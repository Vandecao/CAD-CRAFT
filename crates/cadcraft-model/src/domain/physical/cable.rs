// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Cable identity and core membership; start with Cable.
use super::*;
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;

/// Cabo com capacidade nominal; veias cadastradas são consultadas no Project.
/// Cadastro parcial é um estado de engenharia válido, sem continuidade entre veias.
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
    pub fn id(&self) -> CableId {
        self.id
    }
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }
    pub fn designation(&self) -> &DesignationPart {
        &self.designation
    }
    pub fn description(&self) -> Option<&Text> {
        self.description.as_ref()
    }
    pub fn manufacturer(&self) -> Option<&Text> {
        self.manufacturer.as_ref()
    }
    pub fn catalog_code(&self) -> Option<&Text> {
        self.catalog_code.as_ref()
    }
    pub fn kind(&self) -> CableKind {
        self.kind
    }
    pub fn nominal_core_count(&self) -> u32 {
        self.nominal_core_count.get()
    }
    pub fn properties(&self) -> &TechnicalProperties {
        &self.properties
    }
    pub fn with_manufacturer(mut self, value: Text) -> Self {
        self.manufacturer = Some(value);
        self
    }
    pub fn with_designation(mut self, value: DesignationPart) -> Self {
        self.designation = value;
        self
    }
    pub fn with_description(mut self, value: Text) -> Self {
        self.description = Some(value);
        self
    }
    pub fn with_description_option(mut self, value: Option<Text>) -> Self {
        self.description = value;
        self
    }
    pub fn with_manufacturer_option(mut self, value: Option<Text>) -> Self {
        self.manufacturer = value;
        self
    }
    pub fn with_catalog_code_option(mut self, value: Option<Text>) -> Self {
        self.catalog_code = value;
        self
    }
    pub fn with_kind(mut self, value: CableKind) -> Self {
        self.kind = value;
        self
    }
    pub fn with_nominal_core_count(mut self, value: u32) -> Result<Self> {
        self.nominal_core_count =
            NonZeroU32::new(value).ok_or(DomainError::InvalidNumber("quantidade de veias"))?;
        Ok(self)
    }
    pub fn with_catalog_code(mut self, value: Text) -> Self {
        self.catalog_code = Some(value);
        self
    }
    pub fn with_properties(mut self, value: TechnicalProperties) -> Self {
        self.properties = value;
        self
    }
}
