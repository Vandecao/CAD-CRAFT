// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum PhysicalEdit {
    UpsertTerminal(Terminal),
    DeleteTerminal(TerminalId),
    UpsertStrip(TerminalStrip),
    DeleteStrip(TerminalStripId),
    UpsertCable(Cable),
    DeleteCable(CableId),
    UpsertCore(CableCore),
    DeleteCore(CableCoreId),
    UpsertConductor(Conductor),
    DeleteConductor(ConductorId),
    SetCoreConductor {
        core: CableCoreId,
        conductor: Option<ConductorId>,
    },
    SetConductorConnection {
        conductor: ConductorId,
        link: Option<ConductorConnection>,
    },
    UpsertBridge(TerminalBridge),
    CreateBridge {
        strip: TerminalStripId,
        source: ConnectionPointId,
        target: ConnectionPointId,
        kind: TerminalBridgeKind,
    },
    DeleteBridge(ConnectionId),
    /// Cria ou altera conector (designação, gênero, relações, ordem de contatos).
    UpsertConnector(Connector),
    /// Remove somente o agrupamento; recusado se houver acoplamento.
    DeleteConnector(ConnectorId),
    /// Acoplamento documental explícito; não cria conexões lógicas.
    UpsertMating(ConnectorMating),
    DeleteMating(ConnectorMatingId),
    /// Associa (ou desassocia, com `None`) ponto funcional a borne/contato e,
    /// para borne multinível, ao nível. Não cria nem remove conexões.
    SetPointTerminal {
        point: ConnectionPointId,
        terminal: Option<TerminalId>,
        level: Option<TerminalLevelId>,
    },
    /// Edições aplicadas atomicamente como uma única transação reversível.
    Batch(Vec<PhysicalEdit>),
}

