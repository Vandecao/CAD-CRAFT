// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Document geometry explicitly bound to the existing electrical graph; start with ConnectionRoute.
use super::*;
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};

pub(in crate::domain) fn drawing_error() -> DomainError {
    DomainError::InvalidRelation("desenho de conexão inconsistente ou referenciado")
}

/// Explicit symbol occurrence and local port identifier.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "PortRefData")]
pub struct PortRef {
    representation: RepresentationId,
    port: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PortRefData {
    representation: RepresentationId,
    port: String,
}
impl TryFrom<PortRefData> for PortRef {
    type Error = DomainError;
    fn try_from(data: PortRefData) -> Result<Self> {
        Self::new(data.representation, data.port)
    }
}
impl PortRef {
    /// Create an explicit port reference with a validated local port identifier.
    pub fn new(representation: RepresentationId, port: impl Into<String>) -> Result<Self> {
        let port = port.into();
        if port.trim().is_empty() {
            return Err(drawing_error());
        }
        Ok(Self {
            representation,
            port,
        })
    }
    /// Return the representation identifier.
    pub fn representation_id(&self) -> RepresentationId {
        self.representation
    }
    /// Return the port identifier.
    pub fn port_id(&self) -> &str {
        &self.port
    }
}

/// Documentary wire geometry bound to an existing logical connection.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RouteData")]
pub struct ConnectionRoute {
    id: RepresentationId,
    project_id: ProjectId,
    page_id: PageId,
    connection_id: ConnectionId,
    source: PortRef,
    target: PortRef,
    points: Vec<Point2D>,
    /// Authored constraints, independent of endpoint repair's generated bends.
    /// Old snapshots conservatively treat their complete polyline as authored.
    routing_base: Vec<Point2D>,
    /// Present for new interactive routes: only clicked anchors are constraints.
    #[serde(skip_serializing_if = "Option::is_none")]
    routing_intent: Option<RouteIntent>,
    #[serde(default, skip_serializing_if = "is_false")]
    auto_clearance: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    number_label_position: Option<Point2D>,
}
fn is_false(value: &bool) -> bool {
    !*value
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RouteData {
    id: RepresentationId,
    project_id: ProjectId,
    page_id: PageId,
    connection_id: ConnectionId,
    source: PortRef,
    target: PortRef,
    points: Vec<Point2D>,
    #[serde(default)]
    routing_base: Option<Vec<Point2D>>,
    #[serde(default)]
    routing_intent: Option<RouteIntent>,
    #[serde(default)]
    auto_clearance: bool,
    #[serde(default)]
    number_label_position: Option<Point2D>,
}
impl TryFrom<RouteData> for ConnectionRoute {
    type Error = DomainError;
    fn try_from(d: RouteData) -> Result<Self> {
        validate_route_points(&d.points)?;
        if d.source == d.target {
            return Err(drawing_error());
        }
        let routing_base = d.routing_base.unwrap_or_else(|| d.points.clone());
        validate_route_points(&routing_base)?;
        let expected = if let Some(intent) = &d.routing_intent {
            if intent.points()? != routing_base {
                return Err(drawing_error());
            }
            intent.points_at(d.points[0], *d.points.last().ok_or(drawing_error())?)?
        } else {
            repair_endpoints(
                &routing_base,
                d.points[0],
                *d.points.last().ok_or(drawing_error())?,
            )?
        };
        if !d.auto_clearance && expected != d.points {
            return Err(drawing_error());
        }
        Ok(Self {
            id: d.id,
            project_id: d.project_id,
            page_id: d.page_id,
            connection_id: d.connection_id,
            source: d.source,
            target: d.target,
            points: d.points,
            routing_base,
            routing_intent: d.routing_intent,
            auto_clearance: d.auto_clearance,
            number_label_position: d.number_label_position,
        })
    }
}
impl ConnectionRoute {
    /// Create a wire route with fresh occurrence identity and validated orthogonal geometry.
    pub fn new(
        project_id: ProjectId,
        page_id: PageId,
        connection_id: ConnectionId,
        source: PortRef,
        target: PortRef,
        points: Vec<Point2D>,
    ) -> Result<Self> {
        Self::try_from(RouteData {
            id: RepresentationId::new(),
            project_id,
            page_id,
            connection_id,
            source,
            target,
            points,
            routing_base: None,
            routing_intent: None,
            auto_clearance: false,
            number_label_position: None,
        })
    }
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> RepresentationId {
        self.id
    }
    /// Return the project identifier.
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }
    /// Return the page identifier.
    pub fn page_id(&self) -> PageId {
        self.page_id
    }
    /// Return the connection identifier.
    pub fn connection_id(&self) -> ConnectionId {
        self.connection_id
    }
    /// Return the source.
    pub fn source(&self) -> &PortRef {
        &self.source
    }
    /// Return the target.
    pub fn target(&self) -> &PortRef {
        &self.target
    }
    /// Return the points in their stored order.
    pub fn points(&self) -> &[Point2D] {
        &self.points
    }
    /// Return the optional number label position; absence represents an unset value or relation.
    pub fn number_label_position(&self) -> Option<Point2D> {
        self.number_label_position
    }
    pub(in crate::domain) fn with_number_label_position(
        mut self,
        position: Option<Point2D>,
    ) -> Self {
        self.number_label_position = position;
        self
    }
    /// Replace the route polyline and authored constraints after geometry validation.
    pub fn with_points(mut self, points: Vec<Point2D>) -> Result<Self> {
        validate_route_points(&points)?;
        if points != self.points {
            self.routing_base = points.clone();
            self.routing_intent = None;
            self.auto_clearance = false;
            self.points = points;
        }
        Ok(self)
    }
    /// Pure preview shared with confirmation. A common translation preserves the route; independent
    /// movement repairs only terminal segments. Creates no identity, electrical mutation or history
    /// transaction.
    pub fn points_with_endpoints(&self, a: Point2D, b: Point2D) -> Result<Vec<Point2D>> {
        if let Some(intent) = &self.routing_intent {
            return self.intent_with_endpoints(intent, a, b)?.points_at(a, b);
        }
        let base = self.base_with_endpoints(a, b)?;
        repair_endpoints(&base, a, b)
    }
    /// Confirm exactly the preview while retaining authored constraints. This
    /// must not call with_points: automatic terminal bends are never edits.
    pub(in crate::domain) fn with_endpoints(mut self, a: Point2D, b: Point2D) -> Result<Self> {
        if let Some(intent) = &self.routing_intent {
            let intent = self.intent_with_endpoints(intent, a, b)?;
            self.points = intent.points_at(a, b)?;
            self.routing_base = intent.points()?;
            self.routing_intent = Some(intent);
            self.auto_clearance = false;
            return Ok(self);
        }
        self.routing_base = self.base_with_endpoints(a, b)?;
        self.points = repair_endpoints(&self.routing_base, a, b)?;
        self.auto_clearance = false;
        Ok(self)
    }
    pub(in crate::domain) fn with_auto_clearance(mut self, points: Vec<Point2D>) -> Result<Self> {
        validate_route_points(&points)?;
        if points.first() != self.points.first() || points.last() != self.points.last() {
            return Err(drawing_error());
        }
        self.auto_clearance = points != self.points;
        self.points = points;
        Ok(self)
    }
    pub(in crate::domain) fn has_auto_clearance(&self) -> bool {
        self.auto_clearance
    }
    pub(in crate::domain) fn with_intent(mut self, intent: RouteIntent) -> Result<Self> {
        if intent.points()? != self.points {
            return Err(drawing_error());
        }
        self.routing_intent = Some(intent);
        Ok(self)
    }
    fn intent_with_endpoints(
        &self,
        intent: &RouteIntent,
        a: Point2D,
        b: Point2D,
    ) -> Result<RouteIntent> {
        let old_a = self.points[0];
        let old_b = *self.points.last().ok_or(drawing_error())?;
        let da = (a.x() - old_a.x(), a.y() - old_a.y());
        let db = (b.x() - old_b.x(), b.y() - old_b.y());
        if da == db {
            RouteIntent::new(
                intent
                    .anchors
                    .iter()
                    .map(|p| p.translated(da.0, da.1))
                    .collect::<Result<_>>()?,
                intent.horizontal_first.clone(),
            )
        } else {
            Ok(intent.clone())
        }
    }
    fn base_with_endpoints(&self, a: Point2D, b: Point2D) -> Result<Vec<Point2D>> {
        let old_a = self.points[0];
        let old_b = *self.points.last().ok_or(drawing_error())?;
        let da = (a.x() - old_a.x(), a.y() - old_a.y());
        let db = (b.x() - old_b.x(), b.y() - old_b.y());
        let base = if da == db {
            self.routing_base
                .iter()
                .map(|p| p.translated(da.0, da.1))
                .collect()
        } else {
            Ok(self.routing_base.clone())
        }?;
        validate_route_points(&base)?;
        Ok(base)
    }
    /// Whether the supplied point or entity lies in this record.
    pub fn contains(&self, p: Point2D) -> bool {
        self.points
            .windows(2)
            .any(|s| point_on_segment(p, s[0], s[1]))
    }
    /// The same route under a rigid sheet transform, used by macros. A 90°/270° rotation swaps each
    /// intended L span's orientation.
    pub(in crate::domain) fn transformed(&self, t: Transform2D) -> Result<Self> {
        let map = |points: &[Point2D]| {
            points
                .iter()
                .map(|p| t.apply(*p))
                .collect::<Result<Vec<_>>>()
        };
        let mut next = self.clone();
        next.points = map(&self.points)?;
        validate_route_points(&next.points)?;
        next.routing_base = map(&self.routing_base)?;
        next.number_label_position = self.number_label_position.map(|p| t.apply(p)).transpose()?;
        next.routing_intent = match &self.routing_intent {
            Some(intent) => Some(RouteIntent::new(
                map(&intent.anchors)?,
                if t.swaps_axes() {
                    intent.horizontal_first.iter().map(|h| !h).collect()
                } else {
                    intent.horizontal_first.clone()
                },
            )?),
            None => None,
        };
        Ok(next)
    }
}

/// Persistent route constraints. First/last anchors establish the initial
/// endpoints; interior anchors are user waypoints, never generated elbows.
/// Each span keeps the L orientation chosen when it was confirmed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RouteIntentData")]
pub struct RouteIntent {
    anchors: Vec<Point2D>,
    horizontal_first: Vec<bool>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RouteIntentData {
    anchors: Vec<Point2D>,
    horizontal_first: Vec<bool>,
}
impl TryFrom<RouteIntentData> for RouteIntent {
    type Error = DomainError;
    fn try_from(d: RouteIntentData) -> Result<Self> {
        Self::new(d.anchors, d.horizontal_first)
    }
}
impl RouteIntent {
    /// Create a route intent; reject invalid input.
    pub fn new(anchors: Vec<Point2D>, horizontal_first: Vec<bool>) -> Result<Self> {
        if anchors.len() < 2 || anchors.len() > 4096 || horizontal_first.len() + 1 != anchors.len()
        {
            return Err(drawing_error());
        }
        let intent = Self {
            anchors,
            horizontal_first,
        };
        intent.points()?;
        Ok(intent)
    }
    /// Return the points.
    pub fn points(&self) -> Result<Vec<Point2D>> {
        self.points_at(
            self.anchors[0],
            *self.anchors.last().ok_or(drawing_error())?,
        )
    }
    fn points_at(&self, a: Point2D, b: Point2D) -> Result<Vec<Point2D>> {
        let mut anchors = self.anchors.clone();
        anchors[0] = a;
        *anchors.last_mut().ok_or(drawing_error())? = b;
        let points = orthogonal_route_points(&anchors, &self.horizontal_first)?;
        validate_route_points(&points)?;
        Ok(points)
    }
}

/// Shared deterministic L construction for draft, committed intent and edits.
/// Consecutive coincident anchors collapse only in presentation, not in intent.
pub fn orthogonal_route_points(
    anchors: &[Point2D],
    horizontal_first: &[bool],
) -> Result<Vec<Point2D>> {
    if horizontal_first.len() != anchors.len().saturating_sub(1) {
        return Err(drawing_error());
    }
    let mut points = anchors.first().copied().into_iter().collect::<Vec<_>>();
    for (span, horizontal) in anchors.windows(2).zip(horizontal_first) {
        let (a, b) = (span[0], span[1]);
        if a.x() != b.x() && a.y() != b.y() {
            points.push(if *horizontal {
                Point2D::new(b.x(), a.y())?
            } else {
                Point2D::new(a.x(), b.y())?
            });
        }
        points.push(b);
    }
    points.dedup();
    Ok(points)
}
fn repair_endpoints(old: &[Point2D], a: Point2D, b: Point2D) -> Result<Vec<Point2D>> {
    if a == old[0] && b == old[old.len() - 1] {
        return Ok(old.to_vec());
    }
    let mut points = vec![a];
    if old.len() == 2 {
        if a.x() != b.x() && a.y() != b.y() {
            points.push(if old[0].y() == old[1].y() {
                Point2D::new(b.x(), a.y())?
            } else {
                Point2D::new(a.x(), b.y())?
            });
        }
    } else {
        let first = old[1];
        points.push(if old[0].y() == first.y() {
            Point2D::new(first.x(), a.y())?
        } else {
            Point2D::new(a.x(), first.y())?
        });
        points.extend_from_slice(&old[1..old.len() - 1]);
        let last = old[old.len() - 2];
        points.push(if last.y() == old[old.len() - 1].y() {
            Point2D::new(last.x(), b.y())?
        } else {
            Point2D::new(b.x(), last.y())?
        });
    }
    points.push(b);
    points.dedup();
    // Endpoint repair inserts one elbow before/after the authored interior.
    // When that elbow and the next authored vertex lie on the same straight
    // line, keeping both can make a visible out-and-back horizontal tail.
    // The authored base remains untouched, so returning to the old endpoints
    // restores every original vertex. Explicit junction incidence is checked
    // by Project before a transform is published.
    if old.len() >= 4 && a != old[0] && points.len() >= 4 {
        let first_authored = old[1];
        if points[2] == first_authored && collinear_axis(points[1], points[2], points[3]) {
            points.remove(2);
        }
    }
    if old.len() >= 4 && b != old[old.len() - 1] && points.len() >= 4 {
        let last_authored = old[old.len() - 2];
        let n = points.len();
        if points[n - 3] == last_authored
            && collinear_axis(points[n - 4], points[n - 3], points[n - 2])
        {
            points.remove(n - 3);
        }
    }
    validate_route_points(&points)?;
    Ok(points)
}
fn collinear_axis(a: Point2D, b: Point2D, c: Point2D) -> bool {
    (a.x() == b.x() && b.x() == c.x()) || (a.y() == b.y() && b.y() == c.y())
}
/// Point on an axis-aligned segment, within the domain tolerance: port
/// positions come from symbol transforms (f64), clicked points from the grid.
pub(in crate::domain) fn point_on_segment(p: Point2D, a: Point2D, b: Point2D) -> bool {
    use crate::domain::tolerance::EPS_MM as EPS;
    let within = |v: f64, lo: f64, hi: f64| v >= lo.min(hi) - EPS && v <= lo.max(hi) + EPS;
    (a.x() == b.x() && (p.x() - a.x()).abs() <= EPS && within(p.y(), a.y(), b.y()))
        || (a.y() == b.y() && (p.y() - a.y()).abs() <= EPS && within(p.x(), a.x(), b.x()))
}
pub(in crate::domain) fn validate_route_points(points: &[Point2D]) -> Result<()> {
    if points.len() < 2
        || points.len() > 4096
        || points
            .windows(2)
            .any(|s| s[0] == s[1] || (s[0].x() != s[1].x() && s[0].y() != s[1].y()))
    {
        return Err(drawing_error());
    }
    Ok(())
}

/// Explicit document junction. Auxiliary edges belong exclusively to this junction and represent
/// neither conductors nor physical devices.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionJunction {
    pub(in crate::domain) id: ConnectionJunctionId,
    pub(in crate::domain) page_id: PageId,
    pub(in crate::domain) position: Point2D,
    pub(in crate::domain) routes: Vec<RepresentationId>,
    pub(in crate::domain) connections: Vec<ConnectionId>,
}
impl ConnectionJunction {
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> ConnectionJunctionId {
        self.id
    }
    /// Return the page identifier.
    pub fn page_id(&self) -> PageId {
        self.page_id
    }
    /// Return the position in millimetre coordinates.
    pub fn position(&self) -> Point2D {
        self.position
    }
    /// Return the routes in their stored order.
    pub fn routes(&self) -> &[RepresentationId] {
        &self.routes
    }
    /// Return the connections in their stored order.
    pub fn connections(&self) -> &[ConnectionId] {
        &self.connections
    }
}

/// Validated route, port-binding, junction or coordinated transform operation.
#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionDrawingEdit {
    /// Create route with intent.
    CreateRouteWithIntent {
        /// Explicit source endpoint.
        source: PortRef,
        /// Explicit target of this record or operation.
        target: PortRef,
        /// Authored route constraints, excluding automatically generated bends.
        intent: RouteIntent,
    },
    /// Bind port.
    BindPort {
        /// Explicit symbol occurrence and local port.
        port: PortRef,
        /// Explicit functional connection point.
        point: Option<ConnectionPointId>,
    },
    /// Create route.
    CreateRoute {
        /// Explicit source endpoint.
        source: PortRef,
        /// Explicit target of this record or operation.
        target: PortRef,
        /// Connection points or authored geometric vertices.
        points: Vec<Point2D>,
    },
    /// Route with supplied geometry and no automatic symbol avoidance, preserving imported paths.
    CreateRouteExact {
        /// Explicit source endpoint.
        source: PortRef,
        /// Explicit target of this record or operation.
        target: PortRef,
        /// Connection points or authored geometric vertices.
        points: Vec<Point2D>,
    },
    /// Set route.
    SetRoute {
        /// Stable identifier of the affected object.
        id: RepresentationId,
        /// Connection points or authored geometric vertices.
        points: Vec<Point2D>,
    },
    /// Set number label position.
    SetNumberLabelPosition {
        /// Stable identifier of the affected object.
        id: RepresentationId,
        /// Position in the containing page or terminal.
        position: Option<Point2D>,
    },
    /// Delete route.
    DeleteRoute {
        /// Stable identifier of the affected object.
        id: RepresentationId,
    },
    /// Create junction.
    CreateJunction {
        /// Page identifier used by this operation or filter.
        page: PageId,
        /// Position in the containing page or terminal.
        position: Point2D,
        /// Route occurrence identifiers participating in this operation.
        routes: Vec<RepresentationId>,
    },
    /// Delete junction.
    DeleteJunction {
        /// Stable identifier of the affected object.
        id: ConnectionJunctionId,
    },
    /// Transform.
    Transform(Vec<(RepresentationId, Transform2D)>),
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::domain) struct DrawingChange<K, V> {
    pub key: K,
    pub before: Option<V>,
    pub after: Option<V>,
}
/// Only touched entities, including geometry and auxiliary edges. Replay checks expected state
/// before writing and never clones Project.
#[derive(Clone, Debug, PartialEq)]
pub struct ConnectionDrawingDelta {
    pub(in crate::domain) project: ProjectId,
    pub(in crate::domain) bindings: Vec<DrawingChange<PortRef, ConnectionPointId>>,
    pub(in crate::domain) routes: Vec<DrawingChange<RepresentationId, ConnectionRoute>>,
    pub(in crate::domain) junctions: Vec<DrawingChange<ConnectionJunctionId, ConnectionJunction>>,
    pub(in crate::domain) representations: Vec<DrawingChange<RepresentationId, Representation>>,
    pub(in crate::domain) connections: Vec<DrawingChange<ConnectionId, Connection>>,
}
impl ConnectionDrawingDelta {
    pub(in crate::domain) fn new(project: ProjectId) -> Self {
        Self {
            project,
            bindings: vec![],
            routes: vec![],
            junctions: vec![],
            representations: vec![],
            connections: vec![],
        }
    }
    /// Return the number of contained entries or entity changes recorded by this value.
    pub fn len(&self) -> usize {
        self.bindings.len()
            + self.routes.len()
            + self.junctions.len()
            + self.representations.len()
            + self.connections.len()
    }
    /// Whether this collection or recorded edit contains no effective entries or changes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Snapshot list detects duplicate bindings before index construction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::domain) struct PortBinding {
    pub port: PortRef,
    pub point: ConnectionPointId,
}
