// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Free graphic properties; start with CadApp::show_free_graphic_properties.
use super::*;

impl CadApp {
    pub(super) fn show_free_graphic_properties(&mut self, ui: &mut Ui) {
        let Some(id) = (match self.session.selection {
            Selection::Representation(id) => Some(id),
            _ => None,
        }) else {
            self.free_content.free_edit = None;
            return;
        };
        let Some(graphic) = self
            .session
            .project()
            .and_then(|p| p.representation(id))
            .and_then(Representation::graphic)
            .cloned()
        else {
            self.free_content.free_edit = None;
            return;
        };
        if self.free_content.free_edit_revision != self.session.revision
            || self
                .free_content
                .free_edit
                .as_ref()
                .is_none_or(|draft| draft.0 != id)
        {
            self.free_content.free_edit = Some((
                id,
                graphic.primitive().clone(),
                graphic.stroke_mm(),
                graphic.alignment(),
            ));
            self.free_content.free_edit_revision = self.session.revision;
        }
        let mut apply = false;
        let mut style_change: Option<PrimitiveStyle> = None;
        ui.collapsing("Conteúdo livre", |ui| {
            // Apply stroke pattern and fill changes immediately.
            let mut style = graphic.style();
            egui::ComboBox::from_id_salt("free_graphic_dash")
                .selected_text(style.dash.label())
                .show_ui(ui, |ui| {
                    for dash in LineDash::ALL {
                        ui.selectable_value(&mut style.dash, dash, dash.label());
                    }
                });
            if graphic.primitive().is_closed() {
                ui.checkbox(&mut style.fill, "Preenchimento");
            }
            if style != graphic.style() {
                style_change = Some(style);
            }
            if let Some((_, primitive, stroke, alignment)) = &mut self.free_content.free_edit {
                ui.horizontal(|ui| {
                    ui.label("Traço (mm)");
                    ui.add(egui::DragValue::new(stroke).range(0.01..=10.0).speed(0.01));
                });
                match primitive {
                    SymbolPrimitive::Text { text, height, .. } => {
                        ui.label("Texto fixo Unicode");
                        ui.add(egui::TextEdit::singleline(text));
                        ui.horizontal(|ui| {
                            ui.label("Altura (mm)");
                            ui.add(egui::DragValue::new(height).range(0.1..=100.0).speed(0.1));
                        });
                        egui::ComboBox::from_id_salt("edit_free_align")
                            .selected_text(format!("{:?}", alignment))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(alignment, TextAlignment::Left, "Esquerda");
                                ui.selectable_value(alignment, TextAlignment::Center, "Centro");
                                ui.selectable_value(alignment, TextAlignment::Right, "Direita");
                            });
                    }
                    SymbolPrimitive::Line { end, .. } => {
                        let mut x = end.x();
                        let mut y = end.y();
                        ui.horizontal(|ui| {
                            ui.label("Extremo local X/Y (mm)");
                            ui.add(egui::DragValue::new(&mut x).speed(0.1));
                            ui.add(egui::DragValue::new(&mut y).speed(0.1));
                        });
                        if let Ok(p) = Point2D::new(x, y) {
                            *end = p;
                        }
                    }
                    SymbolPrimitive::Rectangle { rect } => {
                        let mut w = rect.size().width();
                        let mut h = rect.size().height();
                        ui.horizontal(|ui| {
                            ui.label("Largura/altura (mm)");
                            ui.add(egui::DragValue::new(&mut w).range(0.1..=10000.0).speed(0.1));
                            ui.add(egui::DragValue::new(&mut h).range(0.1..=10000.0).speed(0.1));
                        });
                        if let Ok(size) = Size2D::new(w, h)
                            && let Ok(next) = Rect2D::new(rect.origin(), size)
                        {
                            *rect = next;
                        }
                    }
                    SymbolPrimitive::Circle { radius, .. } => {
                        ui.horizontal(|ui| {
                            ui.label("Raio (mm)");
                            ui.add(egui::DragValue::new(radius).range(0.1..=10000.0).speed(0.1));
                        });
                    }
                    SymbolPrimitive::Arc {
                        radius,
                        start_degrees,
                        sweep_degrees,
                        ..
                    } => {
                        ui.horizontal(|ui| {
                            ui.label("Raio (mm)");
                            ui.add(egui::DragValue::new(radius).range(0.1..=10000.0).speed(0.1));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Início/varredura (°)");
                            ui.add(egui::DragValue::new(start_degrees).speed(1.0));
                            ui.add(
                                egui::DragValue::new(sweep_degrees)
                                    .range(-360.0..=360.0)
                                    .speed(1.0),
                            );
                        });
                    }
                    _ => {}
                }
                apply = ui.button("Aplicar conteúdo livre").clicked();
            }
        });
        if let Some(style) = style_change
            && let Err(e) = graphic
                .clone()
                .with_style(style)
                .and_then(|g| self.session.edit_free_graphic(id, g))
        {
            self.log(Level::Error, e.to_string());
        }
        if apply && let Some((_, primitive, stroke, alignment)) = &self.free_content.free_edit {
            match PageGraphic::new(primitive.clone(), *stroke, *alignment)
                .and_then(|g| g.with_style(graphic.style()))
                .and_then(|g| self.session.edit_free_graphic(id, g))
            {
                Ok(_) => {}
                Err(e) => self.log(Level::Error, e.to_string()),
            }
        }
    }
}
