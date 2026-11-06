// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! DXF entity writing; start with Writer path, text and image methods.

use super::*;

impl Writer {
    pub(super) fn point(&mut self, base: i32, x: f64, y: f64) {
        self.pair(base, &num(x));
        self.pair(base + 10, &num(y));
        self.pair(base + 20, "0");
    }
    pub(super) fn common(&mut self, layer: &str, linetype: Option<&str>, color: Option<u8>) {
        self.pair(8, layer);
        if let Some(lt) = linetype {
            self.pair(6, lt);
        }
        if let Some(c) = color {
            self.pair(62, &c.to_string());
        }
    }
    pub(super) fn linetype(&mut self, dash: &[f64]) -> Option<String> {
        if dash.is_empty() {
            return None;
        }
        let key: Vec<i64> = dash.iter().map(|d| (d * 1000.0).round() as i64).collect();
        let next = self.dashes.len() + 1;
        Some(
            self.dashes
                .entry(key)
                .or_insert_with(|| format!("CAD-CRAFT-TRACO-{next}"))
                .clone(),
        )
    }

    pub(super) fn path(
        &mut self,
        points: &[Point2D],
        closed: bool,
        stroke: Option<&cadcraft_render::display::DisplayStroke>,
        fill: Option<DisplayColor>,
        layer: DisplayLayer,
    ) {
        if let Some(color) = fill
            && points.len() > 2
        {
            let name = layer_name(layer, None);
            self.layers.insert((layer, None));
            // Leque a partir do primeiro ponto (formas convexas da folha).
            for i in 1..points.len() - 1 {
                let (a, b, c) = (points[0], points[i], points[i + 1]);
                self.pair(0, "SOLID");
                self.common(&name, None, entity_color(color));
                self.point(10, a.x(), a.y());
                self.point(11, b.x(), b.y());
                self.point(12, c.x(), c.y());
                self.point(13, c.x(), c.y());
                self.report.entities += 1;
            }
        }
        let Some(stroke) = stroke else {
            return;
        };
        let key = width_key(stroke.width);
        let name = layer_name(layer, Some(stroke.width));
        self.layers.insert((layer, Some(key)));
        let linetype = self.linetype(&stroke.dash);
        let color = entity_color(stroke.color);
        if points.len() == 2 && !closed {
            self.pair(0, "LINE");
            self.common(&name, linetype.as_deref(), color);
            self.point(10, points[0].x(), points[0].y());
            self.point(11, points[1].x(), points[1].y());
        } else {
            self.pair(0, "POLYLINE");
            self.common(&name, linetype.as_deref(), color);
            self.pair(66, "1");
            self.point(10, 0.0, 0.0);
            self.pair(70, if closed { "1" } else { "0" });
            for p in points {
                self.pair(0, "VERTEX");
                self.pair(8, &name);
                self.point(10, p.x(), p.y());
            }
            self.pair(0, "SEQEND");
            self.pair(8, &name);
        }
        self.report.entities += 1;
    }

    pub(super) fn text(&mut self, text: &DisplayText) {
        let [a, b, c, d, _, _] = text.matrix;
        let (ux, uy) = (a, b);
        let v_len = c.hypot(d);
        let u_len = ux.hypot(uy);
        if v_len <= 0.0 || u_len <= 0.0 {
            return;
        }
        let natural = text.natural_width();
        let stretch = match text.width {
            Some(w) if natural > 0.0 => w / natural,
            _ => 1.0,
        };
        // Altura de maiúscula; fator de largura relativo à altura.
        let height = text.size * font::CAP_HEIGHT * v_len;
        let factor = u_len * stretch / v_len;
        let mirrored = a * d - b * c < 0.0;
        let rotation = uy.atan2(ux).to_degrees();
        let (x, y) = text.map(0.0, text.baseline);
        let bytes = encode_text(&text.text, &mut self.report.escaped);
        let name = layer_name(text.layer, None);
        self.layers.insert((text.layer, None));
        self.pair(0, "TEXT");
        self.common(&name, None, entity_color(text.color));
        self.point(10, x, y);
        self.pair(40, &num(height));
        self.pair_bytes(1, &bytes);
        if rotation.abs() > 1e-9 {
            self.pair(50, &num(rotation));
        }
        if (factor - 1.0).abs() > 1e-6 {
            self.pair(41, &num(factor));
        }
        if mirrored {
            self.pair(71, "2");
        }
        self.report.entities += 1;
    }

    pub(super) fn image(&mut self, rect: Rect2D) {
        let (o, m) = (rect.origin(), rect.max());
        let pts = [
            (o.x(), o.y()),
            (m.x(), o.y()),
            (m.x(), m.y()),
            (o.x(), m.y()),
        ]
        .map(|(x, y)| Point2D::new(x, y).expect("retângulo válido"));
        let guide = cadcraft_render::display::DisplayStroke {
            width: 0.18,
            dash: vec![],
            color: DisplayColor::Guide,
        };
        self.path(&pts, true, Some(&guide), None, DisplayLayer::Guides);
        self.path(
            &[pts[0], pts[2]],
            false,
            Some(&guide),
            None,
            DisplayLayer::Guides,
        );
        self.path(
            &[pts[1], pts[3]],
            false,
            Some(&guide),
            None,
            DisplayLayer::Guides,
        );
        self.report.images += 1;
    }
}
