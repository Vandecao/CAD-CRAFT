// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::*;
use serde::{Deserialize, Serialize};

/// Entidade destacada; relações são validadas por Project na inserção/importação.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionPoint {
    pub(super) id: ConnectionPointId,
    pub(super) project_id: ProjectId,
    function_id: FunctionId,
    pub(super) designation: DesignationPart,
    pub(super) terminal_id: Option<TerminalId>,
    pub(super) properties: TechnicalProperties,
    /// Nível do borne multinível associado; obrigatório somente nesse caso.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) terminal_level: Option<TerminalLevelId>,
}
impl ConnectionPoint {
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
    pub fn id(&self) -> ConnectionPointId {
        self.id
    }
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }
    pub fn properties(&self) -> &TechnicalProperties {
        &self.properties
    }
    pub fn with_properties(mut self, properties: TechnicalProperties) -> Self {
        self.properties = properties;
        self
    }
    pub fn function_id(&self) -> FunctionId {
        self.function_id
    }
    pub fn designation(&self) -> &DesignationPart {
        &self.designation
    }
    pub fn terminal_id(&self) -> Option<TerminalId> {
        self.terminal_id
    }
    pub fn terminal_level(&self) -> Option<TerminalLevelId> {
        self.terminal_level
    }
    pub fn with_terminal(mut self, terminal: TerminalId) -> Self {
        self.terminal_id = Some(terminal);
        self
    }
    /// Associação a um nível específico de borne multinível.
    pub fn with_terminal_level(mut self, terminal: TerminalId, level: TerminalLevelId) -> Self {
        self.terminal_id = Some(terminal);
        self.terminal_level = Some(level);
        self
    }
}
