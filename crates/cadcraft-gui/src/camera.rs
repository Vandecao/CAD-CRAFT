// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Cartesian millimeters to egui logical points. The viewport is always supplied
//! by layout, never calculated from the paper. No domain object is changed here.
use cad_craft::domain::{Point2D, Rect2D};
use eframe::egui::{Pos2, Rect, Vec2, pos2};

pub const MIN_SCALE: f64 = 0.0001;
pub const MAX_SCALE: f64 = 10_000.0;
pub const SCALE_100: f64 = 96.0 / 25.4;
const MAX_CENTER: f64 = 1.0e12;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    center: [f64; 2],
    scale: f64,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0],
            scale: SCALE_100,
        }
    }
}
impl Camera {
    pub fn new(center: [f64; 2], scale: f64) -> Option<Self> {
        (center
            .iter()
            .all(|x| x.is_finite() && x.abs() <= MAX_CENTER)
            && (MIN_SCALE..=MAX_SCALE).contains(&scale))
        .then_some(Self { center, scale })
    }
    pub fn scale(self) -> f64 {
        self.scale
    }
    pub fn percent(self) -> f64 {
        100.0 * self.scale / SCALE_100
    }
    pub fn world_to_screen(self, p: Point2D, viewport: Rect) -> Option<Pos2> {
        let x = viewport.center().x as f64 + (p.x() - self.center[0]) * self.scale;
        let y = viewport.center().y as f64 - (p.y() - self.center[1]) * self.scale;
        // Avoid huge tessellation coordinates even when imported Core values are finite.
        (x.is_finite() && y.is_finite() && x.abs() < 1.0e8 && y.abs() < 1.0e8)
            .then_some(pos2(x as f32, y as f32))
    }
    fn screen_f64(self, p: Point2D, viewport: Rect) -> Option<[f64; 2]> {
        let point = [
            viewport.center().x as f64 + (p.x() - self.center[0]) * self.scale,
            viewport.center().y as f64 - (p.y() - self.center[1]) * self.scale,
        ];
        point.iter().all(|v| v.is_finite()).then_some(point)
    }
    /// Clip before casting to f32. Offscreen corners do not make a large sheet
    /// disappear when it covers the viewport.
    pub fn clipped_rect(self, bounds: Rect2D, viewport: Rect) -> Option<Rect> {
        let a = self.screen_f64(bounds.origin(), viewport)?;
        let b = self.screen_f64(bounds.max(), viewport)?;
        let min = [
            a[0].min(b[0]).max(viewport.left() as f64),
            a[1].min(b[1]).max(viewport.top() as f64),
        ];
        let max = [
            a[0].max(b[0]).min(viewport.right() as f64),
            a[1].max(b[1]).min(viewport.bottom() as f64),
        ];
        (min[0] <= max[0] && min[1] <= max[1]).then(|| {
            Rect::from_min_max(
                pos2(min[0] as f32, min[1] as f32),
                pos2(max[0] as f32, max[1] as f32),
            )
        })
    }
    /// Liang–Barsky clipping in f64, shared by paper and representation edges.
    pub fn clipped_segment(self, a: Point2D, b: Point2D, viewport: Rect) -> Option<[Pos2; 2]> {
        let a = self.screen_f64(a, viewport)?;
        let b = self.screen_f64(b, viewport)?;
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let mut lo: f64 = 0.0;
        let mut hi: f64 = 1.0;
        for (p, q) in [
            (-dx, a[0] - viewport.left() as f64),
            (dx, viewport.right() as f64 - a[0]),
            (-dy, a[1] - viewport.top() as f64),
            (dy, viewport.bottom() as f64 - a[1]),
        ] {
            if p == 0.0 {
                if q < 0.0 {
                    return None;
                }
            } else {
                let t = q / p;
                if p < 0.0 {
                    lo = lo.max(t);
                } else {
                    hi = hi.min(t);
                }
                if lo > hi {
                    return None;
                }
            }
        }
        Some([
            pos2((a[0] + lo * dx) as f32, (a[1] + lo * dy) as f32),
            pos2((a[0] + hi * dx) as f32, (a[1] + hi * dy) as f32),
        ])
    }
    pub fn screen_to_world(self, p: Pos2, viewport: Rect) -> Option<Point2D> {
        Point2D::new(
            self.center[0] + (p.x - viewport.center().x) as f64 / self.scale,
            self.center[1] - (p.y - viewport.center().y) as f64 / self.scale,
        )
        .ok()
    }
    pub fn zoom_at(&mut self, factor: f64, cursor: Pos2, viewport: Rect) -> bool {
        if !factor.is_finite() || factor <= 0.0 || !viewport.is_finite() {
            return false;
        }
        let Some(anchor) = self.screen_to_world(cursor, viewport) else {
            return false;
        };
        let scale = (self.scale * factor).clamp(MIN_SCALE, MAX_SCALE);
        let center = [
            anchor.x() - (cursor.x - viewport.center().x) as f64 / scale,
            anchor.y() + (cursor.y - viewport.center().y) as f64 / scale,
        ];
        if let Some(next) = Self::new(center, scale) {
            *self = next;
            true
        } else {
            false
        }
    }
    pub fn pan(&mut self, delta: Vec2) {
        let center = [
            self.center[0] - delta.x as f64 / self.scale,
            self.center[1] + delta.y as f64 / self.scale,
        ];
        if let Some(next) = Self::new(center, self.scale) {
            *self = next;
        }
    }
    pub fn fit(&mut self, bounds: Rect2D, viewport: Rect) -> bool {
        if !viewport.is_finite() || viewport.width() <= 0.0 || viewport.height() <= 0.0 {
            return false;
        }
        let size = bounds.size();
        let origin = bounds.origin();
        let scale = ((viewport.width() as f64 * 0.88) / size.width().max(10.0))
            .min(viewport.height() as f64 * 0.88 / size.height().max(10.0));
        // Do not report a successful fit if the required zoom is unsupported.
        if scale < MIN_SCALE {
            return false;
        }
        let center = [
            origin.x() + size.width() / 2.0,
            origin.y() + size.height() / 2.0,
        ];
        if let Some(next) = Self::new(center, scale.min(MAX_SCALE)) {
            *self = next;
            true
        } else {
            false
        }
    }
}

