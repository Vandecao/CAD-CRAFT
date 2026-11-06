// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Derived connectivity groups; start with Network and its read-only accessors.
use super::{ComponentId, ConnectionId, ConnectionPointId, DeviceId, FunctionId};

/// Snapshot of a connected component of the logical graph. Has no persistent identity,
/// serialization or mutators; query Project again after edits. Members are sorted and unique.
/// Function, device and component associations do not create internal connectivity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Network {
    pub(in crate::domain) points: Vec<ConnectionPointId>,
    pub(in crate::domain) connections: Vec<ConnectionId>,
    pub(in crate::domain) functions: Vec<FunctionId>,
    pub(in crate::domain) devices: Vec<DeviceId>,
    pub(in crate::domain) components: Vec<ComponentId>,
}

impl Network {
    /// Return the points in their stored order.
    pub fn points(&self) -> &[ConnectionPointId] {
        &self.points
    }
    /// Return the connections in their stored order.
    pub fn connections(&self) -> &[ConnectionId] {
        &self.connections
    }
    /// Return the functions in their stored order.
    pub fn functions(&self) -> &[FunctionId] {
        &self.functions
    }
    /// Return the devices in their stored order.
    pub fn devices(&self) -> &[DeviceId] {
        &self.devices
    }

    /// Components explicitly assigned to the function or owning terminals mapped by its points. Device
    /// assignment alone is insufficient.
    pub fn components(&self) -> &[ComponentId] {
        &self.components
    }

    /// A simple connected graph has a cycle when |E| >= |V|. Topological cycles are allowed and are not
    /// electrical/ERC diagnoses.
    pub fn has_cycle(&self) -> bool {
        self.connections.len() >= self.points.len()
    }
}
