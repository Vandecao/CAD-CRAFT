// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! "Assistente" tab next to the insertion centre: help and AI assistant.
//!
//! Sign-in is OAuth only ("Sign in with ChatGPT", [`oauth`]); nothing about
//! the account is stored. Questions run in a background thread on a copy of
//! the project ([`cad_craft::assistant::conversation`]); the answer brings
//! text, navigation and command buttons and proposal cards. A proposal
//! changes the project only when the user clicks "Aplicar", through the same
//! undoable session operations as the menus (one Ctrl+Z each).
pub mod chatgpt;
pub mod oauth;

use crate::app::{CadApp, Level};
use crate::commands::CommandId;
use crate::state::Selection;
use cad_craft::assistant::conversation::{Conversation, Provider, Snapshot, Turn};
use cad_craft::assistant::help::CommandInfo;
use cad_craft::assistant::proposal::{Change, Proposal};
use cad_craft::assistant::protocol::NavTarget;
use cad_craft::domain::{PageKind, Project};
use eframe::egui::{self, Context, RichText, Ui};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

/// Factory of the provider used by tests instead of ChatGPT.
pub type ProviderFactory = Box<dyn Fn() -> Box<dyn Provider + Send>>;

#[derive(Clone, Debug, PartialEq)]
pub enum ProposalState {
    Pending,
    Applied,
    Discarded,
    Failed(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Entry {
    User(String),
    Assistant {
        text: String,
        navigation: Vec<NavTarget>,
        commands: Vec<String>,
        proposals: Vec<(Proposal, ProposalState, u64)>,
        notes: Vec<String>,
        queries: Vec<String>,
    },
    Error(String),
}

enum Message {
    Delta(String),
    Answer(Result<Turn, String>),
    SignedIn(Result<Vec<(String, String)>, String>),
}

struct Job {
    receiver: mpsc::Receiver<Message>,
    cancel: Arc<AtomicBool>,
}

pub struct AssistantPanel {
    /// The tab shows the assistant instead of the insertion centre.
    pub open: bool,
    tokens: Arc<Mutex<Option<oauth::Tokens>>>,
    pub signed_in: bool,
    pub models: Vec<(String, String)>,
    pub model: String,
    pub input: String,
    pub entries: Vec<Entry>,
    /// Text being streamed for the current answer.
    pub streaming: String,
    pub status: String,
    /// Command highlighted in the ribbon by "Mostrar comando".
    pub highlight: Option<CommandId>,
    conversation: Arc<Mutex<Conversation>>,
    job: Option<Job>,
    /// Tests replace ChatGPT with a scripted provider.
    pub test_provider: Option<ProviderFactory>,
}

impl Default for AssistantPanel {
    fn default() -> Self {
        Self {
            open: false,
            tokens: Arc::new(Mutex::new(None)),
            signed_in: false,
            models: vec![],
            model: String::new(),
            input: String::new(),
            entries: vec![],
            streaming: String::new(),
            status: String::new(),
            highlight: None,
            conversation: Arc::new(Mutex::new(Conversation::default())),
            job: None,
            test_provider: None,
        }
    }
}

impl AssistantPanel {
    pub fn busy(&self) -> bool {
        self.job.is_some()
    }
    /// Signed in, or a test provider is installed.
    pub fn ready(&self) -> bool {
        self.signed_in || self.test_provider.is_some()
    }
    /// Drops the tokens from memory.
    pub fn sign_out(&mut self) {
        if let Ok(mut t) = self.tokens.lock() {
            *t = None;
        }
        self.signed_in = false;
        self.models.clear();
        self.model.clear();
        self.status = "Sessão encerrada; nenhum dado da conta foi salvo.".into();
    }
    pub fn stop(&mut self) {
        if let Some(job) = &self.job {
            job.cancel.store(true, Ordering::Relaxed);
        }
    }
}

fn describe_selection(app: &CadApp) -> String {
    let Some(p) = app.session.project() else {
        return String::new();
    };
    match app.session.selection {
        Selection::Page(id) => p
            .page(id)
            .map(|pg| format!("página {}", pg.number().as_str()))
            .unwrap_or_default(),
        Selection::Representation(id) => p
            .representation(id)
            .and_then(|r| p.device_for_target(r.target()))
            .and_then(|d| p.device(d))
            .map(|d| format!("dispositivo {}", d.designation()))
            .unwrap_or_else(|| "um objeto da folha".into()),
        Selection::Document(id) => p
            .document(id)
            .map(|d| format!("documento {}", d.name().as_str()))
            .unwrap_or_default(),
        _ => String::new(),
    }
}

impl CadApp {
    fn assistant_catalog(&self) -> Vec<CommandInfo> {
        self.chrome
            .commands
            .iter()
            .map(|c| CommandInfo {
                name: c.name.to_owned(),
                description: c.description.clone(),
                tab: c.tab.label().to_owned(),
                group: c.group.to_owned(),
                shortcut: c.shortcut.map(str::to_owned),
                keywords: c.keywords.to_owned(),
            })
            .collect()
    }

    /// Opens or closes the "Assistente" tab (status bar button).
    pub(crate) fn toggle_assistant(&mut self) {
        let panel = self
            .preferences
            .panel_mut(crate::preferences::PanelId::Insertion);
        if self.assistant.open && panel.visible && !panel.collapsed {
            self.assistant.open = false;
        } else {
            panel.visible = true;
            panel.collapsed = false;
            self.assistant.open = true;
        }
    }

    /// Starts the browser sign-in in the background.
    pub(crate) fn assistant_sign_in(&mut self) {
        if self.assistant.busy() {
            return;
        }
        if self.preferences.assistant_host_id.is_empty() {
            match oauth::new_host_id() {
                Ok(id) => self.preferences.assistant_host_id = id,
                Err(e) => {
                    self.assistant.status = e;
                    return;
                }
            }
        }
        let host = self.preferences.assistant_host_id.clone();
        let tokens = self.assistant.tokens.clone();
        let (sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        std::thread::spawn(move || {
            let result = oauth::sign_in(&host, &flag).and_then(|t| {
                *tokens.lock().map_err(|_| "sessão indisponível")? = Some(t);
                chatgpt::list_models(&tokens)
            });
            let _ = sender.send(Message::SignedIn(result));
        });
        self.assistant.status = "Conclua o login na janela do navegador…".into();
        self.assistant.job = Some(Job { receiver, cancel });
    }

    /// Sends the question in the input box.
    pub(crate) fn assistant_ask(&mut self) {
        let question = self.assistant.input.trim().to_owned();
        if question.is_empty() || self.assistant.busy() || !self.assistant.ready() {
            return;
        }
        let provider: Box<dyn Provider + Send> = match &self.assistant.test_provider {
            Some(factory) => factory(),
            None => Box::new(chatgpt::ChatGpt {
                tokens: self.assistant.tokens.clone(),
                model: self.assistant.model.clone(),
            }),
        };
        let project: Option<Project> = self.session.project().cloned();
        let active = self.session.active;
        let selection = describe_selection(self);
        let catalog = self.assistant_catalog();
        let conversation = self.assistant.conversation.clone();
        let (sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        let q = question.clone();
        std::thread::spawn(move || {
            let mut provider = provider;
            let snapshot = Snapshot {
                project: project.as_ref(),
                active_page: active,
                selection,
                catalog: &catalog,
            };
            let delta = sender.clone();
            let result = match conversation.lock() {
                Ok(mut c) => c.ask(
                    provider.as_mut(),
                    &snapshot,
                    &q,
                    &mut |t| {
                        let _ = delta.send(Message::Delta(t.to_owned()));
                    },
                    &flag,
                ),
                Err(_) => Err("conversa indisponível".into()),
            };
            let _ = sender.send(Message::Answer(result));
        });
        self.assistant.entries.push(Entry::User(question));
        self.assistant.input.clear();
        self.assistant.streaming.clear();
        self.assistant.job = Some(Job { receiver, cancel });
    }

    /// Receives streamed text and finished answers.
    pub(crate) fn poll_assistant(&mut self, ctx: &Context) {
        let Some(job) = &self.assistant.job else {
            return;
        };
        loop {
            match job.receiver.try_recv() {
                Ok(Message::Delta(t)) => self.assistant.streaming.push_str(&t),
                Ok(Message::Answer(result)) => {
                    self.assistant.job = None;
                    self.assistant.streaming.clear();
                    let revision = self.session.revision;
                    self.assistant.entries.push(match result {
                        Ok(turn) => Entry::Assistant {
                            text: turn.text,
                            navigation: turn.navigation,
                            commands: turn.commands,
                            proposals: turn
                                .proposals
                                .into_iter()
                                .map(|p| (p, ProposalState::Pending, revision))
                                .collect(),
                            notes: turn.notes,
                            queries: turn.queries,
                        },
                        Err(e) => Entry::Error(e),
                    });
                    return;
                }
                Ok(Message::SignedIn(result)) => {
                    self.assistant.job = None;
                    match result {
                        Ok(models) => {
                            self.assistant.signed_in = true;
                            self.assistant.model = models
                                .first()
                                .map(|(slug, _)| slug.clone())
                                .unwrap_or_default();
                            self.assistant.models = models;
                            self.assistant.status =
                                "Conectado por OAuth · nenhuma senha salva".into();
                        }
                        Err(e) => {
                            if let Ok(mut t) = self.assistant.tokens.lock() {
                                *t = None;
                            }
                            self.assistant.status = e;
                        }
                    }
                    return;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(50));
                    return;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.assistant.job = None;
                    return;
                }
            }
        }
    }

    /// Goes to a page, device or wire (no change to the project).
    pub(crate) fn assistant_navigate(&mut self, target: &NavTarget) {
        let Some(p) = self.session.project() else {
            return;
        };
        let found = match target {
            NavTarget::Page(number) => {
                cad_craft::assistant::tools::page_by_number(p, number).map(|page| (page, None))
            }
            NavTarget::Device(designation) => {
                let wanted = designation.trim_start_matches('-').to_lowercase();
                p.devices()
                    .find(|d| {
                        d.designation()
                            .to_string()
                            .trim_start_matches('-')
                            .to_lowercase()
                            == wanted
                    })
                    .and_then(|d| {
                        p.representations()
                            .find(|r| p.device_for_target(r.target()) == Some(d.id()))
                            .map(|r| (r.page_id(), Some(r.id())))
                    })
            }
            NavTarget::Wire(number) => p
                .connections()
                .find(|c| {
                    c.number()
                        .is_some_and(|n| n.as_str().eq_ignore_ascii_case(number))
                })
                .and_then(|c| p.connection_routes().find(|r| r.connection_id() == c.id()))
                .map(|r| (r.page_id(), Some(r.id()))),
        };
        match found {
            Some((page, rep)) => {
                self.open_page(page);
                if let Some(rep) = rep {
                    self.session.selection = Selection::Representation(rep);
                }
            }
            None => self.log(
                Level::Warning,
                "Assistente: item não encontrado no projeto.",
            ),
        }
    }

    /// Opens the ribbon tab of a command and highlights it.
    pub(crate) fn assistant_show_command(&mut self, name: &str) {
        if let Some(c) = self.chrome.commands.iter().find(|c| c.name == name) {
            self.preferences.ribbon_tab = c.tab;
            self.preferences.ribbon_collapsed = false;
            self.assistant.highlight = Some(c.id);
            self.log(
                Level::Info,
                format!(
                    "Comando \"{}\": aba {}, grupo {}.",
                    c.name,
                    c.tab.label(),
                    c.group
                ),
            );
        }
    }

    /// Applies a proposal through the undoable session operations.
    pub(crate) fn assistant_apply(&mut self, change: &Change) -> Result<(), String> {
        let s = &mut self.session;
        let r = match change {
            Change::NumberConnections(preview) => s.number_connections(preview).map(|_| ()),
            Change::NumberDevices(preview) => s.number_devices(preview).map(|_| ()),
            Change::NumberPages(preview) => s.number_pages(preview).map(|_| ()),
            Change::WireNumber {
                connection,
                number,
                locked,
            } => s
                .set_connection_number(*connection, Some(number.clone()), *locked)
                .map(|_| ()),
            Change::NewDocument { name } => s.new_document(name).map(|_| ()),
            Change::NewPage {
                document,
                name,
                number,
                size,
            } => s
                .new_page(*document, name, number, PageKind::MultiLine, *size)
                .map(|_| ()),
        };
        r.map_err(|e| e.to_string())
    }

    pub(crate) fn show_assistant(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        self.poll_assistant(&ctx);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Provedor").small());
            ui.label("ChatGPT (OpenAI)");
        });
        if self.assistant.signed_in {
            ui.horizontal(|ui| {
                let current = self
                    .assistant
                    .models
                    .iter()
                    .find(|(s, _)| *s == self.assistant.model)
                    .map(|(_, n)| n.clone())
                    .unwrap_or_else(|| self.assistant.model.clone());
                egui::ComboBox::from_id_salt("assistant_model")
                    .selected_text(current)
                    .width(130.0)
                    .show_ui(ui, |ui| {
                        for (slug, name) in self.assistant.models.clone() {
                            ui.selectable_value(&mut self.assistant.model, slug, name);
                        }
                    });
                if ui.button("Sair").clicked() {
                    self.assistant.sign_out();
                }
            });
        } else if self.assistant.test_provider.is_none() {
            ui.horizontal(|ui| {
                let busy = self.assistant.busy();
                if ui
                    .add_enabled(!busy, egui::Button::new("Entrar com ChatGPT"))
                    .on_hover_text(
                        "Login OAuth no navegador. Nenhuma senha ou chave é pedida ou salva.",
                    )
                    .clicked()
                {
                    self.assistant_sign_in();
                }
                if busy && ui.button("Cancelar").clicked() {
                    self.assistant.stop();
                }
            });
        }
        if !self.assistant.status.is_empty() {
            ui.label(RichText::new(&self.assistant.status).small());
        }
        let context = match self.session.project() {
            Some(p) => {
                let page = self
                    .session
                    .active
                    .and_then(|id| p.page(id))
                    .map(|pg| format!(" · folha {}", pg.number().as_str()))
                    .unwrap_or_default();
                format!("Contexto: {}{page}", p.name().as_str())
            }
            None => "Contexto: nenhum projeto aberto (só ajuda)".into(),
        };
        ui.label(RichText::new(context).small());
        ui.separator();
        let input_height = 70.0;
        let mut actions: Vec<(usize, PanelAction)> = vec![];
        egui::ScrollArea::vertical()
            .id_salt("assistant_chat")
            .stick_to_bottom(true)
            .max_height((ui.available_height() - input_height).max(80.0))
            .show(ui, |ui| {
                if self.assistant.entries.is_empty() {
                    ui.label(
                        RichText::new(
                            "Pergunte sobre o projeto aberto ou sobre como usar o CAD-CRAFT: onde fica um comando, como numerar fios, como corrigir avisos das Verificações…",
                        )
                        .small(),
                    );
                }
                for (i, entry) in self.assistant.entries.iter().enumerate() {
                    show_entry(ui, i, entry, self.session.revision, &mut actions);
                }
                if self.assistant.busy() && self.assistant.signed_in | self.assistant.test_provider.is_some() {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(RichText::new("Pensando…").small());
                    });
                }
            });
        for (i, action) in actions {
            self.run_panel_action(i, action);
        }
        ui.separator();
        let enabled = self.assistant.ready() && !self.assistant.busy();
        let hint = if self.assistant.ready() {
            "Pergunte sobre o projeto aberto…"
        } else {
            "Entre com ChatGPT para conversar"
        };
        let edit = ui.add_enabled(
            enabled,
            egui::TextEdit::multiline(&mut self.assistant.input)
                .hint_text(hint)
                .desired_rows(2)
                .desired_width(f32::INFINITY),
        );
        let enter =
            edit.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift);
        ui.horizontal(|ui| {
            if self.assistant.busy() {
                if ui.button("Interromper").clicked()
                    || ui.input(|i| i.key_pressed(egui::Key::Escape))
                {
                    self.assistant.stop();
                }
            } else if ui
                .add_enabled(enabled, egui::Button::new("Enviar"))
                .clicked()
                || enter
            {
                self.assistant_ask();
            }
            ui.label(RichText::new("Enter envia · Esc interrompe").small());
        });
    }

    fn run_panel_action(&mut self, entry: usize, action: PanelAction) {
        match action {
            PanelAction::Navigate(target) => self.assistant_navigate(&target),
            PanelAction::Command(name) => self.assistant_show_command(&name),
            PanelAction::Apply(index) => {
                let Some(Entry::Assistant { proposals, .. }) = self.assistant.entries.get(entry)
                else {
                    return;
                };
                let Some((proposal, ProposalState::Pending, revision)) =
                    proposals.get(index).cloned()
                else {
                    return;
                };
                let state = if revision != self.session.revision {
                    ProposalState::Failed(
                        "o projeto mudou depois da proposta; pergunte de novo".into(),
                    )
                } else {
                    match self.assistant_apply(&proposal.change) {
                        Ok(()) => {
                            self.log(
                                Level::Info,
                                format!("Assistente: {} aplicada (Ctrl+Z desfaz).", proposal.title),
                            );
                            ProposalState::Applied
                        }
                        Err(e) => ProposalState::Failed(e),
                    }
                };
                if let Some(Entry::Assistant { proposals, .. }) =
                    self.assistant.entries.get_mut(entry)
                {
                    proposals[index].1 = state;
                }
            }
            PanelAction::Discard(index) => {
                if let Some(Entry::Assistant { proposals, .. }) =
                    self.assistant.entries.get_mut(entry)
                {
                    proposals[index].1 = ProposalState::Discarded;
                }
            }
        }
    }
}

