// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Typed coordinates for the placement tools ("Inserção de coordenadas");
//! start with CadApp::apply_typed_point.
//!
//! While a symbol, connection or graphic line is being placed, a small bar
//! takes X and Y in millimetres, absolute or relative to the last point.
//! Enter (or "Aplicar") acts exactly like a click at that point, so the
//! mouse and the keyboard can be mixed and every result is one undo step.
use crate::{
    app::{CadApp, Level},
    interaction::Tool,
};
use cad_craft::{
    domain::*,
    error::{DomainError, Result},
};
use eframe::egui::{self, Context, Key, Rect};

/// State of the coordinate bar.
#[derive(Debug, Default)]
pub struct CoordinateInput {
    /// The bar is shown during placement tools.
    pub open: bool,
    /// X in millimetres (absolute or offset).
    pub x: f64,
    /// Y in millimetres (absolute or offset).
    pub y: f64,
    /// X/Y are offsets from the last point.
    pub relative: bool,
    /// Last point placed with typed coordinates.
    last: Option<Point2D>,
    /// A field of the bar had keyboard focus in the previous frame.
    had_focus: bool,
}

impl CadApp {
    /// Whether the active tool takes typed points.
    pub fn coordinate_tool_active(&self) -> bool {
        self.session.graphics.insertion.is_some()
            || matches!(
                self.session.graphics.tool,
                Tool::DrawConnection | Tool::InsertGraphic(_)
            )
    }

    /// Reference for relative coordinates: the last typed point, the last
    /// corner of the wire being drawn or the start of the line.
    fn coordinate_reference(&self) -> Point2D {
        let g = &self.session.graphics;
        g.connection
            .as_ref()
            .and_then(|d| d.anchors.last().copied())
            .or(g.graphic_start)
            .or(self.coordinates.last)
            .unwrap_or_else(|| Point2D::new(0.0, 0.0).expect("origin"))
    }

    /// Act like a click at the typed point for the active tool; returns the
    /// point used.
    pub fn apply_typed_point(&mut self) -> Result<Point2D> {
        let c = &self.coordinates;
        let point = if c.relative {
            self.coordinate_reference().translated(c.x, c.y)?
        } else {
            Point2D::new(c.x, c.y)?
        };
        let page = self
            .session
            .active
            .ok_or(DomainError::InvalidRelation(crate::strings::OPEN_PAGE))?;
        if let Some(draft) = &mut self.session.graphics.insertion {
            let t = draft.transform;
            draft.transform =
                Transform2D::new(point, t.rotation_degrees(), t.mirror_x(), t.mirror_y())?;
            draft.positioned = true;
            self.session.confirm_symbol_insertion()?;
        } else {
            match self.session.graphics.tool {
                Tool::DrawConnection => {
                    let project = self.session.project().ok_or(DomainError::InvalidRelation(
                        crate::strings::NO_OPEN_PROJECT,
                    ))?;
                    let scene = &mut self.panels_state.connection_scene;
                    scene.sync(project, page, self.session.revision);
                    // Exact point: a port or wire is used only when it lies
                    // on the typed coordinates; otherwise it is a free corner.
                    let hit = scene.snap(point, 8.0e6, 0.0, false);
                    if let Some(d) = &mut self.session.graphics.connection {
                        d.update(hit.position);
                    }
                    self.session.connection_click(hit)?;
                }
                Tool::InsertGraphic(kind) => match self.session.graphics.graphic_start {
                    None => self.session.graphics.graphic_start = Some(point),
                    Some(start) => {
                        let end = Point2D::new(point.x() - start.x(), point.y() - start.y())?;
                        let graphic = PageGraphic::from_drag(
                            kind,
                            Point2D::new(0.0, 0.0)?,
                            end,
                            self.free_content.free_stroke,
                        )?;
                        self.session
                            .insert_graphic(graphic, Transform2D::new(start, 0.0, false, false)?)?;
                        self.session.graphics.tool = Tool::InsertGraphic(kind);
                        self.session.graphics.graphic_start = None;
                    }
                },
                _ => {
                    return Err(DomainError::InvalidRelation(
                        "escolha um símbolo, uma conexão ou uma linha para usar coordenadas",
                    ));
                }
            }
        }
        self.coordinates.last = Some(point);
        Ok(point)
    }

    /// The X/Y bar at the top left of the editor, shown when enabled and a
    /// placement tool is active.
    pub(crate) fn coordinate_bar(&mut self, ctx: &Context, rect: Rect) {
        if !self.coordinates.open || !self.coordinate_tool_active() {
            self.coordinates.had_focus = false;
            return;
        }
        let enter = self.coordinates.had_focus && ctx.input(|i| i.key_pressed(Key::Enter));
        let mut apply = enter;
        egui::Area::new(egui::Id::new("coordinate_bar"))
            .order(egui::Order::Foreground)
            .fixed_pos(rect.left_top() + egui::vec2(8.0, 8.0))
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let c = &mut self.coordinates;
                        ui.label("Coordenadas");
                        let x = ui.add(
                            egui::DragValue::new(&mut c.x)
                                .prefix("X ")
                                .suffix(" mm")
                                .speed(1.0)
                                .max_decimals(3),
                        );
                        let y = ui.add(
                            egui::DragValue::new(&mut c.y)
                                .prefix("Y ")
                                .suffix(" mm")
                                .speed(1.0)
                                .max_decimals(3),
                        );
                        ui.checkbox(&mut c.relative, "Relativo")
                            .on_hover_text("X/Y somados ao último ponto");
                        c.had_focus = x.has_focus() || y.has_focus();
                        if ui.button("Aplicar (Enter)").clicked() {
                            apply = true;
                        }
                    });
                });
            });
        if apply {
            match self.apply_typed_point() {
                Ok(p) => {
                    let mm = |v: f64| format!("{v:.2}").replace('.', ",");
                    self.log(
                        Level::Info,
                        format!("Ponto em X {} Y {} mm.", mm(p.x()), mm(p.y())),
                    )
                }
                Err(e) => self.log(Level::Error, e.to_string()),
            }
        }
    }
}
