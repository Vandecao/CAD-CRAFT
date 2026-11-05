// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Physical component and optional catalog part; start with Component::new.
use super::*;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Entidade destacada; relações são validadas por Project na inserção/importação.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub(in crate::domain) id: ComponentId,
    pub(in crate::domain) project_id: ProjectId,
    designation: Designation,
    classification: Text,
    manufacturer: Option<Text>,
    catalog_code: Option<Text>,
    device_id: Option<DeviceId>,
    function_id: Option<FunctionId>,
    pub(in crate::domain) properties: TechnicalProperties,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(in crate::domain) part_id: Option<PartId>,
    #[serde(default = "default_quantity", skip_serializing_if = "is_one")]
    pub(in crate::domain) quantity: u32,
}
fn default_quantity() -> u32 {
    1
}
fn is_one(value: &u32) -> bool {
    *value == 1
}
impl Component {
    pub fn new(
        project_id: ProjectId,
        designation: Designation,
        classification: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            id: ComponentId::new(),
            project_id,
            designation,
            classification: Text::new(classification)?,
            manufacturer: None,
            catalog_code: None,
            device_id: None,
            function_id: None,
            properties: TechnicalProperties::default(),
            part_id: None,
            quantity: 1,
        })
    }
    pub fn id(&self) -> ComponentId {
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
    pub fn manufacturer(&self) -> Option<&Text> {
        self.manufacturer.as_ref()
    }
    pub fn catalog_code(&self) -> Option<&Text> {
        self.catalog_code.as_ref()
    }
    pub fn device_id(&self) -> Option<DeviceId> {
        self.device_id
    }
    pub fn function_id(&self) -> Option<FunctionId> {
        self.function_id
    }
    pub fn part_id(&self) -> Option<&PartId> {
        self.part_id.as_ref()
    }
    pub fn quantity(&self) -> u32 {
        self.quantity
    }
    pub fn with_part(mut self, part_id: PartId, quantity: u32) -> Result<Self> {
        if quantity == 0 {
            return Err(crate::error::DomainError::InvalidNumber(
                "quantidade da peça",
            ));
        }
        self.part_id = Some(part_id);
        self.quantity = quantity;
        Ok(self)
    }
    /// A coerência da associação é verificada ao registrar no projeto.
    pub fn with_assignment(
        mut self,
        device: Option<DeviceId>,
        function: Option<FunctionId>,
    ) -> Self {
        self.device_id = device;
        self.function_id = function;
        self
    }
    pub fn with_catalog(
        mut self,
        manufacturer: impl Into<String>,
        code: impl Into<String>,
    ) -> Result<Self> {
        self.manufacturer = Some(Text::new(manufacturer)?);
        self.catalog_code = Some(Text::new(code)?);
        Ok(self)
    }
    pub fn with_manufacturer(mut self, manufacturer: impl Into<String>) -> Result<Self> {
        self.manufacturer = Some(Text::new(manufacturer)?);
        Ok(self)
    }
    pub fn with_catalog_code(mut self, code: impl Into<String>) -> Result<Self> {
        self.catalog_code = Some(Text::new(code)?);
        Ok(self)
    }
}
