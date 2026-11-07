// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! forms support for the frame editor; start at the parent module.

use super::*;

pub(in crate::frame_editor) fn element_form(e: &FrameElement) -> ElementForm {
    let mut f = ElementForm::default();
    match e {
        FrameElement::Line { start, end, width } => {
            f.nums = [start.x(), start.y(), end.x(), end.y(), *width, 0.0];
        }
        FrameElement::Rectangle { rect, width } => {
            f.nums = [
                rect.origin().x(),
                rect.origin().y(),
                rect.size().width(),
                rect.size().height(),
                *width,
                0.0,
            ];
        }
        FrameElement::Circle {
            center,
            radius,
            width,
        } => {
            f.nums = [center.x(), center.y(), *radius, *width, 0.0, 0.0];
        }
        FrameElement::Arc {
            center,
            radius,
            start_degrees,
            sweep_degrees,
            width,
        } => {
            f.nums = [
                center.x(),
                center.y(),
                *radius,
                *start_degrees,
                *sweep_degrees,
                *width,
            ];
        }
        FrameElement::Text {
            origin,
            text,
            style,
        } => {
            f.nums = [origin.x(), origin.y(), 0.0, 0.0, 0.0, 0.0];
            f.text = text.default.clone();
            f.translations = text
                .translations
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("; ");
            f.style = Some(*style);
        }
        FrameElement::Special {
            origin,
            field,
            index,
            style,
        } => {
            f.nums = [origin.x(), origin.y(), 0.0, 0.0, 0.0, 0.0];
            (f.field, f.key) = field_index(field);
            f.index = index.unwrap_or(0);
            f.style = Some(*style);
        }
        FrameElement::Image { rect, .. } => {
            f.nums = [
                rect.origin().x(),
                rect.origin().y(),
                rect.size().width(),
                rect.size().height(),
                0.0,
                0.0,
            ];
            f.keep_aspect = true;
        }
    }
    f
}

pub(in crate::frame_editor) fn num(ui: &mut Ui, label: &str, value: &mut f64, suffix: &str) {
    ui.label(label);
    ui.add(
        egui::DragValue::new(value)
            .speed(0.5)
            .max_decimals(3)
            .suffix(suffix),
    );
    ui.end_row();
}
pub(in crate::frame_editor) fn element_form_ui(
    ui: &mut Ui,
    e: &FrameElement,
    f: &mut ElementForm,
    apply: &mut bool,
) {
    let mm = " mm";
    egui::Grid::new("frame_element_form")
        .num_columns(2)
        .show(ui, |ui| {
            let labels: &[(&str, &str)] = match e {
                FrameElement::Line { .. } => &[
                    ("X inicial", mm),
                    ("Y inicial", mm),
                    ("X final", mm),
                    ("Y final", mm),
                    ("Espessura", mm),
                ],
                FrameElement::Rectangle { .. } => &[
                    ("X", mm),
                    ("Y", mm),
                    ("Largura", mm),
                    ("Altura", mm),
                    ("Espessura", mm),
                ],
                FrameElement::Circle { .. } => &[
                    ("X centro", mm),
                    ("Y centro", mm),
                    ("Raio", mm),
                    ("Espessura", mm),
                ],
                FrameElement::Arc { .. } => &[
                    ("X centro", mm),
                    ("Y centro", mm),
                    ("Raio", mm),
                    ("Início", "°"),
                    ("Varredura", "°"),
                    ("Espessura", mm),
                ],
                FrameElement::Text { .. } | FrameElement::Special { .. } => &[("X", mm), ("Y", mm)],
                FrameElement::Image { .. } => {
                    &[("X", mm), ("Y", mm), ("Largura", mm), ("Altura", mm)]
                }
            };
            for (i, (label, suffix)) in labels.iter().enumerate() {
                num(ui, label, &mut f.nums[i], suffix);
            }
            match e {
                FrameElement::Text { .. } => {
                    ui.label("Texto");
                    ui.add(egui::TextEdit::singleline(&mut f.text).desired_width(140.0));
                    ui.end_row();
                    ui.label("Traduções");
                    ui.add(
                        egui::TextEdit::singleline(&mut f.translations)
                            .hint_text("en-US=Page; es-ES=Hoja")
                            .desired_width(140.0),
                    );
                    ui.end_row();
                }
                FrameElement::Special { .. } => {
                    ui.label("Campo");
                    egui::ComboBox::from_id_salt("frame_special_field")
                        .width(140.0)
                        .height(420.0)
                        .selected_text(FIELDS[f.field])
                        .show_ui(ui, |ui| {
                            for (i, name) in FIELDS.iter().enumerate() {
                                ui.selectable_value(&mut f.field, i, *name);
                            }
                        });
                    ui.end_row();
                    if f.field == 3 || f.field == 14 {
                        ui.label("Chave");
                        ui.add(
                            egui::TextEdit::singleline(&mut f.key)
                                .hint_text("Autor, Data, Revisão…")
                                .desired_width(140.0),
                        );
                        ui.end_row();
                        ui.label("Índice (0 = sem)");
                        ui.add(egui::DragValue::new(&mut f.index).range(0..=999));
                        ui.end_row();
                    }
                }
                FrameElement::Image { .. } => {
                    ui.label("Manter proporção");
                    ui.checkbox(&mut f.keep_aspect, "");
                    ui.end_row();
                }
                _ => {}
            }
            if let Some(style) = &mut f.style {
                num(ui, "Altura do texto", &mut style.height, mm);
                ui.label("Alinhamento");
                egui::ComboBox::from_id_salt("frame_text_align")
                    .width(140.0)
                    .selected_text(
                        ALIGNS
                            .iter()
                            .find(|a| a.0 == style.align)
                            .map_or("", |a| a.1),
                    )
                    .show_ui(ui, |ui| {
                        for (align, name) in ALIGNS {
                            ui.selectable_value(&mut style.align, align, name);
                        }
                    });
                ui.end_row();
                num(ui, "Rotação", &mut style.rotation_degrees, "°");
                ui.label("Marca d'água");
                ui.checkbox(&mut style.watermark, "");
                ui.end_row();
            }
        });
    if ui.button("Aplicar elemento").clicked() {
        *apply = true;
    }
}

