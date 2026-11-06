// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Shared vector geometry: analytical picking, exact curve/window intersections,
//! and zoom-dependent tessellation only for visible painting.
use crate::camera::Camera;
use cad_craft::domain::*;
use eframe::egui::{self, Painter, Rect, Stroke};

fn distance(a: Point2D, b: Point2D) -> f64 {
    (a.x() - b.x()).hypot(a.y() - b.y())
}
fn segment_distance(p: Point2D, a: Point2D, b: Point2D) -> f64 {
    let (dx, dy) = (b.x() - a.x(), b.y() - a.y());
    let length = dx.hypot(dy);
    if length == 0.0 {
        return distance(p, a);
    }
    let (ux, uy) = (dx / length, dy / length);
    let along = ((p.x() - a.x()) * ux + (p.y() - a.y()) * uy).clamp(0.0, length);
    (p.x() - a.x() - ux * along).hypot(p.y() - a.y() - uy * along)
}
fn rect_points(r: Rect2D) -> [Point2D; 4] {
    let (a, b) = (r.origin(), r.max());
    [
        a,
        Point2D::new(b.x(), a.y()).expect("bounds"),
        b,
        Point2D::new(a.x(), b.y()).expect("bounds"),
    ]
}
fn on_arc(angle: f64, start: f64, sweep: f64) -> bool {
    let angle = angle.rem_euclid(360.0);
    let start = start.rem_euclid(360.0);
    let delta = if sweep >= 0.0 {
        (angle - start).rem_euclid(360.0)
    } else {
        (start - angle).rem_euclid(360.0)
    };
    delta <= sweep.abs() + 1e-10 || (360.0 - delta).abs() < 1e-10
}
fn curve_point(center: Point2D, radius: f64, angle: f64) -> Option<Point2D> {
    let (s, c) = angle.rem_euclid(360.0).to_radians().sin_cos();
    center.translated(c * radius, s * radius).ok()
}
fn rect_distance(r: Rect2D, p: Point2D) -> f64 {
    let (a, b) = (r.origin(), r.max());
    (a.x() - p.x())
        .max(0.0)
        .max(p.x() - b.x())
        .hypot((a.y() - p.y()).max(0.0).max(p.y() - b.y()))
}
pub fn primitive_distance(primitive: &SymbolPrimitive, p: Point2D) -> f64 {
    use SymbolPrimitive as P;
    match primitive {
        P::Line { start, end } => segment_distance(p, *start, *end),
        P::Polyline { points, closed } => {
            let mut d = points
                .windows(2)
                .map(|s| segment_distance(p, s[0], s[1]))
                .fold(f64::INFINITY, f64::min);
            if *closed && let (Some(a), Some(b)) = (points.first(), points.last()) {
                d = d.min(segment_distance(p, *a, *b));
            }
            d
        }
        P::Rectangle { rect } => {
            let q = rect_points(*rect);
            (0..4)
                .map(|i| segment_distance(p, q[i], q[(i + 1) % 4]))
                .fold(f64::INFINITY, f64::min)
        }
        P::Circle { center, radius } => (distance(p, *center) - radius).abs(),
        P::Arc {
            center,
            radius,
            start_degrees,
            sweep_degrees,
        } => {
            let start_degrees = start_degrees.rem_euclid(360.0);
            let a = (p.y() - center.y()).atan2(p.x() - center.x()).to_degrees();
            if on_arc(a, start_degrees, *sweep_degrees) {
                (distance(p, *center) - radius).abs()
            } else {
                [start_degrees, start_degrees + sweep_degrees]
                    .into_iter()
                    .filter_map(|a| curve_point(*center, *radius, a))
                    .map(|q| distance(p, q))
                    .fold(f64::INFINITY, f64::min)
            }
        }
        P::Text { .. } | P::DynamicText { .. } => primitive
            .bounds()
            .map(|b| rect_distance(b, p))
            .unwrap_or(f64::INFINITY),
        P::Anchor { position } => distance(p, *position),
        P::Ellipse {
            center,
            radius_x,
            radius_y,
            rotation_degrees,
        } => {
            let points = ellipse_points(*center, *radius_x, *radius_y, *rotation_degrees, 50.0);
            (0..points.len())
                .map(|i| segment_distance(p, points[i], points[(i + 1) % points.len()]))
                .fold(f64::INFINITY, f64::min)
        }
    }
}

