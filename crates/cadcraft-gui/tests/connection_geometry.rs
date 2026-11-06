// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Fixture helpers use validated synthetic inputs and fail immediately on setup errors.
#![allow(clippy::unwrap_used)]
use cad_craft::domain::Point2D;
use cad_gui::connection_geometry::*;

fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}

fn valid(points: &[Point2D], start: Point2D, end: Point2D) {
    assert_eq!(points.first(), Some(&start));
    assert_eq!(points.last(), Some(&end));
    for pair in points.windows(2) {
        assert_ne!(pair[0], pair[1]);
        assert!(pair[0].x() == pair[1].x() || pair[0].y() == pair[1].y());
    }
}

#[test]
fn orthogonal_routes_use_direction_in_mm_and_keep_explicit_waypoints() {
    let anchors = [p(-12.0, -8.0), p(4.0, 4.0), p(4.0, 8.0)];
    assert_eq!(
        orthogonal_route(&anchors, true),
        vec![p(-12.0, -8.0), p(4.0, -8.0), p(4.0, 4.0), p(4.0, 8.0)]
    );
    assert_eq!(
        orthogonal_route(&anchors[..2], false),
        vec![p(-12.0, -8.0), p(-12.0, 4.0), p(4.0, 4.0)]
    );
}

#[test]
fn route_builder_removes_only_consecutive_zero_segments() {
    assert_eq!(orthogonal_route(&[], true), vec![]);
    let anchors = [
        p(0.0, 0.0),
        p(0.0, 0.0),
        p(2.0, 0.0),
        p(4.0, 0.0),
        p(0.0, 0.0),
    ];
    assert_eq!(
        orthogonal_route(&anchors, true),
        vec![p(0.0, 0.0), p(2.0, 0.0), p(4.0, 0.0), p(0.0, 0.0)]
    );
}

#[test]
fn nearest_segment_clamps_endpoints_and_handles_diagonal_and_zero() {
    assert_eq!(
        nearest_on_segment(p(-4.0, 3.0), p(0.0, 0.0), p(8.0, 0.0)),
        p(0.0, 0.0)
    );
    assert_eq!(
        nearest_on_segment(p(12.0, 3.0), p(0.0, 0.0), p(8.0, 0.0)),
        p(8.0, 0.0)
    );
    assert_eq!(
        nearest_on_segment(p(4.0, 0.0), p(0.0, 0.0), p(4.0, 4.0)),
        p(2.0, 2.0)
    );
    assert_eq!(
        nearest_on_segment(p(4.0, 0.0), p(1.0, 1.0), p(1.0, 1.0)),
        p(1.0, 1.0)
    );
}

#[test]
fn nearest_handles_extreme_finite_coordinates_without_panicking() {
    assert_eq!(
        nearest_on_segment(p(0.0, 1.0), p(-f64::MAX, 0.0), p(f64::MAX, 0.0)),
        p(0.0, 0.0)
    );
    assert_eq!(
        nearest_on_segment(p(0.0, 0.0), p(-f64::MAX, -f64::MAX), p(f64::MAX, f64::MAX)),
        p(0.0, 0.0)
    );
}

#[test]
fn distant_cursor_still_chooses_nearest_endpoint_of_a_tiny_diagonal() {
    assert_eq!(
        nearest_on_segment(p(f64::MAX, f64::MAX), p(1.0, 1.0), p(2.0, 2.0)),
        p(2.0, 2.0)
    );
    assert_eq!(
        nearest_on_segment(p(f64::MAX, f64::MAX), p(1e-300, 1e-300), p(2e-300, 2e-300)),
        p(2e-300, 2e-300)
    );
}

#[test]
fn hit_uses_trace_distance_including_endpoint_caps_and_stable_ties() {
    let route = [p(-8.0, -4.0), p(8.0, -4.0), p(8.0, 8.0)];
    assert_eq!(route_hit(&route, p(0.0, 4.0), 1.0), None);
    assert_eq!(
        route_hit(&route, p(-9.0, -4.0), 1.0),
        Some((0, p(-8.0, -4.0)))
    );
    assert_eq!(
        route_hit(&route, p(8.0, -4.0), 0.0),
        Some((0, p(8.0, -4.0)))
    );
    assert_eq!(route_hit(&route, p(8.5, 3.0), 1.0), Some((1, p(8.0, 3.0))));
}

#[test]
fn hit_rejects_invalid_tolerance_and_skips_degenerate_segments() {
    let route = [p(0.0, 0.0), p(0.0, 0.0), p(0.0, 8.0)];
    for tolerance in [-1.0, f64::NAN, f64::INFINITY] {
        assert!(route_hit(&route, p(0.0, 0.0), tolerance).is_none());
    }
    assert_eq!(route_hit(&route, p(0.0, 2.0), 0.0), Some((1, p(0.0, 2.0))));
    assert!(route_hit(&route[..2], p(0.0, 0.0), 1.0).is_none());
    assert!(route_hit(&[], p(0.0, 0.0), 1.0).is_none());
}