pub(in crate::frame_editor) fn apply_element_form(
    e: &FrameElement,
    f: &ElementForm,
    image_aspect: Option<f64>,
) -> Result<FrameElement, String> {
    let n = f.nums;
    let p = |x: f64, y: f64| Point2D::new(x, y).map_err(|e| e.to_string());
    let r = |x: f64, y: f64, w: f64, h: f64| {
        Rect2D::new(p(x, y)?, Size2D::new(w, h).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())
    };
    let style = || f.style.ok_or_else(|| "estilo ausente".to_owned());
    Ok(match e {
        FrameElement::Line { .. } => FrameElement::Line {
            start: p(n[0], n[1])?,
            end: p(n[2], n[3])?,
            width: n[4],
        },
        FrameElement::Rectangle { .. } => FrameElement::Rectangle {
            rect: r(n[0], n[1], n[2], n[3])?,
            width: n[4],
        },
        FrameElement::Circle { .. } => FrameElement::Circle {
            center: p(n[0], n[1])?,
            radius: n[2],
            width: n[3],
        },
        FrameElement::Arc { .. } => FrameElement::Arc {
            center: p(n[0], n[1])?,
            radius: n[2],
            start_degrees: n[3],
            sweep_degrees: n[4],
            width: n[5],
        },
        FrameElement::Text { .. } => {
            let mut text = LocalizedText::new(f.text.clone());
            for pair in f.translations.split(';').filter(|s| !s.trim().is_empty()) {
                let (lang, value) = pair
                    .split_once('=')
                    .ok_or_else(|| "traduções no formato idioma=texto; …".to_owned())?;
                text.translations
                    .insert(lang.trim().to_owned(), value.trim().to_owned());
            }
            FrameElement::Text {
                origin: p(n[0], n[1])?,
                text,
                style: style()?,
            }
        }
        FrameElement::Special { .. } => FrameElement::Special {
            origin: p(n[0], n[1])?,
            field: field_from(f.field, &f.key)?,
            index: (f.index > 0).then_some(f.index),
            style: style()?,
        },
        FrameElement::Image { asset, .. } => {
            let height = match (f.keep_aspect, image_aspect) {
                (true, Some(aspect)) => n[2] / aspect,
                _ => n[3],
            };
            FrameElement::Image {
                rect: r(n[0], n[1], n[2], height)?,
                asset: *asset,
            }
        }
    })
}

pub(in crate::frame_editor) fn build_frame_draft(
    frame: &FrameTemplate,
    f: &FrameForm,
) -> Result<FrameDraft, String> {
    let e = |e: cad_craft::error::DomainError| e.to_string();
    let mut draft = frame.draft();
    draft.name = f.name.clone();
    draft.description = f.description.clone();
    draft.sheet = Size2D::new(f.sheet[0], f.sheet[1]).map_err(e)?;
    draft.usable = Rect2D::new(
        Point2D::new(f.usable[0], f.usable[1]).map_err(e)?,
        Size2D::new(f.usable[2], f.usable[3]).map_err(e)?,
    )
    .map_err(e)?;
    for (axis, available, noun, side) in [
        (&f.axes[0], f.usable[2], "colunas", "largura"),
        (&f.axes[1], f.usable[3], "linhas", "altura"),
    ] {
        if let Some(message) = axis.overflow(available, noun, side) {
            return Err(message);
        }
    }
    draft.zones = ZoneLayout {
        columns: f.axes[0].build()?,
        rows: f.axes[1].build()?,
        // Frames edited here always draw their divisions from the zones.
        generated_marks: true,
    };
    Ok(draft)
}
