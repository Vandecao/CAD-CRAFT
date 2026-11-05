// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapas 21–22: diálogo Exportar PDF / Imprimir / Exportar DXF / Exportar
//! DWG. A geração roda numa thread separada (progresso e cancelamento);
//! imprimir gera um PDF temporário e o entrega ao sistema; DWG passa pelo
//! conversor externo configurado. O acesso ao sistema fica atrás de
//! [`SystemShell`], substituível nos testes (que nunca abrem nem imprimem
//! nada de verdade).
use crate::app::{CadApp, Level};
use crate::dwg;
use cad_craft::domain::*;
use cad_craft::dxf_export;
use cad_craft::pdf::{self, PaperFit, PdfError, PdfOptions, PdfScope};
use eframe::egui::{self, Context};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PdfMode {
    Export,
    Print,
    /// Etapa 22: um DXF R12 por página numa pasta.
    Dxf,
    /// Etapa 22: DXF convertido em DWG pelo conversor externo.
    Dwg,
}

impl PdfMode {
    fn title(self) -> &'static str {
        match self {
            Self::Export => "Exportar PDF",
            Self::Print => "Imprimir",
            Self::Dxf => "Exportar DXF",
            Self::Dwg => "Exportar DWG",
        }
    }
    fn action(self) -> &'static str {
        match self {
            Self::Export => "Exportar",
            Self::Print => "Imprimir agora",
            Self::Dxf => "Exportar DXF",
            Self::Dwg => "Exportar DWG",
        }
    }
    fn produced(self) -> &'static str {
        match self {
            Self::Export | Self::Print => "PDF gerado em",
            Self::Dxf => "DXF gerado em",
            Self::Dwg => "DWG gerado em",
        }
    }
}

/// Abrir e imprimir arquivos pelo sistema operacional.
pub trait SystemShell: Send {
    fn open(&mut self, path: &Path) -> Result<(), String>;
    fn print(&mut self, path: &Path) -> Result<(), String>;
}

