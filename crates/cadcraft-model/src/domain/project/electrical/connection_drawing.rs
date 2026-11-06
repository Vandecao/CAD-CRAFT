// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Validated connection routes, bindings and junctions; start with edit_connection_drawing.
use super::*;
use serde::{Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet};

pub(in crate::domain::project) fn serialize_bindings<S: Serializer>(
    bindings: &BTreeMap<PortRef, ConnectionPointId>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    bindings
        .iter()
        .map(|(port, &point)| PortBinding {
            port: port.clone(),
            point,
        })
        .collect::<Vec<_>>()
        .serialize(serializer)
}
fn change<K, V: PartialEq>(
    key: K,
    before: Option<V>,
    after: Option<V>,
) -> Option<DrawingChange<K, V>> {
    if before == after {
        None
    } else {
        Some(DrawingChange { key, before, after })
    }
}
fn preflight<K: Ord, V: PartialEq>(
    map: &BTreeMap<K, V>,
    changes: &[DrawingChange<K, V>],
    forward: bool,
) -> Result<()> {
    for c in changes {
        let expected = if forward { &c.before } else { &c.after };
        if map.get(&c.key) != expected.as_ref() {
            return Err(drawing_error());
        }
    }
    Ok(())
}
fn write<K: Ord + Clone, V: Clone>(
    map: &mut BTreeMap<K, V>,
    changes: &[DrawingChange<K, V>],
    forward: bool,
) {
    for c in changes {
        let next = if forward { &c.after } else { &c.before };
        match next {
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
    /// Resolve an explicit symbol port into page coordinates.
    pub fn port_position(&self, port: &PortRef) -> Result<Point2D> {
        let r = self
            .representation(port.representation_id())
            .ok_or(DomainError::MissingRepresentation(port.representation_id()))?;
        self.port_position_at(port, r.transform())
    }
    fn port_position_at(&self, port: &PortRef, t: Transform2D) -> Result<Point2D> {
        let r = self
            .representation(port.representation_id())
            .ok_or(drawing_error())?;
        let def = r
            .symbol()
            .and_then(|id| self.symbol_definition(id))
            .ok_or(drawing_error())?;
        let local = def
            .ports()
            .iter()
            .find(|p| p.id() == port.port_id())
            .ok_or(drawing_error())?;
        t.apply(local.position())
    }
    /// Return the functional point explicitly bound to this symbol port.
    pub fn port_binding(&self, port: &PortRef) -> Option<ConnectionPointId> {
        self.port_bindings.get(port).copied()
    }
    /// Existing points compatible with the occurrence's explicit target, in ID order. Never selects or
    /// binds a port automatically.
    pub fn port_binding_candidates(&self, port: &PortRef) -> Result<Vec<ConnectionPointId>> {
        self.port_position(port)?;
        Ok(self
            .connection_points
            .keys()
            .copied()
            .filter(|&point| self.check_binding(port, point).is_ok())
            .collect())
    }
    /// Iterate explicit symbol-port bindings without inferring connectivity.
    pub fn port_bindings(&self) -> impl Iterator<Item = (&PortRef, ConnectionPointId)> {
        self.port_bindings
            .iter()
            .map(|(port, &point)| (port, point))
    }
    /// Find the connection route by its stable identifier; return None when absent.
    pub fn connection_route(&self, id: RepresentationId) -> Option<&ConnectionRoute> {
        self.connection_routes.get(&id)
    }
    /// Iterate connection routes without exposing mutable access.
    pub fn connection_routes(&self) -> impl Iterator<Item = &ConnectionRoute> {
        self.connection_routes.values()
    }
    /// Find the connection junction by its stable identifier; return None when absent.
    pub fn connection_junction(&self, id: ConnectionJunctionId) -> Option<&ConnectionJunction> {
        self.connection_junctions.get(&id)
    }
    /// Iterate connection junctions without exposing mutable access.
    pub fn connection_junctions(&self) -> impl Iterator<Item = &ConnectionJunction> {
        self.connection_junctions.values()
    }

    pub(in crate::domain::project) fn check_binding(
        &self,
        port: &PortRef,
        point: ConnectionPointId,
    ) -> Result<()> {
        self.port_position(port)?;
        let point = self.connection_point(point).ok_or(drawing_error())?;
        let r = self
            .representation(port.representation_id())
            .ok_or(drawing_error())?;
        let function = self.function(point.function_id()).ok_or(drawing_error())?;
        let compatible = match r.target() {
            RepresentationTarget::ConnectionPoint(id) => id == point.id(),
            RepresentationTarget::Function(id) => id == point.function_id(),
            RepresentationTarget::Device(id) => id == function.device_id(),
            RepresentationTarget::Terminal(id) => Some(id) == point.terminal_id(),
            RepresentationTarget::Component(id) => {
                self.component(id)
                    .is_some_and(|c| c.function_id() == Some(point.function_id()))
                    || point
                        .terminal_id()
                        .and_then(|id| self.terminal(id))
                        .is_some_and(|t| t.owner() == TerminalOwner::Component(id))
            }
            _ => false,
        };
        if compatible {
            Ok(())
        } else {
            Err(drawing_error())
        }
    }
    pub(in crate::domain::project) fn validate_connection_drawing(&self) -> Result<()> {
        for (port, &point) in &self.port_bindings {
            self.check_binding(port, point)?;
        }
        for route in self.connection_routes.values() {
            self.check_project(route.project_id())?;
            validate_route_points(route.points())?;
            if route.has_auto_clearance() {
                let a = *route.points().first().ok_or(drawing_error())?;
                let b = *route.points().last().ok_or(drawing_error())?;
                let raw = route.points_with_endpoints(a, b)?;
                let expected = self.clear_route_geometry(route, raw, &BTreeMap::new())?;
                if expected != route.points() {
                    return Err(drawing_error());
                }
            }
            let source = self.port_binding(route.source()).ok_or(drawing_error())?;
            let target = self.port_binding(route.target()).ok_or(drawing_error())?;
            let edge = self
                .connection(route.connection_id())
                .ok_or(drawing_error())?;
            if edge.endpoints() != (source.min(target), source.max(target)) || source == target {
                return Err(drawing_error());
            }
            for port in [route.source(), route.target()] {
                if self
                    .representation(port.representation_id())
                    .ok_or(drawing_error())?
                    .page_id()
                    != route.page_id()
                {
                    return Err(drawing_error());
                }
            }
            if route.points().first() != Some(&self.port_position(route.source())?)
                || route.points().last() != Some(&self.port_position(route.target())?)
            {
                return Err(drawing_error());
            }
            let r = self.representation(route.id()).ok_or(drawing_error())?;
            if r.page_id() != route.page_id()
                || r.target() != RepresentationTarget::Connection(route.connection_id())
                || r.symbol().is_some()
                || r.transform() != identity()?
                || r.local_bounds() != Some(route_bounds(route.points())?)
            {
                return Err(drawing_error());
            }
        }
        let mut owned = BTreeSet::new();
        let mut positions = BTreeSet::new();
        for j in self.connection_junctions.values() {
            // Point2D excludes NaN; normalize signed zero for geometric identity.
            let coordinate_key = |v: f64| if v == 0.0 { 0 } else { v.to_bits() };
            if !positions.insert((
                j.page_id(),
                coordinate_key(j.position().x()),
                coordinate_key(j.position().y()),
            )) {
                return Err(drawing_error());
            }
            self.require(EntityRef::Page(j.page_id()))?;
            let unique: BTreeSet<_> = j.routes().iter().copied().collect();
            if unique.len() < 2 || unique.len() != j.routes().len() {
                return Err(drawing_error());
            }
            let mut endpoints = BTreeSet::new();
            for id in j.routes() {
                let r = self.connection_route(*id).ok_or(drawing_error())?;
                if r.page_id() != j.page_id() || !r.contains(j.position()) {
                    return Err(drawing_error());
                }
                let edge = self.connection(r.connection_id()).ok_or(drawing_error())?;
                let (a, b) = edge.endpoints();
                endpoints.insert(a);
                endpoints.insert(b);
            }
            // Every auxiliary edge is uniquely owned and only joins participating routes.
            for id in j.connections() {
                if !owned.insert(*id)
                    || self
                        .connection_routes
                        .values()
                        .any(|r| r.connection_id() == *id)
                {
                    return Err(drawing_error());
                }
                let edge = self.connection(*id).ok_or(drawing_error())?;
                let (a, b) = edge.endpoints();
                if !endpoints.contains(&a) || !endpoints.contains(&b) {
                    return Err(drawing_error());
                }
            }
            // Junction connectivity must be guaranteed by its own routes and edges,
            // independently of coincidental links elsewhere in the project.
            let mut graph = ConnectivityGraph::default();
            for id in j.routes() {
                graph.insert(
                    self.connection(
                        self.connection_route(*id)
                            .ok_or(drawing_error())?
                            .connection_id(),
                    )
                    .ok_or(drawing_error())?,
                )?;
            }
            for id in j.connections() {
                graph.insert(self.connection(*id).ok_or(drawing_error())?)?;
            }
            let first = *endpoints.first().ok_or(drawing_error())?;
            if endpoints.iter().any(|&p| !graph.are_connected(first, p)) {
                return Err(drawing_error());
            }
        }
        Ok(())
    }
    /// Validate and apply a route, junction or binding operation as one reversible edit.
    pub fn edit_connection_drawing(
        &mut self,
        edit: ConnectionDrawingEdit,
    ) -> Result<ConnectionDrawingDelta> {
        if let ConnectionDrawingEdit::Transform(ref transforms) = edit {
            return self.transform_representations_with_drawing(transforms);
        }
        let mut exact = false;
        let (edit, intent) = match edit {
            ConnectionDrawingEdit::CreateRouteWithIntent {
                source,
                target,
                intent,
            } => (
                ConnectionDrawingEdit::CreateRoute {
                    source,
                    target,
                    points: intent.points()?,
                },
                Some(intent),
            ),
            ConnectionDrawingEdit::CreateRouteExact {
                source,
                target,
                points,
            } => {
                exact = true;
                (
                    ConnectionDrawingEdit::CreateRoute {
                        source,
                        target,
                        points,
                    },
                    None,
                )
            }
            edit => (edit, None),
        };
        let mut d = ConnectionDrawingDelta::new(self.id());
        match edit {
            ConnectionDrawingEdit::BindPort { port, point } => {
                self.port_position(&port)?;
                if let Some(p) = point {
                    self.check_binding(&port, p)?;
                }
                if let Some(c) = change(port.clone(), self.port_binding(&port), point) {
                    d.bindings.push(c);
                }
            }
            ConnectionDrawingEdit::CreateRoute {
                source,
                target,
                points,
            } => {
                let a = self.port_binding(&source).ok_or(drawing_error())?;
                let b = self.port_binding(&target).ok_or(drawing_error())?;
                let page = self
                    .representation(source.representation_id())
                    .ok_or(drawing_error())?
                    .page_id();
                let edge = Connection::new(self.id(), a, b)?;
                let mut route =
                    ConnectionRoute::new(self.id(), page, edge.id(), source, target, points)?;
                if let Some(intent) = intent {
                    route = route.with_intent(intent)?;
                }
                if !exact {
                    let cleared = self.clear_route_geometry(
                        &route,
                        route.points().to_vec(),
                        &BTreeMap::new(),
                    )?;
                    route = route.with_auto_clearance(cleared)?;
                }
                let representation = route_representation(&route)?;
                d.connections.push(DrawingChange {
                    key: edge.id(),
                    before: None,
                    after: Some(edge),
                });
                d.representations.push(DrawingChange {
                    key: route.id(),
                    before: None,
                    after: Some(representation),
                });
                d.routes.push(DrawingChange {
                    key: route.id(),
                    before: None,
                    after: Some(route),
                });
            }
            ConnectionDrawingEdit::CreateRouteWithIntent { .. }
            | ConnectionDrawingEdit::CreateRouteExact { .. } => unreachable!("normalized above"),
            ConnectionDrawingEdit::SetRoute { id, points } => {
                let before = self.connection_route(id).ok_or(drawing_error())?.clone();
                let after = before.clone().with_points(points)?;
                let cleared =
                    self.clear_route_geometry(&after, after.points().to_vec(), &BTreeMap::new())?;
                let after = after.with_auto_clearance(cleared)?;
                if let Some(c) = change(id, Some(before), Some(after.clone())) {
                    d.routes.push(c);
                    let old = self.representation(id).ok_or(drawing_error())?.clone();
                    let mut new = old.clone();
                    new.local_bounds = Some(route_bounds(after.points())?);
                    if let Some(c) = change(id, Some(old), Some(new)) {
                        d.representations.push(c);
                    }
                }
            }
            ConnectionDrawingEdit::SetNumberLabelPosition { id, position } => {
                let before = self.connection_route(id).ok_or(drawing_error())?.clone();
                let after = before.clone().with_number_label_position(position);
                if let Some(change) = change(id, Some(before), Some(after)) {
                    d.routes.push(change);
                }
            }
            ConnectionDrawingEdit::DeleteRoute { id } => {
                let route = self.connection_route(id).ok_or(drawing_error())?.clone();
                if self
                    .connection_junctions
                    .values()
                    .any(|j| j.routes().contains(&id))
                {
                    return Err(drawing_error());
                }
                let edge = self
                    .connection(route.connection_id())
                    .ok_or(drawing_error())?
                    .clone();
                d.routes.push(DrawingChange {
                    key: id,
                    before: Some(route),
                    after: None,
                });
                d.representations.push(DrawingChange {
                    key: id,
                    before: Some(self.representation(id).ok_or(drawing_error())?.clone()),
                    after: None,
                });
                d.connections.push(DrawingChange {
                    key: edge.id(),
                    before: Some(edge),
                    after: None,
                });
            }
            ConnectionDrawingEdit::CreateJunction {
                page,
                position,
                mut routes,
            } => {
                routes.sort_unstable();
                if routes.len() < 2 || routes.windows(2).any(|w| w[0] == w[1]) {
                    return Err(drawing_error());
                }
                if self
                    .connection_junctions
                    .values()
                    .any(|j| j.page_id() == page && j.position() == position)
                {
                    return Err(drawing_error());
                }
                // Build a deterministic spanning tree over participant routes.
                // Auxiliary edges are owned exclusively by the new junction.
                let mut graph = ConnectivityGraph::default();
                let mut endpoints = vec![];
                for id in &routes {
                    let r = self.connection_route(*id).ok_or(drawing_error())?;
                    if r.page_id() != page || !r.contains(position) {
                        return Err(drawing_error());
                    }
                    let edge = self.connection(r.connection_id()).ok_or(drawing_error())?;
                    graph.insert(edge)?;
                    endpoints.push(edge.endpoints().0);
                }
                let root = endpoints[0];
                let mut edges = vec![];
                for endpoint in endpoints.into_iter().skip(1) {
                    if graph.are_connected(root, endpoint) {
                        continue;
                    }
                    // Do not borrow someone else's edge: deletion must retain an
                    // unambiguous owner. Such a junction would be redundant.
                    if self
                        .connections
                        .values()
                        .any(|c| c.endpoints() == (root.min(endpoint), root.max(endpoint)))
                    {
                        return Err(drawing_error());
                    }
                    let edge = Connection::new(self.id(), root, endpoint)?;
                    graph.insert(&edge)?;
                    edges.push(edge.id());
                    d.connections.push(DrawingChange {
                        key: edge.id(),
                        before: None,
                        after: Some(edge),
                    });
                }
                let j = ConnectionJunction {
                    id: ConnectionJunctionId::new(),
                    page_id: page,
                    position,
                    routes,
                    connections: edges,
                };
                d.junctions.push(DrawingChange {
                    key: j.id(),
                    before: None,
                    after: Some(j),
                });
            }
            ConnectionDrawingEdit::DeleteJunction { id } => {
                let j = self.connection_junction(id).ok_or(drawing_error())?.clone();
                for edge in j.connections() {
                    d.connections.push(DrawingChange {
                        key: *edge,
                        before: Some(self.connection(*edge).ok_or(drawing_error())?.clone()),
                        after: None,
                    });
                }
                d.junctions.push(DrawingChange {
                    key: id,
                    before: Some(j),
                    after: None,
                });
            }
            ConnectionDrawingEdit::Transform(_) => unreachable!(),
        }
        self.restore_connection_drawing(&d, true)?;
        Ok(d)
    }
    /// State-checked, atomic replay. Validation errors restore the touched entities
    /// before returning, including all derived indices. No whole-project clone.
    pub fn restore_connection_drawing(
        &mut self,
        d: &ConnectionDrawingDelta,
        forward: bool,
    ) -> Result<()> {
        self.check_project(d.project)?;
        preflight(&self.port_bindings, &d.bindings, forward)?;
        preflight(&self.connection_routes, &d.routes, forward)?;
        preflight(&self.connection_junctions, &d.junctions, forward)?;
        preflight(&self.representations, &d.representations, forward)?;
        preflight(&self.connections, &d.connections, forward)?;
        if d.is_empty() {
            return Ok(());
        }
        self.write_drawing_delta(d, forward);
        let checked = (|| {
            let document_index = self.build_document_index()?;
            let graph = self.build_graph()?;
            let physical_index = self.build_physical_index()?;
            self.validate_connection_drawing()?;
            self.validate_markers()?;
            self.document_index = document_index;
            self.graph = graph;
            self.physical_index = physical_index;
            Ok(())
        })();
        if checked.is_err() {
            self.write_drawing_delta(d, !forward);
        }
        checked
    }
    fn write_drawing_delta(&mut self, d: &ConnectionDrawingDelta, forward: bool) {
        write(&mut self.port_bindings, &d.bindings, forward);
        write(&mut self.connection_routes, &d.routes, forward);
        write(&mut self.connection_junctions, &d.junctions, forward);
        write(&mut self.representations, &d.representations, forward);
        write(&mut self.connections, &d.connections, forward);
    }
    pub(in crate::domain::project) fn drawing_references_representation(
        &self,
        id: RepresentationId,
    ) -> bool {
        self.connection_routes.contains_key(&id)
            || self
                .port_bindings
                .keys()
                .any(|p| p.representation_id() == id)
    }
    pub(in crate::domain::project) fn drawing_owns_connection(&self, id: ConnectionId) -> bool {
        self.connection_junctions
            .values()
            .any(|j| j.connections().contains(&id))
            || self
                .connection_routes
                .values()
                .any(|r| r.connection_id() == id)
    }
}
fn identity() -> Result<Transform2D> {
    Transform2D::new(Point2D::new(0., 0.)?, 0., false, false)
}
fn route_bounds(points: &[Point2D]) -> Result<Rect2D> {
    let min_x = points.iter().map(|p| p.x()).fold(f64::INFINITY, f64::min);
    let min_y = points.iter().map(|p| p.y()).fold(f64::INFINITY, f64::min);
    let max_x = points
        .iter()
        .map(|p| p.x())
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y = points
        .iter()
        .map(|p| p.y())
        .fold(f64::NEG_INFINITY, f64::max);
    Rect2D::new(
        Point2D::new(min_x, min_y)?,
        Size2D::new(max_x - min_x, max_y - min_y)?,
    )
}
pub(in crate::domain::project) fn route_representation(
    route: &ConnectionRoute,
) -> Result<Representation> {
    let mut r = Representation::new(
        route.project_id(),
        route.page_id(),
        RepresentationTarget::Connection(route.connection_id()),
        identity()?,
    )
    .with_local_bounds(route_bounds(route.points())?)?;
    r.id = route.id();
    Ok(r)
}

impl Project {
    /// Move occurrences and repair their connected routes in one validated reversible edit.
    pub fn transform_representations_with_drawing(
        &mut self,
        transforms: &[(RepresentationId, Transform2D)],
    ) -> Result<ConnectionDrawingDelta> {
        let mut seen = BTreeMap::new();
        let mut d = ConnectionDrawingDelta::new(self.id());
        for &(id, t) in transforms {
            if seen.insert(id, t).is_some() {
                return Err(DomainError::DuplicateRepresentation(id));
            }
            let old = self
                .representation(id)
                .ok_or(DomainError::MissingRepresentation(id))?;
            self.check_symbol_transform(old, t)?;
            if let Some(bounds) = old.local_bounds() {
                t.transform_bounds(bounds)?;
            }
            if self.connection_routes.contains_key(&id) && old.transform() != t {
                return Err(drawing_error());
            }
            let mut new = old.clone();
            new.transform = t;
            if let Some(c) = change(id, Some(old.clone()), Some(new)) {
                d.representations.push(c);
            }
        }
        let affected_pages: BTreeSet<_> = seen
            .keys()
            .filter_map(|id| self.representation(*id))
            .map(Representation::page_id)
            .collect();
        let mut translations = BTreeMap::new();
        for r in self.connection_routes.values() {
            if !affected_pages.contains(&r.page_id()) {
                continue;
            }
            let a = self.port_position_at(
                r.source(),
                seen.get(&r.source().representation_id())
                    .copied()
                    .unwrap_or(
                        self.representation(r.source().representation_id())
                            .ok_or(drawing_error())?
                            .transform(),
                    ),
            )?;
            let b = self.port_position_at(
                r.target(),
                seen.get(&r.target().representation_id())
                    .copied()
                    .unwrap_or(
                        self.representation(r.target().representation_id())
                            .ok_or(drawing_error())?
                            .transform(),
                    ),
            )?;
            let old_a = r.points()[0];
            let old_b = *r.points().last().ok_or(drawing_error())?;
            let da = (a.x() - old_a.x(), a.y() - old_a.y());
            let db = (b.x() - old_b.x(), b.y() - old_b.y());
            if da == db {
                translations.insert(r.id(), da);
            }
            let after = r.clone().with_endpoints(a, b)?;
            let cleared = self.clear_route_geometry(&after, after.points().to_vec(), &seen)?;
            let after = after.with_auto_clearance(cleared)?;
            if let Some(c) = change(r.id(), Some(r.clone()), Some(after.clone())) {
                d.routes.push(c);
                let old = self.representation(r.id()).ok_or(drawing_error())?.clone();
                let mut new = old.clone();
                new.local_bounds = Some(route_bounds(after.points())?);
                // A route's identity transform cannot also be meaningfully changed.
                if let Some(c) = change(r.id(), Some(old), Some(new)) {
                    d.representations.push(c);
                }
            }
        }
        for j in self.connection_junctions.values() {
            let translation = j
                .routes()
                .first()
                .and_then(|id| translations.get(id))
                .copied();
            if let Some(offset) = translation
                && j.routes()
                    .iter()
                    .all(|id| translations.get(id) == Some(&offset))
            {
                let mut next = j.clone();
                next.position = j.position().translated(offset.0, offset.1)?;
                if let Some(c) = change(j.id(), Some(j.clone()), Some(next)) {
                    d.junctions.push(c);
                }
            }
        }
        self.restore_connection_drawing(&d, true)?;
        Ok(d)
    }

    /// Same endpoint repair and symbol clearance used by transform confirmation.
    /// Overrides are preview transforms; no project or electrical state changes.
    pub fn preview_route_points(
        &self,
        route: &ConnectionRoute,
        a: Point2D,
        b: Point2D,
        transforms: &[(RepresentationId, Transform2D)],
    ) -> Result<Vec<Point2D>> {
        let overrides = transforms.iter().copied().collect::<BTreeMap<_, _>>();
        if overrides.len() != transforms.len() {
            return Err(drawing_error());
        }
        let raw = route.points_with_endpoints(a, b)?;
        self.clear_route_geometry(route, raw, &overrides)
    }
}
