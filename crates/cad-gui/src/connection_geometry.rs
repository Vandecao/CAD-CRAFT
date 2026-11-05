// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Pure connection route geometry in document millimeters.
//!
//! These functions neither inspect nor change electrical topology. Coincident
//! vertices and crossings of non-adjacent segments remain ordinary geometry.
use cad_craft::domain::Point2D;
use cad_craft::error::{DomainError, Result};

/// Connect every pair of anchors by a deterministic L. Explicit collinear
/// anchors are retained; only consecutive equal points are removed.
pub fn orthogonal_route(anchors: &[Point2D], horizontal_first: bool) -> Vec<Point2D> {
    cad_craft::domain::orthogonal_route_points(
        anchors,
        &vec![horizontal_first; anchors.len().saturating_sub(1)],
    )
    .expect("finite Point2D and one orientation per span")
}

/// Closest point on the closed segment, including a degenerate point segment.
/// Axis-aligned paths use exact coordinate clamping. Diagonals are supported for
/// hit testing, even though route editing refuses diagonal input.
pub fn nearest_on_segment(cursor: Point2D, a: Point2D, b: Point2D) -> Point2D {
    if a.x() == b.x() {
        return point(a.x(), cursor.y().clamp(a.y().min(b.y()), a.y().max(b.y())));
    }
    if a.y() == b.y() {
        return point(cursor.x().clamp(a.x().min(b.x()), a.x().max(b.x())), a.y());
    }
    // Scaling before subtraction prevents overflow for finite coordinates at
    // opposite extremes. Interpolation also avoids subtracting those extremes.
    let segment_scale = [a.x(), a.y(), b.x(), b.y()]
        .into_iter()
        .map(f64::abs)
        .fold(0.0, f64::max);
    let scale = segment_scale.max(cursor.x().abs()).max(cursor.y().abs());
    let ax = a.x() / scale;
    let ay = a.y() / scale;
    let cx = cursor.x() / scale;
    let cy = cursor.y() / scale;
    // Normalize the segment separately so a distant cursor cannot underflow
    // its direction or squared length to zero.
    let dx = b.x() / segment_scale - a.x() / segment_scale;
    let dy = b.y() / segment_scale - a.y() / segment_scale;
    let length_squared = dx * dx + dy * dy;
    let projection = (cx - ax) * dx + (cy - ay) * dy;
    if projection <= 0.0 {
        return a;
    }
    let t = ((projection / length_squared) * (scale / segment_scale)).clamp(0.0, 1.0);
    point(interpolate(a.x(), b.x(), t), interpolate(a.y(), b.y(), t))
}

/// Closest nonzero segment within a finite nonnegative tolerance in mm.
/// Equal distances choose the lowest segment index; no AABB-only selection.
pub fn route_hit(
    points: &[Point2D],
    cursor: Point2D,
    tolerance_mm: f64,
) -> Option<(usize, Point2D)> {
    if !tolerance_mm.is_finite() || tolerance_mm < 0.0 {
        return None;
    }
    let mut best: Option<(usize, Point2D, f64)> = None;
    for (index, pair) in points.windows(2).enumerate() {
        if pair[0] == pair[1] {
            continue;
        }
        let closest = nearest_on_segment(cursor, pair[0], pair[1]);
        let distance = (cursor.x() - closest.x()).hypot(cursor.y() - closest.y());
        if distance <= tolerance_mm && best.is_none_or(|(_, _, old)| distance < old) {
            best = Some((index, closest, distance));
        }
    }
    best.map(|(index, closest, _)| (index, closest))
}

