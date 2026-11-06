// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Writes neutral frame templates as R12 DXF; start at `export`.
use super::*;

fn num(v: f64) -> String {
    let s = format!("{:.6}", round6(v));
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s.is_empty() || s == "-0" {
        "0".to_owned()
    } else {
        s.to_owned()
    }
}
fn round6(v: f64) -> f64 {
    (v * 1e6).round() / 1e6
}
/// DXF R12 text: encode non-ASCII characters as `\U+XXXX`.
pub(super) fn escape(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        match c {
            '\n' | '\r' => out.push(' '),
            c if c.is_ascii() && !c.is_ascii_control() => out.push(c),
            c if (c as u32) <= 0xFFFF => {
                let _ = write!(out, "\\U+{:04X}", c as u32);
            }
            _ => out.push('?'),
        }
    }
    out
}
fn align_codes(a: TextAlign) -> (i64, i64) {
    use TextAlign::*;
    match a {
        BottomLeft => (0, 1),
        BottomCenter => (1, 1),
        BottomRight => (2, 1),
        MiddleLeft => (0, 2),
        Center => (1, 2),
        MiddleRight => (2, 2),
        TopLeft => (0, 3),
        TopCenter => (1, 3),
        TopRight => (2, 3),
    }
}

/// Exports a frame as DXF R12 in mm. The report lists unsupported content:
/// images and translations beyond the default text.
pub(super) fn export(frame: &FrameTemplate) -> (Vec<u8>, ImportReport) {
    let mut report = ImportReport::default();
    let mut out = String::new();
    let mut pair = |code: i32, value: &str| {
        let _ = cadcraft_model::dxf::write_group(&mut out, code, value);
    };
    let meta = Meta {
        name: frame.name().as_str().to_owned(),
        description: frame.description().to_owned(),
        usable: [
            frame.usable().origin().x(),
            frame.usable().origin().y(),
            frame.usable().size().width(),
            frame.usable().size().height(),
        ],
        zones: frame.zones().clone(),
    };
    pair(
        999,
        &format!(
            "{META_PREFIX}{}",
            escape(&serde_json::to_string(&meta).expect("metadados serializáveis"))
        ),
    );
    pair(0, "SECTION");
    pair(2, "HEADER");
    pair(9, "$ACADVER");
    pair(1, "AC1009");
    pair(9, "$INSUNITS");
    pair(70, "4");
    for (var, x, y) in [
        ("$LIMMIN", 0.0, 0.0),
        ("$LIMMAX", frame.sheet().width(), frame.sheet().height()),
        ("$EXTMIN", 0.0, 0.0),
        ("$EXTMAX", frame.sheet().width(), frame.sheet().height()),
    ] {
        pair(9, var);
        pair(10, &num(x));
        pair(20, &num(y));
    }
    pair(0, "ENDSEC");
    pair(0, "SECTION");
    pair(2, "ENTITIES");
    let layer = |w: f64| format!("{LAYER_PREFIX}{}mm", num(w));
    for e in frame.elements() {
        match e {
            FrameElement::Line { start, end, width } => {
                pair(0, "LINE");
                pair(8, &layer(*width));
                for (c, v) in [
                    (10, start.x()),
                    (20, start.y()),
                    (30, 0.0),
                    (11, end.x()),
                    (21, end.y()),
                    (31, 0.0),
                ] {
                    pair(c, &num(v));
                }
            }
            FrameElement::Rectangle { rect, width } => {
                let (o, m) = (rect.origin(), rect.max());
                pair(0, "POLYLINE");
                pair(8, &layer(*width));
                pair(66, "1");
                pair(10, "0");
                pair(20, "0");
                pair(30, "0");
                pair(70, "1");
                for (x, y) in [
                    (o.x(), o.y()),
                    (m.x(), o.y()),
                    (m.x(), m.y()),
                    (o.x(), m.y()),
                ] {
                    pair(0, "VERTEX");
                    pair(8, &layer(*width));
                    pair(10, &num(x));
                    pair(20, &num(y));
                    pair(30, "0");
                }
                pair(0, "SEQEND");
                pair(8, &layer(*width));
            }
            FrameElement::Circle {
                center,
                radius,
                width,
            } => {
                pair(0, "CIRCLE");
                pair(8, &layer(*width));
                pair(10, &num(center.x()));
                pair(20, &num(center.y()));
                pair(30, "0");
                pair(40, &num(*radius));
            }
            FrameElement::Arc {
                center,
                radius,
                start_degrees,
                sweep_degrees,
                width,
            } => {
                let (a0, a1) = if *sweep_degrees >= 0.0 {
                    (*start_degrees, start_degrees + sweep_degrees)
                } else {
                    (start_degrees + sweep_degrees, *start_degrees)
                };
                pair(0, "ARC");
                pair(8, &layer(*width));
                pair(10, &num(center.x()));
                pair(20, &num(center.y()));
                pair(30, "0");
                pair(40, &num(*radius));
                pair(50, &num(a0.rem_euclid(360.0)));
                pair(
                    51,
                    &num(if (a1 - a0 - 360.0).abs() < 1e-9 {
                        a0.rem_euclid(360.0) + 360.0
                    } else {
                        a1.rem_euclid(360.0)
                    }),
                );
            }
            FrameElement::Text {
                origin,
                text,
                style,
            } => {
                if !text.translations.is_empty() {
                    report.info("traduções de textos não vão para o DXF (só o texto padrão)");
                }
                write_text(&mut pair, origin.x(), origin.y(), &text.default, style);
            }
            FrameElement::Special {
                origin,
                field,
                index,
                style,
            } => {
                write_text(
                    &mut pair,
                    origin.x(),
                    origin.y(),
                    &format!("<<{}>>", special_code(field, *index)),
                    style,
                );
            }
            FrameElement::Image { .. } => report.ignored("imagem (o DXF R12 não guarda imagens)"),
        }
    }
    pair(0, "ENDSEC");
    pair(0, "EOF");
    (out.into_bytes(), report)
}

fn write_text(
    pair: &mut impl FnMut(i32, &str),
    x: f64,
    y: f64,
    text: &str,
    style: &FrameTextStyle,
) {
    let (h, v) = align_codes(style.align);
    pair(0, "TEXT");
    pair(
        8,
        if style.watermark {
            WATERMARK_LAYER
        } else {
            "CAD-CRAFT textos"
        },
    );
    pair(10, &num(x));
    pair(20, &num(y));
    pair(30, "0");
    pair(40, &num(style.height));
    pair(1, &escape(text));
    if style.rotation_degrees != 0.0 {
        pair(50, &num(style.rotation_degrees));
    }
    pair(72, &h.to_string());
    pair(11, &num(x));
    pair(21, &num(y));
    pair(31, "0");
    pair(73, &v.to_string());
}
