// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use crate::{
    domain::*,
    error::{DomainError, Result},
};
use std::collections::{BTreeMap, BTreeSet};

type WireKey = (PhysicalTermination, PhysicalTermination, DesignationPart);

/// Apenas índices derivados; nenhuma fonte persistente de relações.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct PhysicalIndex {
    names: BTreeMap<WireKey, ConductorId>,
    core_names: BTreeSet<(CableId, DesignationPart)>,
    pub cores_by_cable: BTreeMap<CableId, BTreeSet<CableCoreId>>,
    pub core_by_conductor: BTreeMap<ConductorId, CableCoreId>,
    pub strip_by_terminal: BTreeMap<TerminalId, TerminalStripId>,
    pub conductors_by_connection: BTreeMap<ConnectionId, BTreeSet<ConductorId>>,
    pub conductors_by_terminal: BTreeMap<TerminalId, BTreeSet<ConductorId>>,
    pub connector_by_terminal: BTreeMap<TerminalId, ConnectorId>,
    pub mating_by_connector: BTreeMap<ConnectorId, ConnectorMatingId>,
}
impl PhysicalIndex {
    pub fn insert_conductor(&mut self, conductor: &Conductor) -> Result<()> {
        let (a, b) = conductor.endpoints();
        let key = (
            a.min(b).clone(),
            a.max(b).clone(),
            conductor.designation().clone(),
        );
        if let Some(existing) = self.names.get(&key) {
            return Err(DomainError::DuplicateConductorDesignation(*existing));
        }
        self.names.insert(key, conductor.id());
        for end in [a, b] {
            self.conductors_by_terminal
                .entry(end.terminal_id())
                .or_default()
                .insert(conductor.id());
        }
        if let Some(link) = conductor.connection() {
            self.link_connection(conductor.id(), link.connection_id());
        }
        Ok(())
    }
    pub fn insert_core(&mut self, core: &CableCore, capacity: u32) -> Result<()> {
        let key = (core.cable_id(), core.identification().clone());
        if self.core_names.contains(&key) {
            return Err(DomainError::DuplicateCoreIdentification(core.cable_id()));
        }
        let count = self
            .cores_by_cable
            .get(&core.cable_id())
            .map_or(0, BTreeSet::len);
        if count >= capacity as usize {
            return Err(DomainError::CableCapacityExceeded(core.cable_id()));
        }
        if let Some(conductor) = core.conductor_id()
            && self.core_by_conductor.contains_key(&conductor)
        {
            return Err(DomainError::ConductorAlreadyAssigned(conductor));
        }
        self.core_names.insert(key);
        self.cores_by_cable
            .entry(core.cable_id())
            .or_default()
            .insert(core.id());
        if let Some(conductor) = core.conductor_id() {
            self.core_by_conductor.insert(conductor, core.id());
        }
        Ok(())
    }
    pub fn insert_strip(&mut self, strip: &TerminalStrip) -> Result<()> {
        let mut seen = BTreeSet::new();
        for &terminal in strip.terminals() {
            if self.strip_by_terminal.contains_key(&terminal) || !seen.insert(terminal) {
                return Err(DomainError::TerminalAlreadyAssigned(terminal));
            }
        }
        for &terminal in strip.terminals() {
            self.strip_by_terminal.insert(terminal, strip.id());
        }
        Ok(())
    }
    /// A contact belongs to one connector at most and never to a strip too.
    /// Strips are indexed first by `build_physical_index`.
    pub fn insert_connector(&mut self, connector: &Connector) -> Result<()> {
        for &terminal in connector.contacts() {
            if self.strip_by_terminal.contains_key(&terminal)
                || self.connector_by_terminal.contains_key(&terminal)
            {
                return Err(DomainError::TerminalAlreadyAssigned(terminal));
            }
            self.connector_by_terminal.insert(terminal, connector.id());
        }
        Ok(())
    }
    pub fn insert_mating(&mut self, mating: &ConnectorMating) -> Result<()> {
        for connector in [mating.first(), mating.second()] {
            if self.mating_by_connector.contains_key(&connector) {
                return Err(DomainError::InvalidRelation(
                    "conector já possui acoplamento",
                ));
            }
            self.mating_by_connector.insert(connector, mating.id());
        }
        Ok(())
    }
    pub fn link_connection(&mut self, conductor: ConductorId, connection: ConnectionId) {
        self.conductors_by_connection
            .entry(connection)
            .or_default()
            .insert(conductor);
    }
    pub fn unlink_connection(&mut self, conductor: ConductorId, connection: ConnectionId) {
        if let Some(ids) = self.conductors_by_connection.get_mut(&connection) {
            ids.remove(&conductor);
            if ids.is_empty() {
                self.conductors_by_connection.remove(&connection);
            }
        }
    }
}
