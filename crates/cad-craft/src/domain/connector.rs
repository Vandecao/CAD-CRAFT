// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::*;
use serde::{Deserialize, Serialize};

/// Gênero mecânico declarado. `Unspecified` é válido para documentação, mas
/// não permite acoplamento: o pareamento exige gêneros conhecidos.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectorGender {
    #[default]
    Unspecified,
    Male,
    Female,
    Hermaphroditic,
}
impl ConnectorGender {
    pub const ALL: [Self; 4] = [
        Self::Unspecified,
        Self::Male,
        Self::Female,
        Self::Hermaphroditic,
    ];
    /// Macho↔fêmea ou hermafrodita↔hermafrodita; gênero ausente nunca acopla.
    pub fn mates_with(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Male, Self::Female)
                | (Self::Female, Self::Male)
                | (Self::Hermaphroditic, Self::Hermaphroditic)
        )
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Unspecified => "não declarado",
            Self::Male => "macho",
            Self::Female => "fêmea",
            Self::Hermaphroditic => "hermafrodita",
        }
    }
}

/// Conector industrial (ex.: `-X1`): agrupamento ordenado de contatos.
/// Cada contato é um `Terminal` existente — identidade `TerminalId`, numeração
/// documental na designação do borne e posições para condutores. O conector
/// não cria continuidade elétrica nem muda o proprietário dos contatos.
/// Project valida existência, unicidade e coerência com dispositivo/componente.
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
    pub(super) properties: TechnicalProperties,
}
impl Connector {
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
    pub fn id(&self) -> ConnectorId {
        self.id
    }
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }
    pub fn designation(&self) -> &DesignationPart {
        &self.designation
    }
    pub fn gender(&self) -> ConnectorGender {
        self.gender
    }
    pub fn device(&self) -> Option<DeviceId> {
        self.device
    }
    pub fn component(&self) -> Option<ComponentId> {
        self.component
    }
    /// Ordem explícita de exibição; não é identidade nem numeração.
    pub fn contacts(&self) -> &[TerminalId] {
        &self.contacts
    }
    pub fn properties(&self) -> &TechnicalProperties {
        &self.properties
    }
    pub fn with_designation(mut self, designation: DesignationPart) -> Self {
        self.designation = designation;
        self
    }
    pub fn with_gender(mut self, gender: ConnectorGender) -> Self {
        self.gender = gender;
        self
    }
    /// Relação documental; a coerência com os donos dos contatos é verificada
    /// pelo Project.
    pub fn with_device(mut self, device: Option<DeviceId>) -> Self {
        self.device = device;
        self
    }
    pub fn with_component(mut self, component: Option<ComponentId>) -> Self {
        self.component = component;
        self
    }
    pub fn with_contacts(mut self, contacts: Vec<TerminalId>) -> Self {
        self.contacts = contacts;
        self
    }
    pub fn with_properties(mut self, properties: TechnicalProperties) -> Self {
        self.properties = properties;
        self
    }
}

/// Par explícito de contatos de um acoplamento: `first` pertence ao primeiro
/// conector e `second` ao segundo. Igualdade de numeração não cria pares.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContactPair {
    first: TerminalId,
    second: TerminalId,
}
impl ContactPair {
    pub fn new(first: TerminalId, second: TerminalId) -> Self {
        Self { first, second }
    }
    pub fn first(self) -> TerminalId {
        self.first
    }
    pub fn second(self) -> TerminalId {
        self.second
    }
}

/// Acoplamento entre duas metades. Etapa 24 (DEC-11-01, proposta A): o
/// acoplamento possui uma `Connection` lógica por par de contatos que tenham
/// ponto associado, criada e removida na mesma transação (como as pontes), e
/// por isso as redes atravessam o conector. Project valida gêneros,
/// pertencimento, ausência de duplicidades e as conexões de posse.
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
    /// Conexões de posse, na ordem dos pares com ponto. `None` = arquivo
    /// anterior à Etapa 24 (recalculadas na carga).
    #[serde(default)]
    owned: Option<Vec<ConnectionId>>,
}
impl ConnectorMating {
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
    pub fn id(&self) -> ConnectorMatingId {
        self.id
    }
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }
    pub fn first(&self) -> ConnectorId {
        self.first
    }
    pub fn second(&self) -> ConnectorId {
        self.second
    }
    pub fn pairs(&self) -> &[ContactPair] {
        &self.pairs
    }
    pub fn properties(&self) -> &TechnicalProperties {
        &self.properties
    }
    /// Conexões lógicas de posse do acoplamento (continuidade).
    pub fn owned_connections(&self) -> &[ConnectionId] {
        self.owned.as_deref().unwrap_or(&[])
    }
    pub(super) fn owned_is_legacy(&self) -> bool {
        self.owned.is_none()
    }
    pub(super) fn with_owned(mut self, owned: Vec<ConnectionId>) -> Self {
        self.owned = Some(owned);
        self
    }
    pub fn involves(&self, connector: ConnectorId) -> bool {
        self.first == connector || self.second == connector
    }
    /// Contato acoplado ao informado, somente pelo mapa explícito.
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
    pub fn with_pairs(mut self, pairs: Vec<ContactPair>) -> Self {
        self.pairs = pairs;
        self
    }
    pub fn with_properties(mut self, properties: TechnicalProperties) -> Self {
        self.properties = properties;
        self
    }
}
