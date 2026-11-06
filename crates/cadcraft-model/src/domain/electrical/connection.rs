// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Logical edges between connection points; start with Connection::new.
use super::{ConnectionId, ConnectionPointId, ProjectId, Text};
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};

/// Undirected logical edge. Project validates endpoint existence and membership. Connections have
/// neither physical properties nor EntityRef references, and their IDs differ from point IDs:
/// ```compile_fail
/// # use cadcraft_model as cad_craft;
/// use cad_craft::domain::{ConnectionId, ConnectionPointId};
/// let point: ConnectionPointId = ConnectionId::new();
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "ConnectionData")]
pub struct Connection {
    id: ConnectionId,
    project_id: ProjectId,
    source: ConnectionPointId,
    target: ConnectionPointId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(in crate::domain) number: Option<Text>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(in crate::domain) number_locked: bool,
}

impl Connection {
    /// Create a logical edge with fresh identity; reject identical endpoints.
    pub fn new(
        project_id: ProjectId,
        source: ConnectionPointId,
        target: ConnectionPointId,
    ) -> Result<Self> {
        Self::try_from(ConnectionData {
            id: ConnectionId::new(),
            project_id,
            source,
            target,
            number: None,
            number_locked: false,
        })
    }

    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> ConnectionId {
        self.id
    }
    /// Return the project identifier.
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Endpoints in ascending ID order, independent of current direction.
    pub fn endpoints(&self) -> (ConnectionPointId, ConnectionPointId) {
        (self.source, self.target)
    }
    /// Return the documentary number, when assigned.
    pub fn number(&self) -> Option<&Text> {
        self.number.as_ref()
    }
    /// Whether automatic numbering must preserve the assigned wire number.
    pub fn number_locked(&self) -> bool {
        self.number_locked
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConnectionData {
    id: ConnectionId,
    project_id: ProjectId,
    source: ConnectionPointId,
    target: ConnectionPointId,
    #[serde(default)]
    number: Option<Text>,
    #[serde(default)]
    number_locked: bool,
}

impl TryFrom<ConnectionData> for Connection {
    type Error = DomainError;

    fn try_from(data: ConnectionData) -> Result<Self> {
        if data.source == data.target {
            return Err(DomainError::SelfConnection(data.source));
        }
        Ok(Self {
            id: data.id,
            project_id: data.project_id,
            source: data.source.min(data.target),
            target: data.source.max(data.target),
            number: data.number,
            number_locked: data.number_locked,
        })
    }
}