#[test]
fn move_middle_segment_moves_perpendicularly_and_preserves_anchors() {
    let route = [p(0.0, 0.0), p(8.0, 0.0), p(8.0, 8.0), p(16.0, 8.0)];
    assert_eq!(
        move_segment(&route, 1, p(12.0, 999.0)).unwrap(),
        vec![p(0.0, 0.0), p(12.0, 0.0), p(12.0, 8.0), p(16.0, 8.0)]
    );
}

#[test]
fn moving_first_and_last_segments_keeps_electrical_endpoints() {
    let route = [p(0.0, 0.0), p(8.0, 0.0), p(8.0, 8.0)];
    assert_eq!(
        move_segment(&route, 0, p(999.0, -4.0)).unwrap(),
        vec![p(0.0, 0.0), p(0.0, -4.0), p(8.0, -4.0), p(8.0, 8.0)]
    );
    assert_eq!(
        move_segment(&route, 1, p(12.0, 999.0)).unwrap(),
        vec![p(0.0, 0.0), p(12.0, 0.0), p(12.0, 8.0), p(8.0, 8.0)]
    );
}

#[test]
fn moving_single_segment_adds_two_terminal_legs() {
    let route = [p(-8.0, 0.0), p(8.0, 0.0)];
    assert_eq!(
        move_segment(&route, 0, p(0.0, 4.0)).unwrap(),
        vec![p(-8.0, 0.0), p(-8.0, 4.0), p(8.0, 4.0), p(8.0, 0.0)]
    );
    assert_eq!(move_segment(&route, 0, p(999.0, 0.0)).unwrap(), route);
}

#[test]
fn inserting_vertex_subdivides_segment_and_rejects_non_interior_positions() {
    let route = [p(-8.0, 0.0), p(8.0, 0.0)];
    assert_eq!(
        insert_vertex(&route, 0, p(0.0, 0.0)).unwrap(),
        vec![p(-8.0, 0.0), p(0.0, 0.0), p(8.0, 0.0)]
    );
    for position in [p(0.0, 1.0), p(-8.0, 0.0), p(9.0, 0.0)] {
        assert!(insert_vertex(&route, 0, position).is_err());
    }
}

#[test]
fn removal_reroutes_locally_and_preserves_other_waypoints() {
    let route = [p(0.0, 0.0), p(4.0, 0.0), p(4.0, 8.0), p(12.0, 8.0)];
    assert_eq!(
        remove_vertex(&route, 1, false).unwrap(),
        vec![p(0.0, 0.0), p(0.0, 8.0), p(4.0, 8.0), p(12.0, 8.0)]
    );
    assert_eq!(
        remove_vertex(&[p(0.0, 0.0), p(4.0, 0.0), p(8.0, 0.0)], 1, true).unwrap(),
        vec![p(0.0, 0.0), p(8.0, 0.0)]
    );
}

#[test]
fn moving_vertex_keeps_selected_position_with_orthogonal_neighbors() {
    let route = [p(0.0, 0.0), p(4.0, 0.0), p(4.0, 8.0), p(12.0, 8.0)];
    let edited = move_vertex(&route, 1, p(-4.0, -4.0), true).unwrap();
    valid(&edited, route[0], route[3]);
    assert_eq!(
        edited,
        vec![
            p(0.0, 0.0),
            p(-4.0, 0.0),
            p(-4.0, -4.0),
            p(4.0, -4.0),
            p(4.0, 8.0),
            p(12.0, 8.0)
        ]
    );
    assert_eq!(move_vertex(&route, 1, route[1], false).unwrap(), route);
}

#[test]
fn editing_rejects_endpoints_bad_indices_diagonals_and_zero_segments() {
    let route = [p(0.0, 0.0), p(4.0, 0.0), p(4.0, 8.0)];
    for index in [0, 2, 3, usize::MAX] {
        assert!(remove_vertex(&route, index, true).is_err());
        assert!(move_vertex(&route, index, p(2.0, 2.0), true).is_err());
    }
    assert!(move_vertex(&route, 1, route[0], true).is_err());
    for bad in [
        vec![],
        vec![p(0.0, 0.0)],
        vec![p(0.0, 0.0), p(0.0, 0.0)],
        vec![p(0.0, 0.0), p(4.0, 4.0)],
    ] {
        assert!(move_segment(&bad, 0, p(2.0, 2.0)).is_err());
        assert!(insert_vertex(&bad, 0, p(2.0, 2.0)).is_err());
        assert!(remove_vertex(&bad, 1, true).is_err());
        assert!(move_vertex(&bad, 1, p(2.0, 2.0), true).is_err());
    }
    assert!(move_segment(&route, usize::MAX, p(2.0, 2.0)).is_err());
    assert!(insert_vertex(&route, usize::MAX, p(2.0, 2.0)).is_err());
}
