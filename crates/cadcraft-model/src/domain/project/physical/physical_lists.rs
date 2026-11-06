// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Derived physical engineering lists; start with the Project list queries.
use super::*;

/// Derived terminal row, recomputed from existing physical and logical entities rather than stored
/// in Project.
#[derive(Clone, Debug, PartialEq)]
pub struct TerminalListRow {
    /// Associated physical terminal or contact.
    pub terminal: TerminalId,
    /// Documentary designation; does not imply electrical connectivity.
    pub designation: String,
    /// Physical owner of the terminal.
    pub owner: String,
    /// Device owning the terminal, directly or through its component.
    pub owner_device: Option<DeviceId>,
    /// Terminal strip associated with the record.
    pub strip: Option<TerminalStripId>,
    /// Display designation of the associated strip.
    pub strip_designation: Option<String>,
    /// One-based position in the strip's explicit order, independent of identity.
    pub order: Option<usize>,
    /// Registered conductor landing positions.
    pub positions: Vec<String>,
    /// Connection points or authored geometric vertices.
    pub points: Vec<ConnectionPointId>,
    /// Explicit logical connections associated with this record.
    pub connections: Vec<ConnectionId>,
    /// Owned terminal bridge connection identifiers.
    pub bridges: Vec<ConnectionId>,
    /// Physical conductors associated with this record.
    pub conductors: Vec<ConductorId>,
    /// Declared specialization implying no continuity.
    pub kind: TerminalKind,
    /// Levels in declared order, formatted as designation followed by positions.
    pub levels: Vec<String>,
    /// Connector containing the terminal as a contact.
    pub connector: Option<ConnectorId>,
    /// Display designation of the containing connector.
    pub connector_designation: Option<String>,
}

/// Derived connector contact; empty `points` and `conductors` mean free.
#[derive(Clone, Debug, PartialEq)]
pub struct ConnectorContactRow {
    /// Associated physical terminal or contact.
    pub terminal: TerminalId,
    /// Documentary designation; does not imply electrical connectivity.
    pub designation: String,
    /// Registered conductor landing positions.
    pub positions: Vec<String>,
    /// Connection points or authored geometric vertices.
    pub points: Vec<ConnectionPointId>,
    /// Physical conductors associated with this record.
    pub conductors: Vec<ConductorId>,
    /// Counterpart contact from the explicit map, without inferred continuity.
    pub mated_with: Option<TerminalId>,
    /// Display label of the explicitly mapped mating counterpart.
    pub mated_label: Option<String>,
}
impl ConnectorContactRow {
    /// Whether no explicit functional point or physical conductor occupies the contact.
    pub fn is_free(&self) -> bool {
        self.points.is_empty() && self.conductors.is_empty()
    }
}

/// Derived connector data and ordered contact status rows.
#[derive(Clone, Debug, PartialEq)]
pub struct ConnectorListRow {
    /// Associated industrial connector.
    pub connector: ConnectorId,
    /// Documentary designation; does not imply electrical connectivity.
    pub designation: String,
    /// Declared mechanical connector gender.
    pub gender: ConnectorGender,
    /// Associated logical device.
    pub device: Option<DeviceId>,
    /// Display label of the associated device.
    pub device_label: Option<String>,
    /// Explicit mating record, when present.
    pub mating: Option<ConnectorMatingId>,
    /// Explicitly mapped mating counterpart, when present.
    pub mated_with: Option<String>,
    /// Connector contact rows in declared order.
    pub contacts: Vec<ConnectorContactRow>,
}

/// Derived terminal-strip row with explicit terminal ordering.
#[derive(Clone, Debug, PartialEq)]
pub struct StripListRow {
    /// Terminal strip associated with the record.
    pub strip: TerminalStripId,
    /// Documentary designation; does not imply electrical connectivity.
    pub designation: String,
    /// Terminal entries associated with this record.
    pub terminals: Vec<TerminalId>,
    /// Owned terminal bridge connection identifiers.
    pub bridges: Vec<ConnectionId>,
}

