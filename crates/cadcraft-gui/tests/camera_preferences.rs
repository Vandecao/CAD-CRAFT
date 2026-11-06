// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Fixture helpers use validated synthetic inputs and fail immediately on setup errors.
#![allow(clippy::unwrap_used)]
use cad_craft::domain::{Point2D, Rect2D, Size2D};
use cad_gui::{
    camera::{Camera, grid_lines, snap},
    preferences::{PanelId, Preferences, RibbonTab},
};
use eframe::egui::{Pos2, Rect, Vec2};

fn viewport() -> Rect {
    Rect::from_min_size(Pos2::new(100.0, 50.0), Vec2::new(800.0, 600.0))
}
fn point(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}
fn bounds(x: f64, y: f64, w: f64, h: f64) -> Rect2D {
    Rect2D::new(point(x, y), Size2D::new(w, h).unwrap()).unwrap()
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 0.001, "{a} != {b}");
}

#[test]
fn y_is_inverted_only_in_camera() {
    let c = Camera::new([0.0, 0.0], 2.0).unwrap();
    assert_eq!(
        c.world_to_screen(point(10.0, 20.0), viewport()).unwrap(),
        Pos2::new(520.0, 310.0)
    );
}
#[test]
fn camera_roundtrip_handles_negative_coordinates() {
    let c = Camera::new([-35.0, 90.0], 3.5).unwrap();
    for (x, y) in [(-300.0, 510.0), (0.0, 0.0), (108.5, -28.25)] {
        let p = c
            .screen_to_world(
                c.world_to_screen(point(x, y), viewport()).unwrap(),
                viewport(),
            )
            .unwrap();
        near(p.x(), x);
        near(p.y(), y);
    }
}
#[test]
fn zoom_keeps_document_point_under_cursor() {
    let mut c = Camera::default();
    let cursor = Pos2::new(285.0, 192.0);
    let p = c.screen_to_world(cursor, viewport()).unwrap();
    for factor in [2.0, 0.3, 4.0, 0.125] {
        assert!(c.zoom_at(factor, cursor, viewport()));
        let q = c.world_to_screen(p, viewport()).unwrap();
        near(q.x as f64, cursor.x as f64);
        near(q.y as f64, cursor.y as f64);
    }
}
#[test]
fn pan_uses_screen_delta_and_preserves_scale() {
    let mut c = Camera::new([0.0, 0.0], 2.0).unwrap();
    c.pan(Vec2::new(80.0, -40.0));
    assert_eq!(
        c.world_to_screen(point(0.0, 0.0), viewport()).unwrap(),
        Pos2::new(580.0, 310.0)
    );
    near(c.scale(), 2.0);
}
#[test]
fn fit_page_preserves_aspect_and_centers_it() {
    let mut c = Camera::default();
    assert!(c.fit(bounds(0.0, 0.0, 400.0, 200.0), viewport()));
    let center = c.world_to_screen(point(200.0, 100.0), viewport()).unwrap();
    assert_eq!(center, viewport().center());
    assert!(viewport().contains(c.world_to_screen(point(0.0, 0.0), viewport()).unwrap()));
    assert!(viewport().contains(c.world_to_screen(point(400.0, 200.0), viewport()).unwrap()));
}
#[test]
fn fit_content_can_be_outside_sheet() {
    let mut c = Camera::default();
    assert!(c.fit(bounds(-900.0, 500.0, 100.0, 50.0), viewport()));
    assert_eq!(
        c.world_to_screen(point(-850.0, 525.0), viewport()).unwrap(),
        viewport().center()
    );
}
#[test]
fn fit_degenerate_content_is_finite() {
    let mut c = Camera::default();
    assert!(c.fit(bounds(10.0, 20.0, 0.0, 0.0), viewport()));
    assert!(c.scale().is_finite());
}
#[test]
fn invalid_camera_operations_preserve_state() {
    assert!(Camera::new([f64::NAN, 0.0], 2.0).is_none());
    let mut c = Camera::default();
    let old = c;
    assert!(!c.zoom_at(f64::INFINITY, viewport().center(), viewport()));
    assert!(!c.fit(bounds(0.0, 0.0, 10.0, 10.0), Rect::NOTHING));
    assert_eq!(old, c);
}
#[test]
fn grid_stays_world_aligned_after_pan_and_zoom() {
    let mut c = Camera::default();
    c.pan(Vec2::new(57.0, -61.0));
    c.zoom_at(0.03, viewport().center(), viewport());
    let g = grid_lines(&c, viewport(), 2.5);
    assert!(!g.is_empty());
    assert!(g.len() < 2048);
    for (a, b) in g {
        let p = c.screen_to_world(a, viewport()).unwrap();
        let q = c.screen_to_world(b, viewport()).unwrap();
        if (a.x - b.x).abs() < 0.01 {
            near(p.x() / 2.5, (p.x() / 2.5).round());
        } else {
            near(q.y() / 2.5, (q.y() / 2.5).round());
        }
    }
}
#[test]
fn grid_handles_zoom_limits_without_unbounded_geometry() {
    for scale in [0.0001, 10000.0] {
        let c = Camera::new([0.0, 0.0], scale).unwrap();
        assert!(grid_lines(&c, viewport(), 1.0).len() < 2048);
    }
}
#[test]
fn snap_is_independent_of_grid_visibility() {
    let p = snap(point(-3.1, 5.1), 2.5, true);
    near(p.x(), -2.5);
    near(p.y(), 5.0);
    assert_eq!(snap(point(3.1, 5.1), 2.5, false), point(3.1, 5.1));
}
#[test]
fn preferences_roundtrip_includes_panels_and_ribbon() {
    let mut p = Preferences::default();
    p.panel_mut(PanelId::Navigator).width = 384.0;
    p.panel_mut(PanelId::Insertion).visible = false;
    p.ribbon_collapsed = true;
    p.ribbon_tab = RibbonTab::View;
    p.grid_mm = 2.5;
    let restored = Preferences::from_json(&p.to_json().unwrap()).unwrap();
    assert_eq!(p, restored);
}
#[test]
fn malformed_preferences_fall_back_or_are_sanitized() {
    assert!(Preferences::from_json("not json").is_none());
    let mut p = Preferences {
        grid_mm: -10.0,
        ..Default::default()
    };
    p.panel_mut(PanelId::Navigator).width = f32::INFINITY;
    p.sanitize();
    assert!(p.grid_mm > 0.0);
    assert!(p.panel(PanelId::Navigator).width.is_finite());
}
#[test]
fn layout_restore_restores_all_panel_states() {
    let mut p = Preferences::default();
    p.panel_mut(PanelId::Properties).visible = false;
    p.preview_height = 510.0;
    p.restore_layout();
    assert!(p.panel(PanelId::Properties).visible);
    assert!(p.preview_height < 510.0);
}
