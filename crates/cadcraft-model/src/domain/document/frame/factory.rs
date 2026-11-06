// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Factory A3/A4 frames; start with FrameTemplate::factory_a3.
use super::*;

fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).expect("static point")
}
fn line(a: (f64, f64), b: (f64, f64), width: f64) -> FrameElement {
    FrameElement::Line {
        start: p(a.0, a.1),
        end: p(b.0, b.1),
        width,
    }
}
fn label(x: f64, y: f64, text: &str) -> FrameElement {
    FrameElement::Text {
        origin: p(x, y),
        text: LocalizedText::new(text),
        style: FrameTextStyle::new(2.0, TextAlign::TopLeft),
    }
}
fn value(x: f64, y: f64, field: SpecialField) -> FrameElement {
    FrameElement::Special {
        origin: p(x, y),
        field,
        index: None,
        style: FrameTextStyle::new(3.0, TextAlign::BottomLeft),
    }
}
fn key(name: &str) -> Text {
    Text::new(name).expect("static key")
}

/// Original CAD-CRAFT frame: 5 mm border, 5 mm column strip at the top and a 30 mm title block at
/// the bottom with four fields per row.
fn factory(name: &str, sheet: Size2D, columns: usize) -> FrameTemplate {
    let (w, h) = (sheet.width(), sheet.height());
    let (left, right, bottom, top) = (5.0, w - 5.0, 5.0, h - 5.0);
    let band = top - 5.0;
    let title = bottom + 30.0;
    let width = right - left;
    let mut e = vec![
        FrameElement::Rectangle {
            rect: Rect2D::new(
                p(left, bottom),
                Size2D::new(width, top - bottom).expect("size"),
            )
            .expect("rect"),
            width: 0.5,
        },
        line((left, band), (right, band), 0.25),
        line((left, title), (right, title), 0.5),
        line((left, bottom + 15.0), (right, bottom + 15.0), 0.25),
    ];
    let step = width / columns as f64;
    for i in 1..columns {
        let x = left + step * i as f64;
        e.push(line((x, band), (x, top), 0.25));
    }
    let cell = width / 4.0;
    for i in 1..4 {
        let x = left + cell * i as f64;
        e.push(line((x, bottom), (x, title), 0.25));
    }
    let rows: [[(&str, SpecialField); 4]; 2] = [
        [
            ("Projeto", SpecialField::ProjectName),
            ("Descrição da página", SpecialField::PageName),
            ("Autor", SpecialField::ProjectProperty(key("Autor"))),
            ("Página", SpecialField::PageNumber),
        ],
        [
            ("Identificação", SpecialField::ProjectIdentification),
            ("Documento", SpecialField::DocumentName),
            ("Data", SpecialField::PageProperty(key("Data"))),
            ("Folha / total", SpecialField::PagePosition),
        ],
    ];
    for (r, cells) in rows.into_iter().enumerate() {
        let top_of_row = title - 15.0 * r as f64;
        for (c, (text, field)) in cells.into_iter().enumerate() {
            let x = left + cell * c as f64 + 1.5;
            e.push(label(x, top_of_row - 1.0, text));
            e.push(value(x, top_of_row - 13.0, field));
        }
    }
    // The total-sheet suffix completes the sheet position in the last cell.
    e.push(value(
        right - 20.0,
        bottom + 2.0,
        SpecialField::PageCountInDocument,
    ));
    e.push(value(
        right - cell + 1.5,
        title + 1.0,
        SpecialField::FrameName,
    ));
    let usable = Rect2D::new(
        p(left, title),
        Size2D::new(width, band - title).expect("size"),
    )
    .expect("usable");
    let zones = ZoneLayout {
        columns: ZoneAxis::uniform(columns, step, ZoneLabelStyle::Numbers { start: 1 }),
        rows: ZoneAxis::none(),
    };
    FrameTemplate::new(name, sheet, usable, zones, e)
        .expect("valid factory frame")
        .with_description("Moldura original CAD-CRAFT; sem vínculo com normas ou terceiros.")
}

impl FrameTemplate {
    /// Factory A3 landscape frame with ten 41 mm columns.
    pub fn factory_a3() -> Self {
        factory("CAD-CRAFT A3 paisagem", PaperFormat::A3.size(true), 10)
    }
    /// Factory A4 portrait frame with five 40 mm columns.
    pub fn factory_a4() -> Self {
        factory("CAD-CRAFT A4 retrato", PaperFormat::A4.size(false), 5)
    }
}
