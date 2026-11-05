// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Interruption and potential marker edits; start with the Project marker methods.
use super::*;
use std::collections::BTreeSet;

fn invalid() -> DomainError {
    DomainError::InvalidRelation("marcador, grupo ou associação elétrica inconsistente")
}
fn add<K, V: PartialEq>(
    changes: &mut Vec<MarkerChange<K, V>>,
    key: K,
    before: Option<V>,
    after: Option<V>,
) {
    if before != after {
        changes.push(MarkerChange { key, before, after });
    }
}
fn check<K: Ord, V: PartialEq>(
    map: &BTreeMap<K, V>,
    changes: &[MarkerChange<K, V>],
    forward: bool,
) -> Result<()> {
    for c in changes {
        if map.get(&c.key)
            != if forward {
                c.before.as_ref()
            } else {
                c.after.as_ref()
            }
        {
            return Err(invalid());
        }
    }
    Ok(())
}
fn write<K: Ord + Clone, V: Clone>(
    map: &mut BTreeMap<K, V>,
    changes: &[MarkerChange<K, V>],
    forward: bool,
) {
    for c in changes {
        match if forward { &c.after } else { &c.before } {
            Some(v) => {
                map.insert(c.key.clone(), v.clone());
            }
            None => {
                map.remove(&c.key);
            }
        }
    }
}
impl Project {
    pub fn marker(&self, id: RepresentationId) -> Option<&ElectricalMarker> {
        self.markers.get(&id)
    }
    pub fn markers(&self) -> impl Iterator<Item = &ElectricalMarker> {
        self.markers.values()
    }
    pub fn marker_group(&self, id: ReferenceGroupId) -> Option<&MarkerGroup> {
        self.marker_groups.get(&id)
    }
    pub fn marker_groups(&self) -> impl Iterator<Item = &MarkerGroup> {
        self.marker_groups.values()
    }
    pub(in crate::domain::project) fn marker_owns_connection(&self, id: ConnectionId) -> bool {
        self.marker_groups
            .values()
            .any(|g| g.connections.contains(&id))
    }
    fn marker_port(&self, id: RepresentationId) -> Result<PortRef> {
        let r = self.representation(id).ok_or(invalid())?;
        let d = r
            .symbol()
            .and_then(|s| self.symbol_definition(s))
            .ok_or(invalid())?;
        if d.ports().len() != 1
            || d.ports()[0].id() != "p"
            || d.ports()[0].position() != Point2D::new(0., 0.)?
        {
            return Err(invalid());
        }
        PortRef::new(id, "p")
    }
    fn group_pairs(
        &self,
        g: &MarkerGroup,
    ) -> Result<BTreeSet<(ConnectionPointId, ConnectionPointId)>> {
        let mut pairs = BTreeSet::new();
        if !g.continuity {
            return Ok(pairs);
        }
        let points = g
            .members
            .iter()
            .map(|id| {
                self.marker(*id)
                    .and_then(ElectricalMarker::point)
                    .ok_or(invalid())
            })
            .collect::<Result<Vec<_>>>()?;
        for i in 1..points.len() {
            let a = points[if g.mode == ReferenceMode::Star {
                0
            } else {
                i - 1
            }];
            let b = points[i];
            if a != b {
                pairs.insert((a.min(b), a.max(b)));
            }
        }
        Ok(pairs)
    }
    pub(in crate::domain::project) fn validate_markers(&self) -> Result<()> {
        for m in self.markers.values() {
            let r = self.representation(m.id()).ok_or(invalid())?;
            self.check_project(r.project_id())?;
            if let Some(point) = m.point() {
                self.require(EntityRef::ConnectionPoint(point))?;
            }
            if r.target()
                != m.point().map_or(
                    RepresentationTarget::Unassociated,
                    RepresentationTarget::ConnectionPoint,
                )
                || r.visual().label.as_ref() != Some(m.name())
                || self.port_binding(&self.marker_port(m.id())?) != m.point()
            {
                return Err(invalid());
            }
        }
        let mut members = BTreeSet::new();
        let mut owned = BTreeSet::new();
        for g in self.marker_groups.values() {
            if g.members.len() < 2 {
                return Err(invalid());
            }
            for id in &g.members {
                if !members.insert(*id)
                    || self
                        .marker(*id)
                        .is_none_or(|m| *m.kind() != MarkerKind::Interruption)
                {
                    return Err(invalid());
                }
            }
            let expected = self.group_pairs(g)?;
            let mut actual = BTreeSet::new();
            for id in &g.connections {
                if !owned.insert(*id) || self.drawing_owns_connection(*id) {
                    return Err(invalid());
                }
                let edge = self.connection(*id).ok_or(invalid())?;
                if !actual.insert(edge.endpoints()) {
                    return Err(invalid());
                }
            }
            if actual != expected {
                return Err(invalid());
            }
        }
        Ok(())
    }
    pub fn marker_references(&self, id: RepresentationId) -> Result<Vec<RepresentationId>> {
        let m = self.marker(id).ok_or(invalid())?;
        if matches!(m.kind(), MarkerKind::Potential { .. }) {
            let Some(point) = m.point() else {
                return Ok(vec![]);
            };
            let net = self.network_for_point(point)?;
            return Ok(self
                .markers
                .values()
                .filter(|other| {
                    other.id() != id
                        && matches!(other.kind(), MarkerKind::Potential { .. })
                        && other
                            .point()
                            .is_some_and(|point| net.points().binary_search(&point).is_ok())
                })
                .map(ElectricalMarker::id)
                .collect());
        }
        let Some(g) = self
            .marker_groups
            .values()
            .find(|g| g.members.contains(&id))
        else {
            return Ok(vec![]);
        };
        let index = g.members.iter().position(|m| *m == id).ok_or(invalid())?;
        Ok(match g.mode {
            ReferenceMode::Star if index == 0 => g.members[1..].to_vec(),
            ReferenceMode::Star => vec![g.members[0]],
            ReferenceMode::Chain => index
                .checked_sub(1)
                .into_iter()
                .chain((index + 1 < g.members.len()).then_some(index + 1))
                .map(|i| g.members[i])
                .collect(),
        })
    }
    /// Diagnostics only: contradictory declarations never change the graph.
    pub fn potential_conflicts(&self) -> Vec<PotentialConflict> {
        let mut network_by_point = BTreeMap::new();
        for net in self.networks() {
            for p in net.points() {
                network_by_point.insert(*p, net.points()[0]);
            }
        }
        let mut groups: BTreeMap<ConnectionPointId, Vec<&ElectricalMarker>> = BTreeMap::new();
        for m in self
            .markers
            .values()
            .filter(|m| matches!(m.kind(), MarkerKind::Potential { .. }))
        {
            if let Some(net) = m.point().and_then(|point| network_by_point.get(&point)) {
                groups.entry(*net).or_default().push(m);
            }
        }
        let mut result = vec![];
        for group in groups.values() {
            for (i, a) in group.iter().enumerate() {
                for b in &group[i + 1..] {
                    let kinds = match (a.kind(), b.kind()) {
                        (
                            MarkerKind::Potential { kind: Some(x), .. },
                            MarkerKind::Potential { kind: Some(y), .. },
                        ) => x != y,
                        _ => false,
                    };
                    if a.name() != b.name() || kinds {
                        result.push(PotentialConflict {
                            first: a.id().min(b.id()),
                            second: a.id().max(b.id()),
                        });
                    }
                }
            }
        }
        result.sort_by_key(|c| (c.first, c.second));
        result
    }
    pub fn edit_marker(&mut self, edit: MarkerEdit) -> Result<MarkerDelta> {
        let mut d = MarkerDelta::new(self.id());
        match edit {
            MarkerEdit::Insert {
                marker,
                mut representation,
                definition,
            } => {
                let id = marker.id();
                if self.marker(id).is_some()
                    || self.representation(id).is_some()
                    || representation.id() != id
                    || representation.symbol() != Some(definition.id())
                {
                    return Err(invalid());
                }
                if let Some(old) = self.symbol_definition(definition.id()) {
                    if old != definition.as_ref() {
                        return Err(invalid());
                    }
                } else {
                    add(
                        &mut d.definitions,
                        definition.id().clone(),
                        None,
                        Some(*definition),
                    );
                }
                representation.target = marker.point().map_or(
                    RepresentationTarget::Unassociated,
                    RepresentationTarget::ConnectionPoint,
                );
                representation.visual.label = Some(marker.name().clone());
                add(
                    &mut d.bindings,
                    PortRef::new(id, "p")?,
                    None,
                    marker.point(),
                );
                add(&mut d.markers, id, None, Some(marker));
                add(&mut d.representations, id, None, Some(*representation));
            }
            MarkerEdit::Update {
                id,
                name,
                kind,
                point,
            } => {
                let old = self.marker(id).ok_or(invalid())?;
                if old.point() != point
                    && self
                        .marker_groups
                        .values()
                        .any(|g| g.continuity && g.members.contains(&id))
                {
                    return Err(invalid());
                }
                let next = ElectricalMarker::new(id, name, kind, point);
                let old_rep = self.representation(id).ok_or(invalid())?;
                let mut rep = old_rep.clone();
                rep.target = point.map_or(
                    RepresentationTarget::Unassociated,
                    RepresentationTarget::ConnectionPoint,
                );
                rep.visual.label = Some(next.name().clone());
                add(&mut d.markers, id, Some(old.clone()), Some(next));
                add(&mut d.representations, id, Some(old_rep.clone()), Some(rep));
                let port = self.marker_port(id)?;
                add(
                    &mut d.bindings,
                    port.clone(),
                    self.port_binding(&port),
                    point,
                );
            }
            MarkerEdit::Delete { id } => {
                if self.marker_groups.values().any(|g| g.members.contains(&id)) {
                    return Err(invalid());
                }
                let m = self.marker(id).ok_or(invalid())?;
                let r = self.representation(id).ok_or(invalid())?;
                let port = self.marker_port(id)?;
                add(
                    &mut d.bindings,
                    port.clone(),
                    self.port_binding(&port),
                    None,
                );
                add(&mut d.markers, id, Some(m.clone()), None);
                add(&mut d.representations, id, Some(r.clone()), None);
                let definition = r
                    .symbol()
                    .and_then(|id| self.symbol_definition(id))
                    .ok_or(invalid())?;
                if !self
                    .representations()
                    .any(|other| other.id() != id && other.symbol() == r.symbol())
                {
                    add(
                        &mut d.definitions,
                        definition.id().clone(),
                        Some(definition.clone()),
                        None,
                    );
                }
            }
            MarkerEdit::Move {
                id,
                page,
                transform,
            } => {
                self.marker(id).ok_or(invalid())?;
                if self.connection_routes.values().any(|r| {
                    r.source().representation_id() == id || r.target().representation_id() == id
                }) {
                    return Err(invalid());
                }
                let before = self.representation(id).ok_or(invalid())?;
                let mut after = before.clone();
                after.page_id = page;
                after.transform = transform;
                add(
                    &mut d.representations,
                    id,
                    Some(before.clone()),
                    Some(after),
                );
            }
            MarkerEdit::SetGroup {
                id,
                mode,
                members,
                continuity,
            } => {
                let before = self.marker_group(id);
                if before.is_some_and(|g| {
                    g.mode == mode && g.members == members && g.continuity == continuity
                }) {
                    return Ok(d);
                }
                let mut after = MarkerGroup {
                    id,
                    mode,
                    members,
                    continuity,
                    connections: vec![],
                };
                if let Some(old) = before {
                    for edge in &old.connections {
                        add(
                            &mut d.connections,
                            *edge,
                            Some(self.connection(*edge).ok_or(invalid())?.clone()),
                            None,
                        );
                    }
                }
                for (a, b) in self.group_pairs(&after)? {
                    let edge = Connection::new(self.id(), a, b)?;
                    after.connections.push(edge.id());
                    add(&mut d.connections, edge.id(), None, Some(edge));
                }
                add(&mut d.groups, id, before.cloned(), Some(after));
            }
            MarkerEdit::DeleteGroup { id } => {
                let group = self.marker_group(id).ok_or(invalid())?;
                for edge in &group.connections {
                    add(
                        &mut d.connections,
                        *edge,
                        Some(self.connection(*edge).ok_or(invalid())?.clone()),
                        None,
                    );
                }
                add(&mut d.groups, id, Some(group.clone()), None);
            }
        }
        self.restore_marker(&d, true)?;
        Ok(d)
    }
    fn write_marker_delta(&mut self, d: &MarkerDelta, forward: bool) {
        write(&mut self.markers, &d.markers, forward);
        write(&mut self.marker_groups, &d.groups, forward);
        write(&mut self.representations, &d.representations, forward);
        write(&mut self.symbol_definitions, &d.definitions, forward);
        write(&mut self.port_bindings, &d.bindings, forward);
        write(&mut self.connections, &d.connections, forward);
    }
    pub fn restore_marker(&mut self, d: &MarkerDelta, forward: bool) -> Result<()> {
        self.check_project(d.project)?;
        check(&self.markers, &d.markers, forward)?;
        check(&self.marker_groups, &d.groups, forward)?;
        check(&self.representations, &d.representations, forward)?;
        check(&self.symbol_definitions, &d.definitions, forward)?;
        check(&self.port_bindings, &d.bindings, forward)?;
        check(&self.connections, &d.connections, forward)?;
        if d.is_empty() {
            return Ok(());
        }
        self.write_marker_delta(d, forward);
        let indices = (|| {
            Ok((
                self.build_document_index()?,
                self.build_graph()?,
                self.build_physical_index()?,
            ))
        })();
        let (document, graph, physical) = match indices {
            Ok(indices) => indices,
            Err(e) => {
                self.write_marker_delta(d, !forward);
                return Err(e);
            }
        };
        let old_document = std::mem::replace(&mut self.document_index, document);
        let old_graph = std::mem::replace(&mut self.graph, graph);
        let old_physical = std::mem::replace(&mut self.physical_index, physical);
        if let Err(e) = self.validate() {
            self.write_marker_delta(d, !forward);
            self.document_index = old_document;
            self.graph = old_graph;
            self.physical_index = old_physical;
            return Err(e);
        }
        Ok(())
    }
}
