// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! draw support for the physical editor/connectors; start at the parent module.

use super::*;

pub(in crate::physical_editor) fn gender_combo(
    ui: &mut Ui,
    salt: &str,
    value: &mut ConnectorGender,
) {
    ComboBox::from_id_salt(salt)
        .selected_text(value.label())
        .show_ui(ui, |ui| {
            for gender in ConnectorGender::ALL {
                ui.selectable_value(value, gender, gender.label());
            }
        });
}

impl PhysicalEditor {
    pub(in crate::physical_editor) fn connectors_ui(&mut self, session: &mut Session, ui: &mut Ui) {
        let Some(project_id) = session.project().map(Project::id) else {
            return;
        };
        ui.strong("Novo conector");
        ui.horizontal_wrapped(|ui| {
            ComboBox::from_id_salt("connector_new_device")
                .selected_text(
                    self.connector
                        .new_device
                        .and_then(|id| self.cache.devices.iter().find(|(d, _)| *d == id))
                        .map(|(_, name)| name.as_str())
                        .unwrap_or("Dispositivo do conector"),
                )
                .show_ui(ui, |ui| {
                    for (id, name) in &self.cache.devices {
                        ui.selectable_value(&mut self.connector.new_device, Some(*id), name);
                    }
                });
            ui.label("Identificação");
            ui.add(egui::TextEdit::singleline(&mut self.connector.new_name).desired_width(60.));
            gender_combo(ui, "connector_new_gender", &mut self.connector.new_gender);
            ui.label("Contatos");
            ui.add(egui::TextEdit::singleline(&mut self.connector.new_count).desired_width(40.));
            ui.label("Posição de conexão");
            ui.add(egui::TextEdit::singleline(&mut self.connector.new_position).desired_width(40.));
            if ui
                .button("Criar conector")
                .on_hover_text("Cria o conector e seus contatos numerados em uma transação")
                .clicked()
            {
                let edit = self.new_connector_edit(session, project_id);
                if let Ok(PhysicalEdit::Batch(edits)) = &edit
                    && let Some(PhysicalEdit::UpsertConnector(c)) = edits.last()
                {
                    self.connector.selected = Some(c.id());
                }
                self.apply(session, edit);
            }
        });
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            ui.label("Conector");
            ComboBox::from_id_salt("connector_choice")
                .selected_text(
                    self.connector
                        .selected
                        .and_then(|id| self.cache.connectors.iter().find(|c| c.id() == id))
                        .map(|c| c.designation().as_str())
                        .unwrap_or("Selecione conector"),
                )
                .show_ui(ui, |ui| {
                    for c in &self.cache.connectors {
                        if ui
                            .selectable_value(
                                &mut self.connector.selected,
                                Some(c.id()),
                                c.designation().as_str(),
                            )
                            .clicked()
                        {
                            self.connector.name = c.designation().as_str().into();
                            self.connector.gender = c.gender();
                            self.connector.contact = None;
                            self.connector.pairs.clear();
                        }
                    }
                });
        });
        let Some(connector) = self
            .connector
            .selected
            .and_then(|id| self.cache.connectors.iter().find(|c| c.id() == id))
            .cloned()
        else {
            ui.weak("Selecione ou crie um conector para editar contatos e acoplamento.");
            return;
        };
        let row = self
            .cache
            .lists
            .connectors
            .iter()
            .find(|r| r.connector == connector.id())
            .cloned();
        ui.horizontal_wrapped(|ui| {
            ui.label("Identificação");
            ui.add(egui::TextEdit::singleline(&mut self.connector.name).desired_width(60.));
            gender_combo(ui, "connector_gender", &mut self.connector.gender);
            if ui.button("Salvar conector").clicked() {
                let edit = DesignationPart::new(&self.connector.name).map(|name| {
                    PhysicalEdit::UpsertConnector(
                        connector
                            .clone()
                            .with_designation(name)
                            .with_gender(self.connector.gender),
                    )
                });
                self.apply(session, edit);
            }
            if ui
                .button("Excluir conector e contatos")
                .on_hover_text(
                    "Exclui o conector e seus contatos em uma transação; \
                     recusado se algum contato estiver em uso ou acoplado",
                )
                .clicked()
            {
                let mut edits = vec![PhysicalEdit::DeleteConnector(connector.id())];
                edits.extend(
                    connector
                        .contacts()
                        .iter()
                        .map(|t| PhysicalEdit::DeleteTerminal(*t)),
                );
                self.apply(session, Ok(PhysicalEdit::Batch(edits)));
            }
            if ui.button("Localizar conector").clicked() {
                self.request_connector_locate(session, connector.id());
            }
        });
        if let Some(row) = &row {
            let free = row.contacts.iter().filter(|c| c.is_free()).count();
            ui.label(format!(
                "Dispositivo {} · gênero {} · {} contatos ({} livres, {} ocupados){}",
                row.device_label.as_deref().unwrap_or("-"),
                row.gender.label(),
                row.contacts.len(),
                free,
                row.contacts.len() - free,
                row.mated_with
                    .as_ref()
                    .map(|m| format!(" · acoplado a {m}"))
                    .unwrap_or_default()
            ));
        }
        ui.separator();
        ui.strong(format!(
            "Contatos do conector {}",
            connector.designation().as_str()
        ));
        let contacts = connector.contacts().to_vec();
        ScrollArea::vertical()
            .id_salt("connector_contacts")
            .max_height(160.)
            .show(ui, |ui| {
                for (index, id) in contacts.iter().enumerate() {
                    let label = row
                        .as_ref()
                        .and_then(|r| r.contacts.iter().find(|c| c.terminal == *id))
                        .map(|c| {
                            format!(
                                "{} · posições {} · {} · {} pontos · {} condutores{}",
                                c.designation,
                                if c.positions.is_empty() {
                                    "-".into()
                                } else {
                                    c.positions.join("/")
                                },
                                if c.is_free() { "livre" } else { "ocupado" },
                                c.points.len(),
                                c.conductors.len(),
                                c.mated_label
                                    .as_ref()
                                    .map(|m| format!(" · ↔ {m}"))
                                    .unwrap_or_default()
                            )
                        })
                        .unwrap_or_else(|| "?".into());
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(
                                self.connector.contact == Some(*id),
                                format!("{}. {label}", index + 1),
                            )
                            .clicked()
                        {
                            self.connector.contact = Some(*id);
                            self.connector.contact_name = self
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
                        let reorder = |order: Vec<TerminalId>| {
                            Ok(PhysicalEdit::UpsertConnector(
                                connector.clone().with_contacts(order),
                            ))
                        };
                        if ui.small_button("↑").clicked() && index > 0 {
                            let mut order = contacts.clone();
                            order.swap(index, index - 1);
                            self.apply(session, reorder(order));
                        }
                        if ui.small_button("↓").clicked() && index + 1 < contacts.len() {
                            let mut order = contacts.clone();
                            order.swap(index, index + 1);
                            self.apply(session, reorder(order));
                        }
                        if ui
                            .small_button("Remover contato")
                            .on_hover_text(
                                "Recusado se o contato estiver associado, fiado ou acoplado",
                            )
                            .clicked()
                        {
                            let mut order = contacts.clone();
                            order.remove(index);
                            self.apply(
                                session,
                                Ok(PhysicalEdit::Batch(vec![
                                    PhysicalEdit::UpsertConnector(
                                        connector.clone().with_contacts(order),
                                    ),
                                    PhysicalEdit::DeleteTerminal(*id),
                                ])),
                            );
                        }
                    });
                }
            });
        ui.horizontal_wrapped(|ui| {
            ui.label("Novo contato");
            ui.add(egui::TextEdit::singleline(&mut self.connector.add_contact).desired_width(50.));
            if ui.button("Adicionar contato").clicked() {
                let edit = self.add_contact_edit(&connector);
                self.apply(session, edit);
            }
            ui.label("Numerar a partir de");
            ui.add(
                egui::TextEdit::singleline(&mut self.connector.renumber_start).desired_width(40.),
            );
            if ui
                .button("Renumerar contatos")
                .on_hover_text("Numera na ordem do conector; IDs dos contatos não mudam")
                .clicked()
            {
                let edit = (|| {
                    let start = match self.connector.renumber_start.trim() {
                        "" => 1,
                        text => text
                            .parse::<u32>()
                            .map_err(|_| DomainError::InvalidNumber("número inicial"))?,
                    };
                    Ok(PhysicalEdit::Batch(renumber_edits(
                        &self.cache.terminals,
                        &contacts,
                        start,
                    )?))
                })();
                self.apply(session, edit);
            }
        });
        if let Some(contact) = self
            .connector
            .contact
            .filter(|id| contacts.contains(id))
            .and_then(|id| self.cache.terminals.iter().find(|t| t.id() == id))
            .cloned()
        {
            ui.horizontal(|ui| {
                ui.label("Numeração do contato selecionado");
                ui.add(
                    egui::TextEdit::singleline(&mut self.connector.contact_name).desired_width(60.),
                );
                if ui.button("Renomear contato").clicked() {
                    let edit = DesignationPart::new(&self.connector.contact_name).map(|name| {
                        PhysicalEdit::UpsertTerminal(contact.clone().with_designation(name))
                    });
                    self.apply(session, edit);
                }
            });
            self.point_association_ui(session, ui, &contact, "contact");
        }
        ui.separator();
        self.mating_ui(session, ui, &connector, project_id);
    }
}
