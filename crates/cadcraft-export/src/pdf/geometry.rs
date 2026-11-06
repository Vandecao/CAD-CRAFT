// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! PDF placement and paint helpers; start with Placement::new.

use super::*;

pub(super) fn compose(m1: Affine, m2: Affine) -> Affine {
    [
        m1[0] * m2[0] + m1[2] * m2[1],
        m1[1] * m2[0] + m1[3] * m2[1],
        m1[0] * m2[2] + m1[2] * m2[3],
        m1[1] * m2[2] + m1[3] * m2[3],
        m1[0] * m2[4] + m1[2] * m2[5] + m1[4],
        m1[1] * m2[4] + m1[3] * m2[5] + m1[5],
    ]
}
pub(super) fn f32s(m: Affine) -> [f32; 6] {
    m.map(|v| v as f32)
}
impl Placement {
    pub(super) fn new(width: f64, height: f64, paper: Option<(Size2D, PaperFit)>) -> Self {
        let Some((paper, fit)) = paper else {
            return Self {
                media: (width * PT_PER_MM, height * PT_PER_MM),
                matrix: [PT_PER_MM, 0.0, 0.0, PT_PER_MM, 0.0, 0.0],
            };
        };
        // Papel na mesma orientação da folha.
        let (mut pw, mut ph) = (paper.width(), paper.height());
        if (width > height) != (pw > ph) {
            std::mem::swap(&mut pw, &mut ph);
        }
        let scale = match fit {
            PaperFit::Fit => (pw / width).min(ph / height),
            PaperFit::Actual => 1.0,
        };
        let (ox, oy) = ((pw - width * scale) / 2.0, (ph - height * scale) / 2.0);
        let k = scale * PT_PER_MM;
        Self {
            media: (pw * PT_PER_MM, ph * PT_PER_MM),
            matrix: [k, 0.0, 0.0, k, ox * PT_PER_MM, oy * PT_PER_MM],
        }
    }
    pub(super) fn point(&self, x: f64, y: f64) -> (f32, f32) {
        let m = self.matrix;
        ((m[0] * x + m[4]) as f32, (m[3] * y + m[5]) as f32)
    }
    pub(super) fn length(&self, mm: f64) -> f32 {
        (mm * self.matrix[0]) as f32
    }
    pub(super) fn rect(&self, r: Rect2D) -> Rect {
        let (x1, y1) = self.point(r.origin().x(), r.origin().y());
        let (x2, y2) = self.point(r.max().x(), r.max().y());
        Rect::new(x1, y1, x2, y2)
    }
}

/// Valores de cor (cinza ou RGB) da cor lógica.
enum Paint {
    Gray(f32),
    Rgb(f32, f32, f32),
}
fn paint(color: DisplayColor, monochrome: bool) -> Paint {
    match (color, monochrome) {
        (DisplayColor::Ink, _) | (DisplayColor::Reference, true) => Paint::Gray(0.0),
        (DisplayColor::Reference, false) => Paint::Rgb(0.0, 0.36, 0.72),
        (DisplayColor::Watermark, _) => Paint::Gray(0.8),
        (DisplayColor::Guide, _) => Paint::Gray(0.75),
    }
}
pub(super) fn set_stroke(content: &mut Content, color: DisplayColor, monochrome: bool) {
    match paint(color, monochrome) {
        Paint::Gray(g) => content.set_stroke_gray(g),
        Paint::Rgb(r, g, b) => content.set_stroke_rgb(r, g, b),
    };
}
pub(super) fn set_fill(content: &mut Content, color: DisplayColor, monochrome: bool) {
    match paint(color, monochrome) {
        Paint::Gray(g) => content.set_fill_gray(g),
        Paint::Rgb(r, g, b) => content.set_fill_rgb(r, g, b),
    };
}
