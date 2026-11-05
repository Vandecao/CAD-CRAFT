// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 11 inside the physical editor: industrial connectors (contacts are
//! existing terminals), documental mating, explicit point association and
//! terminal specializations. Every write goes through `apply` (history).
use super::*;
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct ConnectorDraft {
    pub selected: Option<ConnectorId>,
    pub contact: Option<TerminalId>,
    new_device: Option<DeviceId>,
    new_name: String,
    new_gender: ConnectorGender,
    new_count: String,
    new_position: String,
    name: String,
    gender: ConnectorGender,
    contact_name: String,
    add_contact: String,
    renumber_start: String,
    partner: Option<ConnectorId>,
    pairs: Vec<ContactPair>,
    pair_first: Option<TerminalId>,
    pair_second: Option<TerminalId>,
}

/// Draft of the specialization of the selected terminal, re-seeded from the
/// Core after every revision so Undo/Redo never leaves stale values.
#[derive(Default)]
pub(super) struct KindDraft {
    seeded: Option<TerminalId>,
    pub(super) kind: TerminalKind,
    pub(super) positions: String,
    pub(super) side_a: String,
    pub(super) side_b: String,
    levels: Vec<(Option<TerminalLevelId>, String, String)>,
    disconnect_open: bool,
    point: Option<ConnectionPointId>,
    point_level: Option<TerminalLevelId>,
}

fn parts(text: &str) -> Result<Vec<DesignationPart>> {
    text.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(DesignationPart::new)
        .collect()
}
fn joined<'a>(values: impl Iterator<Item = &'a DesignationPart>) -> String {
    values
        .map(DesignationPart::as_str)
        .collect::<Vec<_>>()
        .join(", ")
}
fn gender_combo(ui: &mut Ui, salt: &str, value: &mut ConnectorGender) {
    ComboBox::from_id_salt(salt)
        .selected_text(value.label())
        .show_ui(ui, |ui| {
            for gender in ConnectorGender::ALL {
                ui.selectable_value(value, gender, gender.label());
            }
        });
}

/// Pure builder used by the GUI and its tests: draft → candidate terminal.
/// The Core validates the result; nothing is assumed from the kind.
pub(super) fn terminal_from_draft(base: &Terminal, d: &KindDraft) -> Result<Terminal> {
    let mut sides = BTreeMap::new();
    for p in parts(&d.side_a)? {
        sides.insert(p, TerminalSide::A);
    }
    for p in parts(&d.side_b)? {
        if sides.insert(p, TerminalSide::B).is_some() {
            return Err(DomainError::InvalidRelation(
                "posição declarada nos dois lados",
            ));
        }
    }
    let mut t = base.clone().with_kind(d.kind);
    if d.kind == TerminalKind::MultiLevel {
        let levels = d
            .levels
            .iter()
            .map(|(id, name, positions)| {
                let name = DesignationPart::new(name.trim())?;
                match id.and_then(|id| base.level(id)) {
                    Some(level) => level
                        .clone()
                        .with_designation(name)
                        .with_positions(parts(positions)?),
                    None => TerminalLevel::new(name, parts(positions)?),
                }
            })
            .collect::<Result<Vec<_>>>()?;
        t = t.with_levels(levels);
    } else {
        t = t.with_levels(vec![]).with_positions(parts(&d.positions)?)?;
    }
    let state = (d.kind == TerminalKind::Disconnect).then_some(if d.disconnect_open {
        DisconnectState::Open
    } else {
        DisconnectState::Closed
    });
    Ok(t.with_sides(sides).with_disconnect_state(state))
}

impl KindDraft {
    fn seed(&mut self, t: &Terminal) {
        self.seeded = Some(t.id());
        self.kind = t.kind();
        self.positions = joined(t.positions());
        let side = |s| joined(t.sides().iter().filter(|(_, v)| **v == s).map(|(p, _)| p));
        self.side_a = side(TerminalSide::A);
        self.side_b = side(TerminalSide::B);
        self.levels = t
            .levels()
            .iter()
            .map(|l| {
                (
                    Some(l.id()),
                    l.designation().as_str().to_owned(),
                    joined(l.positions()),
                )
            })
            .collect();
        self.disconnect_open = t.disconnect_state() == Some(DisconnectState::Open);
        if self.point_level.is_some_and(|id| t.level(id).is_none()) {
            self.point_level = None;
        }
    }
}

