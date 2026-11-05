// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Connection input arbitration and clipped painting over the existing viewport.
use crate::{
    app::{CadApp, Level},
    camera::{Camera, snap},
    commands::CommandId,
    connection_editor::{RouteDrag, SnapKind},
    connection_geometry::*,
    editing::GraphicEdit,
    interaction::{Interaction, Tool},
    theme,
};
use cad_craft::domain::*;
use eframe::egui::{self, CursorIcon, PointerButton, Rect, Response, Stroke, Ui};
use std::borrow::Cow;

impl CadApp {
    pub(crate) fn interact_connections(
        &mut self,
        ui: &mut Ui,
        response: &Response,
        page: PageId,
        camera: Camera,
        rect: Rect,
    ) -> bool {
        let Some(p) = self.session.project() else {
            return false;
        };
        self.connection_scene.sync(p, page, self.session.revision);
        let cursor = ui
            .input(|i| i.pointer.interact_pos())
            .and_then(|s| camera.screen_to_world(s, rect));
        let tool = self.session.graphics.tool;
        self.session.graphics.junction_outline = self.preferences.junction_outline;
        let active = matches!(
            tool,
            Tool::DrawConnection
                | Tool::InsertJunction
                | Tool::EditRoute
                | Tool::AddVertex
                | Tool::RemoveVertex
        );
        let hit = cursor.map(|c| {
            self.connection_scene.snap(
                c,
                camera.scale(),
                self.preferences.grid_mm,
                self.preferences.snap_enabled,
            )
        });
        self.session.graphics.connection_snap = if response.hovered() {
            hit.clone()
        } else {
            None
        };
        if response.secondary_clicked() && active {
            self.session.graphics.cancel();
            return true;
        }
        if tool == Tool::DrawConnection {
            if response.hovered()
                && let Some(hit) = hit
            {
                ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
                if let Some(d) = &mut self.session.graphics.connection {
                    if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Space)) {
                        d.horizontal_first = !d.horizontal_first;
                    }
                    d.update(hit.position);
                    self.session.graphics.connection_suggestion =
                        d.source.as_ref().and_then(|source| {
                            let origin = *d.anchors.first()?;
                            self.connection_scene
                                .aligned_port(source, origin, hit.position, camera.scale())
                                .map(|(_, target)| [origin, target])
                        });
                }
                if response.clicked_by(PointerButton::Primary) {
                    match self.session.connection_click(hit) {
                        Ok(true) => self.log(
                            Level::Info,
                            "Conexão validada. Clique em outra porta para continuar; Esc encerra.",
                        ),
                        Ok(false) => {}
                        Err(e) => self.log(Level::Error, e.to_string()),
                    }
                }
            }
            return true;
        }
        if tool == Tool::InsertJunction {
            if response.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
            }
            if response.clicked_by(PointerButton::Primary)
                && let Some(hit) = hit
            {
                let routes = self.connection_scene.routes_at(hit.position);
                self.graphic_edit(GraphicEdit::Drawing(
                    ConnectionDrawingEdit::CreateJunction {
                        page,
                        position: hit.position,
                        routes,
                    },
                ));
            }
            return true;
        }
        if matches!(tool, Tool::AddVertex | Tool::RemoveVertex) {
            if response.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
            }
            if response.clicked_by(PointerButton::Primary)
                && let Some(hit) = hit
            {
                let candidate = match hit.kind {
                    SnapKind::Segment { route, index } => Some((route, index, false)),
                    SnapKind::Vertex { route, index } => Some((route, index, true)),
                    _ => None,
                };
                if let Some((id, index, vertex)) = candidate
                    && let Some(route) = self.session.project().and_then(|p| p.connection_route(id))
                {
                    let points = if tool == Tool::AddVertex {
                        insert_vertex(route.points(), index, hit.position)
                    } else if vertex {
                        remove_vertex(route.points(), index, true)
                    } else {
                        Err(cad_craft::error::DomainError::InvalidRelation(
                            "aponte um vértice intermediário",
                        ))
                    };
                    match points {
                        Ok(points) => self.graphic_edit(GraphicEdit::Drawing(
                            ConnectionDrawingEdit::SetRoute { id, points },
                        )),
                        Err(e) => self.log(Level::Error, e.to_string()),
                    }
                } else {
                    self.log(Level::Warning, "Aponte um segmento ou vértice de conexão.");
                }
            }
            return true;
        }
        // Junction markers are independently selectable; no electrical entity is fabricated.
        if response.clicked_by(PointerButton::Primary)
            && let Some(cursor) = cursor
            && let Some((id, _)) = self
                .connection_scene
                .junctions
                .iter()
                .filter(|(_, point)| {
                    (point.x() - cursor.x()).hypot(point.y() - cursor.y()) * camera.scale() <= 8.
                })
                .min_by(|a, b| {
                    ((a.1.x() - cursor.x()).hypot(a.1.y() - cursor.y()))
                        .total_cmp(&((b.1.x() - cursor.x()).hypot(b.1.y() - cursor.y())))
                })
        {
            let id = *id;
            self.session.select_graphics([]);
            self.session.graphics.junction_selection = Some(id);
            self.execute(CommandId::ShowProperties, ui.ctx());
            return true;
        }
        if response.drag_started_by(PointerButton::Primary) && !ui.input(|i| i.modifiers.ctrl) {
            let start = ui
                .input(|i| i.pointer.press_origin())
                .and_then(|s| camera.screen_to_world(s, rect));
            if let Some(start) = start {
                let selected = self.session.graphics.selection.primary();
                let front = ui.input(|i| i.pointer.press_origin()).and_then(|pos| {
                    self.session.project().and_then(|p| {
                        crate::hit_test::hit_candidates(p, page, camera, rect, pos)
                            .first()
                            .copied()
                    })
                });
                let candidate = if tool == Tool::EditRoute {
                    selected
                } else {
                    front
                };
                let route = self
                    .connection_scene
                    .routes
                    .iter()
                    .filter(|(id, _)| Some(*id) == candidate)
                    .find_map(|(id, points)| {
                        route_hit(points, start, 6. / camera.scale())
                            .map(|(index, _)| (*id, points, index))
                    });
                if let Some((id, points, index)) = route {
                    let vertex = points
                        .iter()
                        .enumerate()
                        .skip(1)
                        .take(points.len().saturating_sub(2))
                        .find(|(_, q)| {
                            (q.x() - start.x()).hypot(q.y() - start.y()) * camera.scale() <= 6.
                        });
                    let (index, is_vertex) =
                        vertex.map(|(i, _)| (i, true)).unwrap_or((index, false));
                    let drag = RouteDrag {
                        id,
                        original: points.clone(),
                        preview: points.clone(),
                        index,
                        vertex: is_vertex,
                    };
                    self.session.select_graphics([id]);
                    self.session.graphics.route_drag = Some(drag);
                }
            }
        }
        if let Some(drag) = &mut self.session.graphics.route_drag {
            ui.ctx().set_cursor_icon(if drag.vertex {
                CursorIcon::Move
            } else {
                CursorIcon::ResizeColumn
            });
            if let Some(cursor) = cursor {
                let position = snap(
                    cursor,
                    self.preferences.grid_mm,
                    self.preferences.snap_enabled,
                );
                if let Err(e) = drag.update(position) {
                    self.log(Level::Warning, e.to_string());
                }
            }
            if response.drag_stopped_by(PointerButton::Primary)
                && let Some(drag) = self.session.graphics.route_drag.take()
            {
                self.graphic_edit(GraphicEdit::Drawing(ConnectionDrawingEdit::SetRoute {
                    id: drag.id,
                    points: drag.preview,
                }));
            }
            return true;
        }
        if tool == Tool::EditRoute && response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::Move);
        }
        false
    }
}