/// Iterates straight edges without allocating. Text's nominal rectangle is used
/// only for selection, while its glyph mesh is used for painting.
fn straight_edges(primitive: &SymbolPrimitive, mut visit: impl FnMut(Point2D, Point2D)) {
    match primitive {
        SymbolPrimitive::Line { start, end } => visit(*start, *end),
        SymbolPrimitive::Polyline { points, closed } => {
            for pair in points.windows(2) {
                visit(pair[0], pair[1]);
            }
            if *closed && let (Some(a), Some(b)) = (points.last(), points.first()) {
                visit(*a, *b);
            }
        }
        SymbolPrimitive::Rectangle { rect } => {
            let p = rect_points(*rect);
            for i in 0..4 {
                visit(p[i], p[(i + 1) % 4]);
            }
        }
        SymbolPrimitive::Ellipse {
            center,
            radius_x,
            radius_y,
            rotation_degrees,
        } => {
            let p = ellipse_points(*center, *radius_x, *radius_y, *rotation_degrees, 50.0);
            for i in 0..p.len() {
                visit(p[i], p[(i + 1) % p.len()]);
            }
        }
        SymbolPrimitive::Text { .. } | SymbolPrimitive::DynamicText { .. } => {
            if let Ok(b) = primitive.bounds() {
                let p = rect_points(b);
                for i in 0..4 {
                    visit(p[i], p[(i + 1) % 4]);
                }
            }
        }
        SymbolPrimitive::Anchor { position } => visit(*position, *position),
        _ => {}
    }
}
fn world_curve(primitive: &SymbolPrimitive, t: Transform2D) -> Option<(Point2D, f64, f64, f64)> {
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
fn curve_extrema(
    center: Point2D,
    radius: f64,
    start: f64,
    sweep: f64,
    mut visit: impl FnMut(Point2D),
) {
    for a in [start, start + sweep, 0.0, 90.0, 180.0, 270.0] {
        if on_arc(a, start, sweep)
            && let Some(p) = curve_point(center, radius, a)
        {
            visit(p);
        }
    }
}
fn curve_crosses(center: Point2D, radius: f64, start: f64, sweep: f64, window: Rect2D) -> bool {
    let mut inside = false;
    curve_extrema(center, radius, start, sweep, |p| {
        inside |= window.contains(p)
    });
    if inside {
        return true;
    }
    let (a, b) = (window.origin(), window.max());
    let valid = |x: f64, y: f64| {
        x >= a.x() - 1e-10
            && x <= b.x() + 1e-10
            && y >= a.y() - 1e-10
            && y <= b.y() + 1e-10
            && on_arc(
                (y - center.y()).atan2(x - center.x()).to_degrees(),
                start,
                sweep,
            )
    };
    for x in [a.x(), b.x()] {
        let dx = (x - center.x()) / radius;
        if dx.abs() <= 1.0 {
            let dy = radius * (1.0 - dx * dx).max(0.0).sqrt();
            if valid(x, center.y() + dy) || valid(x, center.y() - dy) {
                return true;
            }
        }
    }
    for y in [a.y(), b.y()] {
        let dy = (y - center.y()) / radius;
        if dy.abs() <= 1.0 {
            let dx = radius * (1.0 - dy * dy).max(0.0).sqrt();
            if valid(center.x() + dx, y) || valid(center.x() - dx, y) {
                return true;
            }
        }
    }
    false
}
pub fn symbol_in_window(
    d: &SymbolDefinition,
    t: Transform2D,
    window: Rect2D,
    crossing: bool,
) -> bool {
    let primitive_hit = |primitive: &SymbolPrimitive| {
        if let Some((c, r, a, s)) = world_curve(primitive, t) {
            if crossing {
                return curve_crosses(c, r, a, s, window);
            }
            let mut all = true;
            curve_extrema(c, r, a, s, |p| all &= window.contains(p));
            return all;
        }
        let mut hit = !crossing;
        straight_edges(primitive, |a, b| {
            let (Ok(a), Ok(b)) = (t.apply(a), t.apply(b)) else {
                hit = false;
                return;
            };
            if crossing {
                hit |= crate::hit_test::segment_intersects(
                    a,
                    b,
                    window.origin().x(),
                    window.max().x(),
                    window.origin().y(),
                    window.max().y(),
                );
            } else {
                hit &= window.contains(a) && window.contains(b);
            }
        });
        if crossing
            && matches!(primitive, SymbolPrimitive::Text { .. })
            && let Ok(bounds) = primitive.bounds()
        {
            hit |= rect_points(window)
                .into_iter()
                .any(|p| t.inverse_apply(p).is_ok_and(|p| bounds.contains(p)));
        }
        hit
    };
    if crossing {
        d.primitives().iter().any(primitive_hit)
    } else {
        d.primitives().iter().all(primitive_hit)
            && d.ports()
                .iter()
                .all(|port| t.apply(port.position()).is_ok_and(|p| window.contains(p)))
    }
}

pub fn paint_symbol(
    painter: &Painter,
    d: &SymbolDefinition,
    t: Transform2D,
    camera: Camera,
    viewport: Rect,
    stroke: Stroke,
) {
    paint_symbol_with(painter, d, t, camera, viewport, stroke, &placeholder);
}

pub fn paint_primitive(
    painter: &Painter,
    primitive: &SymbolPrimitive,
    t: Transform2D,
    camera: Camera,
    viewport: Rect,
    stroke: Stroke,
) {
    paint_styled(
        painter,
        primitive,
        PrimitiveStyle::default(),
        t,
        camera,
        viewport,
        stroke,
        &placeholder,
    );
}

fn paint_primitive_plain(
    painter: &Painter,
    primitive: &SymbolPrimitive,
    t: Transform2D,
    camera: Camera,
    viewport: Rect,
    stroke: Stroke,
) {
    if let SymbolPrimitive::Text {
        origin,
        text,
        height,
    } = primitive
    {
        paint_text(
            painter,
            t,
            camera,
            viewport,
            (*origin, text, *height),
            stroke.color,
        );
    } else if let Some((center, radius, start, sweep)) = world_curve(primitive, t) {
        // Sagitta <= 0.2 logical points; hard cap keeps imported extremes bounded.
        let pixel_radius = radius * camera.scale();
        let angle = 2.0 * (1.0 - 0.2 / pixel_radius.max(0.2)).clamp(-1.0, 1.0).acos();
        let n = ((sweep.to_radians().abs() / angle.max(1e-8)).ceil() as usize).clamp(8, 2048);
        let mut previous = curve_point(center, radius, start);
        for i in 1..=n {
            let next = curve_point(center, radius, start + sweep * i as f64 / n as f64);
            if let (Some(a), Some(b)) = (previous, next)
                && let Some(line) = camera.clipped_segment(a, b, viewport)
            {
                painter.line_segment(line, stroke);
            }
            previous = next;
        }
    } else if let SymbolPrimitive::Anchor { position } = primitive {
        if let Ok(p) = t.apply(*position)
            && let Some(p) = camera.world_to_screen(p, viewport)
            && viewport.contains(p)
        {
            painter.circle_stroke(p, 2.5, stroke);
        }
    } else {
        straight_edges(primitive, |a, b| {
            if let (Ok(a), Ok(b)) = (t.apply(a), t.apply(b))
                && let Some(line) = camera.clipped_segment(a, b, viewport)
            {
                painter.line_segment(line, stroke);
            }
        });
    }
}

fn paint_text(
    painter: &Painter,
    t: Transform2D,
    camera: Camera,
    viewport: Rect,
    text_data: (Point2D, &str, f64),
    color: egui::Color32,
) {
    let (origin, text, height) = text_data;
    let width = height * 0.6 * text.chars().count() as f64;
    let local = move |x: f64, y: f64| origin.translated(x, y);
    paint_text_mapped(
        painter, t, camera, viewport, text, height, width, &local, color,
    );
}

/// Text with nominal bounds [0, width] by [0, height], transformed to world space by local alignment and rotation, then by t.
#[allow(clippy::too_many_arguments)]
fn paint_text_mapped(
    painter: &Painter,
    t: Transform2D,
    camera: Camera,
    viewport: Rect,
    text: &str,
    height: f64,
    width: f64,
    local: &dyn Fn(f64, f64) -> cad_craft::error::Result<Point2D>,
    color: egui::Color32,
) {
    let font_size = (height * camera.scale()).clamp(8.0, 256.0) as f32;
    let galley = painter.layout_no_wrap(text.to_owned(), egui::FontId::monospace(font_size), color);
    let bounds = galley.mesh_bounds;
    if !bounds.is_finite() || bounds.width() <= 0.0 || bounds.height() <= 0.0 {
        return;
    }
    let mut transformed = (*galley).clone();
    let mut all_bounds = Rect::NOTHING;
    for row in &mut transformed.rows {
        let mut data = (*row.row).clone();
        let mut row_bounds = Rect::NOTHING;
        for v in &mut data.visuals.mesh.vertices {
            let x = (v.pos.x + row.pos.x - bounds.left()) / bounds.width();
            let y = (v.pos.y + row.pos.y - bounds.top()) / bounds.height();
            let screen = local(x as f64 * width, (1.0 - y as f64) * height)
                .and_then(|p| t.apply(p))
                .ok()
                .and_then(|p| camera.world_to_screen(p, viewport));
            if let Some(p) = screen {
                v.pos = p;
                row_bounds.extend_with(p);
            } else {
                return;
            }
        }
        data.visuals.mesh_bounds = row_bounds;
        row.pos = egui::Pos2::ZERO;
        row.row = std::sync::Arc::new(data);
        all_bounds = all_bounds.union(row_bounds);
    }
    transformed.mesh_bounds = all_bounds;
    transformed.rect = all_bounds;
    // Keep texel UVs until egui tessellates the entire frame: fonts laid out
    // later can grow the atlas. Normalizing here corrupts earlier glyphs.
    painter.add(egui::Shape::Text(egui::epaint::TextShape::new(
        egui::Pos2::ZERO,
        std::sync::Arc::new(transformed),
        color,
    )));
}

// Styled geometry.

/// Closed local ellipse points, with sample count chosen by scale.
fn ellipse_points(center: Point2D, rx: f64, ry: f64, rotation: f64, pixels: f64) -> Vec<Point2D> {
    let n = ((rx.max(ry) * pixels).sqrt() * 6.0).clamp(24.0, 720.0) as usize;
    let (s, c) = rotation.to_radians().sin_cos();
    (0..n)
        .filter_map(|i| {
            let a = std::f64::consts::TAU * i as f64 / n as f64;
            let (x, y) = (rx * a.cos(), ry * a.sin());
            center.translated(x * c - y * s, x * s + y * c).ok()
        })
        .collect()
}

/// World-space primitive outline: points and closure flag; None for text and anchors.
fn outline(
    primitive: &SymbolPrimitive,
    t: Transform2D,
    camera: Camera,
) -> Option<(Vec<Point2D>, bool)> {
    use SymbolPrimitive as P;
    let (local, closed) = match primitive {
        P::Line { start, end } => (vec![*start, *end], false),
        P::Polyline { points, closed } => (points.clone(), *closed),
        P::Rectangle { rect } => (rect_points(*rect).to_vec(), true),
        P::Ellipse {
            center,
            radius_x,
            radius_y,
            rotation_degrees,
        } => (
            ellipse_points(
                *center,
                *radius_x,
                *radius_y,
                *rotation_degrees,
                camera.scale(),
            ),
            true,
        ),
        P::Circle { .. } | P::Arc { .. } => {
            let (center, radius, start, sweep) = world_curve(primitive, t)?;
            let pixel_radius = radius * camera.scale();
            let angle = 2.0 * (1.0 - 0.2 / pixel_radius.max(0.2)).clamp(-1.0, 1.0).acos();
            let n = ((sweep.to_radians().abs() / angle.max(1e-8)).ceil() as usize).clamp(8, 2048);
            let points: Vec<Point2D> = (0..=n)
                .filter_map(|i| curve_point(center, radius, start + sweep * i as f64 / n as f64))
                .collect();
            // Already in world space; the last point closes the circle.
            return Some((points, matches!(primitive, P::Circle { .. })));
        }
        _ => return None,
    };
    let world: Option<Vec<Point2D>> = local.iter().map(|p| t.apply(*p).ok()).collect();
    Some((world?, closed))
}

/// Screen segment with a dash pattern measured in logical points.
fn dashed(
    painter: &Painter,
    a: egui::Pos2,
    b: egui::Pos2,
    pattern: &[f32],
    phase: &mut f32,
    stroke: Stroke,
) {
    let length = a.distance(b);
    let total: f32 = pattern.iter().sum();
    if pattern.is_empty() || total <= 0.5 || length <= 0.0 {
        painter.line_segment([a, b], stroke);
        return;
    }
    let dir = (b - a) / length;
    let mut pos = 0.0_f32;
    let mut guard = 0;
    while pos < length && guard < 100_000 {
        guard += 1;
        let cycle = phase.rem_euclid(total);
        let mut acc = 0.0;
        let mut index = 0;
        while index + 1 < pattern.len() && acc + pattern[index] <= cycle {
            acc += pattern[index];
            index += 1;
        }
        let remaining = (acc + pattern[index] - cycle).max(0.05);
        let step = remaining.min(length - pos);
        if index % 2 == 0 {
            painter.line_segment([a + dir * pos, a + dir * (pos + step)], stroke);
        }
        pos += step;
        *phase += step;
    }
}

/// Paint a styled primitive; text supplies dynamic-text values.
#[allow(clippy::too_many_arguments)]
pub fn paint_styled(
    painter: &Painter,
    primitive: &SymbolPrimitive,
    style: PrimitiveStyle,
    t: Transform2D,
    camera: Camera,
    viewport: Rect,
    stroke: Stroke,
    text: &dyn Fn(&SymbolTextField) -> String,
) {
    let stroke = match style.width_mm {
        Some(w) => Stroke::new(
            stroke.width.max((w * camera.scale()) as f32).max(1.0),
            stroke.color,
        ),
        None => stroke,
    };
    match primitive {
        SymbolPrimitive::Text { .. } | SymbolPrimitive::Anchor { .. } => {
            paint_primitive_plain(painter, primitive, t, camera, viewport, stroke);
            return;
        }
        SymbolPrimitive::DynamicText {
            origin,
            field,
            height,
            align,
            rotation_degrees,
        } => {
            let value = text(field);
            if !value.trim().is_empty() {
                paint_text_rotated(
                    painter,
                    t,
                    camera,
                    viewport,
                    (*origin, &value, *height),
                    (*align, *rotation_degrees),
                    stroke.color,
                );
            }
            return;
        }
        _ => {}
    }
    let Some((points, closed)) = outline(primitive, t, camera) else {
        return;
    };
    if style.fill && closed && points.len() > 2 {
        let screen: Option<Vec<egui::Pos2>> = points
            .iter()
            .map(|p| camera.world_to_screen(*p, viewport))
            .collect();
        if let Some(screen) = screen {
            painter.add(egui::Shape::convex_polygon(
                screen,
                stroke.color,
                Stroke::NONE,
            ));
        }
    }
    let scale = camera.scale() as f32;
    let pattern: Vec<f32> = style
        .dash
        .pattern_mm()
        .iter()
        .map(|mm| (*mm as f32 * scale).max(1.5))
        .collect();
    let mut phase = 0.0;
    let segment = |a: Point2D, b: Point2D, phase: &mut f32| {
        if let Some([sa, sb]) = camera.clipped_segment(a, b, viewport) {
            dashed(painter, sa, sb, &pattern, phase, stroke);
        }
    };
    for pair in points.windows(2) {
        segment(pair[0], pair[1], &mut phase);
    }
    if closed
        && !matches!(primitive, SymbolPrimitive::Circle { .. })
        && let (Some(a), Some(b)) = (points.last(), points.first())
    {
        segment(*a, *b, &mut phase);
    }
}

/// Styled symbol; text resolves dynamic text for the occurrence.
#[allow(clippy::too_many_arguments)]
pub fn paint_symbol_with(
    painter: &Painter,
    d: &SymbolDefinition,
    t: Transform2D,
    camera: Camera,
    viewport: Rect,
    stroke: Stroke,
    text: &dyn Fn(&SymbolTextField) -> String,
) {
    let Ok(bounds) = t.transform_bounds(d.bounds()) else {
        return;
    };
    if camera.clipped_rect(bounds, viewport.expand(4.0)).is_none() {
        return;
    }
    for (i, primitive) in d.primitives().iter().enumerate() {
        paint_styled(
            painter,
            primitive,
            d.style(i),
            t,
            camera,
            viewport,
            stroke,
            text,
        );
    }
}

/// Displayed value without an occurrence, such as in an editor, library or preview.
pub fn placeholder(field: &SymbolTextField) -> String {
    field.placeholder()
}

fn paint_text_rotated(
    painter: &Painter,
    t: Transform2D,
    camera: Camera,
    viewport: Rect,
    text_data: (Point2D, &str, f64),
    placement: (TextAlign, f64),
    color: egui::Color32,
) {
    let (origin, text, height) = text_data;
    let (align, degrees) = placement;
    let width = height * 0.6 * text.chars().count() as f64;
    let (fx, fy) = align.fractions();
    let (s, c) = degrees.to_radians().sin_cos();
    let local = move |x: f64, y: f64| {
        let (lx, ly) = (x - fx * width, y - fy * height);
        origin.translated(lx * c - ly * s, lx * s + ly * c)
    };
    paint_text_mapped(
        painter, t, camera, viewport, text, height, width, &local, color,
    );
}
