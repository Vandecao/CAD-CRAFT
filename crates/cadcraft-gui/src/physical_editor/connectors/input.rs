// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! input support for the physical editor/connectors; start at the parent module.

use super::*;

impl PhysicalEditor {
    /// Called from `reseed` after any revision change.
    pub(in crate::physical_editor) fn reseed_connectors(&mut self) {
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

    pub(in crate::physical_editor) fn request_connector_locate(
        &mut self,
        session: &Session,
        id: ConnectorId,
    ) {
        match session.project().and_then(|p| p.connector_occurrence(id)) {
            Some(rep) => self.locate = Some(rep),
            None => self.feedback = "Conector sem ocorrência gráfica explícita.".into(),
        }
    }
}
impl PhysicalEditor {
    pub(in crate::physical_editor) fn new_connector_edit(
        &self,
        session: &Session,
        project_id: ProjectId,
    ) -> Result<PhysicalEdit> {
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

    pub(in crate::physical_editor) fn add_contact_edit(
        &self,
        connector: &Connector,
    ) -> Result<PhysicalEdit> {
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
}
