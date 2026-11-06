// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Terminal strip identity and ordering; start with TerminalStrip.
use super::*;
use serde::{Deserialize, Serialize};

/// Ordered physical grouping with neither implicit continuity nor ownership changes.
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
    /// Create an empty detached terminal strip with fresh identity.
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
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> TerminalStripId {
        self.id
    }
    /// Return the project identifier.
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }
    /// Return the designation.
    pub fn designation(&self) -> &DesignationPart {
        &self.designation
    }
    /// Return the optional location; absence represents an unset value or relation.
    pub fn location(&self) -> Option<&Text> {
        self.location.as_ref()
    }
    /// Return the terminals in their stored order.
    pub fn terminals(&self) -> &[TerminalId] {
        &self.terminals
    }
    /// Return the properties.
    pub fn properties(&self) -> &TechnicalProperties {
        &self.properties
    }
    /// Return this value with location replaced.
    pub fn with_location(mut self, value: Text) -> Self {
        self.location = Some(value);
        self
    }
    /// Return this value with properties replaced.
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
    /// Return this value with designation replaced.
    pub fn with_designation(mut self, designation: DesignationPart) -> Self {
        self.designation = designation;
        self
    }
}
