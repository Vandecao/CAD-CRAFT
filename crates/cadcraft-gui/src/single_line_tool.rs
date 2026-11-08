// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Single-line link tool ("Conexão unifilar"); start with
//! Session::single_line_click.
//!
//! Click a symbol to start, click free points for corners and click another
//! symbol to finish. Keys 1–9 set the phase count; Delete over a link
//! removes it. Links are documentation only: no connections, networks or
//! wire numbers.
use crate::{
    app::{CadApp, Level},
    camera::Camera,
    editing::GraphicEdit,
    hit_test::hit_candidates,
    interaction::{Interaction, Tool},
    state::Session,
    theme,
};
use cad_craft::{
    domain::*,
    error::{DomainError, Result},
};
use eframe::egui::{self, CursorIcon, Key, PointerButton, Rect, Response, Stroke, Ui};

/// Phase count used when none was chosen.
const DEFAULT_PHASES: u8 = 3;

impl Session {
    /// Start the single-line tool on the active page.
    pub fn begin_single_line(&mut self) -> Result<()> {
        self.active
            .ok_or(DomainError::InvalidRelation(crate::strings::OPEN_PAGE))?;
        let phases = self.graphics.single_line_phases;
        self.graphics.cancel();
        self.graphics.tool = Tool::DrawSingleLine;
        self.graphics.single_line_phases = if phases == 0 { DEFAULT_PHASES } else { phases };
        Ok(())
    }
    /// Handle a click at `at` over symbol `on`; returns true when a link was
    /// created.
    pub fn single_line_click(&mut self, at: Point2D, on: Option<RepresentationId>) -> Result<bool> {
        let page = self
            .active
            .ok_or(DomainError::InvalidRelation(crate::strings::OPEN_PAGE))?;
        let on = on.filter(|id| {
            self.project()
                .and_then(|p| p.representation(*id))
                .is_some_and(|r| r.page_id() == page && r.symbol().is_some())
        });
        let Some((source, mut points)) = self.graphics.single_line.clone() else {
            let source = on.ok_or(DomainError::InvalidRelation(
                "comece a conexão unifilar clicando em um símbolo",
            ))?;
            self.graphics.single_line = Some((source, vec![at]));
            return Ok(false);
        };
        if points.last() != Some(&at) {
            points.push(at);
        }
        match on.filter(|id| *id != source) {
            Some(target) => {
                let project = self.project().ok_or(DomainError::InvalidRelation(
                    crate::strings::NO_OPEN_PROJECT,
                ))?;
                let phases = match self.graphics.single_line_phases {
                    0 => DEFAULT_PHASES,
                    n => n,
                };
                let link = SingleLineLink::new(project.id(), page, source, target, points, phases)?;
                self.graphics.single_line = None;
                self.dispatch_graphic(GraphicEdit::SingleLine(SingleLineEdit::Create(link)))?;
                self.graphics.tool = Tool::DrawSingleLine;
                self.graphics.single_line_phases = phases;
                Ok(true)
            }
            None => {
                self.graphics.single_line = Some((source, points));
                Ok(false)
            }
        }
    }
}

/// Distance from `p` to the polyline, in millimetres.
fn distance(points: &[Point2D], p: Point2D) -> f64 {
    points
        .windows(2)
        .map(|w| {
            let (a, b) = (w[0], w[1]);
            let (dx, dy) = (b.x() - a.x(), b.y() - a.y());
            let len2 = dx * dx + dy * dy;
            let t = if len2 == 0.0 {
                0.0
            } else {
                (((p.x() - a.x()) * dx + (p.y() - a.y()) * dy) / len2).clamp(0.0, 1.0)
            };
            (p.x() - a.x() - t * dx).hypot(p.y() - a.y() - t * dy)
        })
        .fold(f64::INFINITY, f64::min)
}

impl CadApp {
    /// Canvas input while the single-line tool is active.
    pub(crate) fn interact_single_line(
        &mut self,
        ui: &Ui,
        response: &Response,
        page: PageId,
        camera: Camera,
        rect: Rect,
    ) {
        if response.secondary_clicked() {
            self.session.graphics.cancel();
            return;
        }
        if let Some(n) = ui.input_mut(|i| {
            (1..=9u8).find(|n| {
                let key = Key::from_name(&n.to_string()).expect("digit key");
                i.consume_key(egui::Modifiers::NONE, key)
            })
        }) {
            self.session.graphics.single_line_phases = n;
            self.log(Level::Info, format!("Conexão unifilar com {n} fase(s)."));
        }
        let Some(cursor) = ui.input(|i| i.pointer.interact_pos()) else {
            return;
        };
        if !response.hovered() {
            return;
        }
        ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
        let Some(world) = camera.screen_to_world(cursor, rect) else {
            return;
        };
        let at = crate::camera::snap(
            world,
            self.preferences.grid_mm,
            self.preferences.snap_enabled,
        );
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Delete)) {
            let tolerance = 6.0 / camera.scale();
            let near = self.session.project().and_then(|p| {
                p.single_lines()
                    .filter(|l| l.page_id() == page)
                    .find(|l| distance(l.points(), world) <= tolerance)
                    .map(SingleLineLink::id)
            });
            if let Some(id) = near {
                self.graphic_edit(GraphicEdit::SingleLine(SingleLineEdit::Delete(id)));
                self.session.graphics.tool = Tool::DrawSingleLine;
            }
        }
        if response.clicked_by(PointerButton::Primary) {
            let on = self.session.project().and_then(|p| {
                hit_candidates(p, page, camera, rect, cursor)
                    .into_iter()
                    .find(|id| p.representation(*id).is_some_and(|r| r.symbol().is_some()))
            });
            match self.session.single_line_click(at, on) {
                Ok(true) => self.log(
                    Level::Info,
                    "Conexão unifilar criada (sem efeito nas redes). Clique em outro símbolo para continuar; Esc encerra.",
                ),
                Ok(false) => {}
                Err(e) => self.log(Level::Error, e.to_string()),
            }
        }
    }
}

/// Links of the page and the link being drawn.
pub(crate) fn paint_single_lines(
    painter: &egui::Painter,
    p: &Project,
    page: PageId,
    camera: Camera,
    view: Rect,
    e: &Interaction,
) {
    let screen = |pt: Point2D| camera.world_to_screen(pt, view);
    let stroke = |px: f32, color| Stroke::new((px * camera.scale() as f32).max(1.0), color);
    for link in p.single_lines().filter(|l| l.page_id() == page) {
        let points: Vec<_> = link.points().iter().filter_map(|pt| screen(*pt)).collect();
        painter.add(egui::Shape::line(points, stroke(0.5, theme::INK)));
        let (marks, label) = link.phase_marks();
        for [a, b] in marks {
            if let (Some(a), Some(b)) = (screen(a), screen(b)) {
                painter.line_segment([a, b], stroke(0.35, theme::INK));
            }
        }
        if let Some(at) = label.and_then(screen) {
            painter.text(
                at,
                egui::Align2::CENTER_CENTER,
                link.phases().to_string(),
                egui::FontId::proportional((2.5 * camera.scale() as f32).max(9.0)),
                theme::INK,
            );
        }
    }
    if let Some((_, points)) = &e.single_line {
        let points: Vec<_> = points.iter().filter_map(|pt| screen(*pt)).collect();
        if points.len() > 1 {
            painter.add(egui::Shape::line(
                points.clone(),
                Stroke::new(1.5_f32, theme::ACCENT),
            ));
        }
        for pt in points {
            painter.circle_stroke(pt, 3.0, Stroke::new(1.0_f32, theme::ACCENT));
        }
    }
}
