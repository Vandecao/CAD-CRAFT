// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::*;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Entidade destacada; relações são validadas por Project na inserção/importação.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub(super) id: DocumentId,
    pub(super) project_id: ProjectId,
    name: Text,
    metadata: Metadata,
    pub(super) properties: TechnicalProperties,
}
impl Document {
    pub fn new(project_id: ProjectId, name: impl Into<String>) -> Result<Self> {
        Ok(Self {
            id: DocumentId::new(),
            project_id,
            name: Text::new(name)?,
            metadata: Metadata::new(),
            properties: TechnicalProperties::default(),
        })
    }
    pub fn id(&self) -> DocumentId {
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
    pub fn name(&self) -> &Text {
        &self.name
    }
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) -> Result<()> {
        self.metadata.insert(Text::new(key)?, value.into());
        Ok(())
    }
}
