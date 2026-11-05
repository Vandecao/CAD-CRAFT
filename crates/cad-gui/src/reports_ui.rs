// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Diálogos de geração, atualização e exportação de relatórios tabulares.
use crate::{project_file::FileServices, state::Session};
use cad_craft::domain::*;
use cad_craft::report_export::{export_csv, export_xlsx};
use eframe::egui::{self, Context};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{self, Receiver},
    },
    thread,
    time::Duration,
};

enum WorkResult {
    Generate(ReportDelta),
    Update(ReportDelta, usize),
    Export(PathBuf),
}
struct ReportWork {
    progress: Arc<AtomicUsize>,
    cancel: Arc<AtomicBool>,
    receiver: Receiver<Result<WorkResult, String>>,
}
impl std::fmt::Debug for ReportWork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReportWork").finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportMode {
    Generate,
    Update,
    Export,
}

#[derive(Debug)]
pub struct ReportWindow {
    mode: Option<ReportMode>,
    kind: ReportKind,
    document: Option<DocumentId>,
    filter_document: Option<DocumentId>,
    filter_page: Option<PageId>,
    function_path: String,
    location_path: String,
    frame: Option<FrameTemplateId>,
    paper: PaperFormat,
    reverse: bool,
    xlsx: bool,
    all_types: bool,
    materials_by_device: bool,
    pub result: Option<Result<String, String>>,
    work: Option<ReportWork>,
}
impl Default for ReportWindow {
    fn default() -> Self {
        Self {
            mode: None,
            kind: ReportKind::PageIndex,
            document: None,
            filter_document: None,
            filter_page: None,
            function_path: String::new(),
            location_path: String::new(),
            frame: None,
            paper: PaperFormat::A3,
            reverse: false,
            xlsx: false,
            all_types: false,
            materials_by_device: false,
            result: None,
            work: None,
        }
    }
}
impl ReportWindow {
    pub fn open(&mut self, mode: ReportMode) {
        self.mode = Some(mode);
        self.result = None;
    }
    pub fn is_open(&self) -> bool {
        self.mode.is_some()
    }
    fn filter(&self) -> ReportFilter {
        let path = |s: &str| {
            s.split('/')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .collect()
        };
        ReportFilter {
            document: self.filter_document,
            page: self.filter_page,
            function_path: path(&self.function_path),
            location_path: path(&self.location_path),
            materials_by_device: self.materials_by_device,
        }
    }
    pub fn show(&mut self, session: &mut Session, files: &mut FileServices, ctx: &Context) {
        let Some(mode) = self.mode else { return };
        if let Some(work) = &self.work {
            match work.receiver.try_recv() {
                Ok(result) => {
                    let cancelled = work.cancel.load(Ordering::Relaxed);
                    self.work = None;
                    self.result = Some(if cancelled {
                        Err("Operação cancelada".to_owned())
                    } else {
                        match result {
                            Ok(WorkResult::Generate(delta)) => {
                                let page = delta.created_page();
                                session
                                    .apply_report_delta(delta)
                                    .and_then(|_| {
                                        if let Some(page) = page {
                                            session.open_page(page)?;
                                        }
                                        Ok("Relatório gerado".to_owned())
                                    })
                                    .map_err(|e| e.to_string())
                            }
                            Ok(WorkResult::Update(delta, count)) => session
                                .apply_report_delta(delta)
                                .map(|_| format!("{count} relatório(s) atualizado(s)"))
                                .map_err(|e| e.to_string()),
                            Ok(WorkResult::Export(path)) => {
                                Ok(format!("Lista exportada: {}", path.display()))
                            }
                            Err(e) => Err(e),
                        }
                    });
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(Duration::from_millis(20));
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.work = None;
                    self.result = Some(Err("Tarefa de relatório interrompida".to_owned()));
                }
            }
        }
        let Some(project) = session.project() else {
            self.mode = None;
            return;
        };
        let documents: Vec<_> = project
            .documents()
            .map(|d| (d.id(), d.name().as_str().to_owned()))
            .collect();
        let pages: Vec<_> = project
            .pages()
            .map(|p| (p.id(), p.number().as_str().to_owned()))
            .collect();
        if self.document.is_none() {
            self.document = documents.first().map(|d| d.0);
        }
        let groups: Vec<_> = project
            .pages()
            .filter_map(|p| p.report().map(|r| (r.group, r.kind)))
            .collect();
        let active_group = session
            .active
            .and_then(|id| project.page(id))
            .and_then(Page::report)
            .map(|r| r.group);
        let mut action: Option<ReportAction> = None;
        let mut open = true;
        egui::Window::new(match mode {
            ReportMode::Generate => "Gerar relatório",
            ReportMode::Update => "Atualizar relatórios",
            ReportMode::Export => "Exportar lista",
        })
        .open(&mut open)
        .default_width(510.0)
        .show(ctx, |ui| {
            if let Some(work) = &self.work {
                ui.label(format!(
                    "Processando relatório: etapa {}/2",
                    work.progress.load(Ordering::Relaxed)
                ));
                if ui.button("Cancelar operação").clicked() {
                    work.cancel.store(true, Ordering::Relaxed);
                }
                return;
            }
            if mode != ReportMode::Update {
                egui::ComboBox::from_label("Tipo de relatório")
                    .selected_text(self.kind.label())
                    .show_ui(ui, |ui| {
                        for kind in ReportKind::ALL {
                            ui.selectable_value(&mut self.kind, kind, kind.label());
                        }
                    });
                ui.horizontal(|ui| {
                    ui.label("Filtro por documento");
                    egui::ComboBox::from_id_salt("report_filter_document")
                        .selected_text(
                            self.filter_document
                                .and_then(|id| {
                                    documents.iter().find(|d| d.0 == id).map(|d| d.1.as_str())
                                })
                                .unwrap_or("Projeto inteiro"),
                        )
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.filter_document, None, "Projeto inteiro");
                            for (id, name) in &documents {
                                ui.selectable_value(&mut self.filter_document, Some(*id), name);
                            }
                        });
                });
                ui.horizontal(|ui| {
                    ui.label("Filtro por página");
                    egui::ComboBox::from_id_salt("report_filter_page")
                        .selected_text(
                            self.filter_page
                                .and_then(|id| {
                                    pages.iter().find(|p| p.0 == id).map(|p| p.1.as_str())
                                })
                                .unwrap_or("Todas"),
                        )
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.filter_page, None, "Todas");
                            for (id, name) in &pages {
                                ui.selectable_value(&mut self.filter_page, Some(*id), name);
                            }
                        });
                });
                ui.horizontal(|ui| {
                    ui.label("Caminho =");
                    ui.text_edit_singleline(&mut self.function_path);
                    ui.label("Caminho +");
                    ui.text_edit_singleline(&mut self.location_path);
                });
                if self.kind == ReportKind::Materials {
                    ui.checkbox(&mut self.materials_by_device, "Uma linha por dispositivo");
                }
            }
            match mode {
                ReportMode::Generate => {
                    egui::ComboBox::from_label("Documento de destino")
                        .selected_text(
                            self.document
                                .and_then(|id| {
                                    documents.iter().find(|d| d.0 == id).map(|d| d.1.as_str())
                                })
                                .unwrap_or("Nenhum"),
                        )
                        .show_ui(ui, |ui| {
                            for (id, name) in &documents {
                                ui.selectable_value(&mut self.document, Some(*id), name);
                            }
                        });
                    egui::ComboBox::from_label("Formato do papel")
                        .selected_text(self.paper.label())
                        .show_ui(ui, |ui| {
                            for paper in [
                                PaperFormat::A4,
                                PaperFormat::A3,
                                PaperFormat::A2,
                                PaperFormat::A1,
                                PaperFormat::A0,
                            ] {
                                ui.selectable_value(&mut self.paper, paper, paper.label());
                            }
                        });
                    egui::ComboBox::from_label("Moldura")
                        .selected_text(
                            self.frame
                                .and_then(|id| project.frame(id).map(|f| f.name().as_str()))
                                .unwrap_or("Padrão"),
                        )
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.frame, None, "Padrão");
                            for frame in project.frames() {
                                ui.selectable_value(
                                    &mut self.frame,
                                    Some(frame.id()),
                                    frame.name().as_str(),
                                );
                            }
                        });
                    ui.checkbox(&mut self.reverse, "Ordenação inversa");
                    if let Some(document) = self.document {
                        let mut request =
                            ReportRequest::new(self.kind, document, self.paper.size(true));
                        request.filter = self.filter();
                        request.frame = self.frame;
                        request.sort = if self.reverse {
                            ReportSort::Reverse
                        } else {
                            ReportSort::Default
                        };
                        match project.preview_report(&request) {
                            Ok(preview) => {
                                ui.label(format!(
                                    "Prévia: {} página(s), {} linha(s)",
                                    preview.pages, preview.rows
                                ));
                                if ui.button("Gerar relatório").clicked() {
                                    action = Some(ReportAction::Generate(request));
                                }
                            }
                            Err(e) => {
                                ui.colored_label(egui::Color32::RED, e.to_string());
                            }
                        }
                    }
                }
                ReportMode::Update => {
                    let stale = groups
                        .iter()
                        .filter(|(group, _)| project.report_stale(*group).unwrap_or(false))
                        .count();
                    ui.label(format!("{stale} relatório(s) desatualizado(s)"));
                    if let Some(group) = active_group
                        && ui.button("Atualizar selecionado").clicked()
                    {
                        action = Some(ReportAction::Update(vec![group]));
                    }
                    if ui.button("Atualizar todos").clicked() {
                        action = Some(ReportAction::Update(groups.iter().map(|g| g.0).collect()));
                    }
                }
                ReportMode::Export => {
                    ui.checkbox(&mut self.all_types, "Todos os tipos em abas XLSX");
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.xlsx, false, "CSV");
                        ui.radio_value(&mut self.xlsx, true, "XLSX");
                    });
                    if ui.button("Exportar lista").clicked() {
                        action = Some(ReportAction::Export);
                    }
                }
            }
            if let Some(result) = &self.result {
                ui.label(result.as_ref().map_or_else(|e| e.as_str(), |s| s.as_str()));
            }
        });
        if !open && self.work.is_none() {
            self.mode = None;
        }
        match action {
            Some(ReportAction::Generate(request)) => {
                let mut project = project.clone();
                self.start_work(ctx, move |progress, cancel| {
                    if cancel.load(Ordering::Relaxed) {
                        return Err("Operação cancelada".to_owned());
                    }
                    progress.store(1, Ordering::Relaxed);
                    let delta = project
                        .generate_report(&request)
                        .map_err(|e| e.to_string())?;
                    progress.store(2, Ordering::Relaxed);
                    Ok(WorkResult::Generate(delta))
                });
            }
            Some(ReportAction::Update(groups)) => {
                let count = groups
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len();
                let mut project = project.clone();
                self.start_work(ctx, move |progress, cancel| {
                    if cancel.load(Ordering::Relaxed) {
                        return Err("Operação cancelada".to_owned());
                    }
                    progress.store(1, Ordering::Relaxed);
                    let delta = project.update_reports(&groups).map_err(|e| e.to_string())?;
                    progress.store(2, Ordering::Relaxed);
                    Ok(WorkResult::Update(delta, count))
                });
            }
            Some(ReportAction::Export) => {
                let filter = self.filter();
                let kinds = if self.xlsx && self.all_types {
                    ReportKind::ALL.to_vec()
                } else {
                    vec![self.kind]
                };
                if let Some(path) = files.picker.save_list(self.kind.label(), self.xlsx) {
                    let project = project.clone();
                    let xlsx = self.xlsx;
                    self.start_work(ctx, move |progress, cancel| {
                        if cancel.load(Ordering::Relaxed) {
                            return Err("Operação cancelada".to_owned());
                        }
                        let tables = kinds
                            .iter()
                            .map(|k| project.report_table(*k, &filter))
                            .collect::<cad_craft::error::Result<Vec<_>>>()
                            .map_err(|e| e.to_string())?;
                        progress.store(1, Ordering::Relaxed);
                        if cancel.load(Ordering::Relaxed) {
                            return Err("Operação cancelada".to_owned());
                        }
                        let bytes = if xlsx {
                            export_xlsx(&tables)
                        } else {
                            export_csv(&tables[0])
                        }?;
                        if cancel.load(Ordering::Relaxed) {
                            return Err("Operação cancelada".to_owned());
                        }
                        std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
                        progress.store(2, Ordering::Relaxed);
                        Ok(WorkResult::Export(path))
                    });
                }
            }
            None => {}
        }
    }
    fn start_work(
        &mut self,
        ctx: &Context,
        task: impl FnOnce(Arc<AtomicUsize>, Arc<AtomicBool>) -> Result<WorkResult, String>
        + Send
        + 'static,
    ) {
        let (sender, receiver) = mpsc::channel();
        let progress = Arc::new(AtomicUsize::new(0));
        let cancel = Arc::new(AtomicBool::new(false));
        let ctx = ctx.clone();
        let worker_progress = progress.clone();
        let worker_cancel = cancel.clone();
        thread::spawn(move || {
            let result = task(worker_progress, worker_cancel);
            let _ = sender.send(result);
            ctx.request_repaint();
        });
        self.work = Some(ReportWork {
            progress,
            cancel,
            receiver,
        });
    }
}
enum ReportAction {
    Generate(ReportRequest),
    Update(Vec<ReportId>),
    Export,
}
