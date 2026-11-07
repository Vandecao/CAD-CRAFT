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
        for scale in [0.0001, 0.001, 0.1, 1.0, 10000.0, 100.0] {
            let points = grid_points(&Camera::new(center, scale).unwrap(), viewport, 0.1);
            assert!(points.len() <= 60_000);
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
    // Dense enough to draw (10 px) in a normal viewport: dots are drawn.
    let normal = Rect::from_min_size(Pos2::ZERO, Vec2::new(1200.0, 800.0));
    assert!(!grid_points(&Camera::new([0.0, 0.0], 100.0).unwrap(), normal, 0.1).is_empty());
}
#[test]
fn far_zoom_hides_the_grid_without_changing_snap_resolution() {
    // As in EPLAN: below MIN_GRID_PX the grid is hidden, never coarsened.
    let far = Camera::new([0.0, 0.0], 0.0001).unwrap();
    assert!(grid_points(&far, view(), 2.5).is_empty());
    let threshold = cad_gui::camera::MIN_GRID_PX / 2.5;
    let just_below = Camera::new([0.0, 0.0], threshold * 0.99).unwrap();
    assert!(grid_points(&just_below, view(), 2.5).is_empty());
    let at = Camera::new([0.0, 0.0], threshold * 1.01).unwrap();
    let points = grid_points(&at, view(), 2.5);
    assert!(!points.is_empty());
    // Visible dots keep the real spacing (2.5 mm), not a multiple.
    for p in points {
        let world = at.screen_to_world(p, view()).unwrap();
        assert!((world.x() / 2.5 - (world.x() / 2.5).round()).abs() < 1e-4);
    }
    assert_eq!(
        snap(Point2D::new(3.7, -2.8).unwrap(), 2.5, true),
        Point2D::new(2.5, -2.5).unwrap()
    );
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
