// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Cable identity and core membership; start with Cable.
use super::*;
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;

/// Cable with nominal capacity; query Project for registered cores. Partial registration is valid
/// engineering state and creates no continuity between cores.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cable {
    id: CableId,
    project_id: ProjectId,
    designation: DesignationPart,
    #[serde(default)]
    description: Option<Text>,
    manufacturer: Option<Text>,
    catalog_code: Option<Text>,
    kind: CableKind,
    nominal_core_count: NonZeroU32,
    pub(in crate::domain) properties: TechnicalProperties,
}
impl Cable {
    /// Create a detached cable with fresh identity and a nonzero nominal core count.
    pub fn new(
        project_id: ProjectId,
        designation: DesignationPart,
        kind: CableKind,
        nominal_core_count: u32,
    ) -> Result<Self> {
        let nominal_core_count = NonZeroU32::new(nominal_core_count)
            .ok_or(DomainError::InvalidNumber("quantidade de veias"))?;
        Ok(Self {
            id: CableId::new(),
            project_id,
            designation,
            description: None,
            manufacturer: None,
            catalog_code: None,
            kind,
            nominal_core_count,
            properties: TechnicalProperties::default(),
        })
    }
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> CableId {
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
    /// Return the optional description; absence represents an unset value or relation.
    pub fn description(&self) -> Option<&Text> {
        self.description.as_ref()
    }
    /// Return the optional manufacturer; absence represents an unset value or relation.
    pub fn manufacturer(&self) -> Option<&Text> {
        self.manufacturer.as_ref()
    }
    /// Return the optional catalog code; absence represents an unset value or relation.
    pub fn catalog_code(&self) -> Option<&Text> {
        self.catalog_code.as_ref()
    }
    /// Return the kind.
    pub fn kind(&self) -> CableKind {
        self.kind
    }
    /// Return the nominal core count.
    pub fn nominal_core_count(&self) -> u32 {
        self.nominal_core_count.get()
    }
    /// Return the properties.
    pub fn properties(&self) -> &TechnicalProperties {
        &self.properties
    }
    /// Return this value with manufacturer replaced.
    pub fn with_manufacturer(mut self, value: Text) -> Self {
        self.manufacturer = Some(value);
        self
    }
    /// Return this value with designation replaced.
    pub fn with_designation(mut self, value: DesignationPart) -> Self {
        self.designation = value;
        self
    }
    /// Return this value with description replaced.
    pub fn with_description(mut self, value: Text) -> Self {
        self.description = Some(value);
        self
    }
    /// Return this value with description replaced.
    pub fn with_description_option(mut self, value: Option<Text>) -> Self {
        self.description = value;
        self
    }
    /// Return this value with manufacturer replaced.
    pub fn with_manufacturer_option(mut self, value: Option<Text>) -> Self {
        self.manufacturer = value;
        self
    }
    /// Return this value with catalog code replaced.
    pub fn with_catalog_code_option(mut self, value: Option<Text>) -> Self {
        self.catalog_code = value;
        self
    }
    /// Return this value with kind replaced.
    pub fn with_kind(mut self, value: CableKind) -> Self {
        self.kind = value;
        self
    }
    /// Return this value with nominal core count replaced, rejecting invalid input.
    pub fn with_nominal_core_count(mut self, value: u32) -> Result<Self> {
        self.nominal_core_count =
            NonZeroU32::new(value).ok_or(DomainError::InvalidNumber("quantidade de veias"))?;
        Ok(self)
    }
    /// Return this value with catalog code replaced.
    pub fn with_catalog_code(mut self, value: Text) -> Self {
        self.catalog_code = Some(value);
        self
    }
    /// Return this value with properties replaced.
    pub fn with_properties(mut self, value: TechnicalProperties) -> Self {
        self.properties = value;
        self
    }
}
