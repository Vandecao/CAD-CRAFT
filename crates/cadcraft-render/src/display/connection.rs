// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Connection labels, references and junctions; start with wire_number_placement.

use super::*;

impl Builder<'_> {
    pub(super) fn references(&mut self, page: PageId) {
        self.layer = DisplayLayer::References;
        let style = FrameTextStyle::new(SHEET_REFERENCE_HEIGHT, TextAlign::TopLeft);
        let line = SHEET_REFERENCE_HEIGHT * LINE_HEIGHT;
        for reference in self.project.sheet_references(page) {
            let text = reference.text();
            self.text(reference.origin, &text, style, DisplayColor::Reference);
            let mut x = 0.0;
            for segment in &reference.segments {
                let w = font::text_width(&segment.text, SHEET_REFERENCE_HEIGHT);
                if let Some(target) = segment.target
                    && w > 0.0
                    && let (Ok(origin), Ok(size)) =
                        (reference.origin.translated(x, -line), Size2D::new(w, line))
                    && let Ok(rect) = Rect2D::new(origin, size)
                {
                    self.items.push(DisplayItem::Link { rect, target });
                }
                x += w;
            }
        }
    }

    pub(super) fn junctions(&mut self, page: PageId) {
        self.layer = DisplayLayer::Junctions;
        let project = self.project;
        for j in project
            .connection_junctions()
            .filter(|j| j.page_id() == page)
        {
            let mut points = arc_points(j.position(), JUNCTION_RADIUS, 0.0, 360.0);
            points.pop();
            self.path(points, true, None, Some(DisplayColor::Ink));
        }
    }
}

/// Wire number offset from the stroke in mm.
pub const WIRE_NUMBER_GAP: f64 = 0.8;

/// Wire number position: stored route position with text to its right,
/// or the midpoint of the longest segment. Place text above horizontal
/// segments and right of vertical segments to keep it off the stroke.
pub fn wire_number_placement(
    points: &[Point2D],
    explicit: Option<Point2D>,
) -> Option<(Point2D, TextAlign)> {
    if let Some(at) = explicit {
        return Some((at, TextAlign::MiddleLeft));
    }
    let pair = points.windows(2).max_by(|a, b| {
        let len = |p: &[Point2D]| (p[1].x() - p[0].x()).hypot(p[1].y() - p[0].y());
        len(a).total_cmp(&len(b))
    })?;
    let (dx, dy) = (pair[1].x() - pair[0].x(), pair[1].y() - pair[0].y());
    let (mx, my) = (
        (pair[0].x() + pair[1].x()) / 2.0,
        (pair[0].y() + pair[1].y()) / 2.0,
    );
    if dx.abs() >= dy.abs() {
        Point2D::new(mx, my + WIRE_NUMBER_GAP)
            .ok()
            .map(|p| (p, TextAlign::BottomCenter))
    } else {
        Point2D::new(mx + WIRE_NUMBER_GAP, my)
            .ok()
            .map(|p| (p, TextAlign::MiddleLeft))
    }
}

/// Stroke width of single-line links in mm.
pub const SINGLE_LINE_WIDTH: f64 = 0.5;

impl Builder<'_> {
    /// Single-line links: polyline, phase strokes and, above four phases,
    /// the phase count. Documentation only; no electrical meaning.
    pub(super) fn single_lines(&mut self, page: PageId) {
        self.layer = DisplayLayer::Wires;
        let project = self.project;
        for link in project.single_lines().filter(|l| l.page_id() == page) {
            self.stroke(
                link.points().to_vec(),
                false,
                SINGLE_LINE_WIDTH,
                DisplayColor::Ink,
            );
            let (marks, label) = link.phase_marks();
            for [a, b] in marks {
                self.stroke(vec![a, b], false, ZONE_MARK_WIDTH, DisplayColor::Ink);
            }
            if let Some(at) = label {
                self.text(
                    at,
                    &link.phases().to_string(),
                    FrameTextStyle::new(2.5, TextAlign::Center),
                    DisplayColor::Ink,
                );
            }
        }
    }
}
