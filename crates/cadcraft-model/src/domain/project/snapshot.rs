// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::*;
use serde::{Deserialize, Serialize, Serializer, ser::SerializeSeq};

/// DTO interno independente de JSON/arquivo. Listas tornam duplicação de ID detectável.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProjectData {
    id: ProjectId,
    name: Text,
    description: String,
    #[serde(default)]
    identification: Option<Text>,
    metadata: Metadata,
    properties: TechnicalProperties,
    #[serde(default)]
    numbering_scheme: NumberingScheme,
    #[serde(default)]
    check_scheme: CheckScheme,
    documents: Vec<Document>,
    pages: Vec<Page>,
    devices: Vec<Device>,
    functions: Vec<Function>,
    components: Vec<Component>,
    #[serde(default)]
    parts: Vec<Part>,
    terminals: Vec<Terminal>,
    connection_points: Vec<ConnectionPoint>,
    #[serde(default)]
    connections: Vec<Connection>,
    #[serde(default)]
    conductors: Vec<Conductor>,
    #[serde(default)]
    cables: Vec<Cable>,
    #[serde(default)]
    cable_cores: Vec<CableCore>,
    #[serde(default)]
    terminal_strips: Vec<TerminalStrip>,
    #[serde(default)]
    terminal_bridges: Vec<TerminalBridge>,
    #[serde(default)]
    connectors: Vec<Connector>,
    #[serde(default)]
    connector_matings: Vec<ConnectorMating>,
    #[serde(default)]
    representations: Vec<Representation>,
    #[serde(default)]
    symbol_definitions: Vec<SymbolDefinition>,
    #[serde(default)]
    port_bindings: Vec<PortBinding>,
    #[serde(default)]
    connection_routes: Vec<ConnectionRoute>,
    #[serde(default)]
    connection_junctions: Vec<ConnectionJunction>,
    #[serde(default)]
    markers: Vec<ElectricalMarker>,
    #[serde(default)]
    marker_groups: Vec<MarkerGroup>,
    #[serde(default)]
    frames: Vec<FrameTemplate>,
    #[serde(default)]
    default_frame: Option<FrameTemplateId>,
    #[serde(default)]
    reference_format: ReferenceFormat,
    #[serde(default)]
    assets: Vec<ImageAsset>,
}

