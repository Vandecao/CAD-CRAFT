// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Terminals and their specializations; start with Terminal.
use super::*;
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

fn read_positions<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<BTreeSet<DesignationPart>, D::Error> {
    let values = Vec::<DesignationPart>::deserialize(deserializer)?;
    unique_positions(values).map_err(serde::de::Error::custom)
}
fn unique_positions(values: Vec<DesignationPart>) -> Result<BTreeSet<DesignationPart>> {
    let mut positions = BTreeSet::new();
    for value in values {
        if !positions.insert(value) {
            return Err(DomainError::InvalidRelation("posição física duplicada"));
        }
    }
    Ok(positions)
}
/// Physical owner, independent of graphical representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminalOwner {
    /// Explicit reference to a device.
    Device(DeviceId),
    /// Explicit reference to a component.
    Component(ComponentId),
}
impl From<TerminalOwner> for EntityRef {
    fn from(owner: TerminalOwner) -> Self {
        match owner {
            TerminalOwner::Device(id) => Self::Device(id),
            TerminalOwner::Component(id) => Self::Component(id),
        }
    }
}

/// Declared physical terminal specialization. Classification creates no connections, networks,
/// grounding or continuity; electrical continuity derives only from explicit logical connections,
/// including bridges.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum TerminalKind {
    /// Unclassified general-purpose cable.
    #[default]
    Generic,
    /// Feed-through with two declared sides and a side for every position.
    FeedThrough,
    /// Protective earth classification without automatic grounding.
    ProtectiveEarth,
    /// Multilevel terminal with at least two identifiable isolated levels.
    MultiLevel,
    /// Distribution terminal with at least two connection positions.
    Distribution,
    /// Disconnect terminal with declared sides and static documentary state.
    Disconnect,
}
impl TerminalKind {
    /// All supported variants in display order.
    pub const ALL: [Self; 6] = [
        Self::Generic,
        Self::FeedThrough,
        Self::ProtectiveEarth,
        Self::MultiLevel,
        Self::Distribution,
        Self::Disconnect,
    ];
    /// Return the pt-BR display label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Generic => "genérico",
            Self::FeedThrough => "passagem",
            Self::ProtectiveEarth => "PE (proteção)",
            Self::MultiLevel => "multinível",
            Self::Distribution => "distribuição",
            Self::Disconnect => "seccionável",
        }
    }
    fn is_generic(&self) -> bool {
        *self == Self::Generic
    }
}

/// Physical side of a position, such as field input versus internal side.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TerminalSide {
    /// First declared terminal side.
    A,
    /// Second declared terminal side.
    B,
}
impl TerminalSide {
    /// Return the pt-BR display label.
    pub fn label(self) -> &'static str {
        match self {
            Self::A => "lado A",
            Self::B => "lado B",
        }
    }
}

/// Static design state of a disconnect terminal. Documentary only: changes neither graph nor
/// networks and performs no simulation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisconnectState {
    /// Documented closed disconnect state.
    Closed,
    /// Documented open disconnect state.
    Open,
}

/// Level of a multilevel terminal. TerminalLevelId is stable, while designation is documentary.
/// Positions of distinct levels are disjoint.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminalLevel {
    id: TerminalLevelId,
    designation: DesignationPart,
    #[serde(deserialize_with = "read_positions")]
    positions: BTreeSet<DesignationPart>,
}
impl TerminalLevel {
    /// Create an identifiable terminal level with validated, unique landing positions.
    pub fn new(designation: DesignationPart, positions: Vec<DesignationPart>) -> Result<Self> {
        Ok(Self {
            id: TerminalLevelId::new(),
            designation,
            positions: unique_positions(positions)?,
        })
    }
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> TerminalLevelId {
        self.id
    }
    /// Return the designation.
    pub fn designation(&self) -> &DesignationPart {
        &self.designation
    }
    /// Iterate registered conductor landing positions in designation order.
    pub fn positions(&self) -> impl Iterator<Item = &DesignationPart> {
        self.positions.iter()
    }
    /// Whether the terminal or level contains this landing position.
    pub fn has_position(&self, position: &DesignationPart) -> bool {
        self.positions.contains(position)
    }
    /// Return this value with designation replaced.
    pub fn with_designation(mut self, designation: DesignationPart) -> Self {
        self.designation = designation;
        self
    }
    /// Return this value with positions replaced, rejecting invalid input.
    pub fn with_positions(mut self, positions: Vec<DesignationPart>) -> Result<Self> {
        self.positions = unique_positions(positions)?;
        Ok(self)
    }
}

