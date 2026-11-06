// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Individual cable core identity and assignments; start with CableCore.
use super::*;
use serde::{Deserialize, Serialize};

/// Single persistent source of core-to-conductor assignment; absence means a free core.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CableCore {
    id: CableCoreId,
    project_id: ProjectId,
    cable_id: CableId,
    identification: DesignationPart,
    marking: ConductorMarking,
    cross_section: Option<CrossSectionMm2>,
    pub(in crate::domain) conductor_id: Option<ConductorId>,
    pub(in crate::domain) properties: TechnicalProperties,
}
impl CableCore {
    /// Create a detached core with fresh identity, cable membership and physical marking.
    pub fn new(
        project_id: ProjectId,
        cable_id: CableId,
        identification: DesignationPart,
        marking: ConductorMarking,
    ) -> Self {
        Self {
            id: CableCoreId::new(),
            project_id,
            cable_id,
            identification,
            marking,
            cross_section: None,
            conductor_id: None,
            properties: TechnicalProperties::default(),
        }
    }
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> CableCoreId {
        self.id
    }
    /// Return the project identifier.
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }
    /// Return the cable identifier.
    pub fn cable_id(&self) -> CableId {
        self.cable_id
    }
    /// Return the identification.
    pub fn identification(&self) -> &DesignationPart {
        &self.identification
    }
    /// Return the marking.
    pub fn marking(&self) -> &ConductorMarking {
        &self.marking
    }
    /// Return the optional cross section; absence represents an unset value or relation.
    pub fn cross_section(&self) -> Option<CrossSectionMm2> {
        self.cross_section
    }
    /// Return the conductor identifier.
    pub fn conductor_id(&self) -> Option<ConductorId> {
        self.conductor_id
    }
    /// Return the properties.
    pub fn properties(&self) -> &TechnicalProperties {
        &self.properties
    }
    /// Return this value with cross section replaced.
    pub fn with_cross_section(mut self, value: CrossSectionMm2) -> Self {
        self.cross_section = Some(value);
        self
    }
    /// Return this value with cross section replaced.
    pub fn with_cross_section_option(mut self, value: Option<CrossSectionMm2>) -> Self {
        self.cross_section = value;
        self
    }
    /// Return this value with properties replaced.
    pub fn with_properties(mut self, value: TechnicalProperties) -> Self {
        self.properties = value;
        self
    }
    /// Return this value with identification replaced.
    pub fn with_identification(mut self, value: DesignationPart) -> Self {
        self.identification = value;
        self
    }
    /// Return this value with marking replaced.
    pub fn with_marking(mut self, value: ConductorMarking) -> Self {
        self.marking = value;
        self
    }
}
