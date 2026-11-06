// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! PDF text encoding and transforms; start with Writer::text.

use super::*;

impl Writer<'_> {
    pub(super) fn text(&mut self, content: &mut Content, text: &DisplayText, place: &Placement) {
        let (bytes, missing) = font::encode(&text.text);
        self.report.replaced.extend(missing);
        let natural = text.natural_width();
        let stretch = match text.width {
            Some(w) if natural > 0.0 => w / natural,
            _ => 1.0,
        };
        // Text space (font size 1) → box → sheet mm → paper points.
        let to_box = [text.size * stretch, 0.0, 0.0, text.size, 0.0, text.baseline];
        let matrix = compose(place.matrix, compose(text.matrix, to_box));
        set_fill(content, text.color, self.options.monochrome);
        content.begin_text();
        content.set_font(FONT, 1.0);
        content.set_text_matrix(f32s(matrix));
        content.show(Str(&bytes));
        content.end_text();
    }
}
