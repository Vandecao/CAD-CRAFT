// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use crate::{
    commands::{self, Command, CommandId},
    dialogs::{Dialog, PendingAction},
    editing::{Edit, GraphicEdit},
    navigation::PageIndex,
    panels::{InsertionState, Inspector, NavigatorState},
    preferences::{PanelId, Preferences},
    state::{Selection, Session},
    theme,
};
use cad_craft::domain::*;
use eframe::egui::{self, Context, Key, Rect, Ui, ViewportCommand};
use std::collections::VecDeque;

/// Etapa 12: como tratar a trava ao abrir um `.cadcraft`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LockMode {
    /// Adquire a trava; se outra sessão a mantém, pergunta ao usuário.
    Normal,
    /// Abre sem trava, somente leitura.
    ReadOnly,
    /// Assume a trava existente por decisão explícita do usuário.
    TakeOver,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Level {
    Info,
    Warning,
    Error,
}
pub struct Message {
    pub level: Level,
    pub text: String,
}
pub struct CadApp {
    pub session: Session,
    pub preferences: Preferences,
    pub last_viewport: Option<Rect>,
    pub(crate) commands: Vec<Command>,
    pub(crate) index: PageIndex,
    pub(crate) index_revision: u64,
    pub(crate) navigator: NavigatorState,
    pub(crate) insertion: InsertionState,
    pub(crate) inspector: Inspector,
    pub(crate) reference_inspector: crate::reference_properties::ReferenceInspector,
    pub(crate) connection_scene: crate::connection_editor::ConnectionScene,
    pub(crate) connection_inspector: crate::connection_properties::ConnectionInspector,
    pub(crate) physical_editor: crate::physical_editor::PhysicalEditor,
    pub(crate) backstage: Option<&'static str>,
    pub(crate) backstage_section: &'static str,
    pub(crate) dialog: Option<Dialog>,
    pub(crate) command_query: String,
    pub(crate) messages: VecDeque<Message>,
    pub(crate) message_filter: Option<Level>,
    pub(crate) cursor_world: Option<Point2D>,
    pub(crate) free_text: String,
    pub(crate) free_height: f64,
    pub(crate) free_stroke: f64,
    pub(crate) free_align: TextAlignment,
    pub(crate) free_edit: Option<(RepresentationId, SymbolPrimitive, f64, TextAlignment)>,
    pub(crate) free_edit_revision: u64,
    pub(crate) context_object: Option<RepresentationId>,
    pub(crate) layout_generation: u64,
    pub(crate) snapshot_text: String,
    pub(crate) snapshot_path: String,
    pub(crate) fullscreen: bool,
    pub(crate) exit_confirmed: bool,
    pub(crate) theme_ready: bool,
    last_system_theme: Option<egui::Theme>,
    /// Etapa 12: seletor de arquivos, recuperação, recentes e salvamento automático.
    pub files: crate::project_file::FileServices,
    /// Etapa 14: editor de molduras (fechado quando `frame` é `None`).
    pub frame_editor: crate::frame_editor::FrameEditor,
    /// Etapa 17: editor de símbolos.
    pub symbol_editor: crate::symbol_editor::SymbolEditor,
    pub part_editor: crate::part_editor::PartEditor,
    pub macro_library: crate::macro_library::MacroLibrary,
    pub(crate) numbering: crate::numbering_ui::NumberingWindow,
    /// Etapa 21: exportar PDF e imprimir.
    pub pdf: crate::pdf_export::PdfWindow,
    /// Etapa 24: painel Verificações.
    pub checks: crate::checks_ui::CheckPanel,
    /// Etapa 25: importação de projeto do QElectroTech.
    pub qet: crate::qet_import_ui::QetImportPanel,
    /// Etapa 27: assistente de IA e ajuda (aba ao lado do Centro de inserção).
    pub assistant: crate::assistant::AssistantPanel,
    pub edz: crate::edz_import_ui::EdzImportPanel,
    pub reports: crate::reports_ui::ReportWindow,
    /// Pasta da biblioteca de símbolos já carregada (recarrega ao mudar).
    symbol_library_seen: Option<String>,
}
impl Default for CadApp {
    fn default() -> Self {
        Self {
            session: Session::default(),
            preferences: Preferences::default(),
            last_viewport: None,
            commands: commands::registry(),
            index: PageIndex::default(),
            index_revision: u64::MAX,
            navigator: NavigatorState::default(),
            insertion: InsertionState::default(),
            inspector: Inspector::default(),
            reference_inspector: crate::reference_properties::ReferenceInspector::default(),
            connection_scene: crate::connection_editor::ConnectionScene::default(),
            connection_inspector: crate::connection_properties::ConnectionInspector::default(),
            physical_editor: crate::physical_editor::PhysicalEditor::default(),
            backstage: None,
            backstage_section: "Propriedades",
            dialog: None,
            command_query: String::new(),
            messages: VecDeque::new(),
            message_filter: None,
            cursor_world: None,
            free_text: "Texto".into(),
            free_height: 4.0,
            free_stroke: 0.25,
            free_align: TextAlignment::Left,
            free_edit: None,
            free_edit_revision: u64::MAX,
            context_object: None,
            layout_generation: 0,
            snapshot_text: String::new(),
            snapshot_path: String::new(),
            fullscreen: false,
            exit_confirmed: false,
            theme_ready: false,
            last_system_theme: None,
            files: crate::project_file::FileServices::default(),
            frame_editor: crate::frame_editor::FrameEditor::default(),
            symbol_editor: crate::symbol_editor::SymbolEditor::default(),
            part_editor: crate::part_editor::PartEditor::default(),
            macro_library: crate::macro_library::MacroLibrary::default(),
            numbering: crate::numbering_ui::NumberingWindow::default(),
            pdf: crate::pdf_export::PdfWindow::default(),
            checks: crate::checks_ui::CheckPanel::default(),
            qet: crate::qet_import_ui::QetImportPanel::default(),
            assistant: crate::assistant::AssistantPanel::default(),
            edz: crate::edz_import_ui::EdzImportPanel::default(),
            reports: crate::reports_ui::ReportWindow::default(),
            symbol_library_seen: None,
        }
    }
}
const RECENT_KEY: &str = "cad-craft.recent.v1";
impl CadApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();
        crate::theme::install_symbol_font(&cc.egui_ctx);
        if let Some(storage) = cc.storage
            && let Some(json) = storage.get_string("cad-craft.visual.v1")
        {
            if let Some(p) = Preferences::from_json(&json) {
                app.preferences = p;
            } else {
                app.log(
                    Level::Warning,
                    "Preferências inválidas ou incompatíveis; padrão restaurado.",
                );
            }
        }
        if let Some(storage) = cc.storage
            && let Some(json) = storage.get_string(RECENT_KEY)
        {
            app.files.recent = serde_json::from_str(&json).unwrap_or_default();
        }
        app.enable_recovery(crate::project_file::Recovery::new(
            crate::project_file::Recovery::default_dir(),
        ));
        app.log(
            Level::Info,
            "CAD-CRAFT pronto. Crie, abra um projeto .cadcraft ou abra a demonstração.",
        );
        app
    }
    /// Ativa a pasta de recuperação e oferece a cópia mais recente, se houver.
    pub fn enable_recovery(&mut self, recovery: crate::project_file::Recovery) {
        let entries = recovery.entries();
        self.files.recovery = Some(recovery);
        if let Some(entry) = entries.into_iter().next() {
            self.log(
                Level::Warning,
                format!("Cópia de recuperação encontrada: {}", entry.name),
            );
            if self.dialog.is_none() {
                self.dialog = Some(Dialog::Recover(entry));
            }
        }
    }
    pub fn show(&mut self, ctx: &Context) {
        // Etapa 17: carrega a biblioteca de símbolos quando a pasta muda.
        let folder = self.preferences.symbol_library.trim().to_owned();
        if self.symbol_library_seen.as_deref() != Some(folder.as_str()) {
            self.symbol_library_seen = Some(folder.clone());
            if !folder.is_empty() {
                self.reload_symbol_library();
            }
        }
        let system_theme = ctx.input(|i| i.raw.system_theme);
        if !self.theme_ready {
            theme::apply(ctx);
            self.theme_ready = true;
        } else if system_theme != self.last_system_theme
            || ctx.input(|i| {
                i.events
                    .iter()
                    .any(|e| matches!(e, egui::Event::WindowFocused(true)))
            })
        {
            // winit 0.30.13 may reset the caption on WM_SETTINGCHANGE without
            // emitting ThemeChanged. Reassert on focus as well, never every frame.
            theme::request_native_dark(ctx);
        }
        self.last_system_theme = system_theme;
        self.preferences.sanitize();
        if self.dialog.is_some()
            || self.backstage.is_some()
            || ctx.input(|i| {
                !i.focused
                    || i.events
                        .iter()
                        .any(|e| matches!(e, egui::Event::WindowFocused(false)))
            })
        {
            self.session.graphics.cancel();
            self.session.graphics.hover = None;
        }
        if self.index_revision != self.session.revision {
            self.index = self
                .session
                .project()
                .map(PageIndex::build)
                .unwrap_or_default();
            self.index_revision = self.session.revision;
            self.navigator.invalidate();
        }
        if ctx.input(|i| i.viewport().close_requested())
            && self.session.dirty
            && !self.exit_confirmed
        {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.dialog = Some(Dialog::Confirm(PendingAction::Exit));
        }
        self.shortcuts(ctx);
        self.autosave();
        let title = if let Some(p) = self.session.project() {
            format!(
                "CAD-CRAFT — {}{}{}{}",
                p.name().as_str(),
                self.session
                    .file
                    .as_ref()
                    .map(|f| format!(
                        " [{}{}]",
                        f.file_name(),
                        if f.read_only { ", somente leitura" } else { "" }
                    ))
                    .unwrap_or_default(),
                self.session
                    .active
                    .and_then(|id| p.page(id))
                    .map(|p| format!(" — {}", p.number().as_str()))
                    .unwrap_or_default(),
                if self.session.dirty { " *" } else { "" }
            )
        } else {
            "CAD-CRAFT — Nenhum projeto aberto".into()
        };
        ctx.send_viewport_cmd(ViewportCommand::Title(title));
        self.top_bar(ctx);
        if self.backstage.is_some() {
            self.show_backstage(ctx);
        } else {
            self.show_ribbon(ctx);
            self.global_status(ctx);
            self.show_messages(ctx);
            // Shared width budget leaves at least 240 logical points for the CAD area.
            let screen = ctx.content_rect();
            let visible = [PanelId::Navigator, PanelId::Insertion, PanelId::Properties]
                .into_iter()
                .filter(|p| self.preferences.panel(*p).visible)
                .count()
                .max(1) as f32;
            let max_width = ((screen.width() - 240.0) / visible).max(65.0);
            for panel in [PanelId::Properties, PanelId::Insertion, PanelId::Navigator] {
                if !self.preferences.panel(panel).visible {
                    continue;
                }
                let pref = self.preferences.panel(panel);
                let collapsed = pref.collapsed;
                let width = if collapsed {
                    30.0
                } else {
                    pref.width.min(max_width)
                };
                let id = egui::Id::new(("side", panel, self.layout_generation));
                let side = if panel == PanelId::Navigator {
                    egui::SidePanel::left(id)
                } else {
                    egui::SidePanel::right(id)
                };
                let output = side
                    .resizable(!collapsed)
                    .min_width(if collapsed { 30.0 } else { 65.0 })
                    .max_width(if collapsed { 30.0 } else { max_width })
                    .default_width(width)
                    .frame(egui::Frame::new().fill(theme::PANEL).inner_margin(4))
                    .show(ctx, |ui| {
                        if collapsed {
                            if ui
                                .button("»")
                                .on_hover_text(format!("Expandir {}", panel.label()))
                                .clicked()
                            {
                                self.preferences.panel_mut(panel).collapsed = false;
                            }
                            return;
                        }
                        match panel {
                            PanelId::Navigator => self.show_navigator(ui),
                            PanelId::Insertion => self.show_insertion(ui),
                            PanelId::Properties => self.show_properties(ui),
                            _ => {}
                        }
                    });
                if !collapsed {
                    self.preferences.panel_mut(panel).width = output.response.rect.width();
                }
            }
            egui::CentralPanel::default()
                .frame(egui::Frame::new().fill(theme::TITLE).inner_margin(0))
                .show(ctx, |ui| {
                    if self.frame_editor.is_open() {
                        self.show_frame_editor(ui);
                    } else if self.symbol_editor.is_open() {
                        self.show_symbol_editor(ui);
                    } else {
                        self.show_documents(ui);
                    }
                });
        }
        self.show_dialog(ctx);
        let mut numbering = std::mem::take(&mut self.numbering);
        numbering.show(&mut self.session, ctx);
        self.numbering = numbering;
        self.reports.show(&mut self.session, &mut self.files, ctx);
        self.poll_qet(ctx);
        self.poll_assistant(ctx);
        self.poll_edz(ctx);
        self.show_edz(ctx);
        self.show_checks(ctx);
        self.show_pdf_window(ctx);
        let mut physical_editor = std::mem::take(&mut self.physical_editor);
        physical_editor.show(&mut self.session, ctx);
        let locate = physical_editor.take_locate();
        self.physical_editor = physical_editor;
        let mut part_editor = std::mem::take(&mut self.part_editor);
        part_editor.show(&mut self.session, &self.preferences, ctx);
        let part_locate = part_editor.take_locate();
        self.part_editor = part_editor;
        let mut macro_library = std::mem::take(&mut self.macro_library);
        macro_library.show(&mut self.session, &self.preferences, ctx);
        let macro_insert = macro_library.take_insert();
        self.macro_library = macro_library;
        if let Some(definition) = macro_insert {
            self.start_macro_insertion(definition);
        }
        if let Some(id) = part_locate {
            self.navigate_to_reference(id);
        }
        if let Some(id) = locate {
            self.navigate_to_reference(id);
        }
    }
    fn shortcuts(&mut self, ctx: &Context) {
        // Etapa 24: navegar pelas mensagens de verificação.
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, Key::F8)) {
            self.step_check(-1);
        } else if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::F8)) {
            self.step_check(1);
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, Key::F1)) {
            self.preferences.ribbon_collapsed = !self.preferences.ribbon_collapsed;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::F11)) {
            self.fullscreen = !self.fullscreen;
            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(self.fullscreen));
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape)) {
            if self.frame_editor.is_open() && self.dialog.is_none() && self.frame_editor.cancel() {
                return;
            }
            if self.symbol_editor.is_open() && self.dialog.is_none() {
                self.symbol_editor.cancel();
                return;
            }
            if self.session.graphics.is_active()
                || self.session.graphics.tool == crate::interaction::Tool::Move
            {
                self.session.graphics.cancel();
            } else if self.dialog.is_some() {
                self.dialog = None;
            } else if !self.command_query.is_empty() {
                self.command_query.clear();
            } else if self.backstage.is_some() {
                self.backstage = None;
            } else {
                self.session
                    .select(self.session.active.map_or(Selection::None, Selection::Page));
            }
            return;
        }
        if self.dialog.is_some() || self.backstage.is_some() || ctx.wants_keyboard_input() {
            return;
        }
        // Etapa 17: o editor de símbolos trata Ctrl+Z/Ctrl+Y/Delete no próprio canvas.
        if self.symbol_editor.is_open() {
            return;
        }
        if self.frame_editor.is_open() {
            for (key, command) in [(Key::Z, CommandId::Undo), (Key::Y, CommandId::Redo)] {
                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, key)) {
                    self.execute(command, ctx);
                }
            }
            return;
        }
        // Ctrl+Shift+S before Ctrl+S: consume_key matches modifiers exactly.
        for (modifiers, key, command) in [
            (egui::Modifiers::CTRL, Key::Z, CommandId::Undo),
            (egui::Modifiers::CTRL, Key::Y, CommandId::Redo),
            (
                egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                Key::S,
                CommandId::SaveProjectAs,
            ),
            (egui::Modifiers::CTRL, Key::S, CommandId::SaveProject),
            (egui::Modifiers::CTRL, Key::O, CommandId::OpenProject),
            (egui::Modifiers::CTRL, Key::C, CommandId::Copy),
            (egui::Modifiers::CTRL, Key::X, CommandId::Cut),
            (egui::Modifiers::CTRL, Key::V, CommandId::Paste),
            (egui::Modifiers::CTRL, Key::D, CommandId::Duplicate),
            (egui::Modifiers::NONE, Key::Delete, CommandId::DeleteFree),
        ] {
            if ctx.input_mut(|i| i.consume_key(modifiers, key)) {
                self.execute(command, ctx);
            }
        }
        // Etapa 16: no Windows/winit, Ctrl+C/X/V chegam como eventos de
        // área de transferência (Copy/Cut/Paste), não como teclas; o Paste só
        // existe quando a área de transferência do sistema tem texto (ver
        // `copy_marker`).
        let clipboard_commands: Vec<CommandId> = ctx.input_mut(|i| {
            let mut found = Vec::new();
            i.events.retain(|e| {
                let command = match e {
                    egui::Event::Copy => CommandId::Copy,
                    egui::Event::Cut => CommandId::Cut,
                    egui::Event::Paste(_) => CommandId::Paste,
                    _ => return true,
                };
                found.push(command);
                false
            });
            found
        });
        for command in clipboard_commands {
            self.execute(command, ctx);
        }
        for (key, dir) in [(Key::PageUp, -1isize), (Key::PageDown, 1)] {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, key))
                && !self.index.rows.is_empty()
            {
                let index = self
                    .index
                    .rows
                    .iter()
                    .position(|r| Some(r.id) == self.session.active)
                    .unwrap_or(0);
                let next = index
                    .saturating_add_signed(dir)
                    .min(self.index.rows.len() - 1);
                let id = self.index.rows[next].id;
                self.open_page(id);
            }
        }
    }
    /// Há um diálogo modal aberto (usado também pelos testes de interface).
    pub fn has_dialog(&self) -> bool {
        self.dialog.is_some()
    }
    pub(crate) fn log(&mut self, level: Level, text: impl Into<String>) {
        if self.messages.len() >= 500 {
            self.messages.pop_front();
        }
        self.messages.push_back(Message {
            level,
            text: text.into(),
        });
    }
    pub(crate) fn edit(&mut self, edit: Edit) {
        match self.session.apply(edit) {
            Ok(()) => self.log(Level::Info, "Propriedade atualizada pelo Core."),
            Err(e) => self.log(Level::Error, e.to_string()),
        }
    }
    /// Etapa 19: liga a ferramenta de inserção de macro na página ativa.
    pub fn start_macro_insertion(&mut self, definition: MacroDefinition) {
        if self.session.active.is_none() {
            self.log(Level::Warning, "Abra uma página para inserir a macro.");
            return;
        }
        self.session.graphics.cancel();
        self.log(
            Level::Info,
            format!(
                "Inserindo macro \"{}\": clique na folha; Girar/Espelhar ajustam; Esc encerra.",
                definition.name().as_str()
            ),
        );
        self.session.graphics.macro_draft = Some(crate::interaction::MacroDraft::new(definition));
        self.session.graphics.tool = crate::interaction::Tool::InsertMacro;
    }
    pub fn graphic_edit(&mut self, edit: GraphicEdit) {
        match self.session.dispatch_graphic(edit) {
            Ok(true) => self.log(Level::Info, "Operação gráfica registrada no histórico."),
            Ok(false) => {}
            Err(e) => {
                self.session.graphics.cancel();
                self.log(Level::Error, e.to_string());
            }
        }
    }
    pub(crate) fn open_page(&mut self, id: PageId) {
        if let Err(e) = self.session.open_page(id) {
            self.log(Level::Error, e.to_string());
        }
    }
    /// Há diálogo modal aberto (confirmação, edição etc.).
    pub fn dialog_open(&self) -> bool {
        self.dialog.is_some()
    }
    pub(crate) fn request(&mut self, action: PendingAction, ctx: &Context) {
        if self.session.dirty {
            self.dialog = Some(Dialog::Confirm(action));
        } else {
            self.perform(action, ctx);
        }
    }
    pub(crate) fn perform(&mut self, action: PendingAction, ctx: &Context) {
        // Opening a file can stop at the lock question without replacing the session.
        if let PendingAction::Open(path) = &action {
            match self.open_project_file(path, LockMode::Normal) {
                Ok(true) => self.after_session_change(),
                Ok(false) => {}
                Err(e) => self.log(Level::Error, e),
            }
            return;
        }
        // Confirmed discard of the current session also drops its recovery copy
        // (unless that copy is exactly what is being restored).
        let current = self.session.project().map(|p| p.id().to_string());
        if !matches!(&action, PendingAction::Recover(e) if Some(&e.project_id) == current.as_ref())
        {
            self.discard_recovery();
        }
        let mut start_checks = false;
        let result = match action {
            PendingAction::Open(_) => unreachable!("handled above"),
            PendingAction::Recover(entry) => self.restore_recovery(&entry),
            PendingAction::DemoConnections => crate::connection_demo::project()
                .and_then(|p| {
                    let page = p.pages().next().map(Page::id);
                    self.session.replace(p, true)?;
                    if let Some(page) = page {
                        self.session.open_page(page)?;
                    }
                    Ok(())
                })
                .map_err(|e| e.to_string()),
            PendingAction::Demo => crate::demo::project()
                .and_then(|p| self.session.replace(p, true))
                .map_err(|e| e.to_string()),
            PendingAction::New(name) => Project::new(name)
                .and_then(crate::state::with_factory_frames)
                .and_then(|p| self.session.replace(p, false))
                .map_err(|e| e.to_string()),
            PendingAction::Import(json) => self.session.import_snapshot(&json),
            PendingAction::ImportQet {
                project,
                open_checks,
            } => {
                start_checks = open_checks;
                self.install_imported(*project)
            }
            PendingAction::Close => {
                self.session.close();
                Ok(())
            }
            PendingAction::Exit => {
                self.exit_confirmed = true;
                ctx.send_viewport_cmd(ViewportCommand::Close);
                Ok(())
            }
        };
        match result {
            Ok(()) => {
                self.after_session_change();
                if start_checks {
                    self.start_check(false);
                }
            }
            Err(e) => self.log(Level::Error, e),
        }
    }
    fn after_session_change(&mut self) {
        self.backstage = None;
        self.navigator = NavigatorState::default();
        self.inspector = Inspector::default();
        self.insertion = InsertionState::default();
        self.files.last_autosave = None;
        self.log(Level::Info, "Sessão atualizada.");
    }

    // ------------------------------------------------ Etapa 12: arquivos

    /// `Ok(false)`: aguardando a decisão sobre a trava; a sessão não mudou.
    pub(crate) fn open_project_file(
        &mut self,
        path: &std::path::Path,
        mode: LockMode,
    ) -> Result<bool, String> {
        use crate::project_file::{FileLock, LockError, ProjectFile, load};
        let loaded = load(path)?;
        let legacy = loaded.is_legacy_snapshot();
        // Reopening the file this session already holds reuses its own lock.
        let own = self
            .session
            .file
            .as_mut()
            .filter(|f| f.path == path && f.locked())
            .and_then(|f| f.take_lock());
        let lock = if legacy || mode == LockMode::ReadOnly {
            None
        } else if own.is_some() {
            own
        } else if mode == LockMode::TakeOver {
            Some(FileLock::take_over(path).map_err(|e| e.to_string())?)
        } else {
            match FileLock::acquire(path) {
                Ok(lock) => Some(lock),
                Err(LockError::Held(owner)) => {
                    self.dialog = Some(Dialog::LockConflict {
                        path: path.to_path_buf(),
                        owner,
                        take_over: false,
                    });
                    return Ok(false);
                }
                Err(e) => return Err(e.to_string()),
            }
        };
        let read_only = !legacy && (loaded.read_only || lock.is_none());
        let source = loaded.source_format;
        // The user already confirmed discarding the previous session.
        self.discard_recovery();
        self.session
            .replace(loaded.project, false)
            .map_err(|e| e.to_string())?;
        crate::project_file::push_recent(&mut self.files.recent, path);
        if legacy {
            // Migrated content must be written with "Salvar como" into a .cadcraft.
            self.session.dirty = true;
            self.log(
                Level::Warning,
                "Snapshot técnico JSON (formato v0) migrado. Use Salvar para gravar como .cadcraft.",
            );
        } else {
            self.session.file = Some(ProjectFile::new(
                path.to_path_buf(),
                read_only,
                source,
                lock,
            ));
            if read_only {
                self.log(
                    Level::Warning,
                    "Projeto aberto somente leitura. Use Salvar como para gravar uma cópia.",
                );
            }
        }
        self.after_session_change();
        Ok(true)
    }
    pub(crate) fn open_project_dialog(&mut self, ctx: &Context) {
        if let Some(path) = self.files.picker.open_project() {
            self.request(PendingAction::Open(path), ctx);
        }
    }
    /// Salva no arquivo associado; sem arquivo gravável, comporta-se como "Salvar como".
    pub fn save_project(&mut self) -> bool {
        match &self.session.file {
            Some(file) if !file.read_only => {
                let path = file.path.clone();
                self.save_to(&path)
            }
            _ => self.save_project_as(),
        }
    }
    pub fn save_project_as(&mut self) -> bool {
        let Some(name) = self.session.project().map(|p| p.name().as_str().to_owned()) else {
            return false;
        };
        let Some(path) = self.files.picker.save_project(&name) else {
            return false;
        };
        self.save_to(&crate::project_file::with_extension(path))
    }
    fn save_to(&mut self, path: &std::path::Path) -> bool {
        use crate::project_file::{FileLock, ProjectFile, save};
        let same = self.session.file.as_ref().is_some_and(|f| f.path == path);
        let reuse = same && self.session.file.as_ref().is_some_and(|f| f.locked());
        let new_lock = if reuse {
            None
        } else {
            match FileLock::acquire(path) {
                Ok(lock) => Some(lock),
                Err(e) => {
                    self.log(Level::Error, format!("{}: {e}", path.display()));
                    return false;
                }
            }
        };
        let Some(project) = self.session.project() else {
            return false;
        };
        let id = project.id().to_string();
        if let Err(e) = save(project, path) {
            // The previous file and lock stay as they were.
            self.log(Level::Error, e);
            return false;
        }
        let lock = match new_lock {
            Some(lock) => Some(lock),
            None => self.session.file.as_mut().and_then(|f| f.take_lock()),
        };
        self.session.file = Some(ProjectFile::new(
            path.to_path_buf(),
            false,
            cad_craft::persistence::FORMAT,
            lock,
        ));
        self.session.dirty = false;
        if let Some(recovery) = &self.files.recovery {
            recovery.discard(&id);
        }
        crate::project_file::push_recent(&mut self.files.recent, path);
        self.log(Level::Info, format!("Projeto salvo em {}.", path.display()));
        true
    }
    /// Cópia de recuperação periódica enquanto houver alterações não salvas.
    pub fn autosave(&mut self) {
        let interval = self.files.autosave_interval;
        let due = self
            .files
            .last_autosave
            .is_none_or(|t| t.elapsed() >= interval);
        // Sair confirmado já descartou a cópia: os quadros finais antes do
        // fechamento não podem gravá-la de novo.
        if !self.session.dirty || !due || self.exit_confirmed {
            return;
        }
        let (Some(recovery), Some(project)) = (&self.files.recovery, self.session.project()) else {
            return;
        };
        let origin = self.session.file.as_ref().map(|f| f.path.clone());
        self.files.last_autosave = Some(std::time::Instant::now());
        if let Err(e) = recovery.store(project, origin.as_deref()) {
            self.log(Level::Warning, format!("Cópia de recuperação falhou: {e}"));
        }
    }
    fn discard_recovery(&mut self) {
        if let (Some(recovery), Some(project)) = (&self.files.recovery, self.session.project()) {
            recovery.discard(&project.id().to_string());
        }
    }
    fn restore_recovery(
        &mut self,
        entry: &crate::project_file::RecoveryEntry,
    ) -> Result<(), String> {
        use crate::project_file::{FileLock, ProjectFile};
        let recovery = self
            .files
            .recovery
            .clone()
            .ok_or("Recuperação indisponível nesta sessão")?;
        let project = recovery.load(entry)?;
        self.session
            .replace(project, false)
            .map_err(|e| e.to_string())?;
        // The recovered copy stays until the project is saved or discarded.
        self.session.dirty = true;
        if let Some(origin) = entry.origin.as_ref().filter(|p| p.is_file()) {
            let lock = FileLock::acquire(origin).ok();
            let read_only = lock.is_none();
            self.session.file = Some(ProjectFile::new(
                origin.clone(),
                read_only,
                cad_craft::persistence::FORMAT,
                lock,
            ));
        }
        Ok(())
    }
    pub(crate) fn recovery_entries(&self) -> Vec<crate::project_file::RecoveryEntry> {
        self.files
            .recovery
            .as_ref()
            .map(|r| r.entries())
            .unwrap_or_default()
    }
    pub fn execute(&mut self, id: CommandId, ctx: &Context) {
        if self.assistant.highlight == Some(id) {
            self.assistant.highlight = None;
        }
        if let Some(reason) = self
            .commands
            .iter()
            .find(|c| c.id == id)
            .and_then(|c| c.reason(&self.session))
        {
            self.log(Level::Warning, reason);
            return;
        }
        use CommandId as C;
        match id {
            C::GenerateReport => self.reports.open(crate::reports_ui::ReportMode::Generate),
            C::UpdateReport => self.reports.open(crate::reports_ui::ReportMode::Update),
            C::ExportList => self.reports.open(crate::reports_ui::ReportMode::Export),
            C::CheckProject => self.start_check(false),
            C::CheckPage => self.start_check(true),
            C::NumberDevices | C::NumberConnections | C::NumberPages => self.numbering.open(id),
            C::ExportPdf => self.pdf.open(crate::pdf_export::PdfMode::Export),
            C::Print => self.pdf.open(crate::pdf_export::PdfMode::Print),
            C::ExportDxf => self.pdf.open(crate::pdf_export::PdfMode::Dxf),
            C::ExportDwg => self.pdf.open(crate::pdf_export::PdfMode::Dwg),
            C::InsertFreeText
            | C::InsertLine
            | C::InsertRectangle
            | C::InsertCircle
            | C::InsertArc
            | C::InsertEllipse => {
                self.session.graphics.cancel();
                self.preferences.panel_mut(PanelId::Insertion).visible = true;
                self.preferences.panel_mut(PanelId::Insertion).collapsed = false;
                self.session.graphics.tool = match id {
                    C::InsertFreeText => crate::interaction::Tool::InsertText,
                    C::InsertLine => crate::interaction::Tool::InsertGraphic(GraphicKind::Line),
                    C::InsertRectangle => {
                        crate::interaction::Tool::InsertGraphic(GraphicKind::Rectangle)
                    }
                    C::InsertCircle => crate::interaction::Tool::InsertGraphic(GraphicKind::Circle),
                    C::InsertEllipse => {
                        crate::interaction::Tool::InsertGraphic(GraphicKind::Ellipse)
                    }
                    _ => crate::interaction::Tool::InsertGraphic(GraphicKind::Arc),
                };
            }
            C::Copy | C::Cut => {
                if let Err(e) = self.session.capture_selection(id == C::Cut) {
                    self.log(Level::Error, e.to_string());
                } else {
                    // Texto no clipboard do sistema: garante o evento Paste do
                    // Ctrl+V no Windows (o conteúdo real fica na sessão).
                    let n = self.session.clipboard.as_ref().map_or(0, |c| c.len());
                    ctx.copy_text(format!("CAD-CRAFT: {n} item(ns) na área de transferência"));
                    self.log(
                        Level::Info,
                        if id == C::Cut {
                            "Seleção recortada (Ctrl+Z desfaz); escolha Colar e clique no destino."
                        } else {
                            "Seleção copiada; escolha Colar e clique no destino."
                        },
                    );
                }
            }
            C::Paste | C::PasteSameDevice => {
                if let Some(clip) = &mut self.session.clipboard {
                    clip.keep_device = id == C::PasteSameDevice;
                    self.session.graphics.cancel();
                    self.session.graphics.tool = crate::interaction::Tool::Paste;
                    self.log(
                        Level::Info,
                        "Clique no destino para colar; repita para várias cópias; Esc encerra.",
                    );
                } else {
                    self.log(Level::Warning, "Área de transferência vazia.");
                }
            }
            C::Duplicate => {
                if let Err(e) = self.session.duplicate_selection() {
                    self.log(Level::Error, e.to_string());
                }
            }
            C::DeleteFree => {
                if let Err(e) = self.session.delete_graphic_selection() {
                    self.log(Level::Error, e.to_string());
                }
            }
            C::InsertInterruption | C::InsertPotential => {
                self.session.graphics.cancel();
                self.session.graphics.tool = if id == C::InsertPotential {
                    crate::interaction::Tool::InsertPotential
                } else {
                    crate::interaction::Tool::InsertInterruption
                };
            }
            C::ReferenceProperties => self.open_reference_properties(),
            C::LocateMainFunction | C::LocateAuxiliaryFunctions => {
                let destinations = self
                    .session
                    .selected_reference_device()
                    .and_then(|d| self.session.project()?.device_references(d).ok())
                    .unwrap_or_default();
                let wanted = if id == C::LocateMainFunction {
                    FunctionRole::Main
                } else {
                    FunctionRole::Auxiliary
                };
                let ids: Vec<_> = destinations
                    .iter()
                    .filter(|r| r.role == wanted)
                    .map(|r| r.representation)
                    .collect();
                if ids.len() == 1 {
                    self.navigate_to_reference(ids[0]);
                } else {
                    self.open_reference_properties();
                }
            }
            C::DemoConnections => self.request(PendingAction::DemoConnections, ctx),
            C::TerminalStripEditor
            | C::ConnectorEditor
            | C::CableEditor
            | C::WiringEditor
            | C::TechnicalLists => {
                use crate::physical_editor::PhysicalTab;
                let tab = match id {
                    C::TerminalStripEditor => PhysicalTab::Strips,
                    C::ConnectorEditor => PhysicalTab::Connectors,
                    C::CableEditor => PhysicalTab::Cables,
                    C::WiringEditor => PhysicalTab::Wiring,
                    _ => PhysicalTab::Lists,
                };
                self.physical_editor.open(tab);
                if let crate::state::Selection::Representation(rep) = self.session.selection
                    && let Some(route) =
                        self.session.project().and_then(|p| p.connection_route(rep))
                {
                    self.physical_editor.focus_connection(route.connection_id());
                }
            }
            C::DrawConnection | C::InsertConnectionAngle => {
                if let Err(e) = self.session.begin_connection() {
                    self.log(Level::Error, e.to_string());
                }
                if id == C::InsertConnectionAngle
                    && let Some(d) = &mut self.session.graphics.connection
                {
                    d.horizontal_first = false;
                }
            }
            C::InsertJunction
            | C::EditConnection
            | C::AddConnectionVertex
            | C::RemoveConnectionVertex => {
                self.session.graphics.cancel();
                self.session.graphics.tool = match id {
                    C::InsertJunction => crate::interaction::Tool::InsertJunction,
                    C::AddConnectionVertex => crate::interaction::Tool::AddVertex,
                    C::RemoveConnectionVertex => crate::interaction::Tool::RemoveVertex,
                    _ => crate::interaction::Tool::EditRoute,
                };
            }
            C::DeleteConnection => {
                if let Some(id) = self.session.graphics.selection.primary() {
                    self.graphic_edit(GraphicEdit::Drawing(ConnectionDrawingEdit::DeleteRoute {
                        id,
                    }));
                }
            }
            C::DeleteJunction => {
                if let Some(id) = self.session.graphics.junction_selection {
                    self.graphic_edit(GraphicEdit::Drawing(
                        ConnectionDrawingEdit::DeleteJunction { id },
                    ));
                    self.session.graphics.junction_selection = None;
                }
            }
            C::HighlightNetwork => self.session.highlight_network(),
            C::ToggleJunctionStyle => {
                self.preferences.junction_outline = !self.preferences.junction_outline
            }
            C::SymbolLibrary => {
                let panel = self.preferences.panel_mut(PanelId::Insertion);
                panel.visible = true;
                panel.collapsed = false;
                self.insertion.show_symbols();
            }
            C::Select => self.session.graphics.cancel(),
            C::Move => {
                self.session.graphics.cancel();
                self.session.graphics.tool = crate::interaction::Tool::Move;
            }
            C::Rotate90 | C::Rotate180 | C::Rotate270 | C::MirrorHorizontal | C::MirrorVertical
                if self.session.graphics.macro_draft.is_some() =>
            {
                // Etapa 19: giro/espelho da macro ainda em posicionamento.
                let draft = self
                    .session
                    .graphics
                    .macro_draft
                    .as_mut()
                    .expect("conferido");
                match id {
                    C::Rotate90 => draft.rotation = (draft.rotation + 90.0) % 360.0,
                    C::Rotate180 => draft.rotation = (draft.rotation + 180.0) % 360.0,
                    C::Rotate270 => draft.rotation = (draft.rotation + 270.0) % 360.0,
                    C::MirrorHorizontal => draft.mirror = !draft.mirror,
                    _ => {
                        draft.mirror = !draft.mirror;
                        draft.rotation = (draft.rotation + 180.0) % 360.0;
                    }
                }
            }
            C::Rotate90 => self.graphic_edit(GraphicEdit::Rotate(90.0)),
            C::Rotate180 => self.graphic_edit(GraphicEdit::Rotate(180.0)),
            C::Rotate270 => self.graphic_edit(GraphicEdit::Rotate(270.0)),
            C::MirrorHorizontal => self.graphic_edit(GraphicEdit::MirrorHorizontal),
            C::MirrorVertical => self.graphic_edit(GraphicEdit::MirrorVertical),
            C::Undo => self.graphic_edit(GraphicEdit::Undo),
            C::Redo => self.graphic_edit(GraphicEdit::Redo),
            C::ShowProperties | C::PageProperties => {
                if id == C::PageProperties
                    && let Some(page) = self.session.active
                {
                    self.session.select(Selection::Page(page));
                }
                let panel = self.preferences.panel_mut(PanelId::Properties);
                panel.visible = true;
                panel.collapsed = false;
            }
            C::LocateReference => {
                let destinations = self.session.reference_destinations();
                if destinations.len() == 1 {
                    self.navigate_to_reference(destinations[0]);
                } else {
                    self.open_reference_properties();
                }
            }
            C::NewProject => self.dialog = Some(Dialog::NewProject(String::new())),
            C::Demo => self.request(PendingAction::Demo, ctx),
            C::OpenSnapshot => self.backstage = Some("Importar"),
            C::ImportQet => self.backstage = Some("Importar"),
            C::ImportEdz => self.edz.open = true,
            C::OpenProject => self.open_project_dialog(ctx),
            C::FrameEditor
            | C::FrameNew
            | C::FrameDuplicate
            | C::FrameDelete
            | C::FrameExport
            | C::FrameImport
            | C::FrameImportExternal
            | C::FrameExportDxf => self.frame_command(id),
            C::SymbolNew
            | C::SymbolEditor
            | C::SymbolExport
            | C::SymbolImport
            | C::SymbolImportElmt
            | C::SymbolImportElmtFolder
            | C::SymbolReload
            | C::SymbolUpdateProject => self.symbol_command(id),
            C::PartEditor => self.part_editor.open = true,
            C::MacroNavigator | C::MacroCreate | C::MacroPage => {
                self.macro_library.open = true;
                self.macro_library.message = match id {
                    C::MacroCreate => {
                        "Preencha ID e nome em \"Nova macro\" e clique em Criar da seleção.".into()
                    }
                    C::MacroPage => "Para guardar a página ativa: preencha ID e nome e clique em \
                                     Criar da página ativa. Para inserir: escolha uma macro de \
                                     página e clique em Inserir como página nova."
                        .into(),
                    _ => std::mem::take(&mut self.macro_library.message),
                };
            }
            C::MacroInsert => match self.macro_library.selected().cloned() {
                Some(definition) if definition.is_page() => {
                    match self.session.insert_page_macro(&definition) {
                        Ok((_, report)) => self.log(
                            Level::Info,
                            format!("Página criada pela macro. {}", report.summary()),
                        ),
                        Err(e) => self.log(Level::Error, e.to_string()),
                    }
                }
                Some(definition) => self.start_macro_insertion(definition),
                None => {
                    self.macro_library.open = true;
                    self.log(Level::Warning, "Escolha uma macro no navegador de macros.");
                }
            },
            C::SaveProject => {
                self.save_project();
            }
            C::SaveProjectAs => {
                self.save_project_as();
            }
            C::ExportSnapshot => {
                self.backstage = Some("Exportar");
                match self.session.snapshot() {
                    Ok(json) => self.snapshot_text = json,
                    Err(e) => self.log(Level::Error, e),
                }
            }
            C::CloseProject => self.request(PendingAction::Close, ctx),
            C::Exit => self.request(PendingAction::Exit, ctx),
            C::Settings => self.backstage = Some("Configurações"),
            C::NewDocument => self.dialog = Some(Dialog::NewDocument(String::new())),
            C::NewPage => {
                if let Some(doc) = self.session.selected_document() {
                    self.dialog = Some(Dialog::new_page(doc));
                }
            }
            C::OpenPage => {
                if let Some(id) = self.session.selected_page() {
                    self.open_page(id);
                }
            }
            C::RenamePage | C::IdentifyPage => {
                if let Some(page_id) = self.session.selected_page()
                    && let Some(page) = self.session.project().and_then(|p| p.page(page_id))
                {
                    let number = id == C::IdentifyPage;
                    self.dialog = Some(Dialog::Rename {
                        id: page_id,
                        number,
                        value: if number { page.number() } else { page.name() }
                            .as_str()
                            .into(),
                    });
                }
            }
            C::DeletePage => {
                if let Some(id) = self.session.selected_page() {
                    self.dialog = Some(Dialog::DeletePage(id));
                }
            }
            C::PageUp | C::PageDown => {
                if let Some(page) = self.session.selected_page()
                    && let Err(e) = self
                        .session
                        .move_page(page, if id == C::PageUp { -1 } else { 1 })
                {
                    self.log(Level::Error, e.to_string());
                }
            }
            C::Navigator => self.toggle_panel(PanelId::Navigator),
            C::Properties => self.toggle_panel(PanelId::Properties),
            C::Preview => self.toggle_panel(PanelId::Preview),
            C::Insertion => self.toggle_panel(PanelId::Insertion),
            C::Messages => self.toggle_panel(PanelId::Messages),
            C::FindPages => {
                self.preferences.panel_mut(PanelId::Navigator).visible = true;
                self.preferences.panel_mut(PanelId::Navigator).collapsed = false;
                self.navigator.tab = crate::panels::NavigatorTab::Pages;
                self.navigator.focus_filter = true;
            }
            C::SyncSelection => {
                if let Some(page) = self.session.active {
                    self.session.select(Selection::Page(page));
                    self.navigator.reveal_page(page);
                    let panel = self.preferences.panel_mut(PanelId::Navigator);
                    panel.visible = true;
                    panel.collapsed = false;
                }
            }
            C::RestoreLayout => {
                self.preferences.restore_layout();
                self.layout_generation += 1;
            }
            C::ToggleGrid => self.preferences.grid_visible = !self.preferences.grid_visible,
            C::ToggleSnap => self.preferences.snap_enabled = !self.preferences.snap_enabled,
            C::FitPage | C::FitContent | C::ZoomIn | C::ZoomOut | C::Zoom100 | C::PreviousView => {
                self.camera_command(id)
            }
            C::DeviceNavigator
            | C::StructureNavigator
            | C::ConnectionNavigator
            | C::CableNavigator
            | C::ConnectorNavigator
            | C::TerminalNavigator => {
                use crate::panels::NavigatorTab as N;
                self.navigator.tab = match id {
                    C::DeviceNavigator => N::Devices,
                    C::StructureNavigator => N::Structures,
                    C::ConnectionNavigator => N::Connections,
                    C::CableNavigator => N::Cables,
                    _ => N::Terminals,
                };
                let p = self.preferences.panel_mut(PanelId::Navigator);
                p.visible = true;
                p.collapsed = false;
            }
            C::Validate => {
                if let Some(p) = self.session.project() {
                    match p.validate() {
                        Ok(()) => self.log(
                            Level::Info,
                            "Integridade estrutural do Core válida. Esta operação não é ERC.",
                        ),
                        Err(e) => self.log(Level::Error, e.to_string()),
                    }
                }
            }
            C::Repeat | C::Planned(..) => self.log(
                Level::Warning,
                "Comando planejado, indisponível nesta etapa.",
            ),
        }
    }
    pub(crate) fn toggle_panel(&mut self, id: PanelId) {
        let p = self.preferences.panel_mut(id);
        p.visible = !p.visible;
        if p.visible {
            p.collapsed = false;
        }
    }
    pub(crate) fn command_button(
        &self,
        ui: &mut Ui,
        id: CommandId,
        large: bool,
        icon_only: bool,
    ) -> bool {
        let Some(c) = self.commands.iter().find(|c| c.id == id) else {
            return false;
        };
        let tooltip = format!(
            "{}\n{}{}",
            c.name,
            c.description,
            c.reason(&self.session)
                .map(|s| format!("\n{s}"))
                .unwrap_or_default()
        );
        crate::icons::button(
            ui,
            c.icon,
            if icon_only { "" } else { c.name },
            &tooltip,
            c.available(&self.session),
            self.assistant.highlight == Some(id),
            large,
        )
        .clicked()
    }
}
impl eframe::App for CadApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.show(ctx);
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(json) = self.preferences.to_json() {
            storage.set_string("cad-craft.visual.v1", json);
        }
        if let Ok(json) = serde_json::to_string(&self.files.recent) {
            storage.set_string(RECENT_KEY, json);
        }
    }
    fn persist_egui_memory(&self) -> bool {
        false
    }
}