/// Leitor de PDF padrão do sistema (verbo "imprimir" no Windows).
pub struct NativeShell;
impl SystemShell for NativeShell {
    fn open(&mut self, path: &Path) -> Result<(), String> {
        let mut command = if cfg!(windows) {
            let mut c = std::process::Command::new("cmd");
            c.args(["/C", "start", ""]).arg(path);
            c
        } else if cfg!(target_os = "macos") {
            let mut c = std::process::Command::new("open");
            c.arg(path);
            c
        } else {
            let mut c = std::process::Command::new("xdg-open");
            c.arg(path);
            c
        };
        command.spawn().map(|_| ()).map_err(|e| e.to_string())
    }
    fn print(&mut self, path: &Path) -> Result<(), String> {
        let mut command = if cfg!(windows) {
            // Aspas simples do PowerShell: duplicar as do caminho.
            let quoted = path.display().to_string().replace('\'', "''");
            let mut c = std::process::Command::new("powershell");
            c.args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &format!("Start-Process -FilePath '{quoted}' -Verb Print"),
            ]);
            c
        } else {
            let mut c = std::process::Command::new("lp");
            c.arg(path);
            c
        };
        command.spawn().map(|_| ()).map_err(|e| e.to_string())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ScopeChoice {
    #[default]
    Project,
    Document,
    ActivePage,
    Pages,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PaperChoice {
    #[default]
    A4,
    A3,
}

enum Message {
    Progress(usize, usize),
    /// Resumo do relatório ou erro.
    Done(Result<String, String>),
}

struct Job {
    mode: PdfMode,
    /// Arquivo (PDF) ou pasta (DXF/DWG) produzidos.
    path: PathBuf,
    receiver: mpsc::Receiver<Message>,
    cancel: Arc<AtomicBool>,
    progress: (usize, usize),
}

pub struct PdfWindow {
    mode: Option<PdfMode>,
    pub scope: ScopeChoice,
    pub chosen: BTreeSet<PageId>,
    pub monochrome: bool,
    pub bookmarks: bool,
    pub open_after: bool,
    pub paper: PaperChoice,
    pub fit: PaperFit,
    /// Etapa 22: argumento de versão do ODA File Converter (`ACAD2018`…).
    pub dwg_version: &'static str,
    /// Etapa 22: tempo limite do conversor DWG.
    pub dwg_timeout: Duration,
    job: Option<Job>,
    /// Resultado da última geração (relatório ou erro).
    pub result: Option<Result<String, String>>,
    pub shell: Box<dyn SystemShell>,
}

impl Default for PdfWindow {
    fn default() -> Self {
        Self {
            mode: None,
            scope: ScopeChoice::Project,
            chosen: BTreeSet::new(),
            monochrome: false,
            bookmarks: true,
            open_after: false,
            paper: PaperChoice::A4,
            fit: PaperFit::Fit,
            dwg_version: "ACAD2018",
            dwg_timeout: dwg::DEFAULT_TIMEOUT,
            job: None,
            result: None,
            shell: Box::new(NativeShell),
        }
    }
}

impl PdfWindow {
    pub fn open(&mut self, mode: PdfMode) {
        self.mode = Some(mode);
        self.result = None;
    }
    pub fn is_open(&self) -> bool {
        self.mode.is_some()
    }
    pub fn is_running(&self) -> bool {
        self.job.is_some()
    }

    fn scope(&self, app_active: Option<PageId>, project: &Project) -> Result<PdfScope, String> {
        match self.scope {
            ScopeChoice::Project => Ok(PdfScope::Project),
            ScopeChoice::Document => app_active
                .and_then(|id| project.page(id))
                .map(|p| PdfScope::Document(p.document_id()))
                .or_else(|| {
                    project
                        .documents()
                        .next()
                        .map(|d| PdfScope::Document(d.id()))
                })
                .ok_or_else(|| "nenhum documento".to_owned()),
            ScopeChoice::ActivePage => app_active
                .map(|id| PdfScope::Pages(vec![id]))
                .ok_or_else(|| "nenhuma página ativa".to_owned()),
            ScopeChoice::Pages => Ok(PdfScope::Pages(self.chosen.iter().copied().collect())),
        }
    }

    /// Recebe mensagens da thread; ao terminar, abre/imprime e guarda o relatório.
    fn poll(&mut self, ctx: &Context) -> Option<(Level, String)> {
        let job = self.job.as_mut()?;
        let mut finished = None;
        while let Ok(message) = job.receiver.try_recv() {
            match message {
                Message::Progress(done, total) => job.progress = (done, total),
                Message::Done(result) => finished = Some(result),
            }
        }
        let Some(result) = finished else {
            ctx.request_repaint_after(Duration::from_millis(50));
            return None;
        };
        let job = self.job.take().expect("tarefa em andamento");
        let log = match result {
            Ok(summary) => {
                let follow_up = match job.mode {
                    PdfMode::Print => self.shell.print(&job.path).map(|()| "Enviado à impressão."),
                    PdfMode::Export if self.open_after => self
                        .shell
                        .open(&job.path)
                        .map(|()| "Aberto no leitor de PDF."),
                    PdfMode::Dxf | PdfMode::Dwg if self.open_after => {
                        self.shell.open(&job.path).map(|()| "Pasta aberta.")
                    }
                    _ => Ok(""),
                };
                match follow_up {
                    Ok(extra) => {
                        let text = format!(
                            "{} {}. {summary} {extra}",
                            job.mode.produced(),
                            job.path.display()
                        );
                        self.result = Some(Ok(text.trim_end().to_owned()));
                        (Level::Info, text)
                    }
                    Err(e) => {
                        let text = format!("Arquivos gerados, mas o sistema recusou: {e}");
                        self.result = Some(Err(text.clone()));
                        (Level::Error, text)
                    }
                }
            }
            Err(e) => {
                self.result = Some(Err(e.clone()));
                (Level::Error, e)
            }
        };
        Some(log)
    }
}

/// Trabalho da thread: recebe o callback de progresso e devolve o resumo.
type Work = Box<dyn FnOnce(&mut dyn FnMut(usize, usize) -> bool) -> Result<String, String> + Send>;

fn error_text(e: PdfError) -> String {
    match e {
        PdfError::Cancelled => "Exportação cancelada.".to_owned(),
        other => other.to_string(),
    }
}

fn spawn(work: Work, path: PathBuf, mode: PdfMode) -> Job {
    let (sender, receiver) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    let flag = cancel.clone();
    std::thread::spawn(move || {
        let progress = sender.clone();
        let result = work(&mut |done, total| {
            let _ = progress.send(Message::Progress(done, total));
            !flag.load(Ordering::Relaxed)
        });
        let _ = sender.send(Message::Done(result));
    });
    Job {
        mode,
        path,
        receiver,
        cancel,
        progress: (0, 0),
    }
}

fn pdf_work(project: Project, options: PdfOptions, target: PathBuf) -> Work {
    Box::new(move |progress| {
        let out = pdf::export_pdf(&project, &options, progress).map_err(error_text)?;
        std::fs::write(&target, &out.bytes)
            .map_err(|e| format!("não foi possível gravar {}: {e}", target.display()))?;
        Ok(out.report.summary())
    })
}

fn dxf_work(project: Project, scope: PdfScope, folder: PathBuf) -> Work {
    Box::new(move |progress| {
        let (files, report) =
            dxf_export::export_dxf_pages(&project, &scope, progress).map_err(error_text)?;
        for file in &files {
            std::fs::write(folder.join(&file.name), &file.bytes)
                .map_err(|e| format!("não foi possível gravar {}: {e}", file.name))?;
        }
        Ok(format!("{} arquivo(s). {}", files.len(), report.summary()))
    })
}

fn dwg_work(
    project: Project,
    scope: PdfScope,
    folder: PathBuf,
    converter: PathBuf,
    version: &'static str,
    timeout: Duration,
) -> Work {
    Box::new(move |progress| {
        let (files, report) =
            dxf_export::export_dxf_pages(&project, &scope, progress).map_err(error_text)?;
        let count = dwg::convert_to_dwg(&converter, &files, &folder, version, timeout)?;
        Ok(format!("{count} arquivo(s) DWG. {}", report.summary()))
    })
}

fn temporary_print_file() -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    std::env::temp_dir().join(format!("cad-craft-impressao-{stamp}.pdf"))
}

