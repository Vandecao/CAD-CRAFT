// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Fixture helpers use validated synthetic inputs and fail immediately on setup errors.
#![allow(clippy::unwrap_used)]
use cad_craft::domain::*;
use cad_gui::{
    camera::Camera,
    symbol_view::{paint_symbol, primitive_distance, symbol_in_window},
};
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, vec2};
fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}
fn rect(x: f64, y: f64, w: f64, h: f64) -> Rect2D {
    Rect2D::new(p(x, y), Size2D::new(w, h).unwrap()).unwrap()
}
fn t(x: f64, y: f64, a: f64, mx: bool, my: bool) -> Transform2D {
    Transform2D::new(p(x, y), a, mx, my).unwrap()
}
fn definition(primitives: Vec<SymbolPrimitive>) -> SymbolDefinition {
    SymbolDefinition::new(
        SymbolDefinitionId::new("test.geometry").unwrap(),
        1,
        "Geometry",
        "Test",
        primitives,
        vec![],
    )
    .unwrap()
}
#[test]
fn hit_uses_strokes_not_whole_bounding_box() {
    let line = SymbolPrimitive::Line {
        start: p(-10.0, -10.0),
        end: p(10.0, 10.0),
    };
    assert!(primitive_distance(&line, p(-9.0, 9.0)) > 12.0);
    assert!(primitive_distance(&line, p(0.0, 0.0)) < 1e-9);
    let circle = SymbolPrimitive::Circle {
        center: p(0.0, 0.0),
        radius: 10.0,
    };
    assert_eq!(primitive_distance(&circle, p(0.0, 0.0)), 10.0);
    assert!(primitive_distance(&circle, p(6.0, 8.0)) < 1e-9);
    assert_eq!(
        primitive_distance(
            &SymbolPrimitive::Rectangle {
                rect: rect(-5.0, -3.0, 10.0, 6.0)
            },
            p(0.0, 0.0)
        ),
        3.0
    );
}
#[test]
fn arc_hit_respects_sweep_direction_and_endpoints() {
    let arc = SymbolPrimitive::Arc {
        center: p(0.0, 0.0),
        radius: 5.0,
        start_degrees: 0.0,
        sweep_degrees: -90.0,
    };
    assert!(primitive_distance(&arc, p(0.0, -5.0)) < 1e-9);
    assert!(primitive_distance(&arc, p(0.0, 5.0)) > 7.0);
    assert!(primitive_distance(&arc, p(5.1, 0.0)) < 0.101);
}
#[test]
fn crossing_detects_circle_edge_but_not_empty_center_even_after_mirroring() {
    let d = definition(vec![SymbolPrimitive::Circle {
        center: p(3.0, 0.0),
        radius: 10.0,
    }]);
    let transform = t(-20.0, 10.0, 90.0, true, false);
    // Transformed center is (-20,7); interior-only window is not a stroke hit.
    assert!(!symbol_in_window(
        &d,
        transform,
        rect(-21.0, 6.0, 2.0, 2.0),
        true
    ));
    assert!(symbol_in_window(
        &d,
        transform,
        rect(-20.1, 16.9, 0.2, 0.2),
        true
    ));
    assert!(!symbol_in_window(
        &d,
        transform,
        rect(-20.1, 16.9, 0.2, 0.2),
        false
    ));
    assert!(symbol_in_window(
        &d,
        transform,
        rect(-31.0, -4.0, 22.0, 22.0),
        false
    ));
}
#[test]
fn containment_of_rotated_arc_uses_actual_extrema() {
    let d = definition(vec![SymbolPrimitive::Arc {
        center: p(0.0, 0.0),
        radius: 5.0,
        start_degrees: 0.0,
        sweep_degrees: 90.0,
    }]);
    assert!(symbol_in_window(
        &d,
        t(0.0, 0.0, 45.0, false, false),
        rect(-3.54, 3.53, 7.08, 1.48),
        false
    ));
    assert!(!symbol_in_window(
        &d,
        t(0.0, 0.0, 45.0, false, false),
        rect(-3.54, 3.53, 7.08, 1.0),
        false
    ));
}
#[test]
fn renderer_emits_clipped_geometry_and_transformed_text() {
    let d = definition(vec![
        SymbolPrimitive::Line {
            start: p(-40.0, 0.0),
            end: p(40.0, 0.0),
        },
        SymbolPrimitive::Polyline {
            points: vec![p(-2.0, 0.0), p(0.0, 4.0), p(2.0, 0.0)],
            closed: true,
        },
        SymbolPrimitive::Rectangle {
            rect: rect(-3.0, -3.0, 6.0, 6.0),
        },
        SymbolPrimitive::Circle {
            center: p(0.0, 0.0),
            radius: 5.0,
        },
        SymbolPrimitive::Arc {
            center: p(0.0, 0.0),
            radius: 6.0,
            start_degrees: 0.0,
            sweep_degrees: -180.0,
        },
        SymbolPrimitive::Text {
            origin: p(-1.0, -1.0),
            text: "M".into(),
            height: 2.0,
        },
        SymbolPrimitive::Anchor {
            position: p(0.0, 0.0),
        },
    ]);
    let ctx = egui::Context::default();
    let viewport = Rect::from_min_size(Pos2::ZERO, vec2(200.0, 200.0));
    let camera = Camera::new([0.0, 0.0], 10.0).unwrap();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(viewport),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                paint_symbol(
                    &ui.painter_at(viewport),
                    &d,
                    t(0.0, 0.0, 90.0, true, false),
                    camera,
                    viewport,
                    Stroke::new(1.0_f32, Color32::BLACK),
                );
            });
        },
    );
    assert!(
        output
            .shapes
            .iter()
            .filter(|s| matches!(s.shape, egui::Shape::LineSegment { .. }))
            .count()
            > 20
    );
    assert!(
        output
            .shapes
            .iter()
            .any(|s| matches!(&s.shape,egui::Shape::Text(text) if text.galley.job.text=="M"))
    );
    for s in &output.shapes {
        if let egui::Shape::LineSegment { points, .. } = &s.shape {
            assert!(points.iter().all(|p| viewport.expand(0.01).contains(*p)));
        }
    }
}

