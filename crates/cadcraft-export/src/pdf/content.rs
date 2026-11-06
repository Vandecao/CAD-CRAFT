// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! PDF page drawing commands; start with Writer::content.

use super::*;

impl Writer<'_> {
    /// Conteúdo da página; devolve as imagens usadas (nome, referência).
    pub(super) fn content(
        &mut self,
        list: &DisplayList,
        place: &Placement,
    ) -> (Vec<u8>, Vec<(String, Ref)>) {
        let mono = self.options.monochrome;
        let mut content = Content::new();
        content.set_line_cap(LineCapStyle::RoundCap);
        content.set_line_join(LineJoinStyle::RoundJoin);
        let mut used = vec![];
        for item in &list.items {
            match item {
                DisplayItem::Path {
                    points,
                    closed,
                    stroke,
                    fill,
                    ..
                } => {
                    let (x, y) = place.point(points[0].x(), points[0].y());
                    content.move_to(x, y);
                    for p in &points[1..] {
                        let (x, y) = place.point(p.x(), p.y());
                        content.line_to(x, y);
                    }
                    if *closed {
                        content.close_path();
                    }
                    if let Some(color) = fill {
                        set_fill(&mut content, *color, mono);
                    }
                    if let Some(s) = stroke {
                        set_stroke(&mut content, s.color, mono);
                        content.set_line_width(place.length(s.width).max(0.1));
                        content.set_dash_pattern(s.dash.iter().map(|d| place.length(*d)), 0.0);
                    }
                    match (fill.is_some(), stroke.is_some()) {
                        (true, true) => content.fill_nonzero_and_stroke(),
                        (true, false) => content.fill_nonzero(),
                        _ => content.stroke(),
                    };
                }
                DisplayItem::Text(text) => self.text(&mut content, text, place),
                DisplayItem::Image { asset, rect, .. } => match self.image(*asset) {
                    Some((name, id)) => {
                        let r = place.rect(*rect);
                        content.save_state();
                        content.transform([r.x2 - r.x1, 0.0, 0.0, r.y2 - r.y1, r.x1, r.y1]);
                        content.x_object(Name(name.as_bytes()));
                        content.restore_state();
                        if !used.iter().any(|(n, _)| n == &name) {
                            used.push((name, id));
                        }
                    }
                    None => {
                        self.report.missing_images += 1;
                        let r = place.rect(*rect);
                        set_stroke(&mut content, DisplayColor::Guide, mono);
                        content.set_line_width(0.5);
                        content.set_dash_pattern([], 0.0);
                        content.rect(r.x1, r.y1, r.x2 - r.x1, r.y2 - r.y1);
                        content.move_to(r.x1, r.y1);
                        content.line_to(r.x2, r.y2);
                        content.move_to(r.x2, r.y1);
                        content.line_to(r.x1, r.y2);
                        content.stroke();
                    }
                },
                DisplayItem::Link { .. } => {}
            }
        }
        (deflate(&content.finish()), used)
    }
}
