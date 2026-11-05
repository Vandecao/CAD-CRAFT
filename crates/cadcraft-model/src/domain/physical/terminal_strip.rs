// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Terminal strip identity and ordering; start with TerminalStrip.
use super::*;
use serde::{Deserialize, Serialize};

/// Agrupamento físico ordenado, sem continuidade implícita nem mudança de proprietário.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminalStrip {
    id: TerminalStripId,
    project_id: ProjectId,
    designation: DesignationPart,
    location: Option<Text>,
    pub(in crate::domain) terminals: Vec<TerminalId>,
    pub(in crate::domain) properties: TechnicalProperties,
}
impl TerminalStrip {
    pub fn new(project_id: ProjectId, designation: DesignationPart) -> Self {
        Self {
            id: TerminalStripId::new(),
            project_id,
            designation,
            location: None,
            terminals: Vec::new(),
            properties: TechnicalProperties::default(),
        }
    }
    pub fn id(&self) -> TerminalStripId {
        self.id
    }
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }
    pub fn designation(&self) -> &DesignationPart {
        &self.designation
    }
    pub fn location(&self) -> Option<&Text> {
        self.location.as_ref()
    }
    pub fn terminals(&self) -> &[TerminalId] {
        &self.terminals
    }
    pub fn properties(&self) -> &TechnicalProperties {
        &self.properties
    }
    pub fn with_location(mut self, value: Text) -> Self {
        self.location = Some(value);
        self
    }
    pub fn with_properties(mut self, value: TechnicalProperties) -> Self {
        self.properties = value;
        self
    }
    /// Replaces the explicit display order. Project validates membership and
    /// uniqueness transactionally; this builder does not imply continuity.
    pub fn with_terminals(mut self, terminals: Vec<TerminalId>) -> Self {
        self.terminals = terminals;
        self
    }
    pub fn with_designation(mut self, designation: DesignationPart) -> Self {
        self.designation = designation;
        self
    }
}
