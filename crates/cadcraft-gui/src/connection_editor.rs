// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Transient electrical editor state and page-scoped snap cache.
use crate::{camera::snap, connection_geometry::*, editing::GraphicEdit, state::Session};
use cad_craft::{
    domain::*,
    error::{DomainError, Result},
};

#[derive(Clone, Debug, PartialEq)]
pub enum SnapKind {
    Port {
        port: PortRef,
        bound: bool,
    },
    Junction(ConnectionJunctionId),
    Vertex {
        route: RepresentationId,
        index: usize,
    },
    Segment {
        route: RepresentationId,
        index: usize,
    },
    Grid,
}
#[derive(Clone, Debug)]
pub struct ConnectionSnap {
    pub position: Point2D,
    pub kind: SnapKind,
}
#[derive(Default, Debug)]
pub struct ConnectionScene {
    revision: Option<u64>,
    page: Option<PageId>,
    pub ports: Vec<(PortRef, Point2D, bool)>,
    pub routes: Vec<(RepresentationId, Vec<Point2D>)>,
    pub junctions: Vec<(ConnectionJunctionId, Point2D)>,
}
impl ConnectionScene {
    pub fn sync(&mut self, p: &Project, page: PageId, revision: u64) {
        if self.revision == Some(revision) && self.page == Some(page) {
            return;
        }
        *self = Self {
            revision: Some(revision),
            page: Some(page),
            ..Self::default()
        };
        for r in p.representations_on_page(page).unwrap_or_default() {
            if !r.visual().visible {
                continue;
            }
            if let Some(d) = r.symbol().and_then(|id| p.symbol_definition(id)) {
                for port in d.ports() {
                    if let Ok(reference) = PortRef::new(r.id(), port.id())
                        && let Ok(position) = p.port_position(&reference)
                    {
                        let bound = p.port_binding(&reference).is_some();
                        self.ports.push((reference, position, bound));
                    }
                }
            }
            if let Some(route) = p.connection_route(r.id()) {
                self.routes.push((route.id(), route.points().to_vec()));
            }
        }
        self.junctions = p
            .connection_junctions()
            .filter(|j| j.page_id() == page)
            .map(|j| (j.id(), j.position()))
            .collect();
    }
    /// Priority is electrical port, junction, vertex, segment, then logical grid.
    /// Distances are measured in logical pixels; DPI affects rendering only.
    pub fn snap(&self, cursor: Point2D, scale: f64, grid: f64, enabled: bool) -> ConnectionSnap {
        let tolerance = 8.0 / scale;
        let mut best: Option<(u8, f64, ConnectionSnap)> = None;
        let mut offer = |priority: u8, position: Point2D, kind: SnapKind| {
            let distance = (position.x() - cursor.x()).hypot(position.y() - cursor.y());
            if distance <= tolerance
                && best
                    .as_ref()
                    .is_none_or(|(pr, d, _)| priority < *pr || (priority == *pr && distance < *d))
            {
                best = Some((priority, distance, ConnectionSnap { position, kind }));
            }
        };
        for (port, position, bound) in &self.ports {
            offer(
                if *bound { 0 } else { 1 },
                *position,
                SnapKind::Port {
                    port: port.clone(),
                    bound: *bound,
                },
            );
        }
        for (id, position) in &self.junctions {
            offer(2, *position, SnapKind::Junction(*id));
        }
        for (route, points) in &self.routes {
            for (index, &position) in points.iter().enumerate() {
                offer(
                    3,
                    position,
                    SnapKind::Vertex {
                        route: *route,
                        index,
                    },
                );
            }
            if let Some((index, position)) = route_hit(points, cursor, tolerance) {
                let position = if enabled {
                    on_segment_grid(position, points[index], points[index + 1], grid)
                } else {
                    position
                };
                offer(
                    4,
                    position,
                    SnapKind::Segment {
                        route: *route,
                        index,
                    },
                );
            }
        }
        best.map(|(_, _, s)| s).unwrap_or(ConnectionSnap {
            position: snap(cursor, grid, enabled),
            kind: SnapKind::Grid,
        })
    }
    pub fn routes_at(&self, position: Point2D) -> Vec<RepresentationId> {
        self.routes
            .iter()
            .filter(|(_, p)| route_hit(p, position, 1e-7).is_some())
            .map(|(id, _)| *id)
            .collect()
    }
    /// Assistance is visual only. A click on the candidate port still confirms via Core.
    pub fn aligned_port(
        &self,
        source: &PortRef,
        origin: Point2D,
        cursor: Point2D,
        scale: f64,
    ) -> Option<(PortRef, Point2D)> {
        self.ports
            .iter()
            .filter(|(r, p, b)| {
                *b && r != source
                    && (p.x() == origin.x() || p.y() == origin.y())
                    && (p.x() - cursor.x()).hypot(p.y() - cursor.y()) * scale <= 40.
            })
            .min_by(|a, b| {
                ((a.1.x() - cursor.x()).hypot(a.1.y() - cursor.y()))
                    .total_cmp(&((b.1.x() - cursor.x()).hypot(b.1.y() - cursor.y())))
            })
            .map(|(r, p, _)| (r.clone(), *p))
    }
}
#[derive(Clone, Debug)]
pub struct ConnectionDraft {
    pub page: PageId,
    pub source: Option<PortRef>,
    pub anchors: Vec<Point2D>,
    pub span_orientations: Vec<bool>,
    pub horizontal_first: bool,
    pub preview: Vec<Point2D>,
}
impl ConnectionDraft {
    pub fn new(page: PageId) -> Self {
        Self {
            page,
            source: None,
            anchors: vec![],
            span_orientations: vec![],
            horizontal_first: true,
            preview: vec![],
        }
    }
    pub fn update(&mut self, cursor: Point2D) {
        let mut anchors = self.anchors.clone();
        if !anchors.is_empty() {
            anchors.push(cursor);
        }
        let mut orientations = self.span_orientations.clone();
        orientations.resize(anchors.len().saturating_sub(1), self.horizontal_first);
        self.preview = orthogonal_route_points(&anchors, &orientations).unwrap_or_default();
    }
}
#[derive(Clone, Debug)]
pub struct RouteDrag {
    pub id: RepresentationId,
    pub original: Vec<Point2D>,
    pub preview: Vec<Point2D>,
    pub index: usize,
    pub vertex: bool,
}
impl RouteDrag {
    pub fn update(&mut self, position: Point2D) -> Result<()> {
        self.preview = if self.vertex {
            move_vertex(&self.original, self.index, position, true)?
        } else {
            move_segment(&self.original, self.index, position)?
        };
        Ok(())
    }
}
impl Session {
    pub fn highlight_network(&mut self) {
        let routes = self.graphics.selection.primary().and_then(|id| {
            let p = self.project()?;
            let route = p.connection_route(id)?;
            let point = p.port_binding(route.source())?;
            let network = p.network_for_point(point).ok()?;
            Some(
                p.connection_routes()
                    .filter(|r| network.connections().contains(&r.connection_id()))
                    .map(ConnectionRoute::id)
                    .collect(),
            )
        });
        self.graphics.highlighted_routes = routes.unwrap_or_default();
    }
    pub fn begin_connection(&mut self) -> Result<()> {
        let page = self.active.ok_or(DomainError::InvalidRelation(
            "abra uma página para conectar",
        ))?;
        self.graphics.cancel();
        self.graphics.tool = crate::interaction::Tool::DrawConnection;
        self.graphics.connection = Some(ConnectionDraft::new(page));
        Ok(())
    }
    pub fn connection_click(&mut self, hit: ConnectionSnap) -> Result<bool> {
        let draft = self
            .graphics
            .connection
            .as_ref()
            .ok_or(DomainError::InvalidRelation("ative a ferramenta Conexão"))?;
        if Some(draft.page) != self.active {
            return Err(DomainError::InvalidRelation(
                "a página da conexão foi alterada",
            ));
        }
        if let SnapKind::Port { port, bound } = hit.kind {
            if !bound {
                return Err(DomainError::InvalidRelation(
                    "porta sem associação elétrica; vincule um ponto nas propriedades",
                ));
            }
            let p = self.project().ok_or(DomainError::InvalidRelation(
                crate::strings::NO_OPEN_PROJECT,
            ))?;
            let position = p.port_position(&port)?;
            if let Some(source) = &draft.source {
                let mut anchors = draft.anchors.clone();
                anchors.push(position);
                let mut orientations = draft.span_orientations.clone();
                orientations.push(draft.horizontal_first);
                let edit = ConnectionDrawingEdit::CreateRouteWithIntent {
                    source: source.clone(),
                    target: port,
                    intent: RouteIntent::new(anchors, orientations)?,
                };
                self.dispatch_graphic(GraphicEdit::Drawing(edit))?;
                self.begin_connection()?;
                return Ok(true);
            }
            let draft = self.graphics.connection.as_mut().expect("draft checked");
            draft.source = Some(port);
            draft.anchors.push(position);
        } else {
            let draft = self.graphics.connection.as_mut().expect("draft checked");
            if draft.source.is_none() {
                return Err(DomainError::InvalidRelation(
                    "inicie em uma porta elétrica associada",
                ));
            }
            if draft.anchors.last() != Some(&hit.position) {
                draft.anchors.push(hit.position);
                draft.span_orientations.push(draft.horizontal_first);
            }
        }
        Ok(false)
    }
}
