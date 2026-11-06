// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Project and page validation in the background. Start with the panel renderer: filters, counters, double-click and F8 navigation, justified exclusions and CSV/XLSX export. Listed messages are reevaluated after edits and disappear when fixed.
use crate::app::{CadApp, Level};
use cad_craft::domain::*;
use cad_craft::report_export::{export_rows_csv, export_rows_xlsx};
use eframe::egui::{self, Context};
use std::collections::BTreeSet;
use std::sync::mpsc;

struct CheckJob {
    receiver: mpsc::Receiver<CheckReport>,
    /// Reevaluation retains only messages already listed.
    keep: Option<BTreeSet<String>>,
    revision: u64,
}

#[derive(Default)]
pub struct CheckPanel {
    open: bool,
    pub scope: Option<CheckScope>,
    /// Listed messages after applying the validation scheme.
    pub report: Option<CheckReport>,
    pub severity: Option<CheckSeverity>,
    pub rule: Option<&'static str>,
    pub selected: Option<usize>,
    job: Option<CheckJob>,
    /// Session revision at which the list was calculated.
    revision: u64,
    /// Message being excluded: key and justification.
    ignoring: Option<(String, String)>,
    pub status: String,
}

impl CheckPanel {
    pub fn is_open(&self) -> bool {
        self.open
    }
    pub fn is_running(&self) -> bool {
        self.job.is_some()
    }
    /// Messages visible through the filters, with their index in the complete list.
    pub fn visible(&self) -> Vec<(usize, &CheckMessage)> {
        let Some(report) = &self.report else {
            return vec![];
        };
        report
            .messages
            .iter()
            .enumerate()
            .filter(|(_, m)| self.severity.is_none_or(|s| m.severity == s))
            .filter(|(_, m)| self.rule.is_none_or(|r| m.rule == r))
            .collect()
    }
}

fn spawn(
    project: Project,
    scope: CheckScope,
    keep: Option<BTreeSet<String>>,
    revision: u64,
) -> CheckJob {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(project.check(scope));
    });
    CheckJob {
        receiver,
        keep,
        revision,
    }
}

impl CadApp {
    /// Validate the project or active page.
    pub(crate) fn start_check(&mut self, page_only: bool) {
        let Some(project) = self.session.project() else {
            return;
        };
        let scope = match (page_only, self.session.active) {
            (true, Some(page)) => CheckScope::Page(page),
            (true, None) => {
                self.log(Level::Warning, "Abra uma página para verificar.");
                return;
            }
            _ => CheckScope::Project,
        };
        self.checks.open = true;
        self.checks.scope = Some(scope);
        self.checks.selected = None;
        self.checks.status = "Verificando…".into();
        self.checks.job = Some(spawn(project.clone(), scope, None, self.session.revision));
    }

    /// F8 / Shift+F8: navigate to the next or previous visible message.
    pub(crate) fn step_check(&mut self, step: isize) {
        let visible: Vec<usize> = self.checks.visible().iter().map(|(i, _)| *i).collect();
        if visible.is_empty() {
            return;
        }
        let position = self
            .checks
            .selected
            .and_then(|s| visible.iter().position(|i| *i == s));
        let next = match position {
            None if step > 0 => 0,
            None => visible.len() - 1,
            Some(p) => (p as isize + step).rem_euclid(visible.len() as isize) as usize,
        };
        self.go_to_check(visible[next]);
    }

    fn go_to_check(&mut self, index: usize) {
        self.checks.selected = Some(index);
        let Some(message) = self
            .checks
            .report
            .as_ref()
            .and_then(|r| r.messages.get(index))
            .cloned()
        else {
            return;
        };
        let location = self.session.project().and_then(|p| {
            message
                .targets
                .iter()
                .find_map(|t| p.check_target_location(*t))
        });
        match location {
            Some((_, Some(rep))) => self.navigate_to_reference(rep),
            Some((page, None)) => self.open_page(page),
            None => self.log(
                Level::Info,
                format!("{}: sem posição na folha (item físico).", message.rule),
            ),
        }
    }

