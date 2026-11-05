// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::*;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Entidade destacada; relações são validadas por Project na inserção/importação.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    pub(super) id: DeviceId,
    pub(super) project_id: ProjectId,
    pub(super) designation: Designation,
    pub(super) classification: Text,
    pub(super) properties: TechnicalProperties,
    #[serde(default)]
    pub(super) designation_locked: bool,
}
impl Device {
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
    pub fn id(&self) -> DeviceId {
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
    pub fn designation(&self) -> &Designation {
        &self.designation
    }
    pub fn classification(&self) -> &Text {
        &self.classification
    }
    pub fn designation_locked(&self) -> bool {
        self.designation_locked
    }
}
