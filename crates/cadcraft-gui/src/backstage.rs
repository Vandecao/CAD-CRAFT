// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! File menu and project entry screen; start with the backstage renderer.
use crate::{
    app::{CadApp, Level},
    commands::{CommandId, Icon},
    dialogs::{Dialog, PendingAction},
    icons, theme,
};
use eframe::egui::{self, Context, RichText};
use std::io::Read;
impl CadApp {
    pub(crate) fn show_backstage(&mut self, ctx: &Context) {
        self.last_viewport = None;
        egui::SidePanel::left("backstage_navigation")
            .resizable(false)
            .exact_width(190.0)
            .frame(egui::Frame::new().fill(theme::TITLE).inner_margin(8))
            .show(ctx, |ui| {
                if icons::button(
                    ui,
                    Icon::ArrowLeft,
                    "Voltar",
                    "Retornar à área de trabalho — Esc",
                    true,
                    false,
                    false,
                )
                .clicked()
                {
                    self.dialogs_state.backstage = None;
                }
                ui.add_space(12.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (label, available) in [
                        ("Informações", true),
                        ("Novo", true),
                        ("Abrir", true),
                        ("Salvar", self.session.project().is_some()),
                        ("Salvar como", self.session.project().is_some()),
                        ("Fazer backup", false),
                        ("Copiar projeto", false),
                        ("Excluir projeto", false),
                        ("Fechar projeto", self.session.project().is_some()),
                        ("Gerenciamento de projetos", false),
                        ("Controle de revisão", false),
                        ("Imprimir", self.session.project().is_some()),
                        ("Exportar PDF", self.session.project().is_some()),
                        ("Exportar DXF", self.session.project().is_some()),
                        ("Exportar DWG", self.session.project().is_some()),
                        ("Exportar lista", self.session.project().is_some()),
                        ("Importar", true),
                        ("Exportar", self.session.project().is_some()),
                        ("Extras", true),
                        ("Configurações", true),
                        ("Ajuda", true),
                        ("Sair", true),
                    ] {
                        let response = ui.add_enabled(
                            available,
                            egui::Button::new(label)
                                .selected(self.dialogs_state.backstage == Some(label))
                                .min_size(egui::vec2(170.0, 31.0)),
                        );
                        if response
                            .on_disabled_hover_text(
                                "Recurso planejado ou contexto indisponível nesta etapa",
                            )
                            .clicked()
                        {
                            match label {
                                "Novo" => self.execute(CommandId::NewProject, ctx),
                                "Salvar" => self.execute(CommandId::SaveProject, ctx),
                                "Salvar como" => self.execute(CommandId::SaveProjectAs, ctx),
                                "Fechar projeto" => self.request(PendingAction::Close, ctx),
                                "Sair" => self.request(PendingAction::Exit, ctx),
                                "Exportar" => self.execute(CommandId::ExportSnapshot, ctx),
                                "Imprimir" => self.execute(CommandId::Print, ctx),
                                "Exportar PDF" => self.execute(CommandId::ExportPdf, ctx),
                                "Exportar DXF" => self.execute(CommandId::ExportDxf, ctx),
                                "Exportar DWG" => self.execute(CommandId::ExportDwg, ctx),
                                "Exportar lista" => self.execute(CommandId::ExportList, ctx),
                                _ => self.dialogs_state.backstage = Some(label),
                            }
                        }
                    }
                });
            });
        egui::CentralPanel::default().frame(egui::Frame::new().fill(theme::PANEL).inner_margin(20)).show(ctx,|ui|{
            let page=self.dialogs_state.backstage.unwrap_or("Informações");let session_kind=if self.session.demonstration{"Demonstração".to_owned()}else{match &self.session.file{Some(f)=>format!("Arquivo .cadcraft ({}){}",f.file_name(),if f.read_only{", somente leitura"}else{""}),None=>"Projeto em memória (ainda não salvo)".to_owned()}};ui.label(RichText::new(page).size(30.0));ui.add_space(18.0);
            egui::ScrollArea::vertical().auto_shrink([false,false]).show(ui,|ui|{
                match page {
                    "Informações"=>{
                        ui.horizontal(|ui|{for section in ["Propriedades","Estatística","Estrutura","Status"]{ui.selectable_value(&mut self.dialogs_state.backstage_section,section,section);}});ui.separator();
                        if let Some(p)=self.session.project(){
                            match self.dialogs_state.backstage_section {
                                "Estatística"=>{egui::Grid::new("project_stats").num_columns(2).striped(true).show(ui,|ui|{for(label,count)in[("Documentos",p.documents().count()),("Páginas",p.pages().count()),("Representações",p.representations().count()),("Componentes",p.components().count()),("Dispositivos",p.devices().count()),("Conexões lógicas",p.connections().count()),("Condutores físicos",p.conductors().count())]{ui.label(label);ui.label(count.to_string());ui.end_row();}});},
                                "Estrutura"=>{for doc in p.documents(){egui::CollapsingHeader::new(doc.name().as_str()).default_open(true).show(ui,|ui|{for page in p.ordered_pages(doc.id()).unwrap_or_default(){ui.label(format!("{} · {}  ={}  +{}",page.number().as_str(),page.name().as_str(),crate::navigation::path_text(page.structure().functional_path()),crate::navigation::path_text(page.structure().location_path())));}});}},
                                "Status"=>{ui.label(if self.session.dirty{"Há alterações não salvas."}else{"Sem alterações desde a abertura ou o último salvamento."});ui.label(if self.session.demonstration{"Projeto demonstrativo próprio."}else{"Sessão documental."});ui.label(match &self.session.file{Some(f)=>format!("Arquivo: {}{}",f.path.display(),if f.read_only{" (somente leitura)"}else{""}),None=>"Sem arquivo .cadcraft associado.".into()});ui.weak("Cópias de recuperação são automáticas enquanto houver alterações não salvas; não substituem backup externo.");},
                                _=>{egui::Grid::new("project_information").num_columns(2).striped(true).spacing([25.0,10.0]).show(ui,|ui|{for(label,value)in[("Nome",p.name().as_str()),("Descrição",p.description()),("Identificação",p.identification().map(|s|s.as_str()).unwrap_or("—")),("Tipo de sessão",session_kind.as_str())]{ui.label(label);ui.label(value);ui.end_row();}for(k,v)in p.metadata(){ui.label(k.as_str());ui.label(v);ui.end_row();}});
                                    if ui.button("Editar propriedades no inspetor").clicked(){self.session.select(crate::state::Selection::Project);self.preferences.panel_mut(crate::preferences::PanelId::Properties).visible=true;self.preferences.panel_mut(crate::preferences::PanelId::Properties).collapsed=false;self.dialogs_state.backstage=None;}
                                }
                            }
                        }else{ui.label("Nenhum projeto aberto.");if ui.button("Abrir demonstração CAD-CRAFT").clicked(){self.execute(CommandId::Demo,ctx);}}
                    },
                    "Abrir"=>self.open_ui(ui,ctx),
                    "Importar"=>{egui::ScrollArea::vertical().id_salt("importar").show(ui,|ui|{self.qet_import_ui(ui);ui.separator();self.snapshot_ui(ui,true);});},
                    "Exportar"=>self.snapshot_ui(ui,false),
                    "Configurações"=>{
                        ui.heading("Preferências visuais");ui.weak("Salvas localmente pelo aplicativo. Independentes dos dados elétricos.");
                        ui.checkbox(&mut self.preferences.ribbon_collapsed,"Ribbon recolhida");ui.checkbox(&mut self.preferences.grid_visible,"Exibir grade");ui.checkbox(&mut self.preferences.snap_enabled,"Snap à grade");ui.horizontal(|ui|{ui.label("Pasta da biblioteca de molduras");ui.add(egui::TextEdit::singleline(&mut self.preferences.frame_library).hint_text("Ex.: C:/Molduras").desired_width(320.0));});ui.horizontal(|ui|{ui.label("Pasta da biblioteca de símbolos");ui.add(egui::TextEdit::singleline(&mut self.preferences.symbol_library).hint_text("Ex.: C:/Simbolos").desired_width(320.0));});ui.horizontal(|ui|{ui.label("Escala de importação .elmt (mm por px)");ui.add(egui::DragValue::new(&mut self.preferences.elmt_mm_per_px).range(0.01..=10.0).speed(0.01));});
                        ui.horizontal(|ui|{ui.label("Pasta da biblioteca de peças");ui.add(egui::TextEdit::singleline(&mut self.preferences.part_library).hint_text("Ex.: C:/Pecas").desired_width(320.0));});
                        ui.horizontal(|ui|{ui.label("Pasta da biblioteca de macros");ui.add(egui::TextEdit::singleline(&mut self.preferences.macro_library).hint_text("Ex.: C:/Macros").desired_width(320.0));});
                        ui.horizontal(|ui|{ui.label("Conversor DWG (ODA File Converter)");ui.add(egui::TextEdit::singleline(&mut self.preferences.dwg_converter).hint_text("Ex.: C:/Program Files/ODA/ODAFileConverter.exe").desired_width(320.0));});
                        if let Some(project) = self.session.project() {
                            let mut scheme = project.numbering_scheme().clone();
                            ui.separator(); ui.heading("Numeração do projeto");
                            ui.checkbox(&mut scheme.use_on_insert, "Usar a regra ao inserir, colar e inserir macro");
                            ui.horizontal(|ui| {
                                ui.label("Dispositivos");
                                ui.selectable_value(&mut scheme.device_format, cad_craft::domain::DeviceNumberFormat::Sequential, "Sequencial por prefixo");
                                ui.selectable_value(&mut scheme.device_format, cad_craft::domain::DeviceNumberFormat::PageColumn, "Página/coluna");
                            });
                            ui.horizontal(|ui| { ui.label("Início"); ui.add(egui::DragValue::new(&mut scheme.start).range(1..=u32::MAX)); ui.label("Passo"); ui.add(egui::DragValue::new(&mut scheme.step).range(1..=u32::MAX)); });
                            if &scheme != project.numbering_scheme() { let _ = self.session.set_numbering_scheme(scheme); }
                        }
                        self.check_settings_ui(ui);
                        ui.horizontal(|ui|{ui.label("Espaçamento (mm)");ui.add(egui::DragValue::new(&mut self.preferences.grid_mm).range(0.1..=1000.0).speed(0.1));});
                        for id in crate::preferences::PanelId::ALL {ui.checkbox(&mut self.preferences.panel_mut(id).visible,id.label());}
                        if ui.button("Restaurar layout padrão").clicked(){self.preferences.restore_layout();self.chrome.layout_generation+=1;}
                    },
                    "Extras"=>{ui.label("Operações disponíveis");for id in [CommandId::Validate,CommandId::RestoreLayout,CommandId::Demo]{if self.command_button(ui,id,false,false){self.execute(id,ctx);}}},
                    "Ajuda"=>{
                        ui.heading("Navegação CAD-CRAFT");
                        for line in ["Clique seleciona; duplo clique ou Enter no navegador abre uma página.","Setas navegam, expandem e recolhem a árvore.","Roda sobre o desenho: zoom no cursor. Arraste central: pan.","Ctrl+PageUp / Ctrl+PageDown: página anterior / seguinte, fora de campos de texto.","Ctrl+F1: recolher Ribbon. F11: tela cheia. Esc: cancelar ou voltar.","Fechar uma aba mantém a página no projeto.","As representações mostram limites e âncoras, não símbolos elétricos.","Use os cabeçalhos e o menu Painéis para restaurar regiões ocultas.","Documentação: docs/internal/reports/ETAPA_05_INTERFACE_GRAFICA_EGUI.md"]{ui.label(line);}
                    },_=>{ui.weak("Recurso planejado para etapa futura.");}
                }
            });
        });
    }
    /// Etapa 12: procurar arquivo, recentes e cópias de recuperação.
    fn open_ui(&mut self, ui: &mut egui::Ui, ctx: &Context) {
        ui.heading("Abrir projeto");
        ui.weak("Projetos .cadcraft e snapshots técnicos JSON (formato v0, migrados ao abrir).");
        if ui.button("Procurar arquivo…").clicked() {
            self.open_project_dialog(ctx);
        }
        ui.add_space(12.0);
        ui.strong("Recentes");
        let mut open = None;
        if self.files.recent.is_empty() {
            ui.weak("Nenhum projeto recente.");
        }
        for path in &self.files.recent {
            let label = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string());
            if ui
                .add_enabled(path.is_file(), egui::Button::new(label))
                .on_hover_text(path.display().to_string())
                .on_disabled_hover_text("Arquivo não encontrado")
                .clicked()
            {
                open = Some(path.clone());
            }
        }
        if let Some(path) = open {
            self.request(PendingAction::Open(path), ctx);
        }
        let entries = self.recovery_entries();
        if !entries.is_empty() {
            ui.add_space(12.0);
            ui.strong("Recuperação");
            let mut action = None;
            for entry in entries {
                ui.horizontal(|ui| {
                    ui.label(&entry.name);
                    if ui.button("Restaurar").clicked() {
                        action = Some((entry.clone(), true));
                    }
                    if ui.button("Descartar").clicked() {
                        action = Some((entry.clone(), false));
                    }
                });
            }
            match action {
                Some((entry, true)) => self.request(PendingAction::Recover(entry), ctx),
                Some((entry, false)) => {
                    if let Some(recovery) = &self.files.recovery {
                        recovery.discard(&entry.project_id);
                    }
                }
                None => {}
            }
        }
    }
    fn snapshot_ui(&mut self, ui: &mut egui::Ui, import: bool) {
        ui.heading(if import {
            "Importar snapshot técnico JSON"
        } else {
            "Exportar snapshot técnico JSON"
        });
        ui.weak("Serialização estrutural experimental do Core. Sem formato .cadcraft, migrações, histórico ou backup. Limite de importação: 8 MiB.");
        ui.label("Caminho do arquivo JSON");
        ui.add(
            egui::TextEdit::singleline(&mut self.dialogs_state.snapshot_path)
                .desired_width(f32::INFINITY),
        );
        ui.horizontal(|ui| {
            if import && ui.button("Ler arquivo").clicked() {
                match read_snapshot(&self.dialogs_state.snapshot_path) {
                    Ok(json) => self.dialogs_state.snapshot_text = json,
                    Err(e) => self.log(Level::Error, e),
                }
            }
            if !import && ui.button("Gravar snapshot técnico").clicked() {
                match self.session.snapshot() {
                    Ok(json) => {
                        let path = self.dialogs_state.snapshot_path.clone();
                        if std::path::Path::new(&path).exists() {
                            self.dialogs_state.dialog =
                                Some(Dialog::OverwriteSnapshot { path, json });
                        } else if let Err(e) = self.write_snapshot(&path, &json, false) {
                            self.log(Level::Error, e);
                        } else {
                            self.log(
                                Level::Info,
                                "Snapshot técnico gravado. Não substitui backup externo.",
                            );
                        }
                    }
                    Err(e) => self.log(Level::Error, e),
                }
            }
            if !import && ui.button("Copiar JSON").clicked() {
                ui.ctx().copy_text(self.dialogs_state.snapshot_text.clone());
            }
        });
        ui.label(if import {
            "Ou cole um snapshot técnico abaixo:"
        } else {
            "Prévia do snapshot:"
        });
        egui::ScrollArea::both().max_height(330.0).show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.dialogs_state.snapshot_text)
                    .code_editor()
                    .desired_width(f32::INFINITY)
                    .desired_rows(16)
                    .interactive(import)
                    .char_limit(8 * 1024 * 1024 + 1),
            );
        });
        if import && ui.button("Importar como sessão").clicked() {
            self.request(
                PendingAction::Import(self.dialogs_state.snapshot_text.clone()),
                ui.ctx(),
            );
        }
        if let Some(message) = self.panels_state.messages.back() {
            ui.label(&message.text);
        }
    }
    pub(crate) fn write_snapshot(
        &mut self,
        path: &str,
        json: &str,
        overwrite: bool,
    ) -> Result<(), String> {
        if json.len() > 8 * 1024 * 1024 {
            return Err("Snapshot excede o limite de intercâmbio de 8 MiB.".into());
        }
        if !path.to_lowercase().ends_with(".json") {
            return Err("Escolha um caminho com extensão .json para o snapshot técnico.".into());
        }
        crate::snapshots::write_from(std::path::Path::new(path), &mut json.as_bytes(), overwrite)
            .map_err(|e| e.to_string())?;
        self.session.dirty = false;
        Ok(())
    }
}
fn read_snapshot(path: &str) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > 8 * 1024 * 1024 {
        return Err("Snapshot técnico excede 8 MiB.".into());
    }
    let mut text = String::new();
    file.take(8 * 1024 * 1024 + 1)
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    if text.len() > 8 * 1024 * 1024 {
        return Err("Snapshot técnico excede 8 MiB.".into());
    }
    Ok(text)
}