impl PhysicalEditor {
    /// Called from `reseed` after any revision change.
    pub(super) fn reseed_connectors(&mut self) {
        let d = &mut self.connector;
        match d
            .selected
            .and_then(|id| self.cache.connectors.iter().find(|c| c.id() == id))
        {
            Some(c) => {
                d.name = c.designation().as_str().into();
                d.gender = c.gender();
                if d.contact.is_some_and(|t| !c.contacts().contains(&t)) {
                    d.contact = None;
                }
            }
            None => {
                d.selected = None;
                d.contact = None;
            }
        }
        if d.partner
            .is_some_and(|id| !self.cache.connectors.iter().any(|c| c.id() == id))
        {
            d.partner = None;
            d.pairs.clear();
        }
        match d
            .contact
            .and_then(|id| self.cache.terminals.iter().find(|t| t.id() == id))
        {
            Some(t) => d.contact_name = t.designation().as_str().into(),
            None => d.contact = None,
        }
        // The specialization draft follows the Core state of its terminal.
        if let Some(t) = self
            .kind
            .seeded
            .and_then(|id| self.cache.terminals.iter().find(|t| t.id() == id))
            .cloned()
        {
            self.kind.seed(&t);
        } else {
            self.kind.seeded = None;
        }
    }

    fn request_connector_locate(&mut self, session: &Session, id: ConnectorId) {
        match session.project().and_then(|p| p.connector_occurrence(id)) {
            Some(rep) => self.locate = Some(rep),
            None => self.feedback = "Conector sem ocorrência gráfica explícita.".into(),
        }
    }

