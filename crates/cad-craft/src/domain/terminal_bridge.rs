// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::*;
use serde::{Deserialize, Serialize};

/// Electrical continuity explicitly owned by one terminal strip. Its stable
/// identity is the ID of the logical Connection it owns, never a graphic line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminalBridgeKind {
    Fixed,
    Inserted,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminalBridge {
    project_id: ProjectId,
    strip_id: TerminalStripId,
    connection_id: ConnectionId,
    kind: TerminalBridgeKind,
    properties: TechnicalProperties,
}
impl TerminalBridge {
    pub(super) fn new(
        project_id: ProjectId,
        strip_id: TerminalStripId,
        connection_id: ConnectionId,
        kind: TerminalBridgeKind,
    ) -> Self {
        Self {
            project_id,
            strip_id,
            connection_id,
            kind,
            properties: TechnicalProperties::default(),
        }
    }
    pub fn id(&self) -> ConnectionId {
        self.connection_id
    }
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }
    pub fn strip_id(&self) -> TerminalStripId {
        self.strip_id
    }
    pub fn kind(&self) -> TerminalBridgeKind {
        self.kind
    }
    pub fn properties(&self) -> &TechnicalProperties {
        &self.properties
    }
    pub fn with_kind(mut self, kind: TerminalBridgeKind) -> Self {
        self.kind = kind;
        self
    }
    pub fn with_properties(mut self, properties: TechnicalProperties) -> Self {
        self.properties = properties;
        self
    }
}
