// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! geometry support for the frame editor; start at the parent module.

use super::*;

pub(in crate::frame_editor) fn pt(x: f64, y: f64) -> Option<Point2D> {
    Point2D::new(x, y).ok()
}
pub(in crate::frame_editor) fn dist_segment(p: Point2D, a: Point2D, b: Point2D) -> f64 {
    let (dx, dy) = (b.x() - a.x(), b.y() - a.y());
    let len2 = dx * dx + dy * dy;
    let t = if len2 == 0.0 {
        0.0
    } else {
        (((p.x() - a.x()) * dx + (p.y() - a.y()) * dy) / len2).clamp(0.0, 1.0)
    };
    let (x, y) = (a.x() + t * dx, a.y() + t * dy);
    ((p.x() - x).powi(2) + (p.y() - y).powi(2)).sqrt()
}
pub(in crate::frame_editor) fn dist_rect(p: Point2D, r: Rect2D, filled: bool) -> f64 {
    let (a, b) = (r.origin(), r.max());
    if filled && r.contains(p) {
        return 0.0;
    }
    let c = [
        pt(a.x(), a.y()),
        pt(b.x(), a.y()),
        pt(b.x(), b.y()),
        pt(a.x(), b.y()),
    ];
    (0..4)
        .filter_map(|i| Some(dist_segment(p, c[i]?, c[(i + 1) % 4]?)))
        .fold(f64::INFINITY, f64::min)
}
/// Caixa aproximada do texto (mesmo critério do desenho dos símbolos).
pub(in crate::frame_editor) fn text_box(
    origin: Point2D,
    chars: usize,
    style: &FrameTextStyle,
) -> Option<Rect2D> {
    let (w, h) = (style.height * 0.6 * chars.max(1) as f64, style.height);
    let (fx, fy) = style.align.fractions();
    Rect2D::new(
        pt(origin.x() - fx * w, origin.y() - fy * h)?,
        Size2D::new(w, h).ok()?,
    )
    .ok()
}
/// Distância (mm) do ponto ao elemento, para seleção.
pub fn element_distance(project: &Project, e: &FrameElement, p: Point2D) -> f64 {
    match e {
        FrameElement::Line { start, end, .. } => dist_segment(p, *start, *end),
        FrameElement::Rectangle { rect, .. } => dist_rect(p, *rect, false),
        FrameElement::Image { rect, .. } => dist_rect(p, *rect, true),
        FrameElement::Circle { center, radius, .. } | FrameElement::Arc { center, radius, .. } => {
            (center.distance_to(p).unwrap_or(f64::INFINITY) - radius).abs()
        }
        FrameElement::Text {
            origin,
            text,
            style,
        } => text_box(*origin, text.default.chars().count(), style)
            .map_or(f64::INFINITY, |r| dist_rect(p, r, true)),
        FrameElement::Special {
            origin,
            field,
            style,
            ..
        } => {
            let _ = project;
            text_box(*origin, field.label().chars().count() + 2, style)
                .map_or(f64::INFINITY, |r| dist_rect(p, r, true))
        }
    }
}
