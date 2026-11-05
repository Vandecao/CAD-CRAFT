// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 20: diálogo único de numeração (dispositivos, conexões, páginas).
//! Regra e filtro → prévia antes/depois → Aplicar em uma transação.
use crate::{
    commands::CommandId,
    state::{Selection, Session},
};
use cad_craft::domain::*;
use eframe::egui::{self, Context};
use std::collections::BTreeSet;

/// Alcance escolhido no diálogo de dispositivos.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Filter {
    #[default]
    Project,
    Document,
    Page,
    Selection,
}

#[derive(Default)]
pub struct NumberingWindow {
    mode: Option<CommandId>,
    pub message: String,
    prefix: String,
    start: u32,
    filter: Filter,
}

enum Preview {
    Devices(DeviceNumberPreview),
    Connections(ConnectionNumberPreview),
    Pages(PageNumberPreview),
}

impl Preview {
    fn has_conflict(&self) -> bool {
        match self {
            Self::Devices(p) => p.rows.iter().any(|r| r.conflict.is_some()),
            Self::Connections(p) => p.rows.iter().any(|r| r.conflict.is_some()),
            Self::Pages(_) => false,
        }
    }
}

/// Dispositivos da seleção gráfica ou do item elétrico selecionado.
fn selected_devices(session: &Session, project: &Project) -> Vec<DeviceId> {
    let mut out = BTreeSet::new();
    for id in session.graphics.selection.ids() {
        if let Some(device) = project
            .representation(*id)
            .and_then(|r| project.device_for_target(r.target()))
        {
            out.insert(device);
        }
    }
    if let Selection::Electrical(target) = &session.selection
        && let Some(device) = project.device_for_target(*target)
    {
        out.insert(device);
    }
    out.into_iter().collect()
}

fn order_controls(ui: &mut egui::Ui, scheme: &mut NumberingScheme) {
    ui.horizontal(|ui| {
        ui.label("Ordem");
        ui.selectable_value(
            &mut scheme.order,
            NumberingOrder::PageColumnRow,
            "Página → coluna → linha",
        );
        ui.selectable_value(
            &mut scheme.order,
            NumberingOrder::PositionXY,
            "Linha a linha",
        );
    });
    ui.horizontal(|ui| {
        ui.label("Início");
        ui.add(egui::DragValue::new(&mut scheme.start).range(1..=u32::MAX));
        ui.label("Passo");
        ui.add(egui::DragValue::new(&mut scheme.step).range(1..=u32::MAX));
    });
}

impl NumberingWindow {
    pub fn open(&mut self, mode: CommandId) {
        self.mode = Some(mode);
        self.message.clear();
        if self.start == 0 {
            self.start = 1;
        }
    }

