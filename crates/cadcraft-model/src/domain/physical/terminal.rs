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
/// Proprietário físico; não é uma representação gráfica.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminalOwner {
    Device(DeviceId),
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

/// Especialização física declarada do borne. A classificação nunca cria
/// conexões, redes, terra ou continuidade: a continuidade elétrica continua
/// derivada apenas de conexões lógicas explícitas (inclusive pontes).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum TerminalKind {
    #[default]
    Generic,
    /// Passagem: dois lados declarados, todas as posições com lado.
    FeedThrough,
    /// Proteção (PE): classificação tipada, sem ligação automática ao terra.
    ProtectiveEarth,
    /// Multinível: dois ou mais níveis identificáveis e isolados.
    MultiLevel,
    /// Distribuição: duas ou mais posições de conexão.
    Distribution,
    /// Seccionável: lados declarados e estado estático documental.
    Disconnect,
}
impl TerminalKind {
    pub const ALL: [Self; 6] = [
        Self::Generic,
        Self::FeedThrough,
        Self::ProtectiveEarth,
        Self::MultiLevel,
        Self::Distribution,
        Self::Disconnect,
    ];
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

/// Lado físico de uma posição (ex.: entrada de campo × lado interno).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TerminalSide {
    A,
    B,
}
impl TerminalSide {
    pub fn label(self) -> &'static str {
        match self {
            Self::A => "lado A",
            Self::B => "lado B",
        }
    }
}

/// Estado estático de projeto de um borne seccionável. É informação
/// documental: não altera o grafo nem as redes, e não há simulação.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisconnectState {
    Closed,
    Open,
}

/// Nível de um borne multinível. A identidade (`TerminalLevelId`) é estável;
/// a designação é documental. As posições de níveis distintos são disjuntas.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminalLevel {
    id: TerminalLevelId,
    designation: DesignationPart,
    #[serde(deserialize_with = "read_positions")]
    positions: BTreeSet<DesignationPart>,
}
impl TerminalLevel {
    pub fn new(designation: DesignationPart, positions: Vec<DesignationPart>) -> Result<Self> {
        Ok(Self {
            id: TerminalLevelId::new(),
            designation,
            positions: unique_positions(positions)?,
        })
    }
    pub fn id(&self) -> TerminalLevelId {
        self.id
    }
    pub fn designation(&self) -> &DesignationPart {
        &self.designation
    }
    pub fn positions(&self) -> impl Iterator<Item = &DesignationPart> {
        self.positions.iter()
    }
    pub fn has_position(&self, position: &DesignationPart) -> bool {
        self.positions.contains(position)
    }
    pub fn with_designation(mut self, designation: DesignationPart) -> Self {
        self.designation = designation;
        self
    }
    pub fn with_positions(mut self, positions: Vec<DesignationPart>) -> Result<Self> {
        self.positions = unique_positions(positions)?;
        Ok(self)
    }
}

/// Entidade destacada; relações são validadas por Project na inserção/importação.
/// Campos de especialização (Etapa 11) têm default e são omitidos quando não
/// usados, mantendo snapshots anteriores válidos e reserializáveis sem mudança.
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
    pub fn id(&self) -> TerminalId {
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
    pub fn with_designation(mut self, designation: DesignationPart) -> Self {
        self.designation = designation;
        self
    }
    pub fn owner(&self) -> TerminalOwner {
        self.owner
    }
    pub fn designation(&self) -> &DesignationPart {
        &self.designation
    }
    pub fn positions(&self) -> impl Iterator<Item = &DesignationPart> {
        self.positions.iter()
    }
    pub fn has_position(&self, position: &DesignationPart) -> bool {
        self.positions.contains(position)
    }
    pub fn with_positions(mut self, positions: Vec<DesignationPart>) -> Result<Self> {
        self.positions = unique_positions(positions)?;
        Ok(self)
    }
    pub fn kind(&self) -> TerminalKind {
        self.kind
    }
    pub fn with_kind(mut self, kind: TerminalKind) -> Self {
        self.kind = kind;
        self
    }
    pub fn sides(&self) -> &BTreeMap<DesignationPart, TerminalSide> {
        &self.sides
    }
    pub fn side_of(&self, position: &DesignationPart) -> Option<TerminalSide> {
        self.sides.get(position).copied()
    }
    pub fn with_sides(mut self, sides: BTreeMap<DesignationPart, TerminalSide>) -> Self {
        self.sides = sides;
        self
    }
    pub fn levels(&self) -> &[TerminalLevel] {
        &self.levels
    }
    pub fn level(&self, id: TerminalLevelId) -> Option<&TerminalLevel> {
        self.levels.iter().find(|l| l.id() == id)
    }
    pub fn level_of_position(&self, position: &DesignationPart) -> Option<&TerminalLevel> {
        self.levels.iter().find(|l| l.has_position(position))
    }
    /// Substitui os níveis. As posições do borne passam a ser a união das
    /// posições dos níveis, para que condutores terminem sempre num nível.
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
    pub fn disconnect_state(&self) -> Option<DisconnectState> {
        self.disconnect_state
    }
    pub fn with_disconnect_state(mut self, state: Option<DisconnectState>) -> Self {
        self.disconnect_state = state;
        self
    }
    /// Regras puras e verificáveis da especialização declarada. Relações com
    /// pontos, pontes, condutores e conectores são verificadas pelo Project.
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