enum PanelAction {
    Navigate(NavTarget),
    Command(String),
    Apply(usize),
    Discard(usize),
}

fn show_entry(
    ui: &mut Ui,
    i: usize,
    entry: &Entry,
    revision: u64,
    actions: &mut Vec<(usize, PanelAction)>,
) {
    match entry {
        Entry::User(text) => {
            egui::Frame::new()
                .fill(ui.visuals().selection.bg_fill.gamma_multiply(0.35))
                .inner_margin(6)
                .corner_radius(4)
                .show(ui, |ui| {
                    ui.label(text);
                });
        }
        Entry::Error(text) => {
            ui.colored_label(
                egui::Color32::from_rgb(230, 120, 100),
                format!("Erro: {text}"),
            );
        }
        Entry::Assistant {
            text,
            navigation,
            commands,
            proposals,
            notes,
            queries,
        } => {
            if !queries.is_empty() {
                ui.label(
                    RichText::new(format!("consultou: {}", queries.join(", ")))
                        .small()
                        .weak(),
                );
            }
            ui.label(answer_layout(ui, text));
            ui.horizontal_wrapped(|ui| {
                for target in navigation {
                    let label = match target {
                        NavTarget::Page(n) => format!("Ir à folha {n}"),
                        NavTarget::Device(d) => format!("Ir a {d}"),
                        NavTarget::Wire(w) => format!("Ir ao fio {w}"),
                    };
                    if ui.button(label).clicked() {
                        actions.push((i, PanelAction::Navigate(target.clone())));
                    }
                }
                for name in commands {
                    if ui.button(format!("Mostrar \"{name}\"")).clicked() {
                        actions.push((i, PanelAction::Command(name.clone())));
                    }
                }
            });
            for (k, (proposal, state, at)) in proposals.iter().enumerate() {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.label(RichText::new(format!("Proposta: {}", proposal.title)).strong());
                    for line in &proposal.lines {
                        ui.label(RichText::new(format!("• {line}")).small());
                    }
                    match state {
                        ProposalState::Pending => {
                            ui.label(
                                RichText::new("Nada muda até você aplicar. Ctrl+Z desfaz.").small(),
                            );
                            ui.horizontal(|ui| {
                                let fresh = *at == revision;
                                if ui
                                    .add_enabled(fresh, egui::Button::new("Aplicar"))
                                    .on_disabled_hover_text("O projeto mudou depois da proposta.")
                                    .clicked()
                                {
                                    actions.push((i, PanelAction::Apply(k)));
                                }
                                if ui.button("Descartar").clicked() {
                                    actions.push((i, PanelAction::Discard(k)));
                                }
                            });
                        }
                        ProposalState::Applied => {
                            ui.label(RichText::new("Aplicada (Ctrl+Z desfaz).").small());
                        }
                        ProposalState::Discarded => {
                            ui.label(RichText::new("Descartada.").small().weak());
                        }
                        ProposalState::Failed(e) => {
                            ui.colored_label(
                                egui::Color32::from_rgb(230, 120, 100),
                                format!("Não aplicada: {e}"),
                            );
                        }
                    }
                });
            }
            for note in notes {
                ui.label(RichText::new(note).small().weak());
            }
        }
    }
    ui.add_space(4.0);
}

/// Answer text with the model's `**bold**` markup drawn as strong text
/// instead of showing the asterisks. Other text is kept as written.
pub fn answer_layout(ui: &Ui, text: &str) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let parts: Vec<&str> = text.split("**").collect();
    // An odd number of markers leaves the last one unpaired: keep it as text.
    let paired = parts.len() % 2 == 1;
    for (i, part) in parts.iter().enumerate() {
        let bold = i % 2 == 1 && (paired || i + 1 < parts.len());
        let shown = if i % 2 == 1 && !bold {
            format!("**{part}")
        } else {
            (*part).to_owned()
        };
        let rich = if bold {
            RichText::new(shown).strong()
        } else {
            RichText::new(shown)
        };
        rich.append_to(
            &mut job,
            ui.style(),
            egui::FontSelection::Default,
            egui::Align::Min,
        );
    }
    job
}
