// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Display geometry helpers; start with arc_points and the Builder path methods.

use super::*;

/// Pontos de um arco (graus, anti-horário) com erro máximo de tesselagem.
pub(super) fn arc_points(center: Point2D, radius: f64, start: f64, sweep: f64) -> Vec<Point2D> {
    let step = if radius > CURVE_TOLERANCE {
        (2.0 * (1.0 - CURVE_TOLERANCE / radius).acos()).to_degrees()
    } else {
        45.0
    };
    let n = ((sweep.abs() / step.max(1e-6)).ceil() as usize).clamp(8, 720);
    (0..=n)
        .filter_map(|i| {
            let a = (start + sweep * i as f64 / n as f64).to_radians();
            center.translated(radius * a.cos(), radius * a.sin()).ok()
        })
        .collect()
}

pub(super) fn ellipse_points(center: Point2D, rx: f64, ry: f64, rotation: f64) -> Vec<Point2D> {
    let mut points = arc_points(
        Point2D::new(0.0, 0.0).expect("origem"),
        rx.max(ry),
        0.0,
        360.0,
    );
    points.pop();
    let (s, c) = rotation.to_radians().sin_cos();
    let n = points.len();
    (0..n)
        .filter_map(|i| {
            let a = std::f64::consts::TAU * i as f64 / n as f64;
            let (x, y) = (rx * a.cos(), ry * a.sin());
            center.translated(x * c - y * s, x * s + y * c).ok()
        })
        .collect()
}

pub(super) fn rect_corners(r: Rect2D) -> Vec<Point2D> {
    let (a, b) = (r.origin(), r.max());
    [
        (a.x(), a.y()),
        (b.x(), a.y()),
        (b.x(), b.y()),
        (a.x(), b.y()),
    ]
    .into_iter()
    .filter_map(|(x, y)| Point2D::new(x, y).ok())
    .collect()
}

/// Círculo/arco da primitiva levado ao mundo (espelhos invertem o sentido).
pub(super) fn world_curve(
    primitive: &SymbolPrimitive,
    t: Transform2D,
) -> Option<(Point2D, f64, f64, f64)> {
    let (center, radius, mut start, mut sweep) = match primitive {
        SymbolPrimitive::Circle { center, radius } => (*center, *radius, 0.0, 360.0),
        SymbolPrimitive::Arc {
            center,
            radius,
            start_degrees,
            sweep_degrees,
        } => (*center, *radius, *start_degrees, *sweep_degrees),
        _ => return None,
    };
    start = start.rem_euclid(360.0);
    if t.mirror_x() {
        start = 180.0 - start;
        sweep = -sweep;
    }
    if t.mirror_y() {
        start = -start;
        sweep = -sweep;
    }
    Some((
        t.apply(center).ok()?,
        radius,
        (start + t.rotation_degrees()).rem_euclid(360.0),
        sweep,
    ))
}

impl Builder<'_> {
    pub(super) fn path(
        &mut self,
        points: Vec<Point2D>,
        closed: bool,
        stroke: Option<DisplayStroke>,
        fill: Option<DisplayColor>,
    ) {
        if points.len() < 2 || (stroke.is_none() && fill.is_none()) {
            return;
        }
        let layer = self.layer;
        self.items.push(DisplayItem::Path {
            points,
            closed,
            stroke,
            fill,
            layer,
        });
    }
    pub(super) fn stroke(
        &mut self,
        points: Vec<Point2D>,
        closed: bool,
        width: f64,
        color: DisplayColor,
    ) {
        self.path(
            points,
            closed,
            Some(DisplayStroke {
                width,
                dash: vec![],
                color,
            }),
            None,
        );
    }
}