    pub(super) fn connectors_ui(&mut self, session: &mut Session, ui: &mut Ui) {
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

    fn new_connector_edit(&self, session: &Session, project_id: ProjectId) -> Result<PhysicalEdit> {
        let d = &self.connector;
        let device = d.new_device.ok_or(DomainError::InvalidRelation(
            "selecione o dispositivo do conector",
        ))?;
        let name = if d.new_name.trim().is_empty() {
            session
                .project()
                .and_then(|p| p.device(device))
                .map(|dev| dev.designation().device().clone())
                .ok_or(DomainError::MissingEntity(EntityRef::Device(device)))?
        } else {
            DesignationPart::new(d.new_name.trim())?
        };
        let count = match d.new_count.trim() {
            "" => 0,
            text => text
                .parse::<u32>()
                .map_err(|_| DomainError::InvalidNumber("quantidade de contatos"))?,
        };
        if count > 512 {
            return Err(DomainError::InvalidNumber(
                "quantidade de contatos (máx. 512)",
            ));
        }
        let positions = parts(&d.new_position)?;
        let mut edits = Vec::new();
        let mut contacts = Vec::new();
        for n in 1..=count {
            let t = Terminal::new(
                project_id,
                TerminalOwner::Device(device),
                DesignationPart::new(n.to_string())?,
            )
            .with_positions(positions.clone())?;
            contacts.push(t.id());
            edits.push(PhysicalEdit::UpsertTerminal(t));
        }
        edits.push(PhysicalEdit::UpsertConnector(
            Connector::new(project_id, name)
                .with_gender(d.new_gender)
                .with_device(Some(device))
                .with_contacts(contacts),
        ));
        Ok(PhysicalEdit::Batch(edits))
    }

    fn add_contact_edit(&self, connector: &Connector) -> Result<PhysicalEdit> {
        let owner = connector
            .device()
            .map(TerminalOwner::Device)
            .or_else(|| {
                connector
                    .contacts()
                    .first()
                    .and_then(|id| self.cache.terminals.iter().find(|t| t.id() == *id))
                    .map(Terminal::owner)
            })
            .ok_or(DomainError::InvalidRelation(
                "conector sem dispositivo para o novo contato",
            ))?;
        let positions = parts(&self.connector.new_position)?;
        let terminal = Terminal::new(
            connector.project_id(),
            owner,
            DesignationPart::new(self.connector.add_contact.trim())?,
        )
        .with_positions(positions)?;
        let mut order = connector.contacts().to_vec();
        order.push(terminal.id());
        Ok(PhysicalEdit::Batch(vec![
            PhysicalEdit::UpsertTerminal(terminal),
            PhysicalEdit::UpsertConnector(connector.clone().with_contacts(order)),
        ]))
    }

    fn mating_ui(
        &mut self,
        session: &mut Session,
        ui: &mut Ui,
        connector: &Connector,
        project_id: ProjectId,
    ) {
        ui.strong("Acoplamento entre metades");
        ui.weak(
            "Documental: o mapa é explícito e validado, mas não cria conexões nem \
             continuidade elétrica (decisão pendente DEC-11-01).",
        );
        if let Some(mating) = self
            .cache
            .matings
            .iter()
            .find(|m| m.involves(connector.id()))
            .cloned()
        {
            let label = |t: TerminalId| {
                self.cache
                    .lists
                    .terminals
                    .iter()
                    .find(|r| r.terminal == t)
                    .map(terminal_text)
                    .unwrap_or_else(|| "?".into())
            };
            for pair in mating.pairs() {
                ui.label(format!(
                    "{} ↔ {}",
                    label(pair.first()),
                    label(pair.second())
                ));
            }
            if ui.button("Excluir acoplamento").clicked() {
                self.apply(session, Ok(PhysicalEdit::DeleteMating(mating.id())));
            }
            return;
        }
        let candidates: Vec<_> = self
            .cache
            .connectors
            .iter()
            .filter(|c| c.id() != connector.id())
            .cloned()
            .collect();
        ui.horizontal_wrapped(|ui| {
            ui.label("Conector par");
            ComboBox::from_id_salt("mating_partner")
                .selected_text(
                    self.connector
                        .partner
                        .and_then(|id| candidates.iter().find(|c| c.id() == id))
                        .map(|c| c.designation().as_str())
                        .unwrap_or("Selecione"),
                )
                .show_ui(ui, |ui| {
                    for c in &candidates {
                        if ui
                            .selectable_value(
                                &mut self.connector.partner,
                                Some(c.id()),
                                format!("{} ({})", c.designation().as_str(), c.gender().label()),
                            )
                            .clicked()
                        {
                            self.connector.pairs.clear();
                        }
                    }
                });
        });
        let Some(partner) = self
            .connector
            .partner
            .and_then(|id| candidates.iter().find(|c| c.id() == id))
            .cloned()
        else {
            return;
        };
        let name = |t: TerminalId| {
            self.cache
                .terminals
                .iter()
                .find(|x| x.id() == t)
                .map(|x| x.designation().as_str().to_owned())
                .unwrap_or_else(|| "?".into())
        };
        ui.horizontal_wrapped(|ui| {
            for (salt, value, list, text) in [
                (
                    "pair_first",
                    &mut self.connector.pair_first,
                    connector.contacts(),
                    "Contato deste",
                ),
                (
                    "pair_second",
                    &mut self.connector.pair_second,
                    partner.contacts(),
                    "Contato do par",
                ),
            ] {
                ComboBox::from_id_salt(salt)
                    .selected_text(value.map(name).unwrap_or_else(|| text.into()))
                    .show_ui(ui, |ui| {
                        for t in list {
                            ui.selectable_value(value, Some(*t), name(*t));
                        }
                    });
            }
            if ui.button("Adicionar par").clicked()
                && let (Some(a), Some(b)) = (self.connector.pair_first, self.connector.pair_second)
            {
                self.connector.pairs.push(ContactPair::new(a, b));
            }
            if ui
                .button("Parear por numeração")
                .on_hover_text(
                    "Preenche o rascunho com pares de mesma numeração; \
                     o mapa só vale após confirmação explícita",
                )
                .clicked()
            {
                self.connector.pairs = connector
                    .contacts()
                    .iter()
                    .filter_map(|a| {
                        let number = name(*a);
                        partner
                            .contacts()
                            .iter()
                            .find(|b| name(**b) == number)
                            .map(|b| ContactPair::new(*a, *b))
                    })
                    .collect();
            }
            if ui.button("Limpar pares").clicked() {
                self.connector.pairs.clear();
            }
        });
        for pair in &self.connector.pairs {
            ui.label(format!(
                "{}:{} ↔ {}:{}",
                connector.designation().as_str(),
                name(pair.first()),
                partner.designation().as_str(),
                name(pair.second())
            ));
        }
        if ui.button("Confirmar acoplamento").clicked() {
            let mating = ConnectorMating::new(
                project_id,
                connector.id(),
                partner.id(),
                self.connector.pairs.clone(),
            );
            self.apply(session, Ok(PhysicalEdit::UpsertMating(mating)));
            if self.cache.key.is_none() {
                self.connector.pairs.clear();
            }
        }
    }

    /// Explicit association between functional points of the owner device and
    /// a terminal/contact (and level). It never creates or removes connections.
    pub(super) fn point_association_ui(
        &mut self,
        session: &mut Session,
        ui: &mut Ui,
        terminal: &Terminal,
        salt: &str,
    ) {
        let Some(p) = session.project() else { return };
        let owner = match terminal.owner() {
            TerminalOwner::Device(d) => Some(d),
            TerminalOwner::Component(c) => p.component(c).and_then(Component::device_id),
        };
        let level_name = |level: Option<TerminalLevelId>| {
            level
                .and_then(|id| terminal.level(id))
                .map(|l| format!(" · nível {}", l.designation().as_str()))
                .unwrap_or_default()
        };
        let associated: Vec<(ConnectionPointId, String)> = p
            .connection_points()
            .filter(|pt| pt.terminal_id() == Some(terminal.id()))
            .map(|pt| {
                (
                    pt.id(),
                    format!(
                        "{}{}",
                        p.connection_point_label(pt.id()),
                        level_name(pt.terminal_level())
                    ),
                )
            })
            .collect();
        let candidates: Vec<(ConnectionPointId, String)> = p
            .connection_points()
            .filter(|pt| pt.terminal_id() != Some(terminal.id()))
            .filter(|pt| p.function(pt.function_id()).map(Function::device_id) == owner)
            .map(|pt| {
                let current = pt
                    .terminal_id()
                    .map(|t| format!(" (em {})", p.terminal_label(t)))
                    .unwrap_or_default();
                (
                    pt.id(),
                    format!("{}{current}", p.connection_point_label(pt.id())),
                )
            })
            .collect();
        ui.label("Pontos funcionais associados (explícito, sem criar conexões):");
        if associated.is_empty() {
            ui.weak("Nenhum ponto associado.");
        }
        for (id, label) in associated {
            ui.horizontal(|ui| {
                ui.label(label);
                if ui.small_button("Desassociar").clicked() {
                    self.apply(
                        session,
                        Ok(PhysicalEdit::SetPointTerminal {
                            point: id,
                            terminal: None,
                            level: None,
                        }),
                    );
                }
            });
        }
        if self
            .kind
            .point
            .is_some_and(|id| !candidates.iter().any(|(c, _)| *c == id))
        {
            self.kind.point = None;
        }
        ui.horizontal_wrapped(|ui| {
            ComboBox::from_id_salt(("point_candidate", salt))
                .selected_text(
                    self.kind
                        .point
                        .and_then(|id| candidates.iter().find(|(c, _)| *c == id))
                        .map(|(_, l)| l.as_str())
                        .unwrap_or("Ponto do dispositivo"),
                )
                .show_ui(ui, |ui| {
                    for (id, label) in &candidates {
                        ui.selectable_value(&mut self.kind.point, Some(*id), label);
                    }
                });
            if terminal.kind() == TerminalKind::MultiLevel {
                ComboBox::from_id_salt(("point_level", salt))
                    .selected_text(
                        self.kind
                            .point_level
                            .and_then(|id| terminal.level(id))
                            .map(|l| l.designation().as_str())
                            .unwrap_or("Nível"),
                    )
                    .show_ui(ui, |ui| {
                        for level in terminal.levels() {
                            ui.selectable_value(
                                &mut self.kind.point_level,
                                Some(level.id()),
                                level.designation().as_str(),
                            );
                        }
                    });
            }
            if ui.button("Associar ponto").clicked() {
                let edit = self
                    .kind
                    .point
                    .ok_or(DomainError::InvalidRelation("selecione o ponto funcional"))
                    .map(|point| PhysicalEdit::SetPointTerminal {
                        point,
                        terminal: Some(terminal.id()),
                        level: (terminal.kind() == TerminalKind::MultiLevel)
                            .then_some(self.kind.point_level)
                            .flatten(),
                    });
                self.apply(session, edit);
            }
        });
    }

    /// Specialization editor of the terminal selected in the strip list.
    pub(super) fn terminal_kind_ui(&mut self, session: &mut Session, ui: &mut Ui, t: &Terminal) {
        if self.kind.seeded != Some(t.id()) {
            self.kind.seed(t);
        }
        ui.strong("Especialização do borne");
        if self
            .cache
            .lists
            .terminals
            .iter()
            .any(|r| r.terminal == t.id() && r.connector.is_some())
        {
            ui.weak("Contato de conector: permanece borne genérico.");
            return;
        }
        ui.horizontal_wrapped(|ui| {
            ui.label("Tipo de borne");
            ComboBox::from_id_salt("terminal_kind")
                .selected_text(self.kind.kind.label())
                .show_ui(ui, |ui| {
                    for kind in TerminalKind::ALL {
                        ui.selectable_value(&mut self.kind.kind, kind, kind.label());
                    }
                });
            if self.kind.kind != TerminalKind::MultiLevel {
                ui.label("Posições do borne");
                ui.add(egui::TextEdit::singleline(&mut self.kind.positions).desired_width(90.));
            }
            ui.label("Lado A");
            ui.add(egui::TextEdit::singleline(&mut self.kind.side_a).desired_width(70.));
            ui.label("Lado B");
            ui.add(egui::TextEdit::singleline(&mut self.kind.side_b).desired_width(70.));
            if self.kind.kind == TerminalKind::Disconnect {
                ui.checkbox(&mut self.kind.disconnect_open, "Seccionado (aberto)")
                    .on_hover_text("Estado estático documental; não altera redes nem simula");
            }
        });
        if self.kind.kind == TerminalKind::MultiLevel {
            ui.label("Níveis (identificação · posições separadas por vírgula):");
            let mut remove = None;
            for (index, (_, name, positions)) in self.kind.levels.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("Nível {}", index + 1));
                    ui.add(egui::TextEdit::singleline(name).desired_width(50.));
                    ui.add(egui::TextEdit::singleline(positions).desired_width(110.));
                    if ui.small_button("Remover nível").clicked() {
                        remove = Some(index);
                    }
                });
            }
            if let Some(index) = remove {
                self.kind.levels.remove(index);
            }
            if ui.button("Adicionar nível").clicked() {
                let n = self.kind.levels.len() + 1;
                self.kind.levels.push((None, n.to_string(), String::new()));
            }
        }
        let note = match self.kind.kind {
            TerminalKind::ProtectiveEarth => {
                "PE: não cria terra nem redes; recusa condutor declarado não PE e ponte com borne não PE."
            }
            TerminalKind::MultiLevel => {
                "Níveis isolados: cada ponto associado indica seu nível; continuidade só por ponte explícita."
            }
            TerminalKind::FeedThrough | TerminalKind::Distribution => {
                "Continuidade somente pelos pontos funcionais associados explicitamente."
            }
            TerminalKind::Disconnect => {
                "Estado documental: não abre nem fecha continuidade no grafo (sem simulação)."
            }
            TerminalKind::Generic => "Borne sem especialização.",
        };
        ui.weak(note);
        if ui.button("Aplicar especialização").clicked() {
            let edit = terminal_from_draft(t, &self.kind).map(PhysicalEdit::UpsertTerminal);
            self.apply(session, edit);
        }
        self.point_association_ui(session, ui, t, "strip_terminal");
    }
}
