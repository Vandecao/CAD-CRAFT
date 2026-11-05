// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Explicit circuit annotations. Geometry belongs to the existing Representation.
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerKind {
    Interruption,
    Potential {
        kind: Option<Text>,
        description: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ElectricalMarker {
    id: RepresentationId,
    name: Text,
    kind: MarkerKind,
    point: Option<ConnectionPointId>,
}
impl ElectricalMarker {
    pub fn new(
        id: RepresentationId,
        name: Text,
        kind: MarkerKind,
        point: Option<ConnectionPointId>,
    ) -> Self {
        Self {
            id,
            name,
            kind,
            point,
        }
    }
    pub fn id(&self) -> RepresentationId {
        self.id
    }
    pub fn name(&self) -> &Text {
        &self.name
    }
    pub fn kind(&self) -> &MarkerKind {
        &self.kind
    }
    pub fn point(&self) -> Option<ConnectionPointId> {
        self.point
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferenceMode {
    Star,
    Chain,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkerGroup {
    pub(in crate::domain) id: ReferenceGroupId,
    pub(in crate::domain) mode: ReferenceMode,
    pub(in crate::domain) members: Vec<RepresentationId>,
    pub(in crate::domain) continuity: bool,
    pub(in crate::domain) connections: Vec<ConnectionId>,
}
impl MarkerGroup {
    pub fn id(&self) -> ReferenceGroupId {
        self.id
    }
    pub fn mode(&self) -> ReferenceMode {
        self.mode
    }
    pub fn members(&self) -> &[RepresentationId] {
        &self.members
    }
    pub fn continuity(&self) -> bool {
        self.continuity
    }
    pub fn connections(&self) -> &[ConnectionId] {
        &self.connections
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PotentialConflict {
    pub first: RepresentationId,
    pub second: RepresentationId,
}

#[derive(Clone, Debug)]
pub enum MarkerEdit {
    Insert {
        marker: ElectricalMarker,
        representation: Box<Representation>,
        definition: Box<SymbolDefinition>,
    },
    Update {
        id: RepresentationId,
        name: Text,
        kind: MarkerKind,
        point: Option<ConnectionPointId>,
    },
    Delete {
        id: RepresentationId,
    },
    Move {
        id: RepresentationId,
        page: PageId,
        transform: Transform2D,
    },
    SetGroup {
        id: ReferenceGroupId,
        mode: ReferenceMode,
        members: Vec<RepresentationId>,
        continuity: bool,
    },
    DeleteGroup {
        id: ReferenceGroupId,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub(in crate::domain) struct MarkerChange<K, V> {
    pub key: K,
    pub before: Option<V>,
    pub after: Option<V>,
}
/// State-checked records of only entities changed by one explicit operation.
#[derive(Clone, Debug, PartialEq)]
pub struct MarkerDelta {
    pub(in crate::domain) project: ProjectId,
    pub(in crate::domain) markers: Vec<MarkerChange<RepresentationId, ElectricalMarker>>,
    pub(in crate::domain) groups: Vec<MarkerChange<ReferenceGroupId, MarkerGroup>>,
    pub(in crate::domain) representations: Vec<MarkerChange<RepresentationId, Representation>>,
    pub(in crate::domain) definitions: Vec<MarkerChange<SymbolDefinitionId, SymbolDefinition>>,
    pub(in crate::domain) bindings: Vec<MarkerChange<PortRef, ConnectionPointId>>,
    pub(in crate::domain) connections: Vec<MarkerChange<ConnectionId, Connection>>,
}
impl MarkerDelta {
    pub(in crate::domain) fn new(project: ProjectId) -> Self {
        Self {
            project,
            markers: vec![],
            groups: vec![],
            representations: vec![],
            definitions: vec![],
            bindings: vec![],
            connections: vec![],
        }
    }
    pub fn len(&self) -> usize {
        self.markers.len()
            + self.groups.len()
            + self.representations.len()
            + self.definitions.len()
            + self.bindings.len()
            + self.connections.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
