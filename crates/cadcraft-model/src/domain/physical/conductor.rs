// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Physical conductor and endpoint metadata; start with Conductor.
use super::*;
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};

/// Explicit binding; source_point corresponds to the conductor's source endpoint. The other point
/// is derived from Connection without duplicating its endpoints.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConductorConnection {
    connection_id: ConnectionId,
    source_point: ConnectionPointId,
}
impl ConductorConnection {
    /// Bind a conductor to a logical connection with an explicit source point.
    pub fn new(connection_id: ConnectionId, source_point: ConnectionPointId) -> Self {
        Self {
            connection_id,
            source_point,
        }
    }
    /// Return the connection identifier.
    pub fn connection_id(self) -> ConnectionId {
        self.connection_id
    }
    /// Return the source point.
    pub fn source_point(self) -> ConnectionPointId {
        self.source_point
    }
}

/// Independent physical element whose cable is derived from CableCore in Project.
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
    pub(in crate::domain) connection: Option<ConductorConnection>,
    pub(in crate::domain) properties: TechnicalProperties,
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
    /// Create a detached conductor with fresh identity and distinct physical endpoints.
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
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> ConductorId {
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
    /// Return the endpoints.
    pub fn endpoints(&self) -> (&PhysicalTermination, &PhysicalTermination) {
        (&self.source, &self.target)
    }
    /// Return the cross section.
    pub fn cross_section(&self) -> CrossSectionMm2 {
        self.cross_section
    }
    /// Return the marking.
    pub fn marking(&self) -> &ConductorMarking {
        &self.marking
    }
    /// Return the optional length; absence represents an unset value or relation.
    pub fn length(&self) -> Option<LengthMm> {
        self.length
    }
    /// Return the optional material; absence represents an unset value or relation.
    pub fn material(&self) -> Option<ConductorMaterial> {
        self.material
    }
    /// Return the function.
    pub fn function(&self) -> ConductorFunction {
        self.function
    }
    /// Return the optional connection; absence represents an unset value or relation.
    pub fn connection(&self) -> Option<ConductorConnection> {
        self.connection
    }
    /// Return the properties.
    pub fn properties(&self) -> &TechnicalProperties {
        &self.properties
    }
    /// Return this value with length replaced.
    pub fn with_length(mut self, value: LengthMm) -> Self {
        self.length = Some(value);
        self
    }
    /// Return this value with material replaced.
    pub fn with_material(mut self, value: ConductorMaterial) -> Self {
        self.material = Some(value);
        self
    }
    /// Return this value with function replaced.
    pub fn with_function(mut self, value: ConductorFunction) -> Self {
        self.function = value;
        self
    }
    /// Return this value with properties replaced.
    pub fn with_properties(mut self, value: TechnicalProperties) -> Self {
        self.properties = value;
        self
    }
    /// Return this value with designation replaced.
    pub fn with_designation(mut self, value: DesignationPart) -> Self {
        self.designation = value;
        self
    }
    /// Return this value with marking replaced.
    pub fn with_marking(mut self, value: ConductorMarking) -> Self {
        self.marking = value;
        self
    }
    /// Return this value with cross section replaced.
    pub fn with_cross_section(mut self, value: CrossSectionMm2) -> Self {
        self.cross_section = value;
        self
    }
}