pub(crate) fn display_points<'a>(
    p: &Project,
    r: &'a ConnectionRoute,
    e: &'a Interaction,
) -> Cow<'a, [Point2D]> {
    if let Some(d) = &e.route_drag
        && d.id == r.id()
    {
        return Cow::Borrowed(&d.preview);
    }
    let Some(preview) = e.preview_transforms() else {
        return Cow::Borrowed(r.points());
    };
    let source = preview.get(&r.source().representation_id()).copied();
    let target = preview.get(&r.target().representation_id()).copied();
    let position = |port: &PortRef, t: Option<Transform2D>| -> Option<Point2D> {
        let rep = p.representation(port.representation_id())?;
        let d = p.symbol_definition(rep.symbol()?)?;
        t.unwrap_or(rep.transform())
            .apply(
                d.ports()
                    .iter()
                    .find(|q| q.id() == port.port_id())?
                    .position(),
            )
            .ok()
    };
    let (Some(a), Some(b)) = (position(r.source(), source), position(r.target(), target)) else {
        return Cow::Borrowed(r.points());
    };
    let transforms = preview.iter().map(|(&id, &t)| (id, t)).collect::<Vec<_>>();
    p.preview_route_points(r, a, b, &transforms)
        .map(Cow::Owned)
        .unwrap_or(Cow::Borrowed(r.points()))
}
fn display_junction_position(p: &Project, j: &ConnectionJunction, e: &Interaction) -> Point2D {
    let mut common = None;
    for id in j.routes() {
        let Some(route) = p.connection_route(*id) else {
            return j.position();
        };
        let preview = display_points(p, route, e);
        let a = preview[0];
        let b = preview[preview.len() - 1];
        let old_a = route.points()[0];
        let old_b = route.points()[route.points().len() - 1];
        let offset = (a.x() - old_a.x(), a.y() - old_a.y());
        if offset != (b.x() - old_b.x(), b.y() - old_b.y())
            || common.is_some_and(|previous| previous != offset)
        {
            return j.position();
        }
        common = Some(offset);
    }
    common
        .and_then(|(x, y)| j.position().translated(x, y).ok())
        .unwrap_or(j.position())
}
pub(crate) fn paint_route(
    painter: &egui::Painter,
    camera: Camera,
    view: Rect,
    points: &[Point2D],
    stroke: Stroke,
) {
    for pair in points.windows(2) {
        if let Some(line) = camera.clipped_segment(pair[0], pair[1], view) {
            painter.line_segment(line, stroke);
        }
    }
}
fn dashed(
    painter: &egui::Painter,
    camera: Camera,
    view: Rect,
    points: &[Point2D],
    color: egui::Color32,
) {
    for pair in points.windows(2) {
        if let Some([a, b]) = camera.clipped_segment(pair[0], pair[1], view) {
            let length = a.distance(b);
            if length <= 0. {
                continue;
            }
            let direction = (b - a) / length;
            let mut t = 0.;
            while t < length {
                painter.line_segment(
                    [a + direction * t, a + direction * (t + 5.).min(length)],
                    Stroke::new(1.5_f32, color),
                );
                t += 9.;
            }
        }
    }
}
pub(crate) fn paint_overlay(
    painter: &egui::Painter,
    p: &Project,
    page: PageId,
    camera: Camera,
    view: Rect,
    e: &Interaction,
) {
    for j in p.connection_junctions().filter(|j| j.page_id() == page) {
        if let Some(position) = camera.world_to_screen(display_junction_position(p, j, e), view)
            && view.contains(position)
        {
            let color = if e.junction_selection == Some(j.id()) {
                theme::ACCENT
            } else {
                theme::INK
            };
            if e.junction_outline {
                painter.circle_filled(position, 4.5, theme::PAPER);
                painter.circle_stroke(position, 4.5, Stroke::new(1.5_f32, color));
            } else {
                painter.circle_filled(position, 3.5, color);
            }
        }
    }
    if let Some(d) = &e.connection {
        dashed(
            painter,
            camera,
            view,
            &d.preview,
            egui::Color32::from_rgb(230, 140, 20),
        );
        painter.text(
            view.left_top() + egui::vec2(12., 28.),
            egui::Align2::LEFT_TOP,
            "Conexão: porta → pontos intermediários → porta  |  Espaço alterna L  |  Esc cancela",
            egui::FontId::proportional(12.),
            theme::ACCENT,
        );
    }
    if let Some(points) = e.connection_suggestion {
        dashed(
            painter,
            camera,
            view,
            &points,
            egui::Color32::from_rgb(120, 140, 210),
        );
    }
    if let Some(hit) = &e.connection_snap
        && (e.is_active() || matches!(hit.kind, SnapKind::Port { .. }))
        && let Some(pos) = camera.world_to_screen(hit.position, view)
    {
        let (label, color) = match &hit.kind {
            SnapKind::Port { bound: true, .. } => {
                ("Porta elétrica", egui::Color32::from_rgb(20, 170, 90))
            }
            SnapKind::Port { bound: false, .. } => {
                ("Porta sem associação", egui::Color32::from_rgb(220, 95, 50))
            }
            SnapKind::Junction(_) => ("Junção", theme::ACCENT),
            SnapKind::Vertex { .. } => ("Vértice", theme::ACCENT),
            SnapKind::Segment { .. } => ("Segmento", theme::ACCENT),
            SnapKind::Grid => ("Grade", theme::ACCENT),
        };
        painter.circle_stroke(pos, 6., Stroke::new(1.5_f32, color));
        painter.text(
            pos + egui::vec2(10., -12.),
            egui::Align2::LEFT_BOTTOM,
            label,
            egui::FontId::proportional(11.),
            color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn egui_capture_preserves_clearance_segments_across_zoom_levels() {
        let points = [
            Point2D::new(-10., 0.).unwrap(),
            Point2D::new(-12., 0.).unwrap(),
            Point2D::new(-12., 7.).unwrap(),
            Point2D::new(40., 7.).unwrap(),
            Point2D::new(40., 0.).unwrap(),
        ];
        let view = Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000., 600.));
        for scale in [2., 4., 8.] {
            let ctx = egui::Context::default();
            let camera = Camera::new([0., 0.], scale).unwrap();
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(view),
                    ..Default::default()
                },
                |ctx| {
                    let painter = ctx.layer_painter(egui::LayerId::new(
                        egui::Order::Foreground,
                        egui::Id::new("route-clearance-capture"),
                    ));
                    paint_route(
                        &painter,
                        camera,
                        view,
                        &points,
                        Stroke::new(1_f32, egui::Color32::WHITE),
                    );
                },
            );
            let captured = output
                .shapes
                .iter()
                .filter(|shape| matches!(shape.shape, egui::Shape::LineSegment { .. }))
                .count();
            assert_eq!(captured, 4, "zoom {scale} clipped a clearance segment");
        }
    }
    #[test]
    fn connected_symbol_preview_matches_committed_route() {
        let mut p = crate::connection_demo::project().unwrap();
        let route = p
            .connection_routes()
            .find(|r| r.points().len() == 4)
            .unwrap()
            .clone();
        let id = route.source().representation_id();
        let r = p.representation(id).unwrap();
        let start = r.transform().translation();
        let page = r.page_id();
        let mut e = Interaction::default();
        e.begin_move(&p, page, id, start).unwrap();
        e.update_move(start.translated(5., 0.).unwrap(), 4., false)
            .unwrap();
        let preview = display_points(&p, &route, &e).into_owned();
        let batch = e.finish_move().unwrap();
        p.transform_representations(&batch).unwrap();
        assert_eq!(preview, p.connection_route(route.id()).unwrap().points());
    }
    #[test]
    fn junction_marker_preview_follows_group_routes() {
        let mut p = crate::connection_demo::project().unwrap();
        let page = p.pages().next().unwrap().id();
        let routes = p.connection_routes().map(ConnectionRoute::id).collect();
        let position = Point2D::new(160., 140.).unwrap();
        p.edit_connection_drawing(ConnectionDrawingEdit::CreateJunction {
            page,
            position,
            routes,
        })
        .unwrap();
        let ids: Vec<_> = p
            .representations()
            .filter(|r| r.symbol().is_some())
            .map(Representation::id)
            .collect();
        let r = p.representation(ids[0]).unwrap();
        let start = r.transform().translation();
        let mut e = Interaction::default();
        e.selection.replace(ids.clone());
        e.begin_move(&p, page, ids[0], start).unwrap();
        e.update_move(start.translated(12., 8.).unwrap(), 4., false)
            .unwrap();
        let j = p.connection_junctions().next().unwrap();
        assert_eq!(
            display_junction_position(&p, j, &e),
            position.translated(12., 8.).unwrap()
        );
    }
}
