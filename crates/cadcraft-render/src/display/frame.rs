// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Frame drawing and zone labels; start with frame_zone_labels, then Builder::frame.

use super::*;

/// Zone label positions in mm: centered in the strip of at most 5 mm between
/// the usable area and sheet border on each enabled side.
pub fn frame_zone_labels(frame: &FrameTemplate) -> Vec<(String, Point2D)> {
    let u = frame.usable();
    let (min, max) = (u.origin(), u.max());
    let sheet = frame.sheet();
    let band = |space: f64| space.clamp(0.0, 5.0) / 2.0;
    let pt = |x, y| Point2D::new(x, y).ok();
    let mut out = vec![];
    let columns = &frame.zones().columns;
    for (i, (a, b)) in columns.bounds().into_iter().enumerate() {
        let x = min.x() + (a + b) / 2.0;
        let label = columns.style.label(i);
        if columns.labels_start
            && let Some(p) = pt(x, max.y() + band(sheet.height() - max.y()))
        {
            out.push((label.clone(), p));
        }
        if columns.labels_end
            && let Some(p) = pt(x, min.y() - band(min.y()))
        {
            out.push((label, p));
        }
    }
    let rows = &frame.zones().rows;
    for (i, (a, b)) in rows.bounds().into_iter().enumerate() {
        let y = max.y() - (a + b) / 2.0;
        let label = rows.style.label(i);
        if rows.labels_start
            && let Some(p) = pt(min.x() - band(min.x()), y)
        {
            out.push((label.clone(), p));
        }
        if rows.labels_end
            && let Some(p) = pt(max.x() + band(sheet.width() - max.x()), y)
        {
            out.push((label, p));
        }
    }
    out
}

impl Builder<'_> {
    pub(super) fn frame(&mut self, page: PageId, frame: &FrameTemplate) {
        self.layer = DisplayLayer::Frame;
        let watermark = |e: &&FrameElement| {
            matches!(
                e,
                FrameElement::Text { style, .. } | FrameElement::Special { style, .. } if style.watermark
            )
        };
        let (marks, rest): (Vec<_>, Vec<_>) = frame.elements().iter().partition(watermark);
        for element in marks.into_iter().chain(rest) {
            let ink = DisplayColor::Ink;
            match element {
                FrameElement::Line { start, end, width } => {
                    self.stroke(vec![*start, *end], false, *width, ink);
                }
                FrameElement::Rectangle { rect, width } => {
                    self.stroke(rect_corners(*rect), true, *width, ink);
                }
                FrameElement::Circle {
                    center,
                    radius,
                    width,
                } => {
                    let mut points = arc_points(*center, *radius, 0.0, 360.0);
                    points.pop();
                    self.stroke(points, true, *width, ink);
                }
                FrameElement::Arc {
                    center,
                    radius,
                    start_degrees,
                    sweep_degrees,
                    width,
                } => self.stroke(
                    arc_points(*center, *radius, *start_degrees, *sweep_degrees),
                    false,
                    *width,
                    ink,
                ),
                FrameElement::Text {
                    origin,
                    text,
                    style,
                } => {
                    let color = if style.watermark {
                        DisplayColor::Watermark
                    } else {
                        ink
                    };
                    self.text(*origin, text.get(None), *style, color);
                }
                FrameElement::Special {
                    origin,
                    field,
                    index,
                    style,
                } => {
                    let color = if style.watermark {
                        DisplayColor::Watermark
                    } else {
                        ink
                    };
                    let value = self.project.resolve_special(page, field, *index);
                    self.text(*origin, &value, *style, color);
                }
                FrameElement::Image { rect, asset } => self.items.push(DisplayItem::Image {
                    asset: *asset,
                    rect: *rect,
                    layer: DisplayLayer::Frame,
                }),
            }
        }
        // Divisions follow the zones, like the labels (frames in format 10).
        for (a, b) in frame.zone_marks() {
            self.stroke(vec![a, b], false, ZONE_MARK_WIDTH, DisplayColor::Ink);
        }
        for (label, origin) in frame_zone_labels(frame) {
            self.text(
                origin,
                &label,
                FrameTextStyle::new(LABEL_HEIGHT, TextAlign::Center),
                DisplayColor::Ink,
            );
        }
    }

    /// Fallback margin and header for a page without a frame, matching the screen.
    pub(super) fn provisional_frame(&mut self, page: &Page) {
        self.layer = DisplayLayer::Guides;
        let (w, h) = (page.size().width(), page.size().height());
        let margin = 10.0_f64.min(w * 0.1).min(h * 0.1);
        if let (Ok(origin), Ok(size)) = (
            Point2D::new(margin, margin),
            Size2D::new(w - 2.0 * margin, h - 2.0 * margin),
        ) && let Ok(inner) = Rect2D::new(origin, size)
        {
            self.stroke(rect_corners(inner), true, 0.18, DisplayColor::Guide);
        }
        self.layer = DisplayLayer::Frame;
        if let Ok(origin) = Point2D::new(3.0, h - 3.0) {
            let header = format!(
                "{} · {}  |  {} × {} mm",
                page.number().as_str(),
                page.name().as_str(),
                w,
                h
            );
            self.text(
                origin,
                &header,
                FrameTextStyle::new(3.0, TextAlign::TopLeft),
                DisplayColor::Ink,
            );
        }
    }
}
