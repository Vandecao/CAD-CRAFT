// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Neutral project aggregate and entity storage; start with Project::new and validate.
use super::*;
use crate::document_index::DocumentIndex;
use crate::error::{DomainError, Result};
use crate::graph::ConnectivityGraph;
use crate::physical_index::PhysicalIndex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod checks;
mod document;
mod electrical;
mod integrity;
mod physical;
mod snapshot;
mod symbols;

pub use checks::*;
pub use document::*;
use electrical::connection_drawing;
pub use electrical::*;
pub use physical::*;
pub use symbols::*;

/// Agregado com identidade estável e acesso imutável às entidades registradas.
/// Inserções exigem alvos existentes. Páginas, representações e conexões têm remoção
/// explícita sem cascata, sujeita à ausência de referências externas.
/// Sem histórico ou ERC.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "snapshot::ProjectData")]
pub struct Project {
    id: ProjectId,
    name: Text,
    description: String,
    identification: Option<Text>,
    metadata: Metadata,
    properties: TechnicalProperties,
    numbering_scheme: NumberingScheme,
    /// Etapa 24: esquema de verificação (regras e mensagens ignoradas).
    #[serde(default, skip_serializing_if = "is_default_scheme")]
    check_scheme: CheckScheme,
    #[serde(serialize_with = "snapshot::serialize_entities")]
    documents: BTreeMap<DocumentId, Document>,
    #[serde(serialize_with = "snapshot::serialize_entities")]
    pages: BTreeMap<PageId, Page>,
    #[serde(serialize_with = "snapshot::serialize_entities")]
    devices: BTreeMap<DeviceId, Device>,
    #[serde(serialize_with = "snapshot::serialize_entities")]
    functions: BTreeMap<FunctionId, Function>,
    #[serde(serialize_with = "snapshot::serialize_entities")]
    components: BTreeMap<ComponentId, Component>,
    #[serde(
        serialize_with = "snapshot::serialize_entities",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    parts: BTreeMap<PartId, Part>,
    #[serde(serialize_with = "snapshot::serialize_entities")]
    terminals: BTreeMap<TerminalId, Terminal>,
    #[serde(serialize_with = "snapshot::serialize_entities")]
    connection_points: BTreeMap<ConnectionPointId, ConnectionPoint>,
    #[serde(serialize_with = "snapshot::serialize_entities")]
    connections: BTreeMap<ConnectionId, Connection>,
    #[serde(serialize_with = "snapshot::serialize_entities")]
    conductors: BTreeMap<ConductorId, Conductor>,
    #[serde(serialize_with = "snapshot::serialize_entities")]
    cables: BTreeMap<CableId, Cable>,
    #[serde(serialize_with = "snapshot::serialize_entities")]
    cable_cores: BTreeMap<CableCoreId, CableCore>,
    #[serde(serialize_with = "snapshot::serialize_entities")]
    terminal_strips: BTreeMap<TerminalStripId, TerminalStrip>,
    #[serde(serialize_with = "snapshot::serialize_entities")]
    terminal_bridges: BTreeMap<ConnectionId, TerminalBridge>,
    // Etapa 11: omitted when empty so earlier snapshots reserialize unchanged.
    #[serde(
        serialize_with = "snapshot::serialize_entities",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    connectors: BTreeMap<ConnectorId, Connector>,
    #[serde(
        serialize_with = "snapshot::serialize_entities",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    connector_matings: BTreeMap<ConnectorMatingId, ConnectorMating>,
    #[serde(serialize_with = "snapshot::serialize_entities")]
    representations: BTreeMap<RepresentationId, Representation>,
    #[serde(serialize_with = "snapshot::serialize_entities")]
    symbol_definitions: BTreeMap<SymbolDefinitionId, SymbolDefinition>,
    #[serde(serialize_with = "connection_drawing::serialize_bindings")]
    port_bindings: BTreeMap<PortRef, ConnectionPointId>,
    #[serde(serialize_with = "snapshot::serialize_entities")]
    connection_routes: BTreeMap<RepresentationId, ConnectionRoute>,
    #[serde(serialize_with = "snapshot::serialize_entities")]
    connection_junctions: BTreeMap<ConnectionJunctionId, ConnectionJunction>,
    #[serde(serialize_with = "snapshot::serialize_entities")]
    markers: BTreeMap<RepresentationId, ElectricalMarker>,
    #[serde(serialize_with = "snapshot::serialize_entities")]
    marker_groups: BTreeMap<ReferenceGroupId, MarkerGroup>,
    // Etapa 13: molduras; omitidas quando vazias para preservar snapshots antigos.
    #[serde(
        serialize_with = "snapshot::serialize_entities",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    frames: BTreeMap<FrameTemplateId, FrameTemplate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_frame: Option<FrameTemplateId>,
    #[serde(skip_serializing_if = "frames::is_default_format")]
    reference_format: ReferenceFormat,
    // Etapa 14: imagens das molduras (bytes fora do JSON).
    #[serde(
        serialize_with = "snapshot::serialize_entities",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    assets: BTreeMap<AssetId, ImageAsset>,
    #[serde(skip)]
    document_index: DocumentIndex,
    #[serde(skip)]
    physical_index: PhysicalIndex,
    #[serde(skip)]
    graph: ConnectivityGraph,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Result<Self> {
        Ok(Self {
            id: ProjectId::new(),
            name: Text::new(name)?,
            description: String::new(),
            identification: None,
            metadata: Metadata::new(),
            properties: TechnicalProperties::default(),
            numbering_scheme: NumberingScheme::default(),
            check_scheme: CheckScheme::default(),
            documents: BTreeMap::new(),
            pages: BTreeMap::new(),
            devices: BTreeMap::new(),
            functions: BTreeMap::new(),
            components: BTreeMap::new(),
            parts: BTreeMap::new(),
            terminals: BTreeMap::new(),
            connection_points: BTreeMap::new(),
            connections: BTreeMap::new(),
            conductors: BTreeMap::new(),
            cables: BTreeMap::new(),
            cable_cores: BTreeMap::new(),
            terminal_strips: BTreeMap::new(),
            terminal_bridges: BTreeMap::new(),
            connectors: BTreeMap::new(),
            connector_matings: BTreeMap::new(),
            representations: BTreeMap::new(),
            symbol_definitions: BTreeMap::new(),
            port_bindings: BTreeMap::new(),
            connection_routes: BTreeMap::new(),
            connection_junctions: BTreeMap::new(),
            markers: BTreeMap::new(),
            marker_groups: BTreeMap::new(),
            frames: BTreeMap::new(),
            default_frame: None,
            reference_format: ReferenceFormat::default(),
            assets: BTreeMap::new(),
            document_index: DocumentIndex::default(),
            physical_index: PhysicalIndex::default(),
            graph: ConnectivityGraph::default(),
        })
    }
    pub fn id(&self) -> ProjectId {
        self.id
    }
    pub fn name(&self) -> &Text {
        &self.name
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    pub fn properties(&self) -> &TechnicalProperties {
        &self.properties
    }
    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = description.into();
    }
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) -> Result<()> {
        self.metadata.insert(Text::new(key)?, value.into());
        Ok(())
    }

    pub fn document(&self, id: DocumentId) -> Option<&Document> {
        self.documents.get(&id)
    }
    pub fn documents(&self) -> impl Iterator<Item = &Document> {
        self.documents.values()
    }
    pub fn insert_document(&mut self, entity: Document) -> Result<DocumentId> {
        let id = entity.id();
        if self.documents.contains_key(&id) {
            return Err(DomainError::DuplicateEntity(EntityRef::Document(id)));
        }
        self.check_project(entity.project_id())?;
        self.check_document(&entity)?;
        self.check_properties(entity.properties())?;
        self.documents.insert(id, entity);
        Ok(id)
    }

    pub fn page(&self, id: PageId) -> Option<&Page> {
        self.pages.get(&id)
    }
    pub fn pages(&self) -> impl Iterator<Item = &Page> {
        self.pages.values()
    }
    pub fn insert_page(&mut self, entity: Page) -> Result<PageId> {
        let id = entity.id();
        if self.pages.contains_key(&id) {
            return Err(DomainError::DuplicateEntity(EntityRef::Page(id)));
        }
        self.check_project(entity.project_id())?;
        self.check_page(&entity)?;
        self.check_properties(entity.properties())?;
        self.pages.insert(id, entity);
        Ok(id)
    }

    pub fn device(&self, id: DeviceId) -> Option<&Device> {
        self.devices.get(&id)
    }
    pub fn devices(&self) -> impl Iterator<Item = &Device> {
        self.devices.values()
    }
    pub fn insert_device(&mut self, entity: Device) -> Result<DeviceId> {
        let id = entity.id();
        if self.devices.contains_key(&id) {
            return Err(DomainError::DuplicateEntity(EntityRef::Device(id)));
        }
        self.check_project(entity.project_id())?;
        self.check_device(&entity)?;
        self.check_new_device_designation(&entity)?;
        self.check_properties(entity.properties())?;
        self.devices.insert(id, entity);
        Ok(id)
    }

    pub fn function(&self, id: FunctionId) -> Option<&Function> {
        self.functions.get(&id)
    }
    pub fn functions(&self) -> impl Iterator<Item = &Function> {
        self.functions.values()
    }
    pub fn insert_function(&mut self, entity: Function) -> Result<FunctionId> {
        let id = entity.id();
        if self.functions.contains_key(&id) {
            return Err(DomainError::DuplicateEntity(EntityRef::Function(id)));
        }
        self.check_project(entity.project_id())?;
        self.check_function(&entity)?;
        self.check_properties(entity.properties())?;
        self.functions.insert(id, entity);
        Ok(id)
    }

    pub fn component(&self, id: ComponentId) -> Option<&Component> {
        self.components.get(&id)
    }
    pub fn components(&self) -> impl Iterator<Item = &Component> {
        self.components.values()
    }
    pub fn insert_component(&mut self, entity: Component) -> Result<ComponentId> {
        let id = entity.id();
        if self.components.contains_key(&id) {
            return Err(DomainError::DuplicateEntity(EntityRef::Component(id)));
        }
        self.check_project(entity.project_id())?;
        self.check_component(&entity)?;
        self.check_properties(entity.properties())?;
        self.components.insert(id, entity);
        Ok(id)
    }

    pub fn terminal(&self, id: TerminalId) -> Option<&Terminal> {
        self.terminals.get(&id)
    }
    pub fn terminals(&self) -> impl Iterator<Item = &Terminal> {
        self.terminals.values()
    }
    pub fn insert_terminal(&mut self, entity: Terminal) -> Result<TerminalId> {
        let id = entity.id();
        if self.terminals.contains_key(&id) {
            return Err(DomainError::DuplicateEntity(EntityRef::Terminal(id)));
        }
        self.check_project(entity.project_id())?;
        self.check_terminal(&entity)?;
        self.check_properties(entity.properties())?;
        self.terminals.insert(id, entity);
        Ok(id)
    }

    pub fn connection_point(&self, id: ConnectionPointId) -> Option<&ConnectionPoint> {
        self.connection_points.get(&id)
    }
    pub fn connection_points(&self) -> impl Iterator<Item = &ConnectionPoint> {
        self.connection_points.values()
    }
    pub fn insert_connection_point(
        &mut self,
        entity: ConnectionPoint,
    ) -> Result<ConnectionPointId> {
        let id = entity.id();
        if self.connection_points.contains_key(&id) {
            return Err(DomainError::DuplicateEntity(EntityRef::ConnectionPoint(id)));
        }
        self.check_project(entity.project_id())?;
        self.check_connection_point(&entity)?;
        self.check_properties(entity.properties())?;
        self.connection_points.insert(id, entity);
        Ok(id)
    }
    pub fn pages_in_document(&self, document: DocumentId) -> impl Iterator<Item = &Page> {
        self.pages
            .values()
            .filter(move |page| page.document_id() == document)
    }
    pub fn functions_for_device(&self, device: DeviceId) -> impl Iterator<Item = &Function> {
        self.functions
            .values()
            .filter(move |function| function.device_id() == device)
    }
    pub fn add_page_element(&mut self, page: PageId, element: PageElement) -> Result<()> {
        self.require(EntityRef::Page(page))?;
        self.require(element.into())?;
        let target = self
            .pages
            .get_mut(&page)
            .ok_or(DomainError::MissingEntity(EntityRef::Page(page)))?;
        if !target.elements.insert(element) {
            return Err(DomainError::DuplicateReference(element.into()));
        }
        Ok(())
    }
    /// Substituição atômica, com resolução de todas as referências antes da escrita.
    pub fn set_properties(
        &mut self,
        target: EntityRef,
        properties: TechnicalProperties,
    ) -> Result<()> {
        self.require(target)?;
        self.check_properties(&properties)?;
        if matches!(target, EntityRef::Conductor(_) | EntityRef::CableCore(_)) {
            self.check_section_properties(&properties)?;
        }
        match target {
            EntityRef::Project(_) => self.properties = properties,
            EntityRef::Document(id) => {
                self.documents
                    .get_mut(&id)
                    .ok_or(DomainError::MissingEntity(target))?
                    .properties = properties
            }
            EntityRef::Page(id) => {
                self.pages
                    .get_mut(&id)
                    .ok_or(DomainError::MissingEntity(target))?
                    .properties = properties
            }
            EntityRef::Device(id) => {
                self.devices
                    .get_mut(&id)
                    .ok_or(DomainError::MissingEntity(target))?
                    .properties = properties
            }
            EntityRef::Function(id) => {
                self.functions
                    .get_mut(&id)
                    .ok_or(DomainError::MissingEntity(target))?
                    .properties = properties
            }
            EntityRef::Component(id) => {
                self.components
                    .get_mut(&id)
                    .ok_or(DomainError::MissingEntity(target))?
                    .properties = properties
            }
            EntityRef::Terminal(id) => {
                self.terminals
                    .get_mut(&id)
                    .ok_or(DomainError::MissingEntity(target))?
                    .properties = properties
            }
            EntityRef::ConnectionPoint(id) => {
                self.connection_points
                    .get_mut(&id)
                    .ok_or(DomainError::MissingEntity(target))?
                    .properties = properties
            }
            EntityRef::Conductor(id) => {
                self.conductors
                    .get_mut(&id)
                    .ok_or(DomainError::MissingEntity(target))?
                    .properties = properties
            }
            EntityRef::Cable(id) => {
                self.cables
                    .get_mut(&id)
                    .ok_or(DomainError::MissingEntity(target))?
                    .properties = properties
            }
            EntityRef::CableCore(id) => {
                self.cable_cores
                    .get_mut(&id)
                    .ok_or(DomainError::MissingEntity(target))?
                    .properties = properties
            }
            EntityRef::TerminalStrip(id) => {
                self.terminal_strips
                    .get_mut(&id)
                    .ok_or(DomainError::MissingEntity(target))?
                    .properties = properties
            }
            EntityRef::Connector(id) => {
                self.connectors
                    .get_mut(&id)
                    .ok_or(DomainError::MissingEntity(target))?
                    .properties = properties
            }
        }
        Ok(())
    }
}

/// Etapa 24: o esquema padrão não é gravado (projetos sem ajuste ficam iguais).
fn is_default_scheme(scheme: &CheckScheme) -> bool {
    scheme == &CheckScheme::default()
}
