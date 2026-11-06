// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Industrial connector edits and mating; start with the Project connector methods.
use super::*;
use std::collections::BTreeSet;

/// Derived connector contact status, never persisted. An explicit functional-point binding or
/// terminating physical conductor occupies a contact; mating alone does not.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ContactStatus {
    /// Connection points or authored geometric vertices.
    pub points: Vec<ConnectionPointId>,
    /// Physical conductors associated with this record.
    pub conductors: Vec<ConductorId>,
    /// Explicitly mapped mating counterpart, when present.
    pub mated_with: Option<TerminalId>,
}
impl ContactStatus {
    /// Whether no explicit functional point or physical conductor occupies the contact.
    pub fn is_free(&self) -> bool {
        self.points.is_empty() && self.conductors.is_empty()
    }
}

impl Project {
    /// Find the connector by its stable identifier; return None when absent.
    pub fn connector(&self, id: ConnectorId) -> Option<&Connector> {
        self.connectors.get(&id)
    }
    /// Iterate connectors without exposing mutable access.
    pub fn connectors(&self) -> impl Iterator<Item = &Connector> {
        self.connectors.values()
    }
    /// Find the connector mating by its stable identifier; return None when absent.
    pub fn connector_mating(&self, id: ConnectorMatingId) -> Option<&ConnectorMating> {
        self.connector_matings.get(&id)
    }
    /// Iterate connector matings without exposing mutable access.
    pub fn connector_matings(&self) -> impl Iterator<Item = &ConnectorMating> {
        self.connector_matings.values()
    }
    /// Return the connector explicitly containing this terminal as a contact.
    pub fn connector_for_terminal(&self, terminal: TerminalId) -> Option<&Connector> {
        self.physical_index
            .connector_by_terminal
            .get(&terminal)
            .and_then(|id| self.connector(*id))
    }
    /// Return the explicit mating containing the connector, if any.
    pub fn mating_for_connector(&self, connector: ConnectorId) -> Option<&ConnectorMating> {
        self.physical_index
            .mating_by_connector
            .get(&connector)
            .and_then(|id| self.connector_mating(*id))
    }
    /// Counterpart contact selected by the explicit mating map.
    pub fn mated_contact(&self, contact: TerminalId) -> Option<TerminalId> {
        let connector = self.connector_for_terminal(contact)?;
        self.mating_for_connector(connector.id())?.mate_of(contact)
    }
    /// Functional points explicitly bound to the terminal or contact.
    pub fn points_for_terminal(&self, terminal: TerminalId) -> Vec<ConnectionPointId> {
        self.connection_points
            .values()
            .filter(|p| p.terminal_id() == Some(terminal))
            .map(ConnectionPoint::id)
            .collect()
    }
    /// Derive current point, conductor and mating associations for a contact.
    pub fn contact_status(&self, contact: TerminalId) -> Result<ContactStatus> {
        self.require(EntityRef::Terminal(contact))?;
        Ok(ContactStatus {
            points: self.points_for_terminal(contact),
            conductors: self
                .physical_index
                .conductors_by_terminal
                .get(&contact)
                .into_iter()
                .flatten()
                .copied()
                .collect(),
            mated_with: self.mated_contact(contact),
        })
    }
    /// First explicit graphical occurrence of a connector contact or its associated documentary device.
    /// Never searches by text or proximity.
    pub fn connector_occurrence(&self, id: ConnectorId) -> Option<RepresentationId> {
        let connector = self.connector(id)?;
        connector
            .contacts()
            .iter()
            .find_map(|t| self.physical_occurrence(RepresentationTarget::Terminal(*t)))
            .or_else(|| {
                connector
                    .device()
                    .and_then(|d| self.physical_occurrence(RepresentationTarget::Device(d)))
            })
    }
    pub(in crate::domain::project) fn check_connector(&self, connector: &Connector) -> Result<()> {
        self.check_project(connector.project_id())?;
        self.check_properties(connector.properties())?;
        if let Some(device) = connector.device() {
            self.require(EntityRef::Device(device))?;
        }
        if let Some(id) = connector.component() {
            let component = self
                .component(id)
                .ok_or(DomainError::MissingEntity(EntityRef::Component(id)))?;
            if connector
                .device()
                .is_some_and(|d| component.device_id() != Some(d))
            {
                return Err(DomainError::InvalidRelation(
                    "componente do conector pertence a outro dispositivo",
                ));
            }
        }
        let mut numbers = BTreeSet::new();
        for &id in connector.contacts() {
            let contact = self
                .terminal(id)
                .ok_or(DomainError::MissingEntity(EntityRef::Terminal(id)))?;
            if !numbers.insert(contact.designation().clone()) {
                return Err(DomainError::InvalidRelation(
                    "numeração de contato duplicada no conector",
                ));
            }
            if contact.kind() != TerminalKind::Generic || !contact.levels().is_empty() {
                return Err(DomainError::InvalidRelation(
                    "contato de conector deve ser borne genérico",
                ));
            }
            if let Some(device) = connector.device()
                && self.terminal_owner_device(contact) != Some(device)
            {
                return Err(DomainError::InvalidRelation(
                    "contato pertence a outro dispositivo",
                ));
            }
        }
        Ok(())
    }
    pub(in crate::domain::project) fn check_mating(&self, mating: &ConnectorMating) -> Result<()> {
        self.check_project(mating.project_id())?;
        self.check_properties(mating.properties())?;
        if mating.first() == mating.second() {
            return Err(DomainError::InvalidRelation(
                "conector não acopla consigo mesmo",
            ));
        }
        let first = self
            .connector(mating.first())
            .ok_or(DomainError::MissingEntity(EntityRef::Connector(
                mating.first(),
            )))?;
        let second = self
            .connector(mating.second())
            .ok_or(DomainError::MissingEntity(EntityRef::Connector(
                mating.second(),
            )))?;
        if !first.gender().mates_with(second.gender()) {
            return Err(DomainError::InvalidRelation(
                "gêneros de conector incompatíveis para acoplamento",
            ));
        }
        if mating.pairs().is_empty() {
            return Err(DomainError::InvalidRelation(
                "acoplamento exige mapeamento explícito de contatos",
            ));
        }
        let mut used = BTreeSet::new();
        for pair in mating.pairs() {
            if !first.contacts().contains(&pair.first())
                || !second.contacts().contains(&pair.second())
            {
                return Err(DomainError::InvalidRelation(
                    "contato do acoplamento fora do conector",
                ));
            }
            if !used.insert(pair.first()) || !used.insert(pair.second()) {
                return Err(DomainError::InvalidRelation(
                    "contato mapeado mais de uma vez no acoplamento",
                ));
            }
        }
        // Owned connections must match the explicit contact pairs.
        if !mating.owned_is_legacy() {
            let expected = self.mating_point_pairs(mating);
            let owned = mating.owned_connections();
            if owned.len() != expected.len() {
                return Err(DomainError::InvalidRelation(
                    "conexões do acoplamento não correspondem aos contatos associados",
                ));
            }
            for (id, (a, b)) in owned.iter().zip(expected) {
                let connection = self
                    .connection(*id)
                    .ok_or(DomainError::MissingConnection(*id))?;
                if connection.endpoints() != (a.min(b), a.max(b)) {
                    return Err(DomainError::InvalidRelation(
                        "conexão do acoplamento não liga os contatos acoplados; desacople antes de mudar o ponto do contato",
                    ));
                }
            }
        }
        Ok(())
    }
}