/// Derived cable data, capacity and registered-core usage counts.
#[derive(Clone, Debug, PartialEq)]
pub struct CableListRow {
    /// Associated physical cable.
    pub cable: CableId,
    /// Documentary designation; does not imply electrical connectivity.
    pub designation: String,
    /// Declared category of this record or operation.
    pub kind: CableKind,
    /// Descriptive text.
    pub description: Option<String>,
    /// Declared manufacturer.
    pub manufacturer: Option<String>,
    /// Manufacturer catalog code.
    pub catalog_code: Option<String>,
    /// Declared cable core capacity.
    pub nominal_cores: u32,
    /// Number of registered cores.
    pub registered_cores: usize,
    /// Number of cores assigned to conductors.
    pub used_cores: usize,
    /// Distinct terminals reached by assigned cores with known endpoints.
    pub endpoints: Vec<String>,
}

/// Derived cable-core assignment and endpoint labels.
#[derive(Clone, Debug, PartialEq)]
pub struct CoreListRow {
    /// Associated cable core.
    pub core: CableCoreId,
    /// Associated physical cable.
    pub cable: CableId,
    /// Display designation of the containing cable.
    pub cable_designation: String,
    /// Validated documentary identification.
    pub identification: String,
    /// Physical color or identification marking.
    pub marking: String,
    /// User-supplied cross section; absence is never inferred.
    pub cross_section_mm2: Option<f64>,
    /// Associated physical conductor.
    pub conductor: Option<ConductorId>,
    /// Explicit source endpoint.
    pub source: Option<String>,
    /// Explicit target of this record or operation.
    pub target: Option<String>,
}

/// Derived physical conductor data and cable/core associations.
#[derive(Clone, Debug, PartialEq)]
pub struct ConductorListRow {
    /// Associated physical conductor.
    pub conductor: ConductorId,
    /// Documentary designation; does not imply electrical connectivity.
    pub designation: String,
    /// Explicit source endpoint.
    pub source: String,
    /// Explicit target of this record or operation.
    pub target: String,
    /// Physical terminal at the source endpoint.
    pub source_terminal: TerminalId,
    /// Physical terminal at the target endpoint.
    pub target_terminal: TerminalId,
    /// Conductor cross section in square millimetres.
    pub cross_section_mm2: f64,
    /// Physical color or identification marking.
    pub marking: String,
    /// Associated physical cable.
    pub cable: Option<CableId>,
    /// Associated cable core.
    pub core: Option<CableCoreId>,
    /// Display label of the associated cable.
    pub cable_label: Option<String>,
    /// Explicit logical connection associated with this record.
    pub connection: Option<ConnectionId>,
}

/// Explicit mapping from logical connections to physical conductors. Bridges are marked and never
/// assigned a conductor.
#[derive(Clone, Debug, PartialEq)]
pub struct WiringAssignmentRow {
    /// Explicit logical connection associated with this record.
    pub connection: ConnectionId,
    /// Explicit source endpoint.
    pub source: String,
    /// Explicit target of this record or operation.
    pub target: String,
    /// Functional connection point at the source endpoint.
    pub source_point: ConnectionPointId,
    /// Functional connection point at the target endpoint.
    pub target_point: ConnectionPointId,
    /// Whether the logical connection is owned by a terminal bridge.
    pub bridge: bool,
    /// Whether a graphical route represents this connection.
    pub drawn: bool,
    /// Physical conductors associated with this record.
    pub conductors: Vec<ConductorId>,
}

/// Derived technical lists and explicit logical-to-physical assignments.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PhysicalLists {
    /// Derived terminal-strip rows.
    pub strips: Vec<StripListRow>,
    /// Terminal entries associated with this record.
    pub terminals: Vec<TerminalListRow>,
    /// Derived cable rows.
    pub cables: Vec<CableListRow>,
    /// Derived cable-core rows.
    pub cores: Vec<CoreListRow>,
    /// Physical conductors associated with this record.
    pub conductors: Vec<ConductorListRow>,
    /// Explicit logical-connection to physical-conductor assignment rows.
    pub assignments: Vec<WiringAssignmentRow>,
    /// Derived industrial connector rows.
    pub connectors: Vec<ConnectorListRow>,
}

fn marking_text(marking: &ConductorMarking) -> String {
    match (marking.color_value(), marking.identification()) {
        (Some(color), Some(id)) => format!("{color:?} · {}", id.as_str()),
        (Some(color), None) => format!("{color:?}"),
        (None, Some(id)) => id.as_str().to_owned(),
        (None, None) => String::new(),
    }
}