pub fn snap(p: Point2D, spacing: f64, enabled: bool) -> Point2D {
    if !enabled || !spacing.is_finite() || spacing <= 0.0 {
        return p;
    }
    Point2D::new(
        (p.x() / spacing).round() * spacing,
        (p.y() / spacing).round() * spacing,
    )
    .unwrap_or(p)
}

/// Grid dots closer than this on screen are not drawn: the grid hides when
/// zoomed far out, as in EPLAN, instead of spreading coarser dots.
pub const MIN_GRID_PX: f64 = 6.0;

/// Visible document intersections at the real spacing, or none when they
/// would be closer than [`MIN_GRID_PX`] or exceed the point budget. Snap
/// keeps the logical spacing either way.
pub fn grid_points(camera: &Camera, viewport: Rect, spacing: f64) -> Vec<Pos2> {
    const MAX_POINTS: f64 = 60_000.0;
    if !viewport.is_finite() || !viewport.is_positive() || !spacing.is_finite() || spacing <= 0.0 {
        return vec![];
    }
    if spacing * camera.scale() < MIN_GRID_PX {
        return vec![];
    }
    let (Some(low), Some(high)) = (
        camera.screen_to_world(viewport.left_bottom(), viewport),
        camera.screen_to_world(viewport.right_top(), viewport),
    ) else {
        return vec![];
    };
    let first = [(low.x() / spacing).ceil(), (low.y() / spacing).ceil()];
    let last = [(high.x() / spacing).floor(), (high.y() / spacing).floor()];
    let count = [
        (last[0] - first[0] + 1.0).max(0.0),
        (last[1] - first[1] + 1.0).max(0.0),
    ];
    if count.contains(&0.0)
        || count[0] * count[1] > MAX_POINTS
        || !first
            .iter()
            .chain(last.iter())
            .all(|n| n.is_finite() && n.abs() < (1_u64 << 52) as f64)
    {
        return vec![];
    }
    let mut points = Vec::with_capacity((count[0] * count[1]) as usize);
    for y in 0..count[1] as usize {
        for x in 0..count[0] as usize {
            if let Ok(world) = Point2D::new(
                (first[0] + x as f64) * spacing,
                (first[1] + y as f64) * spacing,
            ) && let Some(screen) = camera.world_to_screen(world, viewport)
                && viewport.contains(screen)
            {
                points.push(screen);
            }
        }
    }
    points
}

/// At most 2 * 1024 lines, aligned to integer multiples of the actual grid.
pub fn grid_lines(camera: &Camera, viewport: Rect, spacing: f64) -> Vec<(Pos2, Pos2)> {
    if !viewport.is_finite() || !spacing.is_finite() || spacing <= 0.0 {
        return vec![];
    }
    let Some(a) = camera.screen_to_world(viewport.left_bottom(), viewport) else {
        return vec![];
    };
    let Some(b) = camera.screen_to_world(viewport.right_top(), viewport) else {
        return vec![];
    };
    let multiple = (8.0 / (spacing * camera.scale()))
        .max(1.0)
        .log2()
        .ceil()
        .exp2();
    let step = spacing * multiple;
    let mut lines = Vec::new();
    for (low, high, vertical) in [(a.x(), b.x(), true), (a.y(), b.y(), false)] {
        let first = (low / step).ceil() * step;
        let count = ((high - first) / step).floor().max(0.0) as usize;
        for i in 0..=count.min(1022) {
            let value = first + i as f64 * step;
            let point = if vertical {
                Point2D::new(value, a.y())
            } else {
                Point2D::new(a.x(), value)
            };
            if let Ok(point) = point
                && let Some(p) = camera.world_to_screen(point, viewport)
            {
                lines.push(if vertical {
                    (pos2(p.x, viewport.top()), pos2(p.x, viewport.bottom()))
                } else {
                    (pos2(viewport.left(), p.y), pos2(viewport.right(), p.y))
                });
            }
        }
    }
    lines
}