pub(super) fn serialize_entities<S, K, V>(
    map: &BTreeMap<K, V>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
    V: Serialize,
{
    let mut sequence = serializer.serialize_seq(Some(map.len()))?;
    for entity in map.values() {
        sequence.serialize_element(entity)?;
    }
    sequence.end()
}

fn collect_unique<K: Ord + Copy, V>(
    values: Vec<V>,
    key: impl Fn(&V) -> K,
    reference: impl Fn(K) -> EntityRef,
) -> Result<BTreeMap<K, V>> {
    let mut map = BTreeMap::new();
    for value in values {
        let id = key(&value);
        if map.insert(id, value).is_some() {
            return Err(DomainError::DuplicateEntity(reference(id)));
        }
    }
    Ok(map)
}

impl TryFrom<ProjectData> for Project {
    type Error = DomainError;
    fn try_from(data: ProjectData) -> Result<Self> {
        let part_count = data.parts.len();
        let mut project = Self {
            id: data.id,
            name: data.name,
            description: data.description,
            identification: data.identification,
            metadata: data.metadata,
            properties: data.properties,
            numbering_scheme: data.numbering_scheme,
            check_scheme: data.check_scheme,
            documents: collect_unique(data.documents, Document::id, EntityRef::Document)?,
            pages: collect_unique(data.pages, Page::id, EntityRef::Page)?,
            devices: collect_unique(data.devices, Device::id, EntityRef::Device)?,
            functions: collect_unique(data.functions, Function::id, EntityRef::Function)?,
            components: collect_unique(data.components, Component::id, EntityRef::Component)?,
            parts: data
                .parts
                .into_iter()
                .map(|part| (part.id().clone(), part))
                .collect(),
            terminals: collect_unique(data.terminals, Terminal::id, EntityRef::Terminal)?,
            connection_points: collect_unique(
                data.connection_points,
                ConnectionPoint::id,
                EntityRef::ConnectionPoint,
            )?,
            connections: BTreeMap::new(),
            conductors: collect_unique(data.conductors, Conductor::id, EntityRef::Conductor)?,
            cables: collect_unique(data.cables, Cable::id, EntityRef::Cable)?,
            cable_cores: collect_unique(data.cable_cores, CableCore::id, EntityRef::CableCore)?,
            terminal_strips: collect_unique(
                data.terminal_strips,
                TerminalStrip::id,
                EntityRef::TerminalStrip,
            )?,
            terminal_bridges: BTreeMap::new(),
            connectors: collect_unique(data.connectors, Connector::id, EntityRef::Connector)?,
            connector_matings: BTreeMap::new(),
            physical_index: PhysicalIndex::default(),
            representations: BTreeMap::new(),
            symbol_definitions: BTreeMap::new(),
            port_bindings: BTreeMap::new(),
            connection_routes: BTreeMap::new(),
            connection_junctions: BTreeMap::new(),
            markers: BTreeMap::new(),
            marker_groups: BTreeMap::new(),
            frames: BTreeMap::new(),
            default_frame: data.default_frame,
            reference_format: data.reference_format,
            assets: BTreeMap::new(),
            document_index: DocumentIndex::default(),
            graph: ConnectivityGraph::default(),
        };
        if project.parts.len() != part_count {
            return Err(DomainError::InvalidRelation("ID de peça duplicado"));
        }
        for connection in data.connections {
            let id = connection.id();
            if project.connections.insert(id, connection).is_some() {
                return Err(DomainError::DuplicateConnectionId(id));
            }
        }
        for bridge in data.terminal_bridges {
            if project
                .terminal_bridges
                .insert(bridge.id(), bridge)
                .is_some()
            {
                return Err(DomainError::InvalidRelation("ponte duplicada"));
            }
        }
        for mating in data.connector_matings {
            if project
                .connector_matings
                .insert(mating.id(), mating)
                .is_some()
            {
                return Err(DomainError::InvalidRelation("acoplamento duplicado"));
            }
        }
        for definition in data.symbol_definitions {
            if project
                .symbol_definitions
                .insert(definition.id().clone(), definition)
                .is_some()
            {
                return Err(DomainError::InvalidRelation(
                    "definição de símbolo duplicada",
                ));
            }
        }
        for representation in data.representations {
            let id = representation.id();
            if project.representations.insert(id, representation).is_some() {
                return Err(DomainError::DuplicateRepresentation(id));
            }
        }
        for binding in data.port_bindings {
            if project
                .port_bindings
                .insert(binding.port, binding.point)
                .is_some()
            {
                return Err(drawing_error());
            }
        }
        for route in data.connection_routes {
            if project
                .connection_routes
                .insert(route.id(), route)
                .is_some()
            {
                return Err(drawing_error());
            }
        }
        for junction in data.connection_junctions {
            if project
                .connection_junctions
                .insert(junction.id(), junction)
                .is_some()
            {
                return Err(drawing_error());
            }
        }
        for marker in data.markers {
            if project.markers.insert(marker.id(), marker).is_some() {
                return Err(DomainError::InvalidRelation("marcador duplicado"));
            }
        }
        for asset in data.assets {
            if project.assets.insert(asset.id(), asset).is_some() {
                return Err(DomainError::InvalidRelation("imagem duplicada"));
            }
        }
        for frame in data.frames {
            if project.frames.insert(frame.id(), frame).is_some() {
                return Err(DomainError::InvalidRelation("moldura duplicada"));
            }
        }
        for group in data.marker_groups {
            if project.marker_groups.insert(group.id(), group).is_some() {
                return Err(DomainError::InvalidRelation(
                    "grupo de referências duplicado",
                ));
            }
        }
        // Etapa 24 (DEC-11-01, migração 8→9): acoplamentos antigos recebem as
        // conexões de posse na carga.
        let legacy: Vec<_> = project
            .connector_matings
            .values()
            .filter(|m| m.owned_is_legacy())
            .cloned()
            .collect();
        for mating in legacy {
            let (owned, created, _) = project.plan_mating_connections(&mating, None)?;
            for connection in created {
                project.connections.insert(connection.id(), connection);
            }
            project
                .connector_matings
                .insert(mating.id(), mating.with_owned(owned));
        }
        project.document_index = project.build_document_index()?;
        project.graph = project.build_graph()?;
        project.physical_index = project.build_physical_index()?;
        project.validate()?;
        Ok(project)
    }
}
