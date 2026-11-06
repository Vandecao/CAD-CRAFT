// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Connectivity queries and derived networks; start with Project::networks.
use super::*;
use crate::graph::ConnectivityGraph;
use std::collections::BTreeSet;

impl Project {
    /// Explicit transitive connectivity. Missing IDs are errors, including a self-query. An existing
    /// point connects to itself even when isolated.
    pub fn are_connected(
        &self,
        source: ConnectionPointId,
        target: ConnectionPointId,
    ) -> Result<bool> {
        self.require(EntityRef::ConnectionPoint(source))?;
        self.require(EntityRef::ConnectionPoint(target))?;
        Ok(self.graph.are_connected(source, target))
    }

    /// Direct neighbors in ID order, excluding the point itself.
    pub fn directly_connected_points(
        &self,
        point: ConnectionPointId,
    ) -> Result<Vec<ConnectionPointId>> {
        self.require(EntityRef::ConnectionPoint(point))?;
        Ok(self.graph.neighbors(point).map(|(id, _)| id).collect())
    }

    /// Incident edges in ConnectionId order.
    pub fn incident_connections(&self, point: ConnectionPointId) -> Result<Vec<ConnectionId>> {
        self.require(EntityRef::ConnectionPoint(point))?;
        let mut edges: Vec<_> = self.graph.neighbors(point).map(|(_, id)| id).collect();
        edges.sort_unstable();
        Ok(edges)
    }

    /// Networks ordered by their smallest point, including isolated points. Recomputed per call without
    /// caching or persistent identity.
    pub fn networks(&self) -> Vec<Network> {
        let components = self.components_by_function();
        let mut visited = BTreeSet::new();
        let mut networks = Vec::new();
        for &point in self.connection_points.keys() {
            if !visited.contains(&point) {
                let (points, connections) = self.graph.component(point, &mut visited);
                networks.push(self.describe_network(points, connections, &components));
            }
        }
        networks
    }

    /// Derive the complete logical network containing the registered point.
    pub fn network_for_point(&self, point: ConnectionPointId) -> Result<Network> {
        self.require(EntityRef::ConnectionPoint(point))?;
        let (points, connections) = self.graph.component(point, &mut BTreeSet::new());
        Ok(self.describe_network(points, connections, &self.components_by_function()))
    }

    /// Whether any network contains points of both devices. A self-query is true only if the device has
    /// a point. Does not imply internal continuity or connection of every pole.
    pub fn are_devices_connected(&self, source: DeviceId, target: DeviceId) -> Result<bool> {
        self.require(EntityRef::Device(source))?;
        self.require(EntityRef::Device(target))?;
        // Only topology and owners are needed; component enrichment is unnecessary.
        let mut visited = BTreeSet::new();
        for &point in self.connection_points.keys() {
            if visited.contains(&point) {
                continue;
            }
            let (points, _) = self.graph.component(point, &mut visited);
            let devices: BTreeSet<_> = points
                .iter()
                .filter_map(|id| self.connection_points.get(id))
                .filter_map(|point| self.functions.get(&point.function_id()))
                .map(Function::device_id)
                .collect();
            if devices.contains(&source) && devices.contains(&target) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn components_by_function(&self) -> BTreeMap<FunctionId, Vec<ComponentId>> {
        let mut index: BTreeMap<FunctionId, Vec<ComponentId>> = BTreeMap::new();
        for component in self.components.values() {
            if let Some(function) = component.function_id() {
                index.entry(function).or_default().push(component.id());
            }
        }
        index
    }

    fn describe_network(
        &self,
        points: Vec<ConnectionPointId>,
        connections: Vec<ConnectionId>,
        components_by_function: &BTreeMap<FunctionId, Vec<ComponentId>>,
    ) -> Network {
        let mut functions = BTreeSet::new();
        let mut devices = BTreeSet::new();
        let mut components = BTreeSet::new();
        // Aggregate insertion and import guarantee all references exist.
        for point in points
            .iter()
            .filter_map(|id| self.connection_points.get(id))
        {
            functions.insert(point.function_id());
            if let Some(terminal) = point.terminal_id().and_then(|id| self.terminals.get(&id))
                && let TerminalOwner::Component(component) = terminal.owner()
            {
                components.insert(component);
            }
        }
        for function in &functions {
            if let Some(function) = self.functions.get(function) {
                devices.insert(function.device_id());
            }
            if let Some(associated) = components_by_function.get(function) {
                components.extend(associated);
            }
        }
        Network {
            points,
            connections,
            functions: functions.into_iter().collect(),
            devices: devices.into_iter().collect(),
            components: components.into_iter().collect(),
        }
    }

    /// Find the connection by its stable identifier; return None when absent.
    pub fn connection(&self, id: ConnectionId) -> Option<&Connection> {
        self.connections.get(&id)
    }

    /// Connections in ascending ID order without mutable aggregate access.
    pub fn connections(&self) -> impl Iterator<Item = &Connection> {
        self.connections.values()
    }

    /// Insert an explicit edge; indirect connectivity does not prevent cycles.
    pub fn insert_connection(&mut self, connection: Connection) -> Result<ConnectionId> {
        let id = connection.id();
        if self.connections.contains_key(&id) {
            return Err(DomainError::DuplicateConnectionId(id));
        }
        self.check_connection(&connection)?;
        self.graph.insert(&connection)?;
        self.connections.insert(id, connection);
        Ok(id)
    }

    /// Remove only the edge, retaining points, functions, terminals and devices.
    pub fn remove_connection(&mut self, id: ConnectionId) -> Result<Connection> {
        if self.drawing_owns_connection(id)
            || self.marker_owns_connection(id)
            || self.terminal_bridges.contains_key(&id)
        {
            return Err(DomainError::ConnectionInUse(id));
        }
        if self
            .document_index
            .by_target
            .contains_key(&RepresentationTarget::Connection(id))
        {
            return Err(DomainError::ConnectionInUse(id));
        }
        if self
            .physical_index
            .conductors_by_connection
            .contains_key(&id)
        {
            return Err(DomainError::ConnectionInUse(id));
        }
        let connection = self
            .connections
            .remove(&id)
            .ok_or(DomainError::MissingConnection(id))?;
        self.graph.remove(&connection);
        Ok(connection)
    }

    fn check_connection(&self, connection: &Connection) -> Result<()> {
        self.check_project(connection.project_id())?;
        let (source, target) = connection.endpoints();
        self.require(EntityRef::ConnectionPoint(source))?;
        self.require(EntityRef::ConnectionPoint(target))
    }

    pub(in crate::domain::project) fn build_graph(&self) -> Result<ConnectivityGraph> {
        let mut graph = ConnectivityGraph::default();
        for connection in self.connections.values() {
            self.check_connection(connection)?;
            graph.insert(connection)?;
        }
        Ok(graph)
    }
}
