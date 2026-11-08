// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Electrical functions belonging to devices; start with Function and FunctionKind.
use super::*;
use serde::{Deserialize, Serialize};
/// Declared electrical purpose of a device function.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FunctionKind {
    /// Coil function.
    Coil,
    /// Auxiliary contact function.
    AuxiliaryContact,
    /// Main contact function.
    MainContact,
    /// User-defined documentary category.
    Other(Text),
}
impl FunctionKind {
    /// Return the pt-BR display label (the user text for `Other`).
    pub fn label(&self) -> &str {
        match self {
            Self::Coil => "Bobina",
            Self::AuxiliaryContact => "Contato auxiliar",
            Self::MainContact => "Contato principal",
            Self::Other(t) => t.as_str(),
        }
    }
}
/// Explicitly assigned role independent of geometry and function kind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FunctionRole {
    /// No explicit classification supplied.
    #[default]
    Unspecified,
    /// Explicit main function.
    Main,
    /// Explicit auxiliary function.
    Auxiliary,
}
/// Detached entity; Project validates relations on insertion or import.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Function {
    pub(in crate::domain) id: FunctionId,
    pub(in crate::domain) project_id: ProjectId,
    pub(in crate::domain) device_id: DeviceId,
    pub(in crate::domain) designation: Designation,
    pub(in crate::domain) kind: FunctionKind,
    #[serde(default)]
    pub(in crate::domain) role: FunctionRole,
    pub(in crate::domain) properties: TechnicalProperties,
}
impl Function {
    /// Create a detached electrical function owned by the supplied device.
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
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> FunctionId {
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
    /// Return the device identifier.
    pub fn device_id(&self) -> DeviceId {
        self.device_id
    }
    /// Return the designation.
    pub fn designation(&self) -> &Designation {
        &self.designation
    }
    /// Return the kind.
    pub fn kind(&self) -> &FunctionKind {
        &self.kind
    }
    /// Return the role.
    pub fn role(&self) -> FunctionRole {
        self.role
    }
    /// Return this value with role replaced.
    pub fn with_role(mut self, role: FunctionRole) -> Self {
        self.role = role;
        self
    }
}