#[derive(Clone, Debug, PartialEq)]
struct PhysicalChange<K, V> {
    key: K,
    before: Option<V>,
    after: Option<V>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PhysicalDelta {
    project: ProjectId,
    terminals: Vec<PhysicalChange<TerminalId, Terminal>>,
    strips: Vec<PhysicalChange<TerminalStripId, TerminalStrip>>,
    cables: Vec<PhysicalChange<CableId, Cable>>,
    cores: Vec<PhysicalChange<CableCoreId, CableCore>>,
    conductors: Vec<PhysicalChange<ConductorId, Conductor>>,
    bridges: Vec<PhysicalChange<ConnectionId, TerminalBridge>>,
    connections: Vec<PhysicalChange<ConnectionId, Connection>>,
    connectors: Vec<PhysicalChange<ConnectorId, Connector>>,
    matings: Vec<PhysicalChange<ConnectorMatingId, ConnectorMating>>,
    points: Vec<PhysicalChange<ConnectionPointId, ConnectionPoint>>,
}
impl PhysicalDelta {
    pub fn len(&self) -> usize {
        self.terminals.len()
            + self.strips.len()
            + self.cables.len()
            + self.cores.len()
            + self.conductors.len()
            + self.bridges.len()
            + self.connections.len()
            + self.connectors.len()
            + self.matings.len()
            + self.points.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

fn preflight<K: Ord, V: PartialEq>(
    map: &BTreeMap<K, V>,
    changes: &[PhysicalChange<K, V>],
    forward: bool,
) -> Result<()> {
    for change in changes {
        let expected = if forward {
            &change.before
        } else {
            &change.after
        };
        if map.get(&change.key) != expected.as_ref() {
            return Err(DomainError::InvalidRelation(
                "estado físico divergente do histórico",
            ));
        }
    }
    Ok(())
}
fn write<K: Ord + Clone, V: Clone>(
    map: &mut BTreeMap<K, V>,
    changes: &[PhysicalChange<K, V>],
    forward: bool,
) {
    for change in changes {
        let next = if forward {
            &change.after
        } else {
            &change.before
        };
        if let Some(next) = next {
            map.insert(change.key.clone(), next.clone());
        } else {
            map.remove(&change.key);
        }
    }
}
fn change<K: Copy, V: PartialEq>(
    key: K,
    before: Option<V>,
    after: Option<V>,
) -> Option<PhysicalChange<K, V>> {
    (before != after).then_some(PhysicalChange { key, before, after })
}

/// Compõe mudanças sucessivas por chave: primeiro `before`, último `after`.
fn merge<'a, K: Ord + Copy + 'a, V: PartialEq + Clone + 'a>(
    parts: impl Iterator<Item = &'a Vec<PhysicalChange<K, V>>>,
) -> Vec<PhysicalChange<K, V>> {
    let mut merged: BTreeMap<K, (Option<V>, Option<V>)> = BTreeMap::new();
    for changes in parts {
        for c in changes {
            match merged.get_mut(&c.key) {
                Some(entry) => entry.1 = c.after.clone(),
                None => {
                    merged.insert(c.key, (c.before.clone(), c.after.clone()));
                }
            }
        }
    }
    merged
        .into_iter()
        .filter_map(|(key, (before, after))| change(key, before, after))
        .collect()
}

impl Project {
    pub fn terminal_bridge(&self, id: ConnectionId) -> Option<&TerminalBridge> {
        self.terminal_bridges.get(&id)
    }
    pub fn terminal_bridges(&self) -> impl Iterator<Item = &TerminalBridge> {
        self.terminal_bridges.values()
    }
    /// Aplica uma edição física validada. Erros não deixam mutação parcial.
    pub fn edit_physical(&mut self, edit: PhysicalEdit) -> Result<PhysicalDelta> {
        if let PhysicalEdit::Batch(edits) = edit {
            return self.edit_physical_batch(edits);
        }
        let delta = self.physical_delta(edit)?;
        self.restore_physical(&delta, true)?;
        Ok(delta)
    }
    /// Lote atômico: estados intermediários não são publicados nem validados
    /// isoladamente; somente o estado final é verificado e vira um único delta.
    fn edit_physical_batch(&mut self, edits: Vec<PhysicalEdit>) -> Result<PhysicalDelta> {
        let graph_before = self.graph.clone();
        let physical_before = self.physical_index.clone();
        let mut applied = Vec::new();
        let mut failure = None;
        for edit in edits {
            match self.physical_delta(edit) {
                Ok(delta) => {
                    self.write_physical(&delta, true);
                    applied.push(delta);
                }
                Err(error) => {
                    failure = Some(error);
                    break;
                }
            }
        }
        let checked = match failure {
            Some(error) => Err(error),
            None => (|| {
                self.graph = self.build_graph()?;
                self.physical_index = self.build_physical_index()?;
                self.validate()
            })(),
        };
        if let Err(error) = checked {
            for delta in applied.iter().rev() {
                self.write_physical(delta, false);
            }
            self.graph = graph_before;
            self.physical_index = physical_before;
            return Err(error);
        }
        Ok(PhysicalDelta {
            project: self.id(),
            terminals: merge(applied.iter().map(|d| &d.terminals)),
            strips: merge(applied.iter().map(|d| &d.strips)),
            cables: merge(applied.iter().map(|d| &d.cables)),
            cores: merge(applied.iter().map(|d| &d.cores)),
            conductors: merge(applied.iter().map(|d| &d.conductors)),
            bridges: merge(applied.iter().map(|d| &d.bridges)),
            connections: merge(applied.iter().map(|d| &d.connections)),
            connectors: merge(applied.iter().map(|d| &d.connectors)),
            matings: merge(applied.iter().map(|d| &d.matings)),
            points: merge(applied.iter().map(|d| &d.points)),
        })
    }
    fn write_physical(&mut self, delta: &PhysicalDelta, forward: bool) {
        write(&mut self.terminals, &delta.terminals, forward);
        write(&mut self.terminal_strips, &delta.strips, forward);
        write(&mut self.cables, &delta.cables, forward);
        write(&mut self.cable_cores, &delta.cores, forward);
        write(&mut self.conductors, &delta.conductors, forward);
        write(&mut self.connections, &delta.connections, forward);
        write(&mut self.terminal_bridges, &delta.bridges, forward);
        write(&mut self.connectors, &delta.connectors, forward);
        write(&mut self.connector_matings, &delta.matings, forward);
        write(&mut self.connection_points, &delta.points, forward);
    }
    fn physical_delta(&self, edit: PhysicalEdit) -> Result<PhysicalDelta> {
        let mut delta = PhysicalDelta {
            project: self.id(),
            terminals: Vec::new(),
            strips: Vec::new(),
            cables: Vec::new(),
            cores: Vec::new(),
            conductors: Vec::new(),
            bridges: Vec::new(),
            connections: Vec::new(),
            connectors: Vec::new(),
            matings: Vec::new(),
            points: Vec::new(),
        };
        match edit {
            PhysicalEdit::UpsertTerminal(value) => {
                let id = value.id();
                if let Some(c) = change(id, self.terminal(id).cloned(), Some(value)) {
                    delta.terminals.push(c);
                }
            }
            PhysicalEdit::DeleteTerminal(id) => {
                let before = self
                    .terminal(id)
                    .cloned()
                    .ok_or(DomainError::MissingEntity(EntityRef::Terminal(id)))?;
                // Points, conductors and strips are verified on the final state;
                // the connector membership gets a precise message here. Inside a
                // Batch the connector may already have released the contact.
                if self.connectors.values().any(|c| c.contacts().contains(&id)) {
                    return Err(DomainError::EntityInUse(EntityRef::Terminal(id)));
                }
                delta.terminals.push(PhysicalChange {
                    key: id,
                    before: Some(before),
                    after: None,
                });
            }
            PhysicalEdit::UpsertStrip(value) => {
                let id = value.id();
                if let Some(c) = change(id, self.terminal_strip(id).cloned(), Some(value)) {
                    delta.strips.push(c);
                }
            }
            PhysicalEdit::DeleteStrip(id) => {
                let before = self
                    .terminal_strip(id)
                    .cloned()
                    .ok_or(DomainError::MissingEntity(EntityRef::TerminalStrip(id)))?;
                delta.strips.push(PhysicalChange {
                    key: id,
                    before: Some(before),
                    after: None,
                });
            }
            PhysicalEdit::UpsertCable(value) => {
                let id = value.id();
                if let Some(c) = change(id, self.cable(id).cloned(), Some(value)) {
                    delta.cables.push(c);
                }
            }
            PhysicalEdit::DeleteCable(id) => {
                let before = self
                    .cable(id)
                    .cloned()
                    .ok_or(DomainError::MissingEntity(EntityRef::Cable(id)))?;
                delta.cables.push(PhysicalChange {
                    key: id,
                    before: Some(before),
                    after: None,
                });
            }
            PhysicalEdit::UpsertCore(value) => {
                let id = value.id();
                if let Some(c) = change(id, self.cable_core(id).cloned(), Some(value)) {
                    delta.cores.push(c);
                }
            }
            PhysicalEdit::DeleteCore(id) => {
                let before = self
                    .cable_core(id)
                    .cloned()
                    .ok_or(DomainError::MissingEntity(EntityRef::CableCore(id)))?;
                delta.cores.push(PhysicalChange {
                    key: id,
                    before: Some(before),
                    after: None,
                });
            }
            PhysicalEdit::UpsertConductor(value) => {
                let id = value.id();
                if let Some(c) = change(id, self.conductor(id).cloned(), Some(value)) {
                    delta.conductors.push(c);
                }
            }
            PhysicalEdit::DeleteConductor(id) => {
                let before = self
                    .conductor(id)
                    .cloned()
                    .ok_or(DomainError::MissingEntity(EntityRef::Conductor(id)))?;
                delta.conductors.push(PhysicalChange {
                    key: id,
                    before: Some(before),
                    after: None,
                });
            }
            PhysicalEdit::SetCoreConductor { core, conductor } => {
                let before = self
                    .cable_core(core)
                    .cloned()
                    .ok_or(DomainError::MissingEntity(EntityRef::CableCore(core)))?;
                let mut after = before.clone();
                after.conductor_id = conductor;
                if let Some(c) = change(core, Some(before), Some(after)) {
                    delta.cores.push(c);
                }
            }
            PhysicalEdit::SetConductorConnection { conductor, link } => {
                let before = self
                    .conductor(conductor)
                    .cloned()
                    .ok_or(DomainError::MissingEntity(EntityRef::Conductor(conductor)))?;
                let mut after = before.clone();
                after.connection = link;
                if let Some(c) = change(conductor, Some(before), Some(after)) {
                    delta.conductors.push(c);
                }
            }
            PhysicalEdit::UpsertBridge(value) => {
                // Edits kind/properties of an existing bridge only. Adopting an
                // arbitrary connection (or moving it to another strip) would let
                // a later DeleteBridge remove a connection it never created.
                let id = value.id();
                let before = self
                    .terminal_bridge(id)
                    .cloned()
                    .ok_or(DomainError::InvalidRelation("ponte ausente"))?;
                if before.strip_id() != value.strip_id()
                    || before.project_id() != value.project_id()
                {
                    return Err(DomainError::InvalidRelation("ponte não muda de régua"));
                }
                if let Some(c) = change(id, Some(before), Some(value)) {
                    delta.bridges.push(c);
                }
            }
            PhysicalEdit::CreateBridge {
                strip,
                source,
                target,
                kind,
            } => {
                let members = self
                    .terminal_strip(strip)
                    .ok_or(DomainError::InvalidRelation("régua da ponte ausente"))?
                    .terminals();
                for point in [source, target] {
                    let terminal = self
                        .connection_point(point)
                        .and_then(ConnectionPoint::terminal_id)
                        .ok_or(DomainError::InvalidRelation(
                            "ponto da ponte sem borne físico",
                        ))?;
                    if !members.contains(&terminal) {
                        return Err(DomainError::InvalidRelation("borne da ponte fora da régua"));
                    }
                }
                let connection = Connection::new(self.id(), source, target)?;
                let bridge = TerminalBridge::new(self.id(), strip, connection.id(), kind);
                delta.connections.push(PhysicalChange {
                    key: connection.id(),
                    before: None,
                    after: Some(connection),
                });
                delta.bridges.push(PhysicalChange {
                    key: bridge.id(),
                    before: None,
                    after: Some(bridge),
                });
            }
            PhysicalEdit::DeleteBridge(id) => {
                let bridge = self
                    .terminal_bridge(id)
                    .ok_or(DomainError::InvalidRelation("ponte ausente"))?
                    .clone();
                let connection = self
                    .connection(id)
                    .ok_or(DomainError::MissingConnection(id))?
                    .clone();
                delta.bridges.push(PhysicalChange {
                    key: id,
                    before: Some(bridge),
                    after: None,
                });
                delta.connections.push(PhysicalChange {
                    key: id,
                    before: Some(connection),
                    after: None,
                });
            }
            PhysicalEdit::UpsertConnector(value) => {
                let id = value.id();
                if let Some(c) = change(id, self.connector(id).cloned(), Some(value)) {
                    delta.connectors.push(c);
                }
            }
            PhysicalEdit::DeleteConnector(id) => {
                let before = self
                    .connector(id)
                    .cloned()
                    .ok_or(DomainError::MissingEntity(EntityRef::Connector(id)))?;
                // No cascade: the explicit mating must be removed first.
                if self.connector_matings.values().any(|m| m.involves(id)) {
                    return Err(DomainError::EntityInUse(EntityRef::Connector(id)));
                }
                delta.connectors.push(PhysicalChange {
                    key: id,
                    before: Some(before),
                    after: None,
                });
            }
            PhysicalEdit::UpsertMating(value) => {
                // Etapa 24 (DEC-11-01): o acoplamento possui uma conexão por par
                // de contatos com ponto; as que ligam os mesmos pontos são mantidas.
                let id = value.id();
                let before = self.connector_mating(id).cloned();
                let (owned, created, removed) =
                    self.plan_mating_connections(&value, before.as_ref())?;
                for connection in removed {
                    delta.connections.push(PhysicalChange {
                        key: connection.id(),
                        before: Some(connection),
                        after: None,
                    });
                }
                for connection in created {
                    delta.connections.push(PhysicalChange {
                        key: connection.id(),
                        before: None,
                        after: Some(connection),
                    });
                }
                if let Some(c) = change(id, before, Some(value.with_owned(owned))) {
                    delta.matings.push(c);
                }
            }
            PhysicalEdit::DeleteMating(id) => {
                let before = self
                    .connector_mating(id)
                    .cloned()
                    .ok_or(DomainError::InvalidRelation("acoplamento ausente"))?;
                for connection in before.owned_connections() {
                    let connection = self
                        .connection(*connection)
                        .ok_or(DomainError::MissingConnection(*connection))?
                        .clone();
                    delta.connections.push(PhysicalChange {
                        key: connection.id(),
                        before: Some(connection),
                        after: None,
                    });
                }
                delta.matings.push(PhysicalChange {
                    key: id,
                    before: Some(before),
                    after: None,
                });
            }
            PhysicalEdit::SetPointTerminal {
                point,
                terminal,
                level,
            } => {
                let before =
                    self.connection_point(point)
                        .cloned()
                        .ok_or(DomainError::MissingEntity(EntityRef::ConnectionPoint(
                            point,
                        )))?;
                if terminal.is_none() && level.is_some() {
                    return Err(DomainError::InvalidRelation(
                        "nível de borne sem borne associado",
                    ));
                }
                let mut after = before.clone();
                after.terminal_id = terminal;
                after.terminal_level = level;
                if let Some(c) = change(point, Some(before), Some(after)) {
                    delta.points.push(c);
                }
            }
            PhysicalEdit::Batch(_) => {
                return Err(DomainError::InvalidRelation("lote físico aninhado"));
            }
        }
        Ok(delta)
    }
    pub fn restore_physical(&mut self, delta: &PhysicalDelta, forward: bool) -> Result<()> {
        self.check_project(delta.project)?;
        preflight(&self.terminals, &delta.terminals, forward)?;
        preflight(&self.terminal_strips, &delta.strips, forward)?;
        preflight(&self.cables, &delta.cables, forward)?;
        preflight(&self.cable_cores, &delta.cores, forward)?;
        preflight(&self.conductors, &delta.conductors, forward)?;
        preflight(&self.terminal_bridges, &delta.bridges, forward)?;
        preflight(&self.connections, &delta.connections, forward)?;
        preflight(&self.connectors, &delta.connectors, forward)?;
        preflight(&self.connector_matings, &delta.matings, forward)?;
        preflight(&self.connection_points, &delta.points, forward)?;
        if delta.is_empty() {
            return Ok(());
        }
        let graph_before = self.graph.clone();
        let physical_before = self.physical_index.clone();
        self.write_physical(delta, forward);
        let checked = (|| {
            self.graph = self.build_graph()?;
            self.physical_index = self.build_physical_index()?;
            self.validate()
        })();
        if checked.is_err() {
            self.write_physical(delta, !forward);
            self.graph = graph_before;
            self.physical_index = physical_before;
        }
        checked
    }
}
