// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Zone division marks and label strips generated from the zones; start
//! with FrameTemplate::zone_strip, then FrameTemplate::zone_marks.
//!
//! Each side that shows labels has a strip between the usable area and the
//! frame border (the rectangle around the usable area, or the sheet edge).
//! When the usable area already touches the border, the strip goes inside
//! the frame, with a separator line, so labels never fall outside it. Zone
//! boundaries get a short mark across the strip, so adding or removing a
//! column redraws the divisions. Older frames drew these marks as fixed
//! lines; adopt_generated_zone_marks converts them when a file is read.
use super::*;

/// Line width of generated division marks, the width of the former fixed lines.
pub const ZONE_MARK_WIDTH: f64 = 0.25;
/// Widest label strip.
const STRIP: f64 = 5.0;

/// A side of the usable area where zone labels can be shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoneSide {
    /// Above the usable area (column labels at the start).
    Top,
    /// Below the usable area (column labels at the end).
    Bottom,
    /// Left of the usable area (row labels at the start).
    Left,
    /// Right of the usable area (row labels at the end).
    Right,
}

/// Label strip of one side, as coordinates across it: from the usable-area
/// edge to the far side (Y for top and bottom, X for left and right).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZoneStrip {
    /// The usable-area edge.
    pub edge: f64,
    /// The other side of the strip.
    pub far: f64,
    /// The strip lies inside the usable area because the area touches the
    /// frame border; a separator line is drawn at `far`.
    pub inward: bool,
}

impl ZoneStrip {
    /// Where labels are centred across the strip.
    pub fn middle(&self) -> f64 {
        (self.edge + self.far) / 2.0
    }
}

impl ZoneAxis {
    /// Offsets of the marks along the axis: every zone end except the one on
    /// the usable-area edge (the frame border is already there).
    fn mark_offsets(&self, available: f64) -> Vec<f64> {
        let mut end = 0.0;
        let mut out = vec![];
        for size in &self.sizes {
            end += size;
            if end < available - EPS {
                out.push(end);
            }
        }
        out
    }
}

impl FrameTemplate {
    /// The frame border: the smallest rectangle element around the usable
    /// area, or the sheet when there is none.
    fn border(&self) -> (Point2D, Point2D) {
        let u = self.usable;
        let (min, max) = (u.origin(), u.max());
        let around = |r: &Rect2D| {
            r.origin().x() <= min.x() + EPS
                && r.origin().y() <= min.y() + EPS
                && r.max().x() >= max.x() - EPS
                && r.max().y() >= max.y() - EPS
        };
        let area = |r: &Rect2D| r.size().width() * r.size().height();
        self.elements
            .iter()
            .filter_map(|e| match e {
                FrameElement::Rectangle { rect, .. } if around(rect) => Some(*rect),
                _ => None,
            })
            .min_by(|a, b| area(a).total_cmp(&area(b)))
            .map(|r| (r.origin(), r.max()))
            .unwrap_or_else(|| {
                (
                    Point2D::new(0.0, 0.0).expect("origin"),
                    Point2D::new(self.sheet.width(), self.sheet.height()).expect("sheet corner"),
                )
            })
    }

    /// Label strip of one side; see [`ZoneStrip`].
    pub fn zone_strip(&self, side: ZoneSide) -> ZoneStrip {
        let u = self.usable;
        let (min, max) = (u.origin(), u.max());
        let (low, high) = self.border();
        // Sign: +1 when the outside of the usable area is towards larger values.
        let (edge, limit, sign) = match side {
            ZoneSide::Top => (max.y(), high.y(), 1.0),
            ZoneSide::Bottom => (min.y(), low.y(), -1.0),
            ZoneSide::Left => (min.x(), low.x(), -1.0),
            ZoneSide::Right => (max.x(), high.x(), 1.0),
        };
        let gap = (limit - edge) * sign;
        if gap > EPS {
            ZoneStrip {
                edge,
                far: edge + sign * gap.min(STRIP),
                inward: false,
            }
        } else {
            ZoneStrip {
                edge,
                far: edge - sign * STRIP,
                inward: true,
            }
        }
    }

    /// Division marks (and separators of inward strips) as segments in sheet
    /// millimetres. Empty unless the frame generates its marks
    /// (`ZoneLayout::generated_marks`).
    pub fn zone_marks(&self) -> Vec<(Point2D, Point2D)> {
        if self.zones.generated_marks {
            self.marks_for_zones()
        } else {
            vec![]
        }
    }

    fn marks_for_zones(&self) -> Vec<(Point2D, Point2D)> {
        let u = self.usable;
        let (min, max) = (u.origin(), u.max());
        let seg = |a: (f64, f64), b: (f64, f64)| {
            Some((Point2D::new(a.0, a.1).ok()?, Point2D::new(b.0, b.1).ok()?))
        };
        // Marks run from the lower to the higher coordinate.
        let ordered = |s: ZoneStrip| (s.edge.min(s.far), s.edge.max(s.far));
        let mut out = vec![];
        let columns = &self.zones.columns;
        let rows = &self.zones.rows;
        let sides = [
            (ZoneSide::Top, columns, columns.labels_start),
            (ZoneSide::Bottom, columns, columns.labels_end),
            (ZoneSide::Left, rows, rows.labels_start),
            (ZoneSide::Right, rows, rows.labels_end),
        ];
        for (side, axis, shown) in sides {
            if !shown || axis.sizes.is_empty() {
                continue;
            }
            let strip = self.zone_strip(side);
            if matches!(side, ZoneSide::Top | ZoneSide::Bottom) {
                for offset in axis.mark_offsets(u.size().width()) {
                    let x = min.x() + offset;
                    let (lo, hi) = ordered(strip);
                    out.extend(seg((x, lo), (x, hi)));
                }
                if strip.inward {
                    out.extend(seg((min.x(), strip.far), (max.x(), strip.far)));
                }
            } else {
                for offset in axis.mark_offsets(u.size().height()) {
                    let y = max.y() - offset;
                    let (lo, hi) = ordered(strip);
                    out.extend(seg((lo, y), (hi, y)));
                }
                if strip.inward {
                    out.extend(seg((strip.far, min.y()), (strip.far, max.y())));
                }
            }
        }
        out
    }

    /// Converts fixed division lines into generated marks: removes the
    /// `Line` elements that coincide with a zone mark and turns generation
    /// on. Returns `false`, changing nothing, when no line coincides (the
    /// frame keeps its exact drawing).
    pub fn adopt_generated_zone_marks(&mut self) -> bool {
        if self.zones.generated_marks {
            return false;
        }
        let marks = self.marks_for_zones();
        let same =
            |a: Point2D, b: Point2D| (a.x() - b.x()).abs() <= EPS && (a.y() - b.y()).abs() <= EPS;
        let is_mark = |e: &FrameElement| match e {
            FrameElement::Line { start, end, .. } => marks.iter().any(|(a, b)| {
                (same(*start, *a) && same(*end, *b)) || (same(*start, *b) && same(*end, *a))
            }),
            _ => false,
        };
        let before = self.elements.len();
        self.elements.retain(|e| !is_mark(e));
        if self.elements.len() == before {
            return false;
        }
        self.zones.generated_marks = true;
        true
    }
}
