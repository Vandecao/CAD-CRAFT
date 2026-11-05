// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::*;
use std::collections::BTreeSet;

/// Situação derivada de um contato de conector. Nunca é persistida.
/// Um contato está ocupado quando há associação explícita a ponto funcional
/// ou condutor físico terminando nele; o acoplamento não o ocupa.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ContactStatus {
    pub points: Vec<ConnectionPointId>,
    pub conductors: Vec<ConductorId>,
    pub mated_with: Option<TerminalId>,
}
impl ContactStatus {
    pub fn is_free(&self) -> bool {
        self.points.is_empty() && self.conductors.is_empty()
    }
}

impl Project {
    pub fn connector(&self, id: ConnectorId) -> Option<&Connector> {
        self.connectors.get(&id)
    }
    pub fn connectors(&self) -> impl Iterator<Item = &Connector> {
        self.connectors.values()
    }
    pub fn connector_mating(&self, id: ConnectorMatingId) -> Option<&ConnectorMating> {
        self.connector_matings.get(&id)
    }
    pub fn connector_matings(&self) -> impl Iterator<Item = &ConnectorMating> {
        self.connector_matings.values()
    }
    pub fn connector_for_terminal(&self, terminal: TerminalId) -> Option<&Connector> {
        self.physical_index
            .connector_by_terminal
            .get(&terminal)
            .and_then(|id| self.connector(*id))
    }
    pub fn mating_for_connector(&self, connector: ConnectorId) -> Option<&ConnectorMating> {
        self.physical_index
            .mating_by_connector
            .get(&connector)
            .and_then(|id| self.connector_mating(*id))
    }
    /// Contato da outra metade segundo o mapa explícito do acoplamento.
    pub fn mated_contact(&self, contact: TerminalId) -> Option<TerminalId> {
        let connector = self.connector_for_terminal(contact)?;
        self.mating_for_connector(connector.id())?.mate_of(contact)
    }
    /// Pontos funcionais explicitamente associados ao borne/contato.
    pub fn points_for_terminal(&self, terminal: TerminalId) -> Vec<ConnectionPointId> {
        self.connection_points
            .values()
            .filter(|p| p.terminal_id() == Some(terminal))
            .map(ConnectionPoint::id)
            .collect()
    }
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
    /// Primeira ocorrência gráfica explícita de um contato do conector, ou do
    /// dispositivo documental associado. Nunca procura por texto/proximidade.
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
    pub(super) fn check_connector(&self, connector: &Connector) -> Result<()> {
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
    pub(super) fn check_mating(&self, mating: &ConnectorMating) -> Result<()> {
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
        // Etapa 24 (DEC-11-01): conexões de posse coerentes com os pares.
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
    /// Ponto de conexão associado ao contato (o de menor ID, se houver vários).
    pub(super) fn contact_point(&self, contact: TerminalId) -> Option<ConnectionPointId> {
        self.connection_points()
            .filter(|p| p.terminal_id() == Some(contact))
            .map(ConnectionPoint::id)
            .min()
    }
    /// Pares de pontos que o acoplamento liga, na ordem dos pares; contatos
    /// sem ponto associado ficam de fora (verificação VER-017).
    pub(super) fn mating_point_pairs(
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
    /// Conexões de posse para o acoplamento: reaproveita as de `before` que
    /// ligam os mesmos pontos (preserva número de fio) e cria as que faltam.
    /// Devolve (ids na ordem dos pares, novas conexões, conexões removidas).
    pub(super) fn plan_mating_connections(
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