impl Project {
    pub(in crate::domain::project) fn terminal_owner_device(
        &self,
        terminal: &Terminal,
    ) -> Option<DeviceId> {
        match terminal.owner() {
            TerminalOwner::Device(id) => Some(id),
            TerminalOwner::Component(id) => self.component(id).and_then(Component::device_id),
        }
    }
    /// Documentary terminal label: `-X1:1` within a strip, otherwise `K1:A1` using the owner.
    pub fn terminal_label(&self, id: TerminalId) -> String {
        let Some(terminal) = self.terminal(id) else {
            return "?".into();
        };
        if let Some(strip) = self.strip_for_terminal(id) {
            return format!(
                "{}:{}",
                strip.designation().as_str(),
                terminal.designation().as_str()
            );
        }
        if let Some(connector) = self.connector_for_terminal(id) {
            return format!(
                "{}:{}",
                connector.designation().as_str(),
                terminal.designation().as_str()
            );
        }
        let owner = self
            .terminal_owner_device(terminal)
            .and_then(|d| self.device(d))
            .map(|d| d.designation().to_string())
            .unwrap_or_default();
        format!("{owner}:{}", terminal.designation().as_str())
    }
    /// Format a connection-point label from explicit terminal or function ownership.
    pub fn connection_point_label(&self, id: ConnectionPointId) -> String {
        let Some(point) = self.connection_point(id) else {
            return "?".into();
        };
        let owner = self
            .function(point.function_id())
            .map(|f| f.designation().to_string())
            .unwrap_or_default();
        format!("{owner}:{}", point.designation().as_str())
    }
    fn termination_label(&self, end: &PhysicalTermination) -> String {
        format!(
            "{}/{}",
            self.terminal_label(end.terminal_id()),
            end.position().as_str()
        )
    }

