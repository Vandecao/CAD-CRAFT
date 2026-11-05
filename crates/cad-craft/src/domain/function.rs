// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FunctionKind {
    Coil,
    AuxiliaryContact,
    MainContact,
    Other(Text),
}
/// Papel explicitamente atribuído, independente da geometria e do tipo funcional.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FunctionRole {
    #[default]
    Unspecified,
    Main,
    Auxiliary,
}
/// Entidade destacada; relações são validadas por Project na inserção/importação.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Function {
    pub(super) id: FunctionId,
    pub(super) project_id: ProjectId,
    pub(super) device_id: DeviceId,
    pub(super) designation: Designation,
    pub(super) kind: FunctionKind,
    #[serde(default)]
    pub(super) role: FunctionRole,
    pub(super) properties: TechnicalProperties,
}
impl Function {
    pub fn new(
        project_id: ProjectId,
        device_id: DeviceId,
        designation: Designation,
        kind: FunctionKind,
    ) -> Self {
        Self {
            id: FunctionId::new(),
            project_id,
            device_id,
            designation,
            kind,
            role: FunctionRole::Unspecified,
            properties: TechnicalProperties::default(),
        }
    }
    pub fn id(&self) -> FunctionId {
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
    pub fn device_id(&self) -> DeviceId {
        self.device_id
    }
    pub fn designation(&self) -> &Designation {
        &self.designation
    }
    pub fn kind(&self) -> &FunctionKind {
        &self.kind
    }
    pub fn role(&self) -> FunctionRole {
        self.role
    }
    pub fn with_role(mut self, role: FunctionRole) -> Self {
        self.role = role;
        self
    }
}
