// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Application dialogs and confirmations; start with the dialog rendering functions.
use crate::{
    app::{CadApp, Level},
    editing::{Edit, GraphicEdit},
    navigation::{kind_label, page_kinds},
};
use cad_craft::domain::*;
use eframe::egui::{self, Context};
pub(crate) enum PendingAction {
    Demo,
    DemoConnections,
    New(String),
    Import(String),
    /// Project converted from QElectroTech, opened as a new project.
    ImportQet {
        project: Box<cad_craft::domain::Project>,
        open_checks: bool,
    },
    /// Open a .cadcraft project or v0 snapshot at the given path.
    Open(std::path::PathBuf),
    /// Restore the specified recovery copy.
    Recover(crate::project_file::RecoveryEntry),
    Close,
    Exit,
}
pub(crate) enum Dialog {
    /// Edit free text after double-clicking the sheet.
    EditFreeText {
        id: RepresentationId,
        text: String,
    },
    NewProject(String),
    NewDocument(String),
    NewPage {
        document: DocumentId,
        name: String,
        number: String,
        kind: PageKind,
        width: f64,
        height: f64,
    },
    Rename {
        id: PageId,
        number: bool,
        value: String,
    },
    DeletePage(PageId),
    Confirm(PendingAction),
    OverwriteSnapshot {
        path: String,
        json: String,
    },
    /// Another session already has the file open.
    LockConflict {
        path: std::path::PathBuf,
        owner: String,
        take_over: bool,
    },
    /// Recovery copy found at application startup.
    Recover(crate::project_file::RecoveryEntry),
    ExternalFrame {
        path: std::path::PathBuf,
        imported: Box<cad_craft::interchange::ImportedFrame>,
        page: Option<PageId>,
        dxf_unit_mm: Option<f64>,
        dxf_sheet: Option<Size2D>,
        preview_dirty: bool,
    },
}
impl Dialog {
    pub fn new_page(document: DocumentId) -> Self {
        Self::NewPage {
            document,
            name: String::new(),
            number: String::new(),
            kind: PageKind::MultiLine,
            width: 420.0,
            height: 297.0,
        }
    }
}
impl CadApp {
    pub(crate) fn show_dialog(&mut self, ctx: &Context) {
        let Some(mut dialog) = self.dialogs_state.dialog.take() else {
            return;
        };
        let mut accept = false;
        let mut cancel = false;
        egui::Modal::new(egui::Id::new("session_dialog")).show(ctx,|ui| {
            ui.set_max_width(470.0);
            match &mut dialog {
                Dialog::NewProject(name)=>{ui.heading("Novo projeto");ui.label("Nome do projeto");ui.text_edit_singleline(name);ui.weak("O projeto começa em memória com as molduras de fábrica. Use Salvar (Ctrl+S) para gravá-lo como arquivo .cadcraft.");},
                Dialog::NewDocument(name)=>{ui.heading(crate::strings::NEW_DOCUMENT);ui.label("Nome do documento");ui.text_edit_singleline(name);},
                Dialog::NewPage{document,name,number,kind,width,height}=>{
                    ui.heading(crate::strings::NEW_PAGE);
                    egui::ComboBox::from_id_salt("new_page_document").selected_text(self.session.project().and_then(|p|p.document(*document)).map(|d|d.name().as_str()).unwrap_or("Documento")).show_ui(ui,|ui| {if let Some(p)=self.session.project(){for doc in p.documents(){ui.selectable_value(document,doc.id(),doc.name().as_str());}}});
                    ui.label("Nome");ui.text_edit_singleline(name);ui.label("Identificação");ui.text_edit_singleline(number);
                    egui::ComboBox::from_id_salt("new_page_kind").selected_text(kind_label(kind)).show_ui(ui,|ui|{for candidate in page_kinds(){let label=kind_label(&candidate).to_owned();ui.selectable_value(kind,candidate,label);}});
                    ui.label("Formato");let current=Size2D::new(*width,*height).ok().and_then(PaperFormat::matching).map(|(f,l)|format!("{} {}",f.label(),if l{"paisagem"}else{"retrato"})).unwrap_or_else(||"Personalizado".into());egui::ComboBox::from_id_salt("new_page_format").selected_text(current).show_ui(ui,|ui|{for f in PaperFormat::ALL{for landscape in [true,false]{if ui.selectable_label(false,format!("{} {}",f.label(),if landscape{"paisagem"}else{"retrato"})).clicked(){let s=f.size(landscape);*width=s.width();*height=s.height();}}}});
                    ui.horizontal(|ui|{ui.label("Largura (mm)");ui.add(egui::DragValue::new(width).speed(1.0));ui.label("Altura (mm)");ui.add(egui::DragValue::new(height).speed(1.0));});
                },
                Dialog::Rename{number,value,..}=>{ui.heading(if *number{"Alterar identificação"}else{crate::strings::RENAME_PAGE});ui.text_edit_singleline(value);},
                Dialog::DeletePage(id)=>{ui.heading(crate::strings::DELETE_PAGE);let name=self.session.project().and_then(|p|p.page(*id)).map(|p|p.name().as_str()).unwrap_or("Página ausente");ui.label(format!("Excluir “{name}” do projeto?"));ui.weak("O Core impede a exclusão de páginas usadas. Não há exclusão em cascata nem Undo/Redo.");},
                Dialog::Confirm(_)=>{ui.heading("Alterações não salvas");ui.label("Há alterações não salvas nesta sessão. Continuar e descartá-las?");ui.weak("Cancele e use Salvar (Ctrl+S) para gravar o projeto .cadcraft.");},
                Dialog::OverwriteSnapshot{path,..}=>{ui.heading("Substituir snapshot existente");ui.label(path.as_str());ui.label("O arquivo já existe. Confirmar a substituição pelo snapshot técnico desta sessão?");},
                Dialog::LockConflict{path,owner,take_over}=>{ui.heading("Projeto em uso");ui.label(path.display().to_string());ui.label(format!("O arquivo está aberto por {owner}."));ui.label("Confirmar abre uma cópia somente leitura; use Salvar como para gravar alterações.");ui.checkbox(take_over,"Ignorar a trava e abrir para edição (somente se a outra sessão foi encerrada)");},
                Dialog::EditFreeText { text, .. } => {
                    ui.heading("Editar texto");
                    let edit = ui.text_edit_singleline(text);
                    if !edit.has_focus() && !edit.lost_focus() { edit.request_focus(); }
                },
                Dialog::Recover(entry)=>{ui.heading("Recuperar projeto não salvo");ui.label(format!("“{}” tinha alterações não salvas quando o CAD-CRAFT foi encerrado.",entry.name));if let Some(origin)=&entry.origin{ui.label(format!("Arquivo de origem: {}",origin.display()));}ui.weak("Confirmar restaura a cópia de recuperação. Cancelar mantém a cópia disponível em Arquivo → Abrir.");},
                Dialog::ExternalFrame { path, imported, page, dxf_unit_mm, dxf_sheet, preview_dirty } => {
                    let frame = &imported.package.frame;
                    ui.heading("Importar de outro sistema");
                    ui.label(path.display().to_string());
                    if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("dxf")) {
                        let mut custom_unit = dxf_unit_mm.is_some();
                        if ui.checkbox(&mut custom_unit, "Definir mm por unidade (em vez do cabeçalho)").changed() {
                            *dxf_unit_mm = custom_unit.then_some(1.0);
                            *preview_dirty = true;
                        }
                        if let Some(unit) = dxf_unit_mm {
                            *preview_dirty |= ui.add(egui::DragValue::new(unit).range(0.000001..=1_000_000.0).speed(0.1)).changed();
                        }
                        let mut custom_sheet = dxf_sheet.is_some();
                        if ui.checkbox(&mut custom_sheet, "Definir tamanho da folha").changed() {
                            *dxf_sheet = custom_sheet.then_some(PaperFormat::A3.size(true));
                            *preview_dirty = true;
                        }
                        if let Some(sheet) = dxf_sheet {
                            let mut width = sheet.width(); let mut height = sheet.height();
                            let changed = ui.horizontal(|ui| { ui.label("Largura/altura mm"); let a = ui.add(egui::DragValue::new(&mut width).range(1.0..=10_000.0)).changed(); let b = ui.add(egui::DragValue::new(&mut height).range(1.0..=10_000.0)).changed(); a || b }).inner;
                            *preview_dirty |= changed;
                            if let Ok(size) = Size2D::new(width, height) { *sheet = size; }
                        }
                    }
                    ui.label(format!("{} · {:.1} × {:.1} mm · {} elementos", frame.name().as_str(), frame.sheet().width(), frame.sheet().height(), frame.elements().len()));
                    if let Some(project) = self.session.project() {
                        egui::ComboBox::from_id_salt("external_frame_page")
                            .selected_text(page.and_then(|id| project.page(id)).map(|p| p.name().as_str()).unwrap_or("Somente biblioteca"))
                            .show_ui(ui, |ui| {
                                *preview_dirty |= ui.selectable_value(page, None, "Somente biblioteca").changed();
                                for candidate in project.pages() {
                                    *preview_dirty |= ui.selectable_value(page, Some(candidate.id()), candidate.name().as_str()).changed();
                                }
                            });
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(420.0, 220.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 0.0, egui::Color32::WHITE);
                        let mut preview_project = Project::new("Prévia").expect("nome fixo");
                        for asset in &imported.package.assets {
                            let _ = preview_project.edit_frame(FrameEdit::AddAsset(asset.clone()));
                        }
                        if let Ok(bounds) = Rect2D::new(Point2D::new(0.0, 0.0).expect("origem"), frame.sheet()) {
                            let mut camera = crate::camera::Camera::default();
                            camera.fit(bounds, rect.shrink(8.0));
                            for element in frame.elements() {
                                crate::frame_view::paint_element(ui.painter(), &preview_project, None, element, camera, rect, None);
                            }
                            crate::frame_view::paint_zones(ui.painter(), frame, camera, rect);
                        }
                    }
                    if ui.button("Atualizar prévia").clicked() {
                        let page_size = page.and_then(|id| self.session.project()?.page(id)).map(Page::size);
                        match crate::frame_editor::read_external_frame(path, page_size, *dxf_unit_mm, *dxf_sheet) {
                            Ok(updated) => { **imported = updated; *preview_dirty = false; },
                            Err(error) => { ui.colored_label(egui::Color32::RED, error.to_string()); },
                        };
                    }
                    if *preview_dirty { ui.weak("Atualize a prévia para confirmar estas opções."); }
                    ui.label(format!("{} aviso(s) de conversão", imported.report.notes.iter().map(|(_, n)| n).sum::<usize>()));
                    egui::ScrollArea::vertical().max_height(130.0).show(ui, |ui| {
                        for line in imported.report.lines() { ui.label(line); }
                    });
                },
            }
            let can_confirm = !matches!(&dialog, Dialog::ExternalFrame { preview_dirty: true, .. });
            ui.add_space(8.0);ui.horizontal(|ui|{accept=ui.add_enabled(can_confirm, egui::Button::new("Confirmar")).clicked();cancel=ui.button(crate::strings::CANCEL).clicked();});
            if can_confirm && ui.input(|i|i.key_pressed(egui::Key::Enter)){accept=true;}
        });
        if cancel {
            return;
        }
        if !accept {
            self.dialogs_state.dialog = Some(dialog);
            return;
        }
        let result: Result<(), String> = match &dialog {
            Dialog::NewProject(name) => {
                if let Err(e) = Text::new(name.clone()) {
                    Err(e.to_string())
                } else {
                    self.request(PendingAction::New(name.clone()), ctx);
                    Ok(())
                }
            }
            Dialog::NewDocument(name) => self
                .session
                .new_document(name)
                .map(|_| ())
                .map_err(|e| e.to_string()),
            Dialog::NewPage {
                document,
                name,
                number,
                kind,
                width,
                height,
            } => Size2D::new(*width, *height)
                .and_then(|size| {
                    self.session
                        .new_page(*document, name, number, kind.clone(), size)
                })
                .map(|id| self.open_page(id))
                .map_err(|e| e.to_string()),
            Dialog::Rename { id, number, value } => self
                .session
                .apply(if *number {
                    Edit::PageNumber(*id, value.clone())
                } else {
                    Edit::PageName(*id, value.clone())
                })
                .map_err(|e| e.to_string()),
            Dialog::DeletePage(id) => self.session.delete_page(*id).map_err(|e| e.to_string()),
            Dialog::OverwriteSnapshot { path, json } => self.write_snapshot(path, json, true),
            Dialog::LockConflict {
                path, take_over, ..
            } => {
                let mode = if *take_over {
                    crate::app::LockMode::TakeOver
                } else {
                    crate::app::LockMode::ReadOnly
                };
                self.open_project_file(path, mode).map(|_| ())
            }
            Dialog::EditFreeText { id, text } => self.edit_free_text(*id, text).map(|_| ()),
            Dialog::Recover(entry) => {
                self.request(PendingAction::Recover(entry.clone()), ctx);
                Ok(())
            }
            Dialog::ExternalFrame { imported, page, .. } => {
                let result = self
                    .session
                    .project()
                    .ok_or_else(|| "Projeto não aberto".to_owned())
                    .and_then(|project| {
                        let (mut edits, id) = project
                            .frame_import_edits(&imported.package)
                            .map_err(|e| e.to_string())?;
                        if let Some(page) = page {
                            edits.push(FrameEdit::SetPageFrame {
                                page: *page,
                                frame: Some(id),
                            });
                        }
                        Ok((edits, id))
                    });
                match result {
                    Ok((edits, id)) => {
                        self.graphic_edit(GraphicEdit::Frame(FrameEdit::Batch(edits)));
                        if self
                            .session
                            .project()
                            .is_some_and(|p| p.frame(id).is_some())
                        {
                            self.open_frame_editor(id);
                            Ok(())
                        } else {
                            Err("A importação foi recusada pelo projeto".to_owned())
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            Dialog::Confirm(_) => Ok(()),
        };
        if let Err(error) = result {
            self.log(Level::Error, error);
            self.dialogs_state.dialog = Some(dialog);
        } else if let Dialog::Confirm(action) = dialog {
            self.perform(action, ctx);
        } else {
            self.log(Level::Info, "Operação concluída.");
        }
    }
}
