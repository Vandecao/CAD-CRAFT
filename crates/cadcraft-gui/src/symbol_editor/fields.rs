// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Numeric fields for the points of the selected symbol primitive; start
//! with primitive_point_fields.

use super::*;

/// X and Y of one point; returns the edited point, or `None` when unchanged
/// or invalid.
fn point_row(ui: &mut Ui, label: &str, point: Point2D) -> Option<Point2D> {
    let (mut x, mut y) = (point.x(), point.y());
    let mut changed = false;
    ui.label(label);
    changed |= ui
        .add(
            egui::DragValue::new(&mut x)
                .speed(0.5)
                .prefix("X ")
                .max_decimals(3),
        )
        .changed();
    changed |= ui
        .add(
            egui::DragValue::new(&mut y)
                .speed(0.5)
                .prefix("Y ")
                .max_decimals(3),
        )
        .changed();
    ui.end_row();
    changed.then(|| Point2D::new(x, y).ok()).flatten()
}

fn length_row(ui: &mut Ui, label: &str, value: &mut f64) -> bool {
    ui.label(label);
    let changed = ui
        .add(
            egui::DragValue::new(value)
                .speed(0.5)
                .range(0.01..=1000.0)
                .max_decimals(3),
        )
        .changed();
    ui.end_row();
    changed
}

/// Coordinates of the primitive in millimetres, edited in place.
pub(in crate::symbol_editor) fn primitive_point_fields(
    ui: &mut Ui,
    primitive: &mut SymbolPrimitive,
) {
    egui::Grid::new("symbol_primitive_points")
        .num_columns(3)
        .show(ui, |ui| match primitive {
            SymbolPrimitive::Line { start, end } => {
                if let Some(p) = point_row(ui, "Início", *start) {
                    *start = p;
                }
                if let Some(p) = point_row(ui, "Fim", *end) {
                    *end = p;
                }
            }
            SymbolPrimitive::Polyline { points, .. } => {
                for (i, point) in points.iter_mut().enumerate() {
                    if let Some(p) = point_row(ui, &format!("Vértice {}", i + 1), *point) {
                        *point = p;
                    }
                }
            }
            SymbolPrimitive::Rectangle { rect } => {
                let (mut w, mut h) = (rect.size().width(), rect.size().height());
                let mut origin = rect.origin();
                let mut changed = false;
                if let Some(p) = point_row(ui, "Canto", origin) {
                    origin = p;
                    changed = true;
                }
                changed |= length_row(ui, "Largura", &mut w);
                changed |= length_row(ui, "Altura", &mut h);
                if changed && let Ok(r) = Size2D::new(w, h).and_then(|s| Rect2D::new(origin, s)) {
                    *rect = r;
                }
            }
            SymbolPrimitive::Circle { center, radius }
            | SymbolPrimitive::Arc { center, radius, .. } => {
                if let Some(p) = point_row(ui, "Centro", *center) {
                    *center = p;
                }
                length_row(ui, "Raio", radius);
            }
            SymbolPrimitive::Ellipse {
                center,
                radius_x,
                radius_y,
                ..
            } => {
                if let Some(p) = point_row(ui, "Centro", *center) {
                    *center = p;
                }
                length_row(ui, "Raio X", radius_x);
                length_row(ui, "Raio Y", radius_y);
            }
            SymbolPrimitive::Text { origin, .. } | SymbolPrimitive::DynamicText { origin, .. } => {
                if let Some(p) = point_row(ui, "Origem", *origin) {
                    *origin = p;
                }
            }
            SymbolPrimitive::Anchor { position } => {
                if let Some(p) = point_row(ui, "Posição", *position) {
                    *position = p;
                }
            }
        });
    if let SymbolPrimitive::Arc {
        start_degrees,
        sweep_degrees,
        ..
    } = primitive
    {
        ui.horizontal(|ui| {
            ui.add(
                egui::DragValue::new(start_degrees)
                    .speed(5.0)
                    .prefix("Início "),
            );
            ui.add(
                egui::DragValue::new(sweep_degrees)
                    .speed(5.0)
                    .prefix("Abertura "),
            );
        });
    }
}