/// Detached entity; Project validates relations on insertion or import. Specialization fields
/// default and are omitted when unused, preserving earlier snapshots and their exact
/// reserialization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Terminal {
    pub(in crate::domain) id: TerminalId,
    pub(in crate::domain) project_id: ProjectId,
    owner: TerminalOwner,
    designation: DesignationPart,
    #[serde(default, deserialize_with = "read_positions")]
    pub(in crate::domain) positions: BTreeSet<DesignationPart>,
    pub(in crate::domain) properties: TechnicalProperties,
    #[serde(default, skip_serializing_if = "TerminalKind::is_generic")]
    kind: TerminalKind,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    sides: BTreeMap<DesignationPart, TerminalSide>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    levels: Vec<TerminalLevel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    disconnect_state: Option<DisconnectState>,
}
impl Terminal {
    /// Create a detached terminal with fresh identity, explicit owner and designation.
    pub fn new(project_id: ProjectId, owner: TerminalOwner, designation: DesignationPart) -> Self {
        Self {
            id: TerminalId::new(),
            project_id,
            owner,
            designation,
            positions: BTreeSet::new(),
            properties: TechnicalProperties::default(),
            kind: TerminalKind::Generic,
            sides: BTreeMap::new(),
            levels: Vec::new(),
            disconnect_state: None,
        }
    }
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> TerminalId {
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
    /// Return this value with designation replaced.
    pub fn with_designation(mut self, designation: DesignationPart) -> Self {
        self.designation = designation;
        self
    }
    /// Return the owner.
    pub fn owner(&self) -> TerminalOwner {
        self.owner
    }
    /// Return the designation.
    pub fn designation(&self) -> &DesignationPart {
        &self.designation
    }
    /// Iterate registered conductor landing positions in designation order.
    pub fn positions(&self) -> impl Iterator<Item = &DesignationPart> {
        self.positions.iter()
    }
    /// Whether the terminal or level contains this landing position.
    pub fn has_position(&self, position: &DesignationPart) -> bool {
        self.positions.contains(position)
    }
    /// Return this value with positions replaced, rejecting invalid input.
    pub fn with_positions(mut self, positions: Vec<DesignationPart>) -> Result<Self> {
        self.positions = unique_positions(positions)?;
        Ok(self)
    }
    /// Return the kind.
    pub fn kind(&self) -> TerminalKind {
        self.kind
    }
    /// Return this value with kind replaced.
    pub fn with_kind(mut self, kind: TerminalKind) -> Self {
        self.kind = kind;
        self
    }
    /// Return the sides.
    pub fn sides(&self) -> &BTreeMap<DesignationPart, TerminalSide> {
        &self.sides
    }
    /// Return the explicitly declared side of a terminal position.
    pub fn side_of(&self, position: &DesignationPart) -> Option<TerminalSide> {
        self.sides.get(position).copied()
    }
    /// Return this value with sides replaced.
    pub fn with_sides(mut self, sides: BTreeMap<DesignationPart, TerminalSide>) -> Self {
        self.sides = sides;
        self
    }
    /// Return the levels in their stored order.
    pub fn levels(&self) -> &[TerminalLevel] {
        &self.levels
    }
    /// Find a terminal level by its stable ID.
    pub fn level(&self, id: TerminalLevelId) -> Option<&TerminalLevel> {
        self.levels.iter().find(|l| l.id() == id)
    }
    /// Find the level that owns this terminal position.
    pub fn level_of_position(&self, position: &DesignationPart) -> Option<&TerminalLevel> {
        self.levels.iter().find(|l| l.has_position(position))
    }
    /// Replace levels. Terminal positions become the union of level positions so conductor endpoints
    /// always refer to a level.
    pub fn with_levels(mut self, levels: Vec<TerminalLevel>) -> Self {
        if !levels.is_empty() {
            self.positions = levels
                .iter()
                .flat_map(|l| l.positions.iter().cloned())
                .collect();
        }
        self.levels = levels;
        self
    }
    /// Return the optional disconnect state; absence represents an unset value or relation.
    pub fn disconnect_state(&self) -> Option<DisconnectState> {
        self.disconnect_state
    }
    /// Return this value with disconnect state replaced.
    pub fn with_disconnect_state(mut self, state: Option<DisconnectState>) -> Self {
        self.disconnect_state = state;
        self
    }
    /// Pure, verifiable specialization rules. Project checks relations to points, bridges, conductors
    /// and connectors.
    pub fn check_specialization(&self) -> Result<()> {
        let invalid = |reason| Err(DomainError::InvalidRelation(reason));
        if self.sides.keys().any(|p| !self.positions.contains(p)) {
            return invalid("lado declarado para posição inexistente");
        }
        if !self.levels.is_empty() && self.kind != TerminalKind::MultiLevel {
            return invalid("somente borne multinível possui níveis");
        }
        if self.disconnect_state.is_some() != (self.kind == TerminalKind::Disconnect) {
            return invalid("estado de seccionamento exige borne seccionável");
        }
        match self.kind {
            TerminalKind::Generic => {}
            TerminalKind::FeedThrough | TerminalKind::Disconnect => {
                if self.positions.len() < 2 || self.sides.len() != self.positions.len() {
                    return invalid("borne de passagem/seccionável exige lado em cada posição");
                }
                let sides: BTreeSet<_> = self.sides.values().collect();
                if sides.len() != 2 {
                    return invalid("borne de passagem/seccionável exige os lados A e B");
                }
            }
            TerminalKind::ProtectiveEarth => {
                if self.positions.is_empty() {
                    return invalid("borne PE exige ao menos uma posição de conexão");
                }
            }
            TerminalKind::Distribution => {
                if self.positions.len() < 2 {
                    return invalid("borne de distribuição exige duas ou mais posições");
                }
            }
            TerminalKind::MultiLevel => {
                if self.levels.len() < 2 {
                    return invalid("borne multinível exige dois ou mais níveis");
                }
                let mut ids = BTreeSet::new();
                let mut names = BTreeSet::new();
                let mut seen = BTreeSet::new();
                for level in &self.levels {
                    if !ids.insert(level.id) || !names.insert(&level.designation) {
                        return invalid("nível duplicado no borne");
                    }
                    if level.positions.is_empty() {
                        return invalid("nível sem posição de conexão");
                    }
                    for position in &level.positions {
                        if !seen.insert(position) {
                            return invalid("posição compartilhada entre níveis");
                        }
                    }
                }
                if seen.len() != self.positions.len()
                    || seen.iter().any(|p| !self.positions.contains(*p))
                {
                    return invalid("posições do borne divergem dos níveis");
                }
            }
        }
        Ok(())
    }
}
