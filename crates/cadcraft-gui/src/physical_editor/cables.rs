// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! cables support for the physical editor; start at the parent module.

use super::*;

impl PhysicalEditor {
    pub(in crate::physical_editor) fn cables_ui(&mut self, session: &mut Session, ui: &mut Ui) {
        let Some(project_id) = session.project().map(Project::id) else {
            return;
        };
        ui.horizontal(|ui| {
            ui.label("Novo cabo");
            ui.text_edit_singleline(&mut self.new_cable_name);
            ui.label("Veias nominais");
            ui.add(egui::TextEdit::singleline(&mut self.new_cable_count).desired_width(65.));
            if ui.button("Criar cabo").clicked() {
                let edit = (|| {
                    let name = DesignationPart::new(&self.new_cable_name)?;
                    let count = self
                        .new_cable_count
                        .parse::<u32>()
                        .map_err(|_| DomainError::InvalidNumber("quantidade de veias"))?;
                    Ok(PhysicalEdit::UpsertCable(Cable::new(
                        project_id,
                        name,
                        CableKind::Unspecified,
                        count,
                    )?))
                })();
                self.apply(session, edit);
            }
        });
        ui.horizontal(|ui| {
            ComboBox::from_id_salt("cable_choice")
                .selected_text(
                    self.selected_cable
                        .and_then(|id| self.cache.cables.iter().find(|c| c.id() == id))
                        .map(|c| c.designation().as_str())
                        .unwrap_or("Selecione cabo"),
                )
                .show_ui(ui, |ui| {
                    for cable in &self.cache.cables {
                        if ui
                            .selectable_value(
                                &mut self.selected_cable,
                                Some(cable.id()),
                                cable.designation().as_str(),
                            )
                            .clicked()
                        {
                            self.cable_name = cable.designation().as_str().into();
                            self.cable_description = cable
                                .description()
                                .map(|v| v.as_str().to_owned())
                                .unwrap_or_default();
                            self.cable_manufacturer = cable
                                .manufacturer()
                                .map(|v| v.as_str().to_owned())
                                .unwrap_or_default();
                            self.cable_catalog = cable
                                .catalog_code()
                                .map(|v| v.as_str().to_owned())
                                .unwrap_or_default();
                            self.cable_count = cable.nominal_core_count().to_string();
                            self.cable_kind_index = match cable.kind() {
                                CableKind::Unspecified => 0,
                                CableKind::Power => 1,
                                CableKind::Control => 2,
                                CableKind::Instrumentation => 3,
                                CableKind::Data => 4,
                            };
                            self.selected_core = None;
                        }
                    }
                });
        });
        let selected = self
            .selected_cable
            .and_then(|id| self.cache.cables.iter().find(|c| c.id() == id))
            .cloned();
        let Some(cable) = selected else {
            ui.weak("Selecione um cabo para editar as propriedades e veias.");
            return;
        };
        egui::Grid::new("cable_fields")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Identificação");
                ui.text_edit_singleline(&mut self.cable_name);
                ui.end_row();
                ui.label("Descrição");
                ui.text_edit_singleline(&mut self.cable_description);
                ui.end_row();
                ui.label("Fabricante");
                ui.text_edit_singleline(&mut self.cable_manufacturer);
                ui.end_row();
                ui.label("Código de produto");
                ui.text_edit_singleline(&mut self.cable_catalog);
                ui.end_row();
                ui.label("Quantidade de veias");
                ui.text_edit_singleline(&mut self.cable_count);
                ui.end_row();
            });
        ComboBox::from_id_salt("cable_kind")
            .selected_text(
                [
                    "Não especificado",
                    "Potência",
                    "Controle",
                    "Instrumentação",
                    "Dados",
                ][self.cable_kind_index.min(4)],
            )
            .show_ui(ui, |ui| {
                for (index, label) in [
                    "Não especificado",
                    "Potência",
                    "Controle",
                    "Instrumentação",
                    "Dados",
                ]
                .iter()
                .enumerate()
                {
                    ui.selectable_value(&mut self.cable_kind_index, index, *label);
                }
            });
        ui.horizontal(|ui| {
            if ui.button("Salvar cabo").clicked() {
                let edit = (|| {
                    let count = self
                        .cable_count
                        .parse::<u32>()
                        .map_err(|_| DomainError::InvalidNumber("quantidade de veias"))?;
                    let kind = [
                        CableKind::Unspecified,
                        CableKind::Power,
                        CableKind::Control,
                        CableKind::Instrumentation,
                        CableKind::Data,
                    ][self.cable_kind_index.min(4)];
                    let updated = cable
                        .clone()
                        .with_designation(DesignationPart::new(&self.cable_name)?)
                        .with_description_option(optional_text(&self.cable_description)?)
                        .with_manufacturer_option(optional_text(&self.cable_manufacturer)?)
                        .with_catalog_code_option(optional_text(&self.cable_catalog)?)
                        .with_kind(kind)
                        .with_nominal_core_count(count)?;
                    Ok(PhysicalEdit::UpsertCable(updated))
                })();
                self.apply(session, edit);
            }
            if ui.button("Excluir cabo").clicked() {
                self.apply(session, Ok(PhysicalEdit::DeleteCable(cable.id())));
            }
        });
        ui.separator();
        ui.strong("Veias cadastradas");
        let cores = self
            .cache
            .cores
            .iter()
            .filter(|c| c.cable_id() == cable.id())
            .cloned()
            .collect::<Vec<_>>();
        ScrollArea::vertical().max_height(180.).show(ui, |ui| {
            for core in &cores {
                let status = core
                    .conductor_id()
                    .map(|id| format!("usada · {id}"))
                    .unwrap_or_else(|| "livre".into());
                if ui
                    .selectable_label(
                        self.selected_core == Some(core.id()),
                        format!("{} · {status}", core.identification().as_str()),
                    )
                    .clicked()
                {
                    self.selected_core = Some(core.id());
                    self.core_name = core.identification().as_str().into();
                    self.core_marking = core
                        .marking()
                        .identification()
                        .map(|v| v.as_str().to_owned())
                        .unwrap_or_default();
                    self.core_section = core
                        .cross_section()
                        .map(|v| v.get().to_string())
                        .unwrap_or_default();
                    self.selected_wire = core.conductor_id();
                    session.select(Selection::Electrical(RepresentationTarget::CableCore(
                        core.id(),
                    )));
                }
            }
        });
        if let Some(core) = self
            .selected_core
            .and_then(|id| cores.iter().find(|c| c.id() == id))
            .cloned()
        {
            ui.horizontal(|ui| {
                ui.label("Designação");
                ui.text_edit_singleline(&mut self.core_name);
                ui.label("Marcação");
                ui.text_edit_singleline(&mut self.core_marking);
                ui.label("Seção mm²");
                ui.add(egui::TextEdit::singleline(&mut self.core_section).desired_width(65.));
            });
            ui.horizontal(|ui| {
                if ui.button("Salvar veia").clicked() {
                    let edit = (|| {
                        let name = DesignationPart::new(&self.core_name)?;
                        let marking = if self.core_marking.trim().is_empty() {
                            core.marking().clone()
                        } else {
                            ConductorMarking::new(
                                core.marking().color_value(),
                                Some(DesignationPart::new(&self.core_marking)?),
                            )?
                        };
                        let area = optional_section(&self.core_section)?;
                        Ok(PhysicalEdit::UpsertCore(
                            core.clone()
                                .with_identification(name)
                                .with_marking(marking)
                                .with_cross_section_option(area),
                        ))
                    })();
                    self.apply(session, edit);
                }
                if ui.button("Excluir veia").clicked() {
                    self.apply(session, Ok(PhysicalEdit::DeleteCore(core.id())));
                }
            });
            ui.horizontal(|ui| {
                ComboBox::from_id_salt("core_wire")
                    .selected_text(
                        self.selected_wire
                            .and_then(|id| self.cache.wires.iter().find(|w| w.id() == id))
                            .map(|w| w.designation().as_str())
                            .unwrap_or("Condutor físico"),
                    )
                    .show_ui(ui, |ui| {
                        for wire in &self.cache.wires {
                            ui.selectable_value(
                                &mut self.selected_wire,
                                Some(wire.id()),
                                wire.designation().as_str(),
                            );
                        }
                    });
                if ui.button("Atribuir veia").clicked() {
                    self.apply(
                        session,
                        self.selected_wire
                            .map(|id| PhysicalEdit::SetCoreConductor {
                                core: core.id(),
                                conductor: Some(id),
                            })
                            .ok_or(DomainError::InvalidRelation("selecione condutor")),
                    );
                }
                if ui.button("Liberar veia").clicked() {
                    self.apply(
                        session,
                        Ok(PhysicalEdit::SetCoreConductor {
                            core: core.id(),
                            conductor: None,
                        }),
                    );
                }
            });
        } else {
            ui.horizontal(|ui| {
                ui.label("Nova veia");
                ui.text_edit_singleline(&mut self.core_name);
                ui.label("Seção mm² (opcional)");
                ui.add(egui::TextEdit::singleline(&mut self.core_section).desired_width(65.));
                if ui.button("Adicionar veia").clicked() {
                    let edit = (|| {
                        let name = DesignationPart::new(&self.core_name)?;
                        let mut core = CableCore::new(
                            project_id,
                            cable.id(),
                            name.clone(),
                            ConductorMarking::identified(name),
                        );
                        if let Some(area) = optional_section(&self.core_section)? {
                            core = core.with_cross_section(area);
                        }
                        Ok(PhysicalEdit::UpsertCore(core))
                    })();
                    self.apply(session, edit);
                }
            });
        }
    }
}