#[test]
fn text_texture_coordinates_survive_font_atlas_growth_in_same_frame() {
    let d = definition(vec![SymbolPrimitive::Text {
        origin: p(0.0, 0.0),
        text: "M".into(),
        height: 2.0,
    }]);
    let ctx = egui::Context::default();
    let viewport = Rect::from_min_size(Pos2::ZERO, vec2(400.0, 400.0));
    let camera = Camera::new([0.0, 0.0], 10.0).unwrap();
    let mut grew = false;
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(viewport),
            ..Default::default()
        },
        |ctx| {
            let painter = ctx.layer_painter(egui::LayerId::background());
            paint_symbol(
                &painter,
                &d,
                t(0.0, 0.0, 0.0, false, false),
                camera,
                viewport,
                Stroke::new(1.0_f32, Color32::BLACK),
            );
            let before = ctx.fonts(|f| f.font_image_size());
            for size in (30..180).step_by(5) {
                let _ = painter.layout_no_wrap(
                    "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".into(),
                    egui::FontId::proportional(size as f32),
                    Color32::BLACK,
                );
            }
            grew = before != ctx.fonts(|f| f.font_image_size());
            paint_symbol(
                &painter,
                &d,
                t(10.0, 0.0, 0.0, false, false),
                camera,
                viewport,
                Stroke::new(1.0_f32, Color32::BLACK),
            );
        },
    );
    assert!(grew, "fixture must grow the font atlas");
    let meshes: Vec<_> = output
        .shapes
        .iter()
        .filter(|s| matches!(s.shape, egui::Shape::Text(_) | egui::Shape::Mesh(_)))
        .map(|s| {
            let primitives = ctx.tessellate(vec![s.clone()], output.pixels_per_point);
            match &primitives[0].primitive {
                egui::epaint::Primitive::Mesh(m) => {
                    m.vertices.iter().map(|v| v.uv).collect::<Vec<_>>()
                }
                _ => panic!("text mesh"),
            }
        })
        .collect();
    assert_eq!(meshes.len(), 2);
    assert_eq!(meshes[0], meshes[1]);
}