    fn poll_checks(&mut self, ctx: &Context) {
        if let Some(job) = &self.checks.job {
            match job.receiver.try_recv() {
                Ok(mut report) => {
                    let job = self.checks.job.take().expect("tarefa");
                    if let Some(keep) = &job.keep {
                        report.messages.retain(|m| keep.contains(&m.key()));
                    }
                    let (e, w, i) = (
                        report.count(CheckSeverity::Error),
                        report.count(CheckSeverity::Warning),
                        report.count(CheckSeverity::Info),
                    );
                    self.checks.status = format!(
                        "{e} erro(s), {w} aviso(s), {i} informação(ões); {} ignorada(s).",
                        report.ignored
                    );
                    if job.keep.is_none() {
                        self.log(Level::Info, format!("Verificação: {}", self.checks.status));
                    }
                    self.checks.revision = job.revision;
                    self.checks.report = Some(report);
                    if self.checks.selected.is_some_and(|s| {
                        self.checks
                            .report
                            .as_ref()
                            .is_none_or(|r| s >= r.messages.len())
                    }) {
                        self.checks.selected = None;
                    }
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(50));
                }
                Err(mpsc::TryRecvError::Disconnected) => self.checks.job = None,
            }
            return;
        }
        // Automatically reevaluate listed messages after edits.
        if let (Some(report), Some(scope), Some(project)) = (
            &self.checks.report,
            self.checks.scope,
            self.session.project(),
        ) && self.session.revision != self.checks.revision
            && !report.messages.is_empty()
        {
            let keep = report.messages.iter().map(CheckMessage::key).collect();
            self.checks.job = Some(spawn(
                project.clone(),
                scope,
                Some(keep),
                self.session.revision,
            ));
        }
    }

    pub(crate) fn show_checks(&mut self, ctx: &Context) {
        self.poll_checks(ctx);
        if !self.checks.open {
            return;
        }
        let mut open = true;
        let mut go = None;
        let mut rerun = None;
        let mut export = None;
        let mut ignore: Option<(String, String)> = None;
        egui::Window::new("Verificações")
            .id(egui::Id::new("checks_window"))
            .open(&mut open)
            .default_width(620.0)
            .show(ctx, |ui| {
                let panel = &mut self.checks;
                ui.horizontal(|ui| {
                    if ui.button("Verificar projeto").clicked() {
                        rerun = Some(false);
                    }
                    if ui.button("Verificar página").clicked() {
                        rerun = Some(true);
                    }
                    if panel.job.is_some() {
                        ui.spinner();
                    }
                    ui.label(&panel.status);
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label("Gravidade");
                    ui.selectable_value(&mut panel.severity, None, "Todas");
                    for s in [
                        CheckSeverity::Error,
                        CheckSeverity::Warning,
                        CheckSeverity::Info,
                    ] {
                        ui.selectable_value(&mut panel.severity, Some(s), s.label());
                    }
                    ui.separator();
                    egui::ComboBox::from_id_salt("check_rule_filter")
                        .selected_text(panel.rule.unwrap_or("Todas as regras"))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut panel.rule, None, "Todas as regras");
                            for r in CHECK_RULES {
                                ui.selectable_value(
                                    &mut panel.rule,
                                    Some(r.code),
                                    format!("{} — {}", r.code, r.title),
                                );
                            }
                        });
                });
                ui.separator();
                let visible: Vec<(usize, CheckMessage)> = panel
                    .visible()
                    .into_iter()
                    .map(|(i, m)| (i, m.clone()))
                    .collect();
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        if visible.is_empty() {
                            ui.weak("Nenhuma mensagem.");
                        }
                        for (index, message) in &visible {
                            let text = format!(
                                "{} · {} · {}",
                                message.rule,
                                message.severity.label(),
                                message.text
                            );
                            let color = match message.severity {
                                CheckSeverity::Error => egui::Color32::from_rgb(220, 80, 70),
                                CheckSeverity::Warning => egui::Color32::from_rgb(220, 160, 40),
                                CheckSeverity::Info => ui.visuals().text_color(),
                            };
                            let response = ui.selectable_label(
                                panel.selected == Some(*index),
                                egui::RichText::new(text).color(color),
                            );
                            if response.double_clicked() {
                                go = Some(*index);
                            } else if response.clicked() {
                                panel.selected = Some(*index);
                            }
                        }
                    });
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Anterior (Shift+F8)").clicked() {
                        go = Some(usize::MAX - 1);
                    }
                    if ui.button("Próxima (F8)").clicked() {
                        go = Some(usize::MAX);
                    }
                    let selected = panel
                        .selected
                        .and_then(|s| panel.report.as_ref()?.messages.get(s))
                        .map(CheckMessage::key);
                    if ui
                        .add_enabled(selected.is_some(), egui::Button::new("Ignorar…"))
                        .clicked()
                        && let Some(key) = selected
                    {
                        panel.ignoring = Some((key, String::new()));
                    }
                    if ui.button("Exportar CSV").clicked() {
                        export = Some(false);
                    }
                    if ui.button("Exportar XLSX").clicked() {
                        export = Some(true);
                    }
                });
                if let Some((key, reason)) = &mut panel.ignoring {
                    ui.horizontal(|ui| {
                        ui.label("Justificativa");
                        ui.text_edit_singleline(reason);
                        if ui
                            .add_enabled(
                                !reason.trim().is_empty(),
                                egui::Button::new("Confirmar ignorar"),
                            )
                            .clicked()
                        {
                            ignore = Some((key.clone(), reason.trim().to_owned()));
                        }
                    });
                }
            });
        if !open {
            self.checks.open = false;
        }
        if let Some(page_only) = rerun {
            self.start_check(page_only);
        }
        match go {
            Some(usize::MAX) => self.step_check(1),
            Some(i) if i == usize::MAX - 1 => self.step_check(-1),
            Some(i) => self.go_to_check(i),
            None => {}
        }
        if let Some((key, reason)) = ignore {
            self.checks.ignoring = None;
            self.ignore_check(key, reason);
        }
        if let Some(xlsx) = export {
            self.export_checks(xlsx);
        }
    }

    fn ignore_check(&mut self, key: String, reason: String) {
        let Some(project) = self.session.project() else {
            return;
        };
        let mut scheme = project.check_scheme().clone();
        let reason = match Text::new(reason) {
            Ok(r) => r,
            Err(e) => {
                self.log(Level::Error, e.to_string());
                return;
            }
        };
        scheme.ignored.push(IgnoredCheck { key, reason });
        match self.session.set_check_scheme(scheme) {
            Ok(_) => {
                self.log(Level::Info, "Mensagem ignorada (Ctrl+Z desfaz).");
                // Automatic reevaluation removes the message from the list.
            }
            Err(e) => self.log(Level::Error, e.to_string()),
        }
    }

    fn export_checks(&mut self, xlsx: bool) {
        let (Some(project), Some(report)) = (self.session.project(), &self.checks.report) else {
            return;
        };
        let rows = project.check_export_rows(report);
        let bytes = if xlsx {
            export_rows_xlsx("Verificações", &CHECK_EXPORT_HEADERS, &rows)
        } else {
            export_rows_csv(&CHECK_EXPORT_HEADERS, &rows)
        };
        let name = format!("{} - verificacoes", project.name().as_str());
        let Some(path) = self.files.picker.save_list(&name, xlsx) else {
            return;
        };
        match bytes.and_then(|b| std::fs::write(&path, b).map_err(|e| e.to_string())) {
            Ok(()) => self.log(
                Level::Info,
                format!("Mensagens exportadas em {}.", path.display()),
            ),
            Err(e) => self.log(Level::Error, e),
        }
    }

    /// Validation settings: enable rules and set their severity.
    pub(crate) fn check_settings_ui(&mut self, ui: &mut egui::Ui) {
        let Some(project) = self.session.project() else {
            return;
        };
        let current = project.check_scheme().clone();
        let mut scheme = current.clone();
        ui.separator();
        ui.heading("Verificações do projeto");
        egui::Grid::new("check_rules")
            .num_columns(3)
            .striped(true)
            .show(ui, |ui| {
                for rule in CHECK_RULES {
                    let mut setting = scheme.setting(rule.code);
                    let before = setting;
                    ui.checkbox(
                        &mut setting.enabled,
                        format!("{} — {}", rule.code, rule.title),
                    );
                    egui::ComboBox::from_id_salt(("check_severity", rule.code))
                        .selected_text(setting.severity.label())
                        .show_ui(ui, |ui| {
                            for s in [
                                CheckSeverity::Error,
                                CheckSeverity::Warning,
                                CheckSeverity::Info,
                            ] {
                                ui.selectable_value(&mut setting.severity, s, s.label());
                            }
                        });
                    ui.end_row();
                    if setting != before {
                        scheme.rules.insert(rule.code.to_owned(), setting);
                    }
                }
            });
        if !scheme.ignored.is_empty() {
            ui.label(format!("{} mensagem(ns) ignorada(s)", scheme.ignored.len()));
            if ui.button("Voltar a mostrar as ignoradas").clicked() {
                scheme.ignored.clear();
            }
        }
        if scheme != current
            && let Err(e) = self.session.set_check_scheme(scheme)
        {
            self.log(Level::Error, e.to_string());
        }
    }
}
