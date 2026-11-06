// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Rebuildable private connectivity index; start with ConnectivityGraph. Only connections are persistent.
use crate::{
    domain::{Connection, ConnectionId, ConnectionPointId},
    error::{DomainError, Result},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ConnectivityGraph {
    adjacency: BTreeMap<ConnectionPointId, BTreeMap<ConnectionPointId, ConnectionId>>,
}

impl ConnectivityGraph {
    pub(crate) fn neighbors(
        &self,
        point: ConnectionPointId,
    ) -> impl Iterator<Item = (ConnectionPointId, ConnectionId)> + '_ {
        self.adjacency
            .get(&point)
            .into_iter()
            .flat_map(|neighbors| neighbors.iter().map(|(&point, &edge)| (point, edge)))
    }

    /// Iterative DFS marks vertices on discovery to bound the stack by O(V). A missing index entry
    /// means an isolated vertex; Project validates its ID.
    pub(crate) fn component(
        &self,
        start: ConnectionPointId,
        visited: &mut BTreeSet<ConnectionPointId>,
    ) -> (Vec<ConnectionPointId>, Vec<ConnectionId>) {
        let mut stack = vec![start];
        let mut points = Vec::new();
        let mut edges = BTreeSet::new();
        visited.insert(start);
        while let Some(point) = stack.pop() {
            points.push(point);
            for (neighbor, edge) in self.neighbors(point) {
                edges.insert(edge);
                if visited.insert(neighbor) {
                    stack.push(neighbor);
                }
            }
        }
        points.sort_unstable();
        (points, edges.into_iter().collect())
    }

    pub(crate) fn are_connected(
        &self,
        source: ConnectionPointId,
        target: ConnectionPointId,
    ) -> bool {
        let mut visited = BTreeSet::from([source]);
        let mut stack = vec![source];
        while let Some(point) = stack.pop() {
            if point == target {
                return true;
            }
            for (neighbor, _) in self.neighbors(point) {
                if visited.insert(neighbor) {
                    stack.push(neighbor);
                }
            }
        }
        false
    }

    /// Check for duplicates before writing either incidence.
    pub(crate) fn insert(&mut self, connection: &Connection) -> Result<()> {
        let (source, target) = connection.endpoints();
        if self
            .adjacency
            .get(&source)
            .is_some_and(|neighbors| neighbors.contains_key(&target))
        {
            return Err(DomainError::DuplicateConnection { source, target });
        }
        self.adjacency
            .entry(source)
            .or_default()
            .insert(target, connection.id());
        self.adjacency
            .entry(target)
            .or_default()
            .insert(source, connection.id());
        Ok(())
    }

    pub(crate) fn remove(&mut self, connection: &Connection) {
        let (source, target) = connection.endpoints();
        for (a, b) in [(source, target), (target, source)] {
            if let Some(neighbors) = self.adjacency.get_mut(&a) {
                neighbors.remove(&b);
                if neighbors.is_empty() {
                    self.adjacency.remove(&a);
                }
            }
        }
    }
}