impl Project {
    /// Connection point associated with the contact; lowest ID if several exist.
    pub(in crate::domain::project) fn contact_point(
        &self,
        contact: TerminalId,
    ) -> Option<ConnectionPointId> {
        self.connection_points()
            .filter(|p| p.terminal_id() == Some(contact))
            .map(ConnectionPoint::id)
            .min()
    }
    /// Point pairs connected by the mating, in pair order. Contacts without associated points are
    /// excluded and reported by VER-017.
    pub(in crate::domain::project) fn mating_point_pairs(
        &self,
        mating: &ConnectorMating,
    ) -> Vec<(ConnectionPointId, ConnectionPointId)> {
        mating
            .pairs()
            .iter()
            .filter_map(|pair| {
                Some((
                    self.contact_point(pair.first())?,
                    self.contact_point(pair.second())?,
                ))
            })
            .collect()
    }
    /// Owned mating connections: reuse entries in `before` with identical endpoints, preserving wire
    /// numbers, and create missing ones. Return ordered pair IDs, new connections and removed
    /// connections.
    pub(in crate::domain::project) fn plan_mating_connections(
        &self,
        mating: &ConnectorMating,
        before: Option<&ConnectorMating>,
    ) -> Result<(Vec<ConnectionId>, Vec<Connection>, Vec<Connection>)> {
        let mut previous: Vec<Connection> = before
            .map(|m| {
                m.owned_connections()
                    .iter()
                    .filter_map(|id| self.connection(*id).cloned())
                    .collect()
            })
            .unwrap_or_default();
        let mut ids = vec![];
        let mut created = vec![];
        for (a, b) in self.mating_point_pairs(mating) {
            let endpoints = (a.min(b), a.max(b));
            if let Some(i) = previous.iter().position(|c| c.endpoints() == endpoints) {
                ids.push(previous.remove(i).id());
            } else {
                let connection = Connection::new(self.id(), a, b)?;
                ids.push(connection.id());
                created.push(connection);
            }
        }
        Ok((ids, created, previous))
    }
}
