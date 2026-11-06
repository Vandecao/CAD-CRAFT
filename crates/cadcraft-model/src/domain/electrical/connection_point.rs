// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Logical function connection points; start with ConnectionPoint.
use super::*;
use serde::{Deserialize, Serialize};

/// Detached entity; Project validates relations on insertion or import.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionPoint {
    pub(in crate::domain) id: ConnectionPointId,
    pub(in crate::domain) project_id: ProjectId,
    function_id: FunctionId,
    pub(in crate::domain) designation: DesignationPart,
    pub(in crate::domain) terminal_id: Option<TerminalId>,
    pub(in crate::domain) properties: TechnicalProperties,
    /// Associated multilevel terminal level; required only for multilevel terminals.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(in crate::domain) terminal_level: Option<TerminalLevelId>,
}
impl ConnectionPoint {
    /// Create a detached connection point owned by the supplied function.
    pub fn new(
        project_id: ProjectId,
        function_id: FunctionId,
        designation: DesignationPart,
    ) -> Self {
        Self {
            id: ConnectionPointId::new(),
            project_id,
            function_id,
            designation,
            terminal_id: None,
            properties: TechnicalProperties::default(),
            terminal_level: None,
        }
    }
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> ConnectionPointId {
        self.id
    }
    /// Return the project identifier.
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }
    /// Return the properties.
    pub fn properties(&self) -> &TechnicalProperties {
        &self.properties
    }
    /// Return this value with properties replaced.
    pub fn with_properties(mut self, properties: TechnicalProperties) -> Self {
        self.properties = properties;
        self
    }
    /// Return the function identifier.
    pub fn function_id(&self) -> FunctionId {
        self.function_id
    }
    /// Return the designation.
    pub fn designation(&self) -> &DesignationPart {
        &self.designation
    }
    /// Return the terminal identifier.
    pub fn terminal_id(&self) -> Option<TerminalId> {
        self.terminal_id
    }
    /// Return the optional terminal level; absence represents an unset value or relation.
    pub fn terminal_level(&self) -> Option<TerminalLevelId> {
        self.terminal_level
    }
    /// Return this value with terminal replaced.
    pub fn with_terminal(mut self, terminal: TerminalId) -> Self {
        self.terminal_id = Some(terminal);
        self
    }
    /// Associate a specific level of a multilevel terminal.
    pub fn with_terminal_level(mut self, terminal: TerminalId, level: TerminalLevelId) -> Self {
        self.terminal_id = Some(terminal);
        self.terminal_level = Some(level);
        self
    }
}
