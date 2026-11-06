// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! wiring support for the physical editor; start at the parent module.

use super::*;

impl PhysicalEditor {
    pub(in crate::physical_editor) fn termination_picker(
        ui: &mut Ui,
        salt: &str,
        terminals: &[TerminalListRow],
        value: &mut Option<(TerminalId, DesignationPart)>,
    ) {
        let label = value
            .as_ref()
            .and_then(|(id, pos)| {
                terminals
                    .iter()
                    .find(|t| t.terminal == *id)
                    .map(|t| format!("{} / {}", terminal_text(t), pos.as_str()))
            })
            .unwrap_or_else(|| "Selecione borne/posição".into());
        ComboBox::from_id_salt(salt)
            .width(220.)
            .selected_text(label)
            .show_ui(ui, |ui| {
                for row in terminals {
                    for position in &row.positions {
                        let Ok(part) = DesignationPart::new(position) else {
                            continue;
                        };
                        let option = Some((row.terminal, part));
                        let text = format!("{} / {position}", terminal_text(row));
                        ui.selectable_value(value, option, text);
                    }
                }
            });
    }
    pub(in crate::physical_editor) fn wiring_ui(&mut self, session: &mut Session, ui: &mut Ui) {
        let Some(project) = session.project() else {
            return;
        };
        let project_id = project.id();
        ui.weak(
            "Rota gráfica, conexão lógica e condutor físico são camadas distintas: \
             associar um condutor não altera a topologia nem as redes.",
        );
        let rows = &self.cache.lists.assignments;
        let mut picked = None;
        ui.strong(format!("Conexões lógicas ({})", rows.len()));
        let height = ui.spacing().interact_size.y;
        ScrollArea::vertical()
            .id_salt("wiring_connections")
            .max_height(150.)
            .show_rows(ui, height, rows.len(), |ui, range| {
                for row in &rows[range] {
                    let state = if row.bridge {
                        "ponte de régua".to_owned()
                    } else if row.conductors.is_empty() {
                        "sem condutor".to_owned()
                    } else {
                        format!("{} condutor(es)", row.conductors.len())
                    };
                    let drawn = if row.drawn { " · desenhada" } else { "" };
                    let text = format!("{} → {} · {state}{drawn}", row.source, row.target);
                    let selected = self.selected_connection == Some(row.connection);
                    if ui.selectable_label(selected, text).clicked() {
                        picked = Some(row.connection);
                    }
                }
            });
        if let Some(id) = picked {
            self.focus_connection(id);
        }
        let mut edit: Option<Result<PhysicalEdit>> = None;
        let mut locate = None;
        let selected = self
            .selected_connection
            .and_then(|id| {
                self.cache
                    .lists
                    .assignments
                    .iter()
                    .find(|r| r.connection == id)
            })
            .cloned();
        let mut ends = (None, None);
        if let Some(row) = &selected {
            let terminal_of = |p: ConnectionPointId| {
                project
                    .connection_point(p)
                    .and_then(ConnectionPoint::terminal_id)
            };
            ends = (terminal_of(row.source_point), terminal_of(row.target_point));
            ui.separator();
            ui.horizontal(|ui| {
                ui.strong("Conexão selecionada");
                if ui.small_button("Localizar").clicked() {
                    locate = Some(RepresentationTarget::Connection(row.connection));
                }
            });
            let describe = |t: Option<TerminalId>| {
                t.map(|t| project.terminal_label(t))
                    .unwrap_or_else(|| "sem borne físico".into())
            };
            egui::Grid::new("wiring_endpoints")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Origem");
                    ui.label(format!("{} · borne {}", row.source, describe(ends.0)));
                    ui.end_row();
                    ui.label("Destino");
                    ui.label(format!("{} · borne {}", row.target, describe(ends.1)));
                    ui.end_row();
                });
            if row.bridge {
                ui.weak("Ponte explícita de régua: continuidade interna, sem condutor externo.");
            } else {
                for wire in row.conductors.iter().filter_map(|id| {
                    self.cache
                        .lists
                        .conductors
                        .iter()
                        .find(|w| w.conductor == *id)
                }) {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{} · {} → {} · {} mm² · {}",
                            wire.designation,
                            wire.source,
                            wire.target,
                            wire.cross_section_mm2,
                            wire.cable_label.as_deref().unwrap_or("fio individual")
                        ));
                        if ui.small_button("Remover associação").clicked() {
                            edit = Some(Ok(PhysicalEdit::SetConductorConnection {
                                conductor: wire.conductor,
                                link: None,
                            }));
                        }
                    });
                }
                match ends {
                    (Some(a), Some(b)) => {
                        let compatible: Vec<_> = self
                            .cache
                            .lists
                            .conductors
                            .iter()
                            .filter(|w| {
                                w.connection.is_none()
                                    && ((w.source_terminal, w.target_terminal) == (a, b)
                                        || (w.source_terminal, w.target_terminal) == (b, a))
                            })
                            .collect();
                        ui.horizontal(|ui| {
                            ComboBox::from_id_salt("wire_to_link")
                                .selected_text(
                                    self.wire_to_link
                                        .and_then(|id| {
                                            compatible.iter().find(|w| w.conductor == id)
                                        })
                                        .map(|w| w.designation.as_str())
                                        .unwrap_or("Condutor compatível livre"),
                                )
                                .show_ui(ui, |ui| {
                                    for w in &compatible {
                                        ui.selectable_value(
                                            &mut self.wire_to_link,
                                            Some(w.conductor),
                                            format!(
                                                "{} · {} → {}",
                                                w.designation, w.source, w.target
                                            ),
                                        );
                                    }
                                });
                            if ui.button("Associar condutor").clicked() {
                                edit = Some(
                                    self.wire_to_link
                                        .and_then(|id| {
                                            compatible.iter().find(|w| w.conductor == id)
                                        })
                                        .map(|w| PhysicalEdit::SetConductorConnection {
                                            conductor: w.conductor,
                                            link: Some(link_for(row, ends, w.source_terminal)),
                                        })
                                        .ok_or(DomainError::InvalidRelation(
                                            "selecione um condutor compatível",
                                        )),
                                );
                            }
                        });
                    }
                    _ => {
                        ui.weak(
                            "Os dois pontos funcionais precisam de borne físico explícito \
                             para receber um condutor.",
                        );
                    }
                }
            }
        }
        ui.separator();
        ui.strong("Cadastrar condutor físico");
        egui::Grid::new("wire_fields")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Designação");
                ui.text_edit_singleline(&mut self.wire_name);
                ui.end_row();
                ui.label("Seção nominal (mm²)");
                ui.text_edit_singleline(&mut self.wire_section);
                ui.end_row();
                ui.label("Marcação (opcional)");
                ui.text_edit_singleline(&mut self.wire_marking);
                ui.end_row();
                ui.label("Origem");
                Self::termination_picker(
                    ui,
                    "wire_source",
                    &self.cache.lists.terminals,
                    &mut self.wire_source,
                );
                ui.end_row();
                ui.label("Destino");
                Self::termination_picker(
                    ui,
                    "wire_target",
                    &self.cache.lists.terminals,
                    &mut self.wire_target,
                );
                ui.end_row();
            });
        let linkable = selected.as_ref().filter(|r| !r.bridge).zip(match ends {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        });
        ui.horizontal(|ui| {
            if let Some((_, (a, b))) = linkable
                && ui.button("Usar bornes da conexão").clicked()
            {
                let first = |t: TerminalId| {
                    self.cache
                        .lists
                        .terminals
                        .iter()
                        .find(|r| r.terminal == t)
                        .and_then(|r| r.positions.first())
                        .and_then(|p| DesignationPart::new(p).ok())
                        .map(|p| (t, p))
                };
                self.wire_source = first(a);
                self.wire_target = first(b);
                if self.wire_source.is_none() || self.wire_target.is_none() {
                    self.feedback = "Borne sem posição de conexão cadastrada.".into();
                }
            }
            let create = ui.button("Criar condutor").clicked();
            let create_link = linkable.is_some() && ui.button("Criar e associar").clicked();
            if create || create_link {
                let built = self.build_wire(project_id);
                edit = Some(built.map(|wire| match linkable.filter(|_| create_link) {
                    None => PhysicalEdit::UpsertConductor(wire),
                    Some((row, _)) => {
                        let link = link_for(row, ends, wire.endpoints().0.terminal_id());
                        let conductor = wire.id();
                        PhysicalEdit::Batch(vec![
                            PhysicalEdit::UpsertConductor(wire),
                            PhysicalEdit::SetConductorConnection {
                                conductor,
                                link: Some(link),
                            },
                        ])
                    }
                }));
            }
        });
        ui.separator();
        self.conductor_table(ui, &mut edit, &mut locate);
        if let Some(id) = picked {
            session.select(Selection::Electrical(RepresentationTarget::Connection(id)));
        }
        if let Some(target) = locate {
            self.request_locate(session, target);
        }
        if let Some(edit) = edit {
            self.apply(session, edit);
        }
    }
}
impl PhysicalEditor {
    pub(in crate::physical_editor) fn conductor_table(
        &mut self,
        ui: &mut Ui,
        edit: &mut Option<Result<PhysicalEdit>>,
        locate: &mut Option<RepresentationTarget>,
    ) {
        let wires = &self.cache.lists.conductors;
        ui.strong(format!("Condutores cadastrados ({})", wires.len()));
        let height = ui.spacing().interact_size.y;
        let mut picked = None;
        ScrollArea::vertical()
            .id_salt("wire_table")
            .max_height(140.)
            .show_rows(ui, height, wires.len(), |ui, range| {
                for w in &wires[range] {
                    let text = format!(
                        "{} · {} → {} · {} mm² · {} · {}",
                        w.designation,
                        w.source,
                        w.target,
                        w.cross_section_mm2,
                        w.cable_label.as_deref().unwrap_or("fio individual"),
                        if w.connection.is_some() {
                            "associado"
                        } else {
                            "sem conexão"
                        }
                    );
                    if ui
                        .selectable_label(self.selected_wire == Some(w.conductor), text)
                        .clicked()
                    {
                        picked = Some(w.conductor);
                    }
                }
            });
        if let Some(id) = picked {
            self.selected_wire = Some(id);
            if let Some(w) = wires.iter().find(|w| w.conductor == id) {
                self.wire_edit_name = w.designation.clone();
                self.wire_edit_section = w.cross_section_mm2.to_string();
            }
        }
        let Some(wire) = self
            .selected_wire
            .and_then(|id| self.cache.wires.iter().find(|w| w.id() == id))
            .cloned()
        else {
            return;
        };
        ui.horizontal(|ui| {
            ui.label("Designação");
            ui.add(egui::TextEdit::singleline(&mut self.wire_edit_name).desired_width(80.));
            ui.label("Seção mm²");
            ui.add(egui::TextEdit::singleline(&mut self.wire_edit_section).desired_width(60.));
            if ui.button("Salvar designação/seção").clicked() {
                *edit = Some((|| {
                    let area = optional_section(&self.wire_edit_section)?.ok_or(
                        DomainError::InvalidNumber("seção nominal obrigatória em mm²"),
                    )?;
                    Ok(PhysicalEdit::UpsertConductor(
                        wire.clone()
                            .with_designation(DesignationPart::new(&self.wire_edit_name)?)
                            .with_cross_section(area),
                    ))
                })());
            }
            if ui.button("Excluir condutor").clicked() {
                *edit = Some(Ok(PhysicalEdit::DeleteConductor(wire.id())));
            }
            if ui.button("Localizar").clicked() {
                *locate = Some(RepresentationTarget::Conductor(wire.id()));
            }
        });
    }
}
/// Orients the conductor link so that its source termination maps to the
/// connection point carrying the same physical terminal.
pub(in crate::physical_editor) fn link_for(
    row: &WiringAssignmentRow,
    ends: (Option<TerminalId>, Option<TerminalId>),
    wire_source: TerminalId,
) -> ConductorConnection {
    let point = if ends.0 == Some(wire_source) {
        row.source_point
    } else {
        row.target_point
    };
    ConductorConnection::new(row.connection, point)
}
