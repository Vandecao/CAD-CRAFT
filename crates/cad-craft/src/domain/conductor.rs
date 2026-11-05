// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::*;
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};

/// Vínculo explícito; source_point corresponde ao extremo source do condutor.
/// O outro ponto é derivado da Connection, sem duplicar seus extremos.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConductorConnection {
    connection_id: ConnectionId,
    source_point: ConnectionPointId,
}
impl ConductorConnection {
    pub fn new(connection_id: ConnectionId, source_point: ConnectionPointId) -> Self {
        Self {
            connection_id,
            source_point,
        }
    }
    pub fn connection_id(self) -> ConnectionId {
        self.connection_id
    }
    pub fn source_point(self) -> ConnectionPointId {
        self.source_point
    }
}

/// Elemento físico independente. Seu cabo é derivado de CableCore no Project.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "ConductorData")]
pub struct Conductor {
    id: ConductorId,
    project_id: ProjectId,
    designation: DesignationPart,
    source: PhysicalTermination,
    target: PhysicalTermination,
    cross_section: CrossSectionMm2,
    marking: ConductorMarking,
    length: Option<LengthMm>,
    material: Option<ConductorMaterial>,
    function: ConductorFunction,
    pub(super) connection: Option<ConductorConnection>,
    pub(super) properties: TechnicalProperties,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConductorData {
    id: ConductorId,
    project_id: ProjectId,
    designation: DesignationPart,
    source: PhysicalTermination,
    target: PhysicalTermination,
    cross_section: CrossSectionMm2,
    marking: ConductorMarking,
    length: Option<LengthMm>,
    material: Option<ConductorMaterial>,
    function: ConductorFunction,
    connection: Option<ConductorConnection>,
    properties: TechnicalProperties,
}
impl TryFrom<ConductorData> for Conductor {
    type Error = DomainError;
    fn try_from(d: ConductorData) -> Result<Self> {
        let mut value = Self::new(
            d.project_id,
            d.designation,
            d.source,
            d.target,
            d.cross_section,
            d.marking,
        )?;
        value.id = d.id;
        value.length = d.length;
        value.material = d.material;
        value.function = d.function;
        value.connection = d.connection;
        value.properties = d.properties;
        Ok(value)
    }
}
impl Conductor {
    pub fn new(
        project_id: ProjectId,
        designation: DesignationPart,
        source: PhysicalTermination,
        target: PhysicalTermination,
        cross_section: CrossSectionMm2,
        marking: ConductorMarking,
    ) -> Result<Self> {
        if source == target {
            return Err(DomainError::InvalidRelation(
                "extremos físicos devem ser distintos",
            ));
        }
        Ok(Self {
            id: ConductorId::new(),
            project_id,
            designation,
            source,
            target,
            cross_section,
            marking,
            length: None,
            material: None,
            function: ConductorFunction::Unspecified,
            connection: None,
            properties: TechnicalProperties::default(),
        })
    }
    pub fn id(&self) -> ConductorId {
        self.id
    }
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }
    pub fn designation(&self) -> &DesignationPart {
        &self.designation
    }
    pub fn endpoints(&self) -> (&PhysicalTermination, &PhysicalTermination) {
        (&self.source, &self.target)
    }
    pub fn cross_section(&self) -> CrossSectionMm2 {
        self.cross_section
    }
    pub fn marking(&self) -> &ConductorMarking {
        &self.marking
    }
    pub fn length(&self) -> Option<LengthMm> {
        self.length
    }
    pub fn material(&self) -> Option<ConductorMaterial> {
        self.material
    }
    pub fn function(&self) -> ConductorFunction {
        self.function
    }
    pub fn connection(&self) -> Option<ConductorConnection> {
        self.connection
    }
    pub fn properties(&self) -> &TechnicalProperties {
        &self.properties
    }
    pub fn with_length(mut self, value: LengthMm) -> Self {
        self.length = Some(value);
        self
    }
    pub fn with_material(mut self, value: ConductorMaterial) -> Self {
        self.material = Some(value);
        self
    }
    pub fn with_function(mut self, value: ConductorFunction) -> Self {
        self.function = value;
        self
    }
    pub fn with_properties(mut self, value: TechnicalProperties) -> Self {
        self.properties = value;
        self
    }
    pub fn with_designation(mut self, value: DesignationPart) -> Self {
        self.designation = value;
        self
    }
    pub fn with_marking(mut self, value: ConductorMarking) -> Self {
        self.marking = value;
        self
    }
    pub fn with_cross_section(mut self, value: CrossSectionMm2) -> Self {
        self.cross_section = value;
        self
    }
}
