// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_craft::domain::Point2D;
use cad_gui::camera::{Camera, grid_points, snap};
use eframe::egui::{Pos2, Rect, Vec2};

fn view() -> Rect {
    Rect::from_min_size(Pos2::new(100.0, 50.0), Vec2::new(80.0, 60.0))
}
#[test]
fn points_are_real_intersections_with_document_spacing_and_negative_coordinates() {
    let camera = Camera::new([0.0, 0.0], 4.0).unwrap();
    let points = grid_points(&camera, view(), 2.5);
    assert_eq!(points.len(), 63);
    for p in [
        Pos2::new(100.0, 50.0),
        Pos2::new(140.0, 80.0),
        Pos2::new(180.0, 110.0),
    ] {
        assert!(points.contains(&p));
    }
    for p in points {
        let world = camera.screen_to_world(p, view()).unwrap();
        assert!((world.x() / 2.5 - (world.x() / 2.5).round()).abs() < 1e-8);
        assert!((world.y() / 2.5 - (world.y() / 2.5).round()).abs() < 1e-8);
    }
}
#[test]
fn pan_and_cursor_zoom_keep_grid_anchored_in_document_space() {
    let mut camera = Camera::new([0.0, 0.0], 4.0).unwrap();
    camera.pan(Vec2::new(7.0, -3.0));
    let origin = Pos2::new(147.0, 77.0);
    assert!(grid_points(&camera, view(), 2.5).contains(&origin));
    camera.zoom_at(2.0, origin, view());
    let points = grid_points(&camera, view(), 2.5);
    assert!(points.contains(&origin));
    assert!(points.contains(&Pos2::new(167.0, 77.0)));
}
#[test]
fn extreme_zoom_and_large_viewports_have_a_finite_bounded_point_budget() {
    let viewport = Rect::from_min_size(Pos2::new(-100.0, 13.0), Vec2::new(16384.0, 8192.0));
    for center in [[0.0, 0.0], [-1e12, 1e12]] {
        for scale in [0.0001, 0.001, 0.1, 1.0, 10000.0] {
            let points = grid_points(&Camera::new(center, scale).unwrap(), viewport, 0.1);
            assert!(points.len() <= 12000);
            assert!(!points.is_empty());
            assert!(
                points
                    .iter()
                    .all(|p| p.is_finite() && viewport.contains(*p))
            );
            let unique: std::collections::HashSet<_> = points
                .iter()
                .map(|p| (p.x.to_bits(), p.y.to_bits()))
                .collect();
            assert_eq!(unique.len(), points.len());
        }
    }
}
#[test]
fn adaptive_render_density_does_not_change_snap_resolution() {
    let camera = Camera::new([0.0, 0.0], 0.0001).unwrap();
    let points = grid_points(&camera, view(), 2.5);
    assert!(!points.is_empty());
    assert!(points.len() < 100);
    assert_eq!(
        snap(Point2D::new(3.7, -2.8).unwrap(), 2.5, true),
        Point2D::new(2.5, -2.5).unwrap()
    );
    // At this zoom the visual step is 32768 * 2.5 mm = 8.192 logical points.
    // Assert in screen space: inverse f32 screen quantization is amplified by 10000.
    for p in points {
        let offset = (p.x - 140.0) / 8.192;
        assert!((offset - offset.round()).abs() < 0.0001);
    }
}
#[test]
fn no_visible_intersection_or_invalid_input_produces_no_points() {
    let camera = Camera::new([1.0, 1.0], 10000.0).unwrap();
    assert!(grid_points(&camera, view(), 4.0).is_empty());
    for spacing in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(grid_points(&camera, view(), spacing).is_empty());
    }
    for viewport in [
        Rect::NOTHING,
        Rect::EVERYTHING,
        Rect::from_min_size(Pos2::ZERO, Vec2::ZERO),
    ] {
        assert!(grid_points(&camera, viewport, 4.0).is_empty());
    }
}
#[test]
fn grid_covers_visible_exterior_without_using_paper_or_screen_origin() {
    let viewport = Rect::from_min_size(Pos2::new(120.0, 60.0), Vec2::new(800.0, 600.0));
    let camera = Camera::new([-500.0, -100.0], 4.0).unwrap();
    let points = grid_points(&camera, viewport, 4.0);
    assert!(points.len() > 100);
    assert!(points.iter().any(|p| p.x > viewport.right() - 20.0));
    for p in points {
        let w = camera.screen_to_world(p, viewport).unwrap();
        assert!(w.x() < 0.0);
        assert!((w.x() / 4.0 - (w.x() / 4.0).round()).abs() < 1e-5);
        assert!((w.y() / 4.0 - (w.y() / 4.0).round()).abs() < 1e-5);
    }
}