    pub fn show(&mut self, session: &mut Session, ctx: &Context) {
        let Some(mode) = self.mode else {
            return;
        };
        let mut open = true;
        let title = match mode {
            CommandId::NumberDevices => "Numerar dispositivos",
            CommandId::NumberConnections => "Numerar conexões",
            _ => "Numerar páginas",
        };
        let mut scheme_change = None;
        let mut apply = None;
        egui::Window::new(title)
            .id(egui::Id::new("numbering_window"))
            .open(&mut open)
            .default_width(560.0)
            .show(ctx, |ui| {
                let Some(project) = session.project() else {
                    ui.weak("Nenhum projeto aberto.");
                    return;
                };
                let page = session.active.filter(|id| project.page(*id).is_some());
                let document = page
                    .and_then(|id| project.page(id))
                    .map(Page::document_id)
                    .or(match session.selection {
                        Selection::Document(id) => Some(id),
                        _ => None,
                    })
                    .or_else(|| project.documents().next().map(Document::id));
                let mut scheme = project.numbering_scheme().clone();
                match mode {
                    CommandId::NumberDevices => {
                        ui.horizontal(|ui| {
                            ui.label("Formato");
                            ui.selectable_value(
                                &mut scheme.device_format,
                                DeviceNumberFormat::Sequential,
                                "Sequencial por prefixo (-K1)",
                            );
                            ui.selectable_value(
                                &mut scheme.device_format,
                                DeviceNumberFormat::PageColumn,
                                "Página/coluna (-K2.4)",
                            );
                        });
                        order_controls(ui, &mut scheme);
                        ui.checkbox(&mut scheme.keep_locked, "Manter designações travadas");
                        ui.horizontal(|ui| {
                            ui.label("Alcance");
                            ui.selectable_value(&mut self.filter, Filter::Project, "Projeto");
                            ui.selectable_value(&mut self.filter, Filter::Document, "Documento");
                            ui.selectable_value(&mut self.filter, Filter::Page, "Página ativa");
                            ui.selectable_value(&mut self.filter, Filter::Selection, "Seleção");
                        });
                    }
                    CommandId::NumberConnections => {
                        ui.horizontal(|ui| {
                            ui.label("Formato");
                            ui.selectable_value(
                                &mut scheme.wire_format,
                                WireNumberFormat::Sequential,
                                "Sequencial",
                            );
                            ui.selectable_value(
                                &mut scheme.wire_format,
                                WireNumberFormat::PageColumn,
                                "Página/coluna",
                            );
                        });
                        order_controls(ui, &mut scheme);
                        ui.weak(
                            "Um número por rede. Potencial nomeado mantém o nome; \
                             números travados não mudam.",
                        );
                    }
                    _ => {
                        ui.horizontal(|ui| {
                            ui.label("Prefixo");
                            ui.text_edit_singleline(&mut self.prefix);
                            ui.label("Início");
                            ui.add(egui::DragValue::new(&mut self.start).range(1..=u32::MAX));
                        });
                        ui.weak("A ordem das páginas não muda; as referências /página.coluna acompanham.");
                    }
                }
                if &scheme != project.numbering_scheme() {
                    scheme_change = Some(scheme);
                }
                ui.separator();
                let filter = match self.filter {
                    Filter::Project => Ok(NumberingFilter::Project),
                    Filter::Document => document
                        .map(NumberingFilter::Document)
                        .ok_or("nenhum documento ativo"),
                    Filter::Page => page
                        .map(|id| NumberingFilter::Pages(vec![id]))
                        .ok_or("nenhuma página ativa"),
                    Filter::Selection => {
                        Ok(NumberingFilter::Selection(selected_devices(session, project)))
                    }
                };
                let preview = match mode {
                    CommandId::NumberDevices => filter
                        .map_err(|e| e.to_owned())
                        .and_then(|f| {
                            project
                                .preview_device_numbering(f)
                                .map_err(|e| e.to_string())
                        })
                        .map(Preview::Devices),
                    CommandId::NumberConnections => project
                        .preview_connection_numbering()
                        .map_err(|e| e.to_string())
                        .map(Preview::Connections),
                    _ => document
                        .ok_or_else(|| "documento ausente".to_owned())
                        .and_then(|id| {
                            project
                                .preview_page_numbering(id, &self.prefix, self.start)
                                .map_err(|e| e.to_string())
                        })
                        .map(Preview::Pages),
                };
                let preview = match preview {
                    Ok(preview) => preview,
                    Err(error) => {
                        ui.colored_label(egui::Color32::RED, error);
                        return;
                    }
                };
                ui.strong("Prévia (antes → depois)");
                egui::ScrollArea::vertical()
                    .max_height(320.0)
                    .show(ui, |ui| match &preview {
                        Preview::Devices(p) => {
                            if p.rows.is_empty() {
                                ui.weak("Nenhum dispositivo no alcance.");
                            }
                            for row in &p.rows {
                                let text = format!("{} → {}", row.before, row.after);
                                match &row.conflict {
                                    Some(c) => {
                                        ui.colored_label(
                                            egui::Color32::RED,
                                            format!("{text} — conflito: {c}"),
                                        );
                                    }
                                    None => {
                                        ui.label(text);
                                    }
                                }
                            }
                        }
                        Preview::Connections(p) => {
                            if p.rows.is_empty() {
                                ui.weak("Nenhuma conexão no projeto.");
                            }
                            // Linhas da mesma rede são consecutivas: uma por rede.
                            for group in p.rows.chunk_by(|a, b| a.after == b.after) {
                                let before: BTreeSet<_> = group
                                    .iter()
                                    .map(|r| r.before.as_ref().map_or("—", Text::as_str))
                                    .collect();
                                let after = group[0].after.as_ref().map_or("—", Text::as_str);
                                let text = format!(
                                    "Rede {after}: {} fio(s), antes {}",
                                    group.len(),
                                    before.into_iter().collect::<Vec<_>>().join(", ")
                                );
                                match &group[0].conflict {
                                    Some(c) => {
                                        ui.colored_label(
                                            egui::Color32::RED,
                                            format!("{text} — conflito: {c}"),
                                        );
                                    }
                                    None => {
                                        ui.label(text);
                                    }
                                }
                            }
                        }
                        Preview::Pages(p) => {
                            for row in &p.rows {
                                ui.label(format!("{} → {}", row.before.as_str(), row.after.as_str()));
                            }
                        }
                    });
                let conflicts = preview.has_conflict();
                if conflicts {
                    ui.colored_label(
                        egui::Color32::RED,
                        "Há conflitos: resolva-os antes de aplicar.",
                    );
                }
                if ui
                    .add_enabled(!conflicts, egui::Button::new("Aplicar numeração"))
                    .clicked()
                {
                    apply = Some(preview);
                }
                if !self.message.is_empty() {
                    ui.colored_label(egui::Color32::RED, &self.message);
                }
            });
        if let Some(scheme) = scheme_change
            && let Err(error) = session.set_numbering_scheme(scheme)
        {
            self.message = error.to_string();
        }
        if let Some(preview) = apply {
            let result = match &preview {
                Preview::Devices(p) => session.number_devices(p),
                Preview::Connections(p) => session.number_connections(p),
                Preview::Pages(p) => session.number_pages(p),
            };
            match result {
                Ok(_) => self.mode = None,
                Err(error) => self.message = error.to_string(),
            }
        }
        if !open {
            self.mode = None;
        }
    }
}
