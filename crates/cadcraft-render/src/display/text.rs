// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Display text placement; start with Builder::text and mapped_text.

use super::*;

impl Builder<'_> {
    /// Frame/sheet text: font size `style.height`, nine-point alignment
    /// in a `width × 1.2·size` box, rotated counterclockwise about its origin.
    pub(super) fn text(
        &mut self,
        origin: Point2D,
        text: &str,
        style: FrameTextStyle,
        color: DisplayColor,
    ) {
        if text.trim().is_empty() || style.height <= 0.0 {
            return;
        }
        let size = style.height;
        let (w, h) = (font::text_width(text, size), size * LINE_HEIGHT);
        let (fx, fy) = style.align.fractions();
        let (s, c) = style.rotation_degrees.to_radians().sin_cos();
        let (ox, oy) = (-fx * w, -fy * h);
        self.items.push(DisplayItem::Text(DisplayText {
            text: text.to_owned(),
            matrix: [
                c,
                s,
                -s,
                c,
                origin.x() + c * ox - s * oy,
                origin.y() + s * ox + c * oy,
            ],
            size,
            baseline: size * BASELINE,
            width: None,
            color,
            layer: self.layer,
        }));
    }

    /// Symbol text: nominal `[0, width] × [0, height]` box mapped onto the
    /// page by affine `map`; capital letters fill the height, as on screen.
    pub(super) fn mapped_text(
        &mut self,
        text: &str,
        (height, width): (f64, f64),
        map: &dyn Fn(f64, f64) -> Option<Point2D>,
        color: DisplayColor,
    ) {
        if text.trim().is_empty() || height <= 0.0 || width <= 0.0 {
            return;
        }
        let (Some(p0), Some(px), Some(py)) = (map(0.0, 0.0), map(1.0, 0.0), map(0.0, 1.0)) else {
            return;
        };
        self.items.push(DisplayItem::Text(DisplayText {
            text: text.to_owned(),
            matrix: [
                px.x() - p0.x(),
                px.y() - p0.y(),
                py.x() - p0.x(),
                py.y() - p0.y(),
                p0.x(),
                p0.y(),
            ],
            size: height / font::CAP_HEIGHT,
            baseline: 0.0,
            width: Some(width),
            color,
            layer: self.layer,
        }));
    }
}
