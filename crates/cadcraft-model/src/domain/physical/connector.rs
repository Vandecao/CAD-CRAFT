// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Industrial connectors and explicit electrical mating records; start with Connector.
use super::*;
use serde::{Deserialize, Serialize};

/// Declared mechanical gender. `Unspecified` is valid documentation but cannot mate; mating
/// requires known genders.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectorGender {
    /// No explicit classification supplied.
    #[default]
    Unspecified,
    /// Male mechanical connector half.
    Male,
    /// Female mechanical connector half.
    Female,
    /// Hermaphroditic mechanical connector half.
    Hermaphroditic,
}
impl ConnectorGender {
    /// All supported variants in display order.
    pub const ALL: [Self; 4] = [
        Self::Unspecified,
        Self::Male,
        Self::Female,
        Self::Hermaphroditic,
    ];
    /// Male with female, or hermaphroditic with hermaphroditic; unspecified never mates.
    pub fn mates_with(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Male, Self::Female)
                | (Self::Female, Self::Male)
                | (Self::Hermaphroditic, Self::Hermaphroditic)
        )
    }
    /// Return the pt-BR display label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Unspecified => "não declarado",
            Self::Male => "macho",
            Self::Female => "fêmea",
            Self::Hermaphroditic => "hermafrodita",
        }
    }
}

/// Industrial connector, such as `-X1`, grouping ordered contacts. Each contact is an existing
/// Terminal with stable TerminalId, documentary designation and conductor positions. The grouping
/// changes neither continuity nor ownership. Project validates existence, uniqueness and
/// device/component consistency.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Connector {
    id: ConnectorId,
    project_id: ProjectId,
    designation: DesignationPart,
    #[serde(default)]
    gender: ConnectorGender,
    #[serde(default)]
    device: Option<DeviceId>,
    #[serde(default)]
    component: Option<ComponentId>,
    #[serde(default)]
    contacts: Vec<TerminalId>,
    #[serde(default)]
    pub(in crate::domain) properties: TechnicalProperties,
}
impl Connector {
    /// Create an empty detached connector with fresh identity and unspecified gender.
    pub fn new(project_id: ProjectId, designation: DesignationPart) -> Self {
        Self {
            id: ConnectorId::new(),
            project_id,
            designation,
            gender: ConnectorGender::Unspecified,
            device: None,
            component: None,
            contacts: Vec::new(),
            properties: TechnicalProperties::default(),
        }
    }
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> ConnectorId {
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
    /// Return the gender.
    pub fn gender(&self) -> ConnectorGender {
        self.gender
    }
    /// Return the optional device; absence represents an unset value or relation.
    pub fn device(&self) -> Option<DeviceId> {
        self.device
    }
    /// Return the optional component; absence represents an unset value or relation.
    pub fn component(&self) -> Option<ComponentId> {
        self.component
    }
    /// Explicit display order, independent of identity and numbering.
    pub fn contacts(&self) -> &[TerminalId] {
        &self.contacts
    }
    /// Return the properties.
    pub fn properties(&self) -> &TechnicalProperties {
        &self.properties
    }
    /// Return this value with designation replaced.
    pub fn with_designation(mut self, designation: DesignationPart) -> Self {
        self.designation = designation;
        self
    }
    /// Return this value with gender replaced.
    pub fn with_gender(mut self, gender: ConnectorGender) -> Self {
        self.gender = gender;
        self
    }
    /// Documentary relation; Project checks consistency with contact owners.
    pub fn with_device(mut self, device: Option<DeviceId>) -> Self {
        self.device = device;
        self
    }
    /// Return this value with component replaced.
    pub fn with_component(mut self, component: Option<ComponentId>) -> Self {
        self.component = component;
        self
    }
    /// Return this value with contacts replaced.
    pub fn with_contacts(mut self, contacts: Vec<TerminalId>) -> Self {
        self.contacts = contacts;
        self
    }
    /// Return this value with properties replaced.
    pub fn with_properties(mut self, properties: TechnicalProperties) -> Self {
        self.properties = properties;
        self
    }
}

/// Explicit mating pair: `first` belongs to the first connector and `second` to the second. Equal
/// numbering never creates pairs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContactPair {
    first: TerminalId,
    second: TerminalId,
}
impl ContactPair {
    /// Declare an explicit correspondence between two terminal contacts.
    pub fn new(first: TerminalId, second: TerminalId) -> Self {
        Self { first, second }
    }
    /// Return the first.
    pub fn first(self) -> TerminalId {
        self.first
    }
    /// Return the second.
    pub fn second(self) -> TerminalId {
        self.second
    }
}

/// Mating between two connector halves. Owns one logical Connection per contact pair with
/// associated points, created and removed in the same transaction as the mating. Networks therefore
/// cross the connector. Project validates genders, membership, uniqueness and owned connections.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectorMating {
    id: ConnectorMatingId,
    project_id: ProjectId,
    first: ConnectorId,
    second: ConnectorId,
    pairs: Vec<ContactPair>,
    #[serde(default)]
    properties: TechnicalProperties,
    /// Owned connections in the order of pairs with points. `None` denotes legacy data reconstructed on
    /// load.
    #[serde(default)]
    owned: Option<Vec<ConnectionId>>,
}
impl ConnectorMating {
    /// Create a detached mating with fresh identity and the supplied explicit contact pairs.
    pub fn new(
        project_id: ProjectId,
        first: ConnectorId,
        second: ConnectorId,
        pairs: Vec<ContactPair>,
    ) -> Self {
        Self {
            id: ConnectorMatingId::new(),
            project_id,
            first,
            second,
            pairs,
            properties: TechnicalProperties::default(),
            owned: Some(Vec::new()),
        }
    }
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> ConnectorMatingId {
        self.id
    }
    /// Return the project identifier.
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }
    /// Return the first.
    pub fn first(&self) -> ConnectorId {
        self.first
    }
    /// Return the second.
    pub fn second(&self) -> ConnectorId {
        self.second
    }
    /// Return the pairs in their stored order.
    pub fn pairs(&self) -> &[ContactPair] {
        &self.pairs
    }
    /// Return the properties.
    pub fn properties(&self) -> &TechnicalProperties {
        &self.properties
    }
    /// Logical connections owned by this mating, providing explicit continuity.
    pub fn owned_connections(&self) -> &[ConnectionId] {
        self.owned.as_deref().unwrap_or(&[])
    }
    pub(in crate::domain) fn owned_is_legacy(&self) -> bool {
        self.owned.is_none()
    }
    pub(in crate::domain) fn with_owned(mut self, owned: Vec<ConnectionId>) -> Self {
        self.owned = Some(owned);
        self
    }
    /// Whether this mating includes the supplied connector.
    pub fn involves(&self, connector: ConnectorId) -> bool {
        self.first == connector || self.second == connector
    }
    /// Mated counterpart determined solely by the explicit contact map.
    pub fn mate_of(&self, contact: TerminalId) -> Option<TerminalId> {
        self.pairs.iter().find_map(|p| {
            if p.first == contact {
                Some(p.second)
            } else if p.second == contact {
                Some(p.first)
            } else {
                None
            }
        })
    }
    /// Return this value with pairs replaced.
    pub fn with_pairs(mut self, pairs: Vec<ContactPair>) -> Self {
        self.pairs = pairs;
        self
    }
    /// Return this value with properties replaced.
    pub fn with_properties(mut self, properties: TechnicalProperties) -> Self {
        self.properties = properties;
        self
    }
}
