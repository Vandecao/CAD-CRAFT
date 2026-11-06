// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Logical device identity independent of graphics; start with Device::new.
use super::*;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Detached entity; Project validates relations on insertion or import.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    pub(in crate::domain) id: DeviceId,
    pub(in crate::domain) project_id: ProjectId,
    pub(in crate::domain) designation: Designation,
    pub(in crate::domain) classification: Text,
    pub(in crate::domain) properties: TechnicalProperties,
    #[serde(default)]
    pub(in crate::domain) designation_locked: bool,
}
impl Device {
    /// Create a detached logical device with fresh identity and validated classification.
    pub fn new(
        project_id: ProjectId,
        designation: Designation,
        classification: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            id: DeviceId::new(),
            project_id,
            designation,
            classification: Text::new(classification)?,
            properties: TechnicalProperties::default(),
            designation_locked: false,
        })
    }
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> DeviceId {
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
    /// Return the designation.
    pub fn designation(&self) -> &Designation {
        &self.designation
    }
    /// Return the classification.
    pub fn classification(&self) -> &Text {
        &self.classification
    }
    /// Whether automatic numbering must preserve this device designation.
    pub fn designation_locked(&self) -> bool {
        self.designation_locked
    }
}