impl CadApp {
    pub(crate) fn show_pdf_window(&mut self, ctx: &Context) {
        if let Some((level, text)) = self.pdf.poll(ctx) {
            self.log(level, text);
        }
        let Some(mode) = self.pdf.mode else {
            return;
        };
        let mut open = true;
        let mut start = false;
        let mut cancel = false;
        let active = self.session.active;
        let converter_problem = dwg::converter_problem(&self.preferences.dwg_converter);
        egui::Window::new(mode.title())
            .id(egui::Id::new("pdf_window"))
            .open(&mut open)
            .default_width(420.0)
            .show(ctx, |ui| {
                let Some(project) = self.session.project() else {
                    ui.weak("Nenhum projeto aberto.");
                    return;
                };
                let w = &mut self.pdf;
                ui.strong("Alcance");
                ui.horizontal_wrapped(|ui| {
                    ui.radio_value(&mut w.scope, ScopeChoice::Project, "Projeto");
                    ui.radio_value(&mut w.scope, ScopeChoice::Document, "Documento ativo");
                    ui.radio_value(&mut w.scope, ScopeChoice::ActivePage, "Página ativa");
                    ui.radio_value(&mut w.scope, ScopeChoice::Pages, "Páginas escolhidas");
                });
                if w.scope == ScopeChoice::Pages {
                    egui::ScrollArea::vertical()
                        .max_height(160.0)
                        .show(ui, |ui| {
                            for document in project.documents() {
                                ui.weak(document.name().as_str());
                                for page in project.ordered_pages(document.id()).unwrap_or_default()
                                {
                                    let mut on = w.chosen.contains(&page.id());
                                    let label = format!(
                                        "{} · {}",
                                        page.number().as_str(),
                                        page.name().as_str()
                                    );
                                    if ui.checkbox(&mut on, label).changed() {
                                        if on {
                                            w.chosen.insert(page.id());
                                        } else {
                                            w.chosen.remove(&page.id());
                                        }
                                    }
                                }
                            }
                        });
                }
                ui.separator();
                match mode {
                    PdfMode::Export | PdfMode::Print => {
                        ui.checkbox(&mut w.monochrome, "Preto e branco");
                        ui.checkbox(&mut w.bookmarks, "Marcadores por documento e página");
                    }
                    PdfMode::Dxf | PdfMode::Dwg => {
                        ui.weak(
                            "DXF R12 em mm, um arquivo por página, camadas por categoria \
                             (CAD-CRAFT Moldura, Simbolos, Fios, Textos, Referencias…).",
                        );
                    }
                }
                match mode {
                    PdfMode::Export => {
                        ui.checkbox(&mut w.open_after, "Abrir ao terminar");
                    }
                    PdfMode::Print => {
                        ui.horizontal(|ui| {
                            ui.label("Papel");
                            ui.radio_value(&mut w.paper, PaperChoice::A4, "A4");
                            ui.radio_value(&mut w.paper, PaperChoice::A3, "A3");
                        });
                        ui.horizontal(|ui| {
                            ui.label("Escala");
                            ui.radio_value(&mut w.fit, PaperFit::Fit, "Ajustar ao papel");
                            ui.radio_value(&mut w.fit, PaperFit::Actual, "Tamanho real (1:1)");
                        });
                    }
                    PdfMode::Dxf => {
                        ui.checkbox(&mut w.open_after, "Abrir pasta ao terminar");
                    }
                    PdfMode::Dwg => {
                        ui.checkbox(&mut w.open_after, "Abrir pasta ao terminar");
                        ui.horizontal(|ui| {
                            ui.label("Versão DWG");
                            for (label, value) in dwg::DWG_VERSIONS {
                                ui.radio_value(&mut w.dwg_version, value, label);
                            }
                        });
                        if let Some(problem) = converter_problem {
                            ui.colored_label(egui::Color32::from_rgb(200, 120, 20), problem);
                        }
                    }
                }
                ui.separator();
                if let Some(job) = &w.job {
                    let (done, total) = job.progress;
                    let fraction = if total == 0 {
                        0.0
                    } else {
                        done as f32 / total as f32
                    };
                    ui.add(
                        egui::ProgressBar::new(fraction).text(format!("Página {done} de {total}")),
                    );
                    if ui.button("Cancelar exportação").clicked() {
                        cancel = true;
                    }
                } else {
                    let enabled = mode != PdfMode::Dwg || converter_problem.is_none();
                    let response = ui.add_enabled(enabled, egui::Button::new(mode.action()));
                    if response
                        .on_disabled_hover_text(converter_problem.unwrap_or_default())
                        .clicked()
                    {
                        start = true;
                    }
                }
                match &w.result {
                    Some(Ok(text)) => {
                        ui.label(text);
                    }
                    Some(Err(text)) => {
                        ui.colored_label(egui::Color32::RED, text);
                    }
                    None => {}
                }
            });
        if cancel && let Some(job) = &self.pdf.job {
            job.cancel.store(true, Ordering::Relaxed);
        }
        if start {
            self.start_export(mode, active);
        }
        if !open && !self.pdf.is_running() {
            self.pdf.mode = None;
        }
    }

