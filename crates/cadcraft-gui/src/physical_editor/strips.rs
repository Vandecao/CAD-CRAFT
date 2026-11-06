// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! strips support for the physical editor; start at the parent module.

use super::*;

impl PhysicalEditor {
    pub(in crate::physical_editor) fn strips_ui(&mut self, session: &mut Session, ui: &mut Ui) {
        let Some(project_id) = session.project().map(Project::id) else {
            return;
        };
        ui.horizontal(|ui| {
            ui.label("Nova régua");
            ui.text_edit_singleline(&mut self.new_strip);
            if ui.button("Criar régua").clicked() {
                let edit = DesignationPart::new(&self.new_strip)
                    .map(|name| PhysicalEdit::UpsertStrip(TerminalStrip::new(project_id, name)));
                self.apply(session, edit);
                self.new_strip.clear();
            }
        });
        ui.horizontal(|ui| {
            ComboBox::from_id_salt("strip_choice")
                .selected_text(
                    self.selected_strip
                        .and_then(|id| self.cache.strips.iter().find(|s| s.id() == id))
                        .map(|s| s.designation().as_str())
                        .unwrap_or("Selecione régua"),
                )
                .show_ui(ui, |ui| {
                    for strip in &self.cache.strips {
                        if ui
                            .selectable_value(
                                &mut self.selected_strip,
                                Some(strip.id()),
                                strip.designation().as_str(),
                            )
                            .clicked()
                        {
                            self.strip_name = strip.designation().as_str().into();
                            self.selected_terminal = None;
                            self.bridge_source = None;
                            self.bridge_target = None;
                        }
                    }
                });
            if let Some(strip) = self
                .selected_strip
                .and_then(|id| self.cache.strips.iter().find(|s| s.id() == id))
                .cloned()
            {
                ui.text_edit_singleline(&mut self.strip_name);
                if ui.button("Salvar identificação").clicked() {
                    self.apply(
                        session,
                        DesignationPart::new(&self.strip_name).map(|name| {
                            PhysicalEdit::UpsertStrip(strip.clone().with_designation(name))
                        }),
                    );
                }
                if ui.button("Excluir régua").clicked() {
                    self.apply(session, Ok(PhysicalEdit::DeleteStrip(strip.id())));
                }
            }
        });
        ui.separator();
        let selected = self
            .selected_strip
            .and_then(|id| self.cache.strips.iter().find(|s| s.id() == id))
            .cloned();
        if let Some(strip) = selected {
            ui.strong(format!("Bornes da régua {}", strip.designation().as_str()));
            let members = strip.terminals().to_vec();
            ScrollArea::vertical().max_height(160.).show(ui, |ui| {
                for (index, id) in members.iter().enumerate() {
                    let label = self
                        .cache
                        .lists
                        .terminals
                        .iter()
                        .find(|t| t.terminal == *id)
                        .map(|t| {
                            format!(
                                "{}{} · dono {} · lados {} · {} conexões · {} pontes · {} condutores",
                                t.designation,
                                if t.kind == TerminalKind::Generic {
                                    String::new()
                                } else {
                                    format!(" [{}]", t.kind.label())
                                },
                                if t.owner.is_empty() { "-" } else { &t.owner },
                                if t.positions.is_empty() {
                                    "-".into()
                                } else {
                                    t.positions.join("/")
                                },
                                t.connections.len(),
                                t.bridges.len(),
                                t.conductors.len()
                            )
                        })
                        .unwrap_or_else(|| "?".into());
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(
                                self.selected_terminal == Some(*id),
                                format!("{} · {label}", index + 1),
                            )
                            .clicked()
                        {
                            self.selected_terminal = Some(*id);
                            self.terminal_name = self
                                .cache
                                .terminals
                                .iter()
                                .find(|t| t.id() == *id)
                                .map(|t| t.designation().as_str().to_owned())
                                .unwrap_or_default();
                            session
                                .select(Selection::Electrical(RepresentationTarget::Terminal(*id)));
                        }
                        if ui.small_button("Localizar").clicked() {
                            self.request_locate(session, RepresentationTarget::Terminal(*id));
                        }
                        if ui.small_button("↑").clicked() && index > 0 {
                            let mut order = members.clone();
                            order.swap(index, index - 1);
                            self.apply(
                                session,
                                Ok(PhysicalEdit::UpsertStrip(
                                    strip.clone().with_terminals(order),
                                )),
                            );
                        }
                        if ui.small_button("↓").clicked() && index + 1 < members.len() {
                            let mut order = members.clone();
                            order.swap(index, index + 1);
                            self.apply(
                                session,
                                Ok(PhysicalEdit::UpsertStrip(
                                    strip.clone().with_terminals(order),
                                )),
                            );
                        }
                        if ui.small_button("Remover da régua").clicked() {
                            let mut order = members.clone();
                            order.remove(index);
                            self.apply(
                                session,
                                Ok(PhysicalEdit::UpsertStrip(
                                    strip.clone().with_terminals(order),
                                )),
                            );
                        }
                    });
                }
            });
            let available = self
                .cache
                .terminals
                .iter()
                .filter(|t| {
                    !self
                        .cache
                        .strips
                        .iter()
                        .any(|s| s.terminals().contains(&t.id()))
                })
                .cloned()
                .collect::<Vec<_>>();
            ui.horizontal(|ui| {
                ComboBox::from_id_salt("terminal_to_attach")
                    .selected_text(
                        self.attach_candidate
                            .and_then(|id| self.cache.terminals.iter().find(|t| t.id() == id))
                            .map(|t| t.designation().as_str())
                            .unwrap_or("Borne existente"),
                    )
                    .show_ui(ui, |ui| {
                        for terminal in &available {
                            ui.selectable_value(
                                &mut self.attach_candidate,
                                Some(terminal.id()),
                                terminal.designation().as_str(),
                            );
                        }
                    });
                if ui.button("Adicionar borne").clicked()
                    && let Some(id) = self.attach_candidate
                {
                    let mut order = members.clone();
                    order.push(id);
                    self.apply(
                        session,
                        Ok(PhysicalEdit::UpsertStrip(
                            strip.clone().with_terminals(order),
                        )),
                    );
                }
            });
            ui.horizontal(|ui| {
                ui.label("Numerar a partir de");
                ui.add(egui::TextEdit::singleline(&mut self.renumber_start).desired_width(50.));
                if ui
                    .button("Renumerar sequencialmente")
                    .on_hover_text(
                        "Atribui números consecutivos na ordem da régua em uma única transação",
                    )
                    .clicked()
                {
                    let edit = (|| {
                        let start = if self.renumber_start.trim().is_empty() {
                            1
                        } else {
                            self.renumber_start
                                .trim()
                                .parse::<u32>()
                                .map_err(|_| DomainError::InvalidNumber("número inicial"))?
                        };
                        let edits = renumber_edits(&self.cache.terminals, &members, start)?;
                        Ok(PhysicalEdit::Batch(edits))
                    })();
                    self.apply(session, edit);
                }
                if ui.small_button("Localizar régua").clicked() {
                    self.request_locate(session, RepresentationTarget::TerminalStrip(strip.id()));
                }
            });
            if let Some(terminal) = self
                .selected_terminal
                .filter(|id| members.contains(id))
                .and_then(|id| self.cache.terminals.iter().find(|t| t.id() == id))
                .cloned()
            {
                ui.horizontal(|ui| {
                    ui.label("Numeração do borne selecionado");
                    ui.add(egui::TextEdit::singleline(&mut self.terminal_name).desired_width(70.));
                    if ui.button("Renomear borne").clicked() {
                        let edit = DesignationPart::new(&self.terminal_name).map(|name| {
                            PhysicalEdit::UpsertTerminal(terminal.clone().with_designation(name))
                        });
                        self.apply(session, edit);
                    }
                });
                ui.separator();
                self.terminal_kind_ui(session, ui, &terminal);
            }
            ui.separator();
            self.bridge_ui(session, ui, &strip);
        } else {
            ui.weak("Selecione uma régua para ordenar bornes e cadastrar pontes.");
        }
        ui.separator();
        ui.strong("Cadastrar borne individual");
        ui.horizontal(|ui| {
            ComboBox::from_id_salt("terminal_owner")
                .selected_text(
                    self.selected_device
                        .and_then(|id| self.cache.devices.iter().find(|(d, _)| *d == id))
                        .map(|(_, name)| name.as_str())
                        .unwrap_or("Dispositivo proprietário"),
                )
                .show_ui(ui, |ui| {
                    for (id, name) in &self.cache.devices {
                        ui.selectable_value(&mut self.selected_device, Some(*id), name);
                    }
                });
            ui.label("Número");
            ui.text_edit_singleline(&mut self.new_terminal_name);
            ui.label("Lado/posição");
            ui.text_edit_singleline(&mut self.terminal_position);
            if ui.button("Criar borne").clicked() {
                let edit = (|| {
                    let device = self.selected_device.ok_or(DomainError::InvalidRelation(
                        "selecione dispositivo proprietário",
                    ))?;
                    let name = DesignationPart::new(&self.new_terminal_name)?;
                    let mut terminal =
                        Terminal::new(project_id, TerminalOwner::Device(device), name);
                    if !self.terminal_position.trim().is_empty() {
                        terminal = terminal
                            .with_positions(vec![DesignationPart::new(&self.terminal_position)?])?;
                    }
                    Ok(PhysicalEdit::UpsertTerminal(terminal))
                })();
                self.apply(session, edit);
            }
        });
        // Only the member picked in the strip list can be deleted here; the
        // attach combo has its own state, and stale ids are not offered.
        if let Some(row) = self
            .selected_terminal
            .and_then(|id| self.cache.lists.terminals.iter().find(|t| t.terminal == id))
            .cloned()
            && ui
                .button(format!("Excluir borne {}", terminal_text(&row)))
                .clicked()
        {
            self.apply(session, Ok(PhysicalEdit::DeleteTerminal(row.terminal)));
        }
    }

    pub(in crate::physical_editor) fn bridge_ui(
        &mut self,
        session: &mut Session,
        ui: &mut Ui,
        strip: &TerminalStrip,
    ) {
        ui.strong("Pontes elétricas explícitas");
        let points = self
            .cache
            .points
            .iter()
            .filter(|p| {
                p.terminal_id()
                    .is_some_and(|id| strip.terminals().contains(&id))
            })
            .cloned()
            .collect::<Vec<_>>();
        ui.horizontal(|ui| {
            for (key, selected, label) in [
                ("bridge_source", &mut self.bridge_source, "Origem"),
                ("bridge_target", &mut self.bridge_target, "Destino"),
            ] {
                ComboBox::from_id_salt(key)
                    .selected_text(
                        selected
                            .and_then(|id| points.iter().find(|p| p.id() == id))
                            .map(|p| p.designation().as_str())
                            .unwrap_or(label),
                    )
                    .show_ui(ui, |ui| {
                        for point in &points {
                            let terminal = point
                                .terminal_id()
                                .and_then(|id| self.cache.terminals.iter().find(|t| t.id() == id))
                                .map(|t| t.designation().as_str())
                                .unwrap_or("?");
                            ui.selectable_value(
                                selected,
                                Some(point.id()),
                                format!("{terminal}:{}", point.designation().as_str()),
                            );
                        }
                    });
            }
            ui.checkbox(&mut self.bridge_kind, "Fixa");
            if ui.button("Criar ponte").clicked() {
                let edit = self
                    .bridge_source
                    .zip(self.bridge_target)
                    .map(|(source, target)| PhysicalEdit::CreateBridge {
                        strip: strip.id(),
                        source,
                        target,
                        kind: if self.bridge_kind {
                            TerminalBridgeKind::Fixed
                        } else {
                            TerminalBridgeKind::Inserted
                        },
                    })
                    .ok_or(DomainError::InvalidRelation(
                        "selecione os dois pontos da ponte",
                    ));
                self.apply(session, edit);
            }
        });
        for bridge in self
            .cache
            .bridges
            .iter()
            .filter(|b| b.strip_id() == strip.id())
            .cloned()
            .collect::<Vec<_>>()
        {
            ui.horizontal(|ui| {
                ui.label(format!("{:?} · {}", bridge.kind(), bridge.id()));
                if ui.small_button("Remover ponte").clicked() {
                    self.apply(session, Ok(PhysicalEdit::DeleteBridge(bridge.id())));
                }
            });
        }
    }
}
