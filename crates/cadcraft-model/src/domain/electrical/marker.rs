// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Explicit circuit annotations. Geometry belongs to the existing Representation.
use super::*;
use serde::{Deserialize, Serialize};

/// Explicit circuit annotation, such as an interruption or named potential.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerKind {
    /// Explicit circuit interruption marker.
    Interruption,
    /// Named potential declaration.
    Potential {
        /// Declared category of this record or operation.
        kind: Option<Text>,
        /// Descriptive text.
        description: String,
    },
}
/// Named circuit annotation optionally associated with a connection point.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ElectricalMarker {
    id: RepresentationId,
    name: Text,
    kind: MarkerKind,
    point: Option<ConnectionPointId>,
}
impl ElectricalMarker {
    /// Create a named circuit annotation with fresh identity and an optional connection point.
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
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> RepresentationId {
        self.id
    }
    /// Return the name.
    pub fn name(&self) -> &Text {
        &self.name
    }
    /// Return the kind.
    pub fn kind(&self) -> &MarkerKind {
        &self.kind
    }
    /// Return the optional point; absence represents an unset value or relation.
    pub fn point(&self) -> Option<ConnectionPointId> {
        self.point
    }
}
/// Explicit connectivity arrangement within a marker reference group.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferenceMode {
    /// Connect group members to the first member.
    Star,
    /// Connect adjacent members in order.
    Chain,
}
/// Explicit marker grouping and the logical connections it owns.
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
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> ReferenceGroupId {
        self.id
    }
    /// Return the mode.
    pub fn mode(&self) -> ReferenceMode {
        self.mode
    }
    /// Return the members in their stored order.
    pub fn members(&self) -> &[RepresentationId] {
        &self.members
    }
    /// Whether this marker group owns logical continuity connections.
    pub fn continuity(&self) -> bool {
        self.continuity
    }
    /// Return the connections in their stored order.
    pub fn connections(&self) -> &[ConnectionId] {
        &self.connections
    }
}
/// Two contradictory potential declarations within one logical network.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PotentialConflict {
    /// First member of the explicit pair.
    pub first: RepresentationId,
    /// Second member of the explicit pair.
    pub second: RepresentationId,
}

/// Reversible marker occurrence, definition or reference-group edit.
#[derive(Clone, Debug)]
pub enum MarkerEdit {
    /// Insert the supplied occurrence and related data.
    Insert {
        /// Circuit marker data.
        marker: ElectricalMarker,
        /// Graphical occurrence associated with this operation.
        representation: Box<Representation>,
        /// Symbol definition registered for this occurrence.
        definition: Box<SymbolDefinition>,
    },
    /// Replace the selected record with validated data.
    Update {
        /// Stable identifier of the affected object.
        id: RepresentationId,
        /// Human-readable name.
        name: Text,
        /// Declared category of this record or operation.
        kind: MarkerKind,
        /// Explicit functional connection point.
        point: Option<ConnectionPointId>,
    },
    /// Remove the selected occurrence and its owned relations.
    Delete {
        /// Stable identifier of the affected object.
        id: RepresentationId,
    },
    /// Move the selected occurrence.
    Move {
        /// Stable identifier of the affected object.
        id: RepresentationId,
        /// Page identifier used by this operation or filter.
        page: PageId,
        /// Validated placement transform in page coordinates.
        transform: Transform2D,
    },
    /// Set group.
    SetGroup {
        /// Stable identifier of the affected object.
        id: ReferenceGroupId,
        /// Explicit group connectivity arrangement.
        mode: ReferenceMode,
        /// Explicit member occurrence identifiers.
        members: Vec<RepresentationId>,
        /// Whether this group owns logical continuity connections.
        continuity: bool,
    },
    /// Delete group.
    DeleteGroup {
        /// Stable identifier of the affected object.
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
    /// Return the number of contained entries or entity changes recorded by this value.
    pub fn len(&self) -> usize {
        self.markers.len()
            + self.groups.len()
            + self.representations.len()
            + self.definitions.len()
            + self.bindings.len()
            + self.connections.len()
    }
    /// Whether this collection or recorded edit contains no effective entries or changes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
