// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Geometry shared by selection and painting: transformed local rectangles or anchors.
use crate::camera::Camera;
use cad_craft::domain::*;
use eframe::egui::{Pos2, Rect};
pub const HIT_POINTS: f64 = 6.0;
pub fn corners(r: &Representation, t: Transform2D) -> Option<[Point2D; 4]> {
    let b = r.local_bounds()?;
    let a = b.origin();
    let z = b.max();
    Some([
        t.apply(a).ok()?,
        t.apply(Point2D::new(z.x(), a.y()).ok()?).ok()?,
        t.apply(z).ok()?,
        t.apply(Point2D::new(a.x(), z.y()).ok()?).ok()?,
    ])
}
pub fn hit_candidates(
    p: &Project,
    page: PageId,
    camera: Camera,
    viewport: Rect,
    cursor: Pos2,
) -> Vec<RepresentationId> {
    let Some(world) = camera.screen_to_world(cursor, viewport) else {
        return vec![];
    };
    let tolerance = HIT_POINTS / camera.scale();
    let mut hits = Vec::new();
    for r in p.representations_on_page(page).unwrap_or_default() {
        if !r.visual().visible {
            continue;
        }
        if let Some(route) = p.connection_route(r.id()) {
            if crate::connection_geometry::route_hit(route.points(), world, tolerance).is_some() {
                hits.push((r.visual().layer, 0, r.id()));
            }
            continue;
        }
        let Ok(local) = r.transform().inverse_apply(world) else {
            continue;
        };
        let distance = if let Some(graphic) = r.graphic() {
            graphic
                .drawable()
                .map(|primitive| crate::symbol_view::primitive_distance(&primitive, local))
                .unwrap_or(f64::INFINITY)
        } else if let Some(definition) = r.symbol().and_then(|id| p.symbol_definition(id)) {
            definition
                .primitives()
                .iter()
                .map(|primitive| crate::symbol_view::primitive_distance(primitive, local))
                .fold(f64::INFINITY, f64::min)
                .min(
                    definition
                        .ports()
                        .iter()
                        .map(|port| {
                            (local.x() - port.position().x()).hypot(local.y() - port.position().y())
                        })
                        .fold(f64::INFINITY, f64::min),
                )
        } else if let Some(b) = r.local_bounds() {
            let a = b.origin();
            let z = b.max();
            let dx = (a.x() - local.x()).max(0.0).max(local.x() - z.x());
            let dy = (a.y() - local.y()).max(0.0).max(local.y() - z.y());
            dx.hypot(dy)
        } else {
            local.x().hypot(local.y())
        };
        // The anchor is painted for every representation, also outside local bounds.
        let anchor = (world.x() - r.transform().translation().x())
            .hypot(world.y() - r.transform().translation().y());
        let stroke_allowance = r.graphic().map_or(0.0, |g| g.stroke_mm() / 2.0);
        if distance <= tolerance + stroke_allowance || anchor <= tolerance {
            hits.push((r.visual().layer, 1, r.id()));
        }
    }
    hits.sort_unstable();
    hits.into_iter().rev().map(|(_, _, id)| id).collect()
}
pub fn rectangle_selection(
    p: &Project,
    page: PageId,
    a: Point2D,
    b: Point2D,
    crossing: bool,
) -> Vec<RepresentationId> {
    let (min_x, max_x, min_y, max_y) = (
        a.x().min(b.x()),
        a.x().max(b.x()),
        a.y().min(b.y()),
        a.y().max(b.y()),
    );
    let inside = |q: Point2D| q.x() >= min_x && q.x() <= max_x && q.y() >= min_y && q.y() <= max_y;
    p.representations_on_page(page)
        .unwrap_or_default()
        .into_iter()
        .filter(|r| {
            if !r.visual().visible {
                return false;
            }
            if let Some(route) = p.connection_route(r.id()) {
                return if crossing {
                    route
                        .points()
                        .windows(2)
                        .any(|w| segment_intersects(w[0], w[1], min_x, max_x, min_y, max_y))
                } else {
                    route.points().iter().all(|q| inside(*q))
                };
            }
            if let Some(d) = r.symbol().and_then(|id| p.symbol_definition(id)) {
                let Ok(window) = Size2D::new(max_x - min_x, max_y - min_y)
                    .and_then(|size| Rect2D::new(Point2D::new(min_x, min_y)?, size))
                else {
                    return false;
                };
                return crate::symbol_view::symbol_in_window(d, r.transform(), window, crossing);
            }
            let Some(points) = corners(r, r.transform()) else {
                return inside(r.transform().translation());
            };
            if !crossing {
                return points.into_iter().all(inside);
            }
            if points.into_iter().any(inside) {
                return true;
            }
            // A rectangle can enclose a selection rectangle without an edge crossing.
            let window = [
                Point2D::new(min_x, min_y),
                Point2D::new(max_x, min_y),
                Point2D::new(max_x, max_y),
                Point2D::new(min_x, max_y),
            ];
            if window.into_iter().flatten().any(|q| {
                r.transform()
                    .inverse_apply(q)
                    .is_ok_and(|q| r.local_bounds().is_some_and(|b| b.contains(q)))
            }) {
                return true;
            }
            (0..4).any(|i| {
                segment_intersects(points[i], points[(i + 1) % 4], min_x, max_x, min_y, max_y)
            })
        })
        .map(Representation::id)
        .collect()
}
pub(crate) fn segment_intersects(
    a: Point2D,
    b: Point2D,
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
) -> bool {
    let mut lo: f64 = 0.0;
    let mut hi: f64 = 1.0;
    let dx = b.x() - a.x();
    let dy = b.y() - a.y();
    for (p, q) in [
        (-dx, a.x() - min_x),
        (dx, max_x - a.x()),
        (-dy, a.y() - min_y),
        (dy, max_y - a.y()),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return false;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                lo = lo.max(t)
            } else {
                hi = hi.min(t)
            }
            if lo > hi {
                return false;
            }
        }
    }
    true
}
