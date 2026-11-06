// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 25: Arquivo > Importar > Projeto QElectroTech (`.qet`).
//! Análise em segundo plano, prévia (contagens e relatório) e criação de um
//! projeto novo (decisão 5); opcionalmente abre as Verificações.
use crate::app::{CadApp, Level};
use crate::dialogs::PendingAction;
use crate::error::{GuiError, GuiResult};
use cad_craft::interchange::NoteKind;
use cad_craft::interchange::qet::{self, QetImport, QetOptions};
use eframe::egui::{self, Context};
use std::sync::mpsc;

/// Limite do arquivo lido (o importador também confere).
const MAX_QET_BYTES: u64 = 64 * 1024 * 1024;

pub struct QetImportPanel {
    pub path: String,
    /// Abrir o painel Verificações depois de criar o projeto.
    pub open_checks: bool,
    pub status: String,
    pub preview: Option<Box<QetImport>>,
    job: Option<mpsc::Receiver<GuiResult<QetImport>>>,
}
impl Default for QetImportPanel {
    fn default() -> Self {
        Self {
            path: String::new(),
            open_checks: true,
            status: String::new(),
            preview: None,
            job: None,
        }
    }
}
impl QetImportPanel {
    pub fn busy(&self) -> bool {
        self.job.is_some()
    }
}

fn read_and_import(path: &str, options: &QetOptions) -> GuiResult<QetImport> {
    let path = std::path::Path::new(path.trim());
    if !path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("qet"))
    {
        return Err("Escolha um projeto do QElectroTech (.qet).".into());
    }
    let size = std::fs::metadata(path).map_err(GuiError::from)?.len();
    if size > MAX_QET_BYTES {
        return Err("Arquivo maior que 64 MB.".into());
    }
    let bytes = std::fs::read(path).map_err(GuiError::from)?;
    qet::import(&bytes, options).map_err(GuiError::from)
}

impl CadApp {
    /// Lê e converte o arquivo em segundo plano; o resultado vira a prévia.
    pub(crate) fn start_qet_analysis(&mut self) {
        let path = self.qet.path.clone();
        let options = QetOptions {
            mm_per_px: self.preferences.elmt_mm_per_px,
        };
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = sender.send(read_and_import(&path, &options));
        });
        self.qet.preview = None;
        self.qet.status = "Analisando…".into();
        self.qet.job = Some(receiver);
    }

    pub(crate) fn poll_qet(&mut self, ctx: &Context) {
        let Some(job) = &self.qet.job else {
            return;
        };
        match job.try_recv() {
            Ok(Ok(imported)) => {
                self.qet.job = None;
                let st = &imported.stats;
                self.qet.status = format!(
                    "{} folha(s), {} dispositivo(s), {} fio(s), {} relatório(s) de folha; {} ignorado(s), {} aproximado(s).",
                    st.pages,
                    st.devices,
                    st.conductors,
                    st.sheet_reports,
                    imported.report.count(NoteKind::Ignored),
                    imported.report.count(NoteKind::Approximated),
                );
                self.qet.preview = Some(Box::new(imported));
            }
            Ok(Err(e)) => {
                self.qet.job = None;
                self.qet.status = format!("Falha: {e}");
                self.log(Level::Error, format!("Importação QElectroTech: {e}"));
            }
            Err(mpsc::TryRecvError::Empty) => {
                ctx.request_repaint_after(std::time::Duration::from_millis(50));
            }
            Err(mpsc::TryRecvError::Disconnected) => self.qet.job = None,
        }
    }

    /// Cria o projeto importado (com confirmação de alterações pendentes).
    pub(crate) fn confirm_qet_import(&mut self, ctx: &Context) {
        let Some(preview) = self.qet.preview.take() else {
            return;
        };
        self.request(
            PendingAction::ImportQet {
                project: Box::new(preview.project),
                open_checks: self.qet.open_checks,
            },
            ctx,
        );
    }

    pub(crate) fn qet_import_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Importar projeto QElectroTech");
        ui.weak(
            "Cria um projeto novo a partir de um .qet (XML): folhas, molduras, símbolos embutidos, \
             dispositivos (mestre e escravos juntos), fios com número travado, relatórios de folha \
             e textos. O que não tiver equivalente vai para o relatório. .qetz ainda não é lido.",
        );
        ui.label("Caminho do projeto .qet");
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.qet.path)
                    .hint_text("Ex.: C:/Projetos/painel.qet")
                    .desired_width(420.0),
            );
            if ui.button("Procurar…").clicked()
                && let Some(path) = self.files.picker.open_qet()
            {
                self.qet.path = path.display().to_string();
            }
        });
        ui.horizontal(|ui| {
            let busy = self.qet.busy();
            if ui
                .add_enabled(
                    !busy && !self.qet.path.trim().is_empty(),
                    egui::Button::new("Analisar"),
                )
                .clicked()
            {
                self.start_qet_analysis();
            }
            if busy {
                ui.spinner();
            }
            ui.checkbox(&mut self.qet.open_checks, "Abrir Verificações depois");
        });
        if !self.qet.status.is_empty() {
            ui.label(&self.qet.status);
        }
        if let Some(preview) = &self.qet.preview {
            ui.label(format!(
                "Projeto \"{}\" · {} símbolo(s), {} ocorrência(s), {} texto(s), {} desenho(s) sem terminal",
                preview.project.name().as_str(),
                preview.stats.symbols,
                preview.stats.occurrences,
                preview.stats.texts,
                preview.stats.drawings
            ));
            egui::ScrollArea::vertical()
                .id_salt("qet_report")
                .max_height(180.0)
                .show(ui, |ui| {
                    for line in preview.report.lines() {
                        ui.label(line);
                    }
                });
        }
        let ctx = ui.ctx().clone();
        if ui
            .add_enabled(
                self.qet.preview.is_some(),
                egui::Button::new("Criar projeto importado"),
            )
            .clicked()
        {
            self.confirm_qet_import(&ctx);
        }
    }
}

impl CadApp {
    /// Instala o projeto importado como sessão nova, ainda sem arquivo:
    /// fica com alterações pendentes até o primeiro Salvar como.
    pub(crate) fn install_imported(
        &mut self,
        project: cad_craft::domain::Project,
    ) -> GuiResult<()> {
        let first = project
            .documents()
            .next()
            .and_then(|d| project.ordered_pages(d.id()).ok())
            .and_then(|pages| pages.first().map(|p| p.id()));
        self.session
            .replace(project, false)
            .map_err(GuiError::from)?;
        if let Some(page) = first {
            self.session.open_page(page).map_err(GuiError::from)?;
        }
        self.session.dirty = true;
        self.qet.status = "Projeto importado: salve-o como .cadcraft.".into();
        Ok(())
    }
}