/// Move a segment perpendicularly. Both electrical endpoints stay fixed;
/// moving a terminal segment introduces a terminal leg. Neighboring segments
/// are reconnected orthogonally, without discarding unrelated waypoints.
pub fn move_segment(points: &[Point2D], index: usize, position: Point2D) -> Result<Vec<Point2D>> {
    validate_route(points)?;
    let pair = points
        .get(index..)
        .and_then(|tail| tail.get(..2))
        .ok_or(invalid())?;
    let a = pair[0];
    let b = pair[1];
    let horizontal = a.y() == b.y();
    let moved_a = if horizontal {
        point(a.x(), position.y())
    } else {
        point(position.x(), a.y())
    };
    let moved_b = if horizontal {
        point(b.x(), position.y())
    } else {
        point(position.x(), b.y())
    };
    if a == moved_a && b == moved_b {
        return Ok(points.to_vec());
    }
    let mut anchors = points[..index].to_vec();
    if index == 0 {
        anchors.push(a);
    }
    anchors.extend([moved_a, moved_b]);
    if index + 2 == points.len() {
        anchors.push(b);
    }
    anchors.extend_from_slice(&points[index + 2..]);
    let result = orthogonal_route(&anchors, horizontal);
    validate_route(&result)?;
    Ok(result)
}

/// Subdivide a segment at a point strictly inside its trace. Off-trace points
/// and existing segment endpoints are refused; use `move_vertex` to reshape.
pub fn insert_vertex(
    points: &[Point2D],
    segment: usize,
    position: Point2D,
) -> Result<Vec<Point2D>> {
    validate_route(points)?;
    let pair = points
        .get(segment..)
        .and_then(|tail| tail.get(..2))
        .ok_or(invalid())?;
    if position == pair[0]
        || position == pair[1]
        || nearest_on_segment(position, pair[0], pair[1]) != position
    {
        return Err(invalid());
    }
    let mut result = points.to_vec();
    result.insert(segment + 1, position);
    Ok(result)
}

/// Remove an interior waypoint and reconnect its neighbors by a deterministic
/// L. A geometrically necessary bend may remain after removing its waypoint.
pub fn remove_vertex(
    points: &[Point2D],
    index: usize,
    horizontal_first: bool,
) -> Result<Vec<Point2D>> {
    validate_vertex(points, index)?;
    let mut anchors = points.to_vec();
    anchors.remove(index);
    let result = orthogonal_route(&anchors, horizontal_first);
    validate_route(&result)?;
    Ok(result)
}

/// Move an interior waypoint, routing both adjacent spans orthogonally.
/// Endpoints and unrelated waypoints are fixed. Collapsing onto an adjacent
/// waypoint is refused explicitly instead of silently deleting it.
pub fn move_vertex(
    points: &[Point2D],
    index: usize,
    position: Point2D,
    horizontal_first: bool,
) -> Result<Vec<Point2D>> {
    validate_vertex(points, index)?;
    if position == points[index - 1] || position == points[index + 1] {
        return Err(invalid());
    }
    let mut anchors = points.to_vec();
    anchors[index] = position;
    let result = orthogonal_route(&anchors, horizontal_first);
    validate_route(&result)?;
    Ok(result)
}

fn invalid() -> DomainError {
    DomainError::InvalidRelation("rota ortogonal ou edição de segmento/vértice inválida")
}

fn validate_route(points: &[Point2D]) -> Result<()> {
    if points.len() < 2
        || points.windows(2).any(|pair| {
            pair[0] == pair[1] || (pair[0].x() != pair[1].x() && pair[0].y() != pair[1].y())
        })
    {
        return Err(invalid());
    }
    Ok(())
}

fn validate_vertex(points: &[Point2D], index: usize) -> Result<()> {
    validate_route(points)?;
    if index == 0 || index >= points.len() - 1 {
        return Err(invalid());
    }
    Ok(())
}

fn point(x: f64, y: f64) -> Point2D {
    // Inputs are finite Point2D coordinates or convex interpolations thereof.
    Point2D::new(x, y).expect("finite route coordinates")
}

fn interpolate(a: f64, b: f64, t: f64) -> f64 {
    if a.is_sign_positive() == b.is_sign_positive() {
        a + (b - a) * t
    } else {
        a * (1.0 - t) + b * t
    }
}