    fn start_export(&mut self, mode: PdfMode, active: Option<PageId>) {
        let Some(project) = self.session.project() else {
            return;
        };
        let scope = match self.pdf.scope(active, project) {
            Ok(scope) => scope,
            Err(e) => {
                self.pdf.result = Some(Err(e));
                return;
            }
        };
        let name = project.name().as_str().to_owned();
        let project = project.clone();
        let (work, path) = match mode {
            PdfMode::Export | PdfMode::Print => {
                let paper = (mode == PdfMode::Print).then(|| {
                    (
                        match self.pdf.paper {
                            PaperChoice::A4 => PaperFormat::A4.size(false),
                            PaperChoice::A3 => PaperFormat::A3.size(false),
                        },
                        self.pdf.fit,
                    )
                });
                let options = PdfOptions {
                    scope,
                    monochrome: self.pdf.monochrome,
                    bookmarks: self.pdf.bookmarks,
                    paper,
                };
                let path = if mode == PdfMode::Print {
                    temporary_print_file()
                } else {
                    match self.files.picker.save_pdf(&name) {
                        Some(path) => path,
                        None => return,
                    }
                };
                (pdf_work(project, options, path.clone()), path)
            }
            PdfMode::Dxf => {
                let Some(folder) = self.files.picker.pick_folder("Pasta de destino dos DXF") else {
                    return;
                };
                (dxf_work(project, scope, folder.clone()), folder)
            }
            PdfMode::Dwg => {
                if let Some(problem) = dwg::converter_problem(&self.preferences.dwg_converter) {
                    self.pdf.result = Some(Err(problem.to_owned()));
                    return;
                }
                let Some(folder) = self.files.picker.pick_folder("Pasta de destino dos DWG") else {
                    return;
                };
                let converter = PathBuf::from(self.preferences.dwg_converter.trim());
                (
                    dwg_work(
                        project,
                        scope,
                        folder.clone(),
                        converter,
                        self.pdf.dwg_version,
                        self.pdf.dwg_timeout,
                    ),
                    folder,
                )
            }
        };
        self.pdf.result = None;
        self.pdf.job = Some(spawn(work, path, mode));
    }
}