    /// Technical lists derived in O(n log n) from current entities.
    pub fn physical_lists(&self) -> PhysicalLists {
        let mut points_by_terminal: BTreeMap<TerminalId, Vec<ConnectionPointId>> = BTreeMap::new();
        for point in self.connection_points.values() {
            if let Some(t) = point.terminal_id() {
                points_by_terminal.entry(t).or_default().push(point.id());
            }
        }
        let mut connections_by_point: BTreeMap<ConnectionPointId, Vec<ConnectionId>> =
            BTreeMap::new();
        for c in self.connections.values() {
            let (a, b) = c.endpoints();
            connections_by_point.entry(a).or_default().push(c.id());
            connections_by_point.entry(b).or_default().push(c.id());
        }
        let mut wires_by_terminal: BTreeMap<TerminalId, Vec<ConductorId>> = BTreeMap::new();
        let mut wires_by_connection: BTreeMap<ConnectionId, Vec<ConductorId>> = BTreeMap::new();
        for wire in self.conductors.values() {
            let (a, b) = wire.endpoints();
            wires_by_terminal
                .entry(a.terminal_id())
                .or_default()
                .push(wire.id());
            if b.terminal_id() != a.terminal_id() {
                wires_by_terminal
                    .entry(b.terminal_id())
                    .or_default()
                    .push(wire.id());
            }
            if let Some(link) = wire.connection() {
                wires_by_connection
                    .entry(link.connection_id())
                    .or_default()
                    .push(wire.id());
            }
        }
        let mut strip_of: BTreeMap<TerminalId, (TerminalStripId, usize)> = BTreeMap::new();
        for strip in self.terminal_strips.values() {
            for (index, t) in strip.terminals().iter().enumerate() {
                strip_of.insert(*t, (strip.id(), index + 1));
            }
        }
        let mut bridges_by_strip: BTreeMap<TerminalStripId, Vec<ConnectionId>> = BTreeMap::new();
        for bridge in self.terminal_bridges.values() {
            bridges_by_strip
                .entry(bridge.strip_id())
                .or_default()
                .push(bridge.id());
        }
        let core_by_wire: BTreeMap<ConductorId, &CableCore> = self
            .cable_cores
            .values()
            .filter_map(|core| core.conductor_id().map(|w| (w, core)))
            .collect();

        let strips = self
            .terminal_strips
            .values()
            .map(|s| StripListRow {
                strip: s.id(),
                designation: s.designation().as_str().into(),
                terminals: s.terminals().to_vec(),
                bridges: bridges_by_strip.get(&s.id()).cloned().unwrap_or_default(),
            })
            .collect();

        let terminals = self
            .terminals
            .values()
            .map(|t| {
                let points = points_by_terminal.get(&t.id()).cloned().unwrap_or_default();
                let mut connections: Vec<ConnectionId> = points
                    .iter()
                    .filter_map(|p| connections_by_point.get(p))
                    .flatten()
                    .copied()
                    .collect();
                connections.sort();
                connections.dedup();
                let bridges = connections
                    .iter()
                    .copied()
                    .filter(|c| self.terminal_bridges.contains_key(c))
                    .collect();
                let device = self.terminal_owner_device(t);
                let strip = strip_of.get(&t.id()).copied();
                TerminalListRow {
                    terminal: t.id(),
                    designation: t.designation().as_str().into(),
                    owner: device
                        .and_then(|d| self.device(d))
                        .map(|d| d.designation().to_string())
                        .unwrap_or_default(),
                    owner_device: device,
                    strip: strip.map(|s| s.0),
                    strip_designation: strip
                        .and_then(|s| self.terminal_strip(s.0))
                        .map(|s| s.designation().as_str().into()),
                    order: strip.map(|s| s.1),
                    positions: t.positions().map(|p| p.as_str().to_owned()).collect(),
                    points,
                    connections,
                    bridges,
                    conductors: wires_by_terminal.get(&t.id()).cloned().unwrap_or_default(),
                    kind: t.kind(),
                    levels: t
                        .levels()
                        .iter()
                        .map(|l| {
                            let positions: Vec<_> = l.positions().map(|p| p.as_str()).collect();
                            format!("{} ({})", l.designation().as_str(), positions.join("/"))
                        })
                        .collect(),
                    connector: self
                        .physical_index
                        .connector_by_terminal
                        .get(&t.id())
                        .copied(),
                    connector_designation: self
                        .connector_for_terminal(t.id())
                        .map(|c| c.designation().as_str().to_owned()),
                }
            })
            .collect();

        let mut cores = Vec::new();
        let mut cables = Vec::new();
        for cable in self.cables.values() {
            let mut registered = 0;
            let mut used = 0;
            let mut endpoints = Vec::new();
            for core in self.cores_for_cable(cable.id()) {
                registered += 1;
                let wire = core.conductor_id().and_then(|w| self.conductor(w));
                if wire.is_some() {
                    used += 1;
                }
                let (source, target) = wire
                    .map(|w| {
                        let (a, b) = w.endpoints();
                        (
                            Some(self.termination_label(a)),
                            Some(self.termination_label(b)),
                        )
                    })
                    .unwrap_or((None, None));
                if let Some(w) = wire {
                    let (a, b) = w.endpoints();
                    endpoints.push(self.terminal_label(a.terminal_id()));
                    endpoints.push(self.terminal_label(b.terminal_id()));
                }
                cores.push(CoreListRow {
                    core: core.id(),
                    cable: cable.id(),
                    cable_designation: cable.designation().as_str().into(),
                    identification: core.identification().as_str().into(),
                    marking: marking_text(core.marking()),
                    cross_section_mm2: core.cross_section().map(CrossSectionMm2::get),
                    conductor: core.conductor_id(),
                    source,
                    target,
                });
            }
            endpoints.sort();
            endpoints.dedup();
            cables.push(CableListRow {
                cable: cable.id(),
                designation: cable.designation().as_str().into(),
                kind: cable.kind(),
                description: cable.description().map(|v| v.as_str().into()),
                manufacturer: cable.manufacturer().map(|v| v.as_str().into()),
                catalog_code: cable.catalog_code().map(|v| v.as_str().into()),
                nominal_cores: cable.nominal_core_count(),
                registered_cores: registered,
                used_cores: used,
                endpoints,
            });
        }

        let conductors = self
            .conductors
            .values()
            .map(|w| {
                let (a, b) = w.endpoints();
                let core = core_by_wire.get(&w.id()).copied();
                let cable = core.and_then(|c| self.cable(c.cable_id()));
                ConductorListRow {
                    conductor: w.id(),
                    designation: w.designation().as_str().into(),
                    source: self.termination_label(a),
                    target: self.termination_label(b),
                    source_terminal: a.terminal_id(),
                    target_terminal: b.terminal_id(),
                    cross_section_mm2: w.cross_section().get(),
                    marking: marking_text(w.marking()),
                    cable: cable.map(Cable::id),
                    core: core.map(CableCore::id),
                    cable_label: core.zip(cable).map(|(c, k)| {
                        format!(
                            "{}:{}",
                            k.designation().as_str(),
                            c.identification().as_str()
                        )
                    }),
                    connection: w.connection().map(ConductorConnection::connection_id),
                }
            })
            .collect();

        let assignments = self
            .connections
            .values()
            .map(|c| {
                let (a, b) = c.endpoints();
                WiringAssignmentRow {
                    connection: c.id(),
                    source: self.connection_point_label(a),
                    target: self.connection_point_label(b),
                    source_point: a,
                    target_point: b,
                    bridge: self.terminal_bridges.contains_key(&c.id()),
                    drawn: self.drawing_owns_connection(c.id()),
                    conductors: wires_by_connection
                        .get(&c.id())
                        .cloned()
                        .unwrap_or_default(),
                }
            })
            .collect();

        let connectors = self
            .connectors
            .values()
            .map(|c| {
                let mating = self.mating_for_connector(c.id());
                let partner = mating
                    .map(|m| {
                        if m.first() == c.id() {
                            m.second()
                        } else {
                            m.first()
                        }
                    })
                    .and_then(|id| self.connector(id));
                ConnectorListRow {
                    connector: c.id(),
                    designation: c.designation().as_str().into(),
                    gender: c.gender(),
                    device: c.device(),
                    device_label: c
                        .device()
                        .and_then(|d| self.device(d))
                        .map(|d| d.designation().to_string()),
                    mating: mating.map(ConnectorMating::id),
                    mated_with: partner.map(|p| p.designation().as_str().to_owned()),
                    contacts: c
                        .contacts()
                        .iter()
                        .filter_map(|id| self.terminal(*id))
                        .map(|t| {
                            let mate = mating.and_then(|m| m.mate_of(t.id()));
                            ConnectorContactRow {
                                terminal: t.id(),
                                designation: t.designation().as_str().into(),
                                positions: t.positions().map(|p| p.as_str().to_owned()).collect(),
                                points: points_by_terminal
                                    .get(&t.id())
                                    .cloned()
                                    .unwrap_or_default(),
                                conductors: wires_by_terminal
                                    .get(&t.id())
                                    .cloned()
                                    .unwrap_or_default(),
                                mated_with: mate,
                                mated_label: mate.map(|m| self.terminal_label(m)),
                            }
                        })
                        .collect(),
                }
            })
            .collect();

        PhysicalLists {
            strips,
            terminals,
            cables,
            cores,
            conductors,
            assignments,
            connectors,
        }
    }

