// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Zone division marks generated from the zones; start with FrameTemplate::zone_marks.
//!
//! Each zone boundary gets a short mark across the label strip on every side
//! where that axis shows labels, so adding or removing a column redraws the
//! divisions, like the labels already were. Older frames drew these marks as
//! fixed lines; adopt_generated_zone_marks converts them when a file is read.
use super::*;

/// Line width of generated division marks, the width of the former fixed lines.
pub const ZONE_MARK_WIDTH: f64 = 0.25;
/// Widest label strip between the usable area and the sheet edge, as for labels.
const STRIP: f64 = 5.0;

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
    /// Division marks as segments in sheet millimetres. Empty unless the
    /// frame generates its marks (`ZoneLayout::generated_marks`).
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
        let strip = |space: f64| space.clamp(0.0, STRIP);
        let seg = |a: (f64, f64), b: (f64, f64)| {
            Some((Point2D::new(a.0, a.1).ok()?, Point2D::new(b.0, b.1).ok()?))
        };
        let mut out = vec![];
        let columns = &self.zones.columns;
        for offset in columns.mark_offsets(u.size().width()) {
            let x = min.x() + offset;
            let top = strip(self.sheet.height() - max.y());
            let bottom = strip(min.y());
            if columns.labels_start && top > EPS {
                out.extend(seg((x, max.y()), (x, max.y() + top)));
            }
            if columns.labels_end && bottom > EPS {
                out.extend(seg((x, min.y() - bottom), (x, min.y())));
            }
        }
        let rows = &self.zones.rows;
        for offset in rows.mark_offsets(u.size().height()) {
            let y = max.y() - offset;
            let left = strip(min.x());
            let right = strip(self.sheet.width() - max.x());
            if rows.labels_start && left > EPS {
                out.extend(seg((min.x() - left, y), (min.x(), y)));
            }
            if rows.labels_end && right > EPS {
                out.extend(seg((max.x(), y), (max.x() + right, y)));
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
