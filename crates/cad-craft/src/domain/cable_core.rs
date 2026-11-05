// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::*;
use serde::{Deserialize, Serialize};

/// Única fonte persistente do vínculo veia–condutor. Ausência significa veia livre.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CableCore {
    id: CableCoreId,
    project_id: ProjectId,
    cable_id: CableId,
    identification: DesignationPart,
    marking: ConductorMarking,
    cross_section: Option<CrossSectionMm2>,
    pub(super) conductor_id: Option<ConductorId>,
    pub(super) properties: TechnicalProperties,
}
impl CableCore {
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
    pub fn id(&self) -> CableCoreId {
        self.id
    }
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }
    pub fn cable_id(&self) -> CableId {
        self.cable_id
    }
    pub fn identification(&self) -> &DesignationPart {
        &self.identification
    }
    pub fn marking(&self) -> &ConductorMarking {
        &self.marking
    }
    pub fn cross_section(&self) -> Option<CrossSectionMm2> {
        self.cross_section
    }
    pub fn conductor_id(&self) -> Option<ConductorId> {
        self.conductor_id
    }
    pub fn properties(&self) -> &TechnicalProperties {
        &self.properties
    }
    pub fn with_cross_section(mut self, value: CrossSectionMm2) -> Self {
        self.cross_section = Some(value);
        self
    }
    pub fn with_cross_section_option(mut self, value: Option<CrossSectionMm2>) -> Self {
        self.cross_section = value;
        self
    }
    pub fn with_properties(mut self, value: TechnicalProperties) -> Self {
        self.properties = value;
        self
    }
    pub fn with_identification(mut self, value: DesignationPart) -> Self {
        self.identification = value;
        self
    }
    pub fn with_marking(mut self, value: ConductorMarking) -> Self {
        self.marking = value;
        self
    }
}
