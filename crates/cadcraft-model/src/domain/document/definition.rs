// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Project document metadata; start with Document::new.
use super::*;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Detached entity; Project validates relations on insertion or import.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub(in crate::domain) id: DocumentId,
    pub(in crate::domain) project_id: ProjectId,
    name: Text,
    metadata: Metadata,
    pub(in crate::domain) properties: TechnicalProperties,
}
impl Document {
    /// Create a detached document with fresh identity and a validated name.
    pub fn new(project_id: ProjectId, name: impl Into<String>) -> Result<Self> {
        Ok(Self {
            id: DocumentId::new(),
            project_id,
            name: Text::new(name)?,
            metadata: Metadata::new(),
            properties: TechnicalProperties::default(),
        })
    }
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> DocumentId {
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
    /// Return the name.
    pub fn name(&self) -> &Text {
        &self.name
    }
    /// Return the metadata.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    /// Insert or replace a validated metadata entry.
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) -> Result<()> {
        self.metadata.insert(Text::new(key)?, value.into());
        Ok(())
    }
}