    /// First explicit physical-object occurrence by documentary relation: the object itself, its
    /// points/connections, then its owner. Never searches by proximity or text.
    pub fn physical_occurrence(&self, target: RepresentationTarget) -> Option<RepresentationId> {
        let first = |t: RepresentationTarget| {
            self.document_index
                .by_target
                .get(&t)
                .and_then(|ids| ids.iter().next().copied())
        };
        if let Some(id) = first(target) {
            return Some(id);
        }
        let terminal_occurrence = |terminal: TerminalId| {
            first(RepresentationTarget::Terminal(terminal))
                .or_else(|| {
                    self.connection_points
                        .values()
                        .filter(|p| p.terminal_id() == Some(terminal))
                        .find_map(|p| first(RepresentationTarget::ConnectionPoint(p.id())))
                })
                .or_else(|| {
                    self.terminal(terminal)
                        .and_then(|t| self.terminal_owner_device(t))
                        .and_then(|d| first(RepresentationTarget::Device(d)))
                })
        };
        match target {
            RepresentationTarget::Terminal(id) => terminal_occurrence(id),
            RepresentationTarget::TerminalStrip(id) => self
                .terminal_strip(id)?
                .terminals()
                .iter()
                .find_map(|t| terminal_occurrence(*t)),
            RepresentationTarget::Conductor(id) => {
                let wire = self.conductor(id)?;
                wire.connection()
                    .and_then(|l| first(RepresentationTarget::Connection(l.connection_id())))
                    .or_else(|| terminal_occurrence(wire.endpoints().0.terminal_id()))
                    .or_else(|| terminal_occurrence(wire.endpoints().1.terminal_id()))
            }
            RepresentationTarget::CableCore(id) => self
                .cable_core(id)?
                .conductor_id()
                .and_then(|w| self.physical_occurrence(RepresentationTarget::Conductor(w))),
            RepresentationTarget::Cable(id) => self
                .cores_for_cable(id)
                .filter_map(CableCore::conductor_id)
                .find_map(|w| self.physical_occurrence(RepresentationTarget::Conductor(w))),
            _ => None,
        }
    }
}
