// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Physical component and optional catalog part; start with Component::new.
use super::*;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Detached entity; Project validates relations on insertion or import.
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
    /// Create a detached physical component with fresh identity and validated classification.
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
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> ComponentId {
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
    /// Return the optional manufacturer; absence represents an unset value or relation.
    pub fn manufacturer(&self) -> Option<&Text> {
        self.manufacturer.as_ref()
    }
    /// Return the optional catalog code; absence represents an unset value or relation.
    pub fn catalog_code(&self) -> Option<&Text> {
        self.catalog_code.as_ref()
    }
    /// Return the device identifier.
    pub fn device_id(&self) -> Option<DeviceId> {
        self.device_id
    }
    /// Return the function identifier.
    pub fn function_id(&self) -> Option<FunctionId> {
        self.function_id
    }
    /// Return the part identifier.
    pub fn part_id(&self) -> Option<&PartId> {
        self.part_id.as_ref()
    }
    /// Return the quantity.
    pub fn quantity(&self) -> u32 {
        self.quantity
    }
    /// Assign a catalog part and nonzero quantity to the detached component.
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
    /// Project validates assignment consistency when registering the component.
    pub fn with_assignment(
        mut self,
        device: Option<DeviceId>,
        function: Option<FunctionId>,
    ) -> Self {
        self.device_id = device;
        self.function_id = function;
        self
    }
    /// Set validated manufacturer and catalog-code text on this component.
    pub fn with_catalog(
        mut self,
        manufacturer: impl Into<String>,
        code: impl Into<String>,
    ) -> Result<Self> {
        self.manufacturer = Some(Text::new(manufacturer)?);
        self.catalog_code = Some(Text::new(code)?);
        Ok(self)
    }
    /// Return this value with manufacturer replaced, rejecting invalid input.
    pub fn with_manufacturer(mut self, manufacturer: impl Into<String>) -> Result<Self> {
        self.manufacturer = Some(Text::new(manufacturer)?);
        Ok(self)
    }
    /// Return this value with catalog code replaced, rejecting invalid input.
    pub fn with_catalog_code(mut self, code: impl Into<String>) -> Result<Self> {
        self.catalog_code = Some(Text::new(code)?);
        Ok(self)
    }
}
