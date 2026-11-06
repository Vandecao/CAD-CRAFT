// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! draw support for the symbol editor; start at the parent module.

use super::*;

impl CadApp {
    pub(crate) fn show_symbol_editor(&mut self, ui: &mut Ui) {
        let title = self
            .symbol_editor
            .draft
            .as_ref()
            .map(|d| d.name.clone())
            .unwrap_or_default();
        let version = self
            .symbol_editor
            .base
            .as_ref()
            .map_or(1, |b| b.version() + 1);
        ui.horizontal_wrapped(|ui| {
            ui.strong(format!("Símbolo: {title} · v{version}"));
            ui.separator();
            for tool in SymbolTool::ALL {
                if ui
                    .selectable_label(self.symbol_editor.tool == tool, tool.label())
                    .clicked()
                {
                    self.symbol_editor.pending.clear();
                    self.symbol_editor.tool = tool;
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(
                    self.symbol_editor.selection.is_some(),
                    egui::Button::new("Excluir seleção"),
                )
                .clicked()
            {
                self.symbol_editor.delete_selection();
            }
            if ui
                .add_enabled(
                    !self.symbol_editor.undo.is_empty(),
                    egui::Button::new("Desfazer"),
                )
                .clicked()
            {
                self.symbol_editor.undo();
            }
            if ui
                .add_enabled(
                    !self.symbol_editor.redo.is_empty(),
                    egui::Button::new("Refazer"),
                )
                .clicked()
            {
                self.symbol_editor.redo();
            }
            if ui.button("Ajustar à tela").clicked() {
                self.symbol_editor.needs_fit = true;
            }
            if ui.button("Salvar na biblioteca").clicked() {
                self.save_symbol_to_library();
            }
            let in_project = self.symbol_editor.id.as_ref().is_some_and(|id| {
                self.session
                    .project()
                    .is_some_and(|p| p.symbol_definition(id).is_some())
            });
            if ui
                .add_enabled(in_project, egui::Button::new("Atualizar no projeto"))
                .clicked()
            {
                self.update_symbol_in_project();
            }
            if ui.button("Fechar editor").clicked() {
                self.close_symbol_editor();
            }
        });
        if !self.symbol_editor.is_open() {
            return;
        }
        ui.small(self.symbol_editor.tool.hint());
        if let Some(m) = &self.symbol_editor.message {
            ui.colored_label(theme::ACCENT, m);
        }
        egui::SidePanel::right("symbol_editor_properties")
            .resizable(false)
            .exact_width(270.0)
            .frame(egui::Frame::new().fill(theme::PANEL).inner_margin(6))
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| self.symbol_properties_ui(ui));
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::TITLE))
            .show_inside(ui, |ui| self.symbol_canvas(ui));
    }

    pub(in crate::symbol_editor) fn symbol_canvas(&mut self, ui: &mut Ui) {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        let ed = &mut self.symbol_editor;
        ed.viewport = Some(rect);
        if ed.needs_fit && rect.width() > 40.0 && rect.height() > 40.0 {
            let bounds = ed
                .build()
                .ok()
                .map(|d| d.bounds())
                .filter(|b| b.size().width() > 1.0 || b.size().height() > 1.0)
                .and_then(|b| {
                    Rect2D::new(
                        b.origin().translated(-10.0, -10.0).ok()?,
                        Size2D::new(b.size().width() + 20.0, b.size().height() + 20.0).ok()?,
                    )
                    .ok()
                })
                .unwrap_or_else(|| {
                    Rect2D::new(
                        Point2D::new(-20.0, -20.0).expect("fixo"),
                        Size2D::new(40.0, 40.0).expect("fixo"),
                    )
                    .expect("fixo")
                });
            ed.camera.fit(bounds, rect.shrink(24.0));
            ed.needs_fit = false;
        }
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0
                && let Some(cursor) = response.hover_pos()
            {
                ed.camera
                    .zoom_at((scroll as f64 / 200.0).exp(), cursor, rect);
            }
        }
        if response.dragged_by(PointerButton::Middle) {
            ed.camera.pan(response.drag_delta());
        }
        let camera = ed.camera;
        let world = |p: egui::Pos2| {
            camera
                .screen_to_world(p, rect)
                .map(|w| snap(w, GRID_MM, true))
        };
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0, theme::PAPER);
        if camera.scale() * GRID_MM >= 4.0 {
            for p in grid_points(&camera, rect, GRID_MM) {
                painter.circle_filled(p, 0.8, theme::GRID_DOT);
            }
        }
        // Origin, also the insertion point.
        if let Some(o) = Point2D::new(0.0, 0.0)
            .ok()
            .and_then(|o| camera.world_to_screen(o, rect))
        {
            let s = Stroke::new(1.0_f32, theme::GRID);
            painter.line_segment([o - egui::vec2(8.0, 0.0), o + egui::vec2(8.0, 0.0)], s);
            painter.line_segment([o - egui::vec2(0.0, 8.0), o + egui::vec2(0.0, 8.0)], s);
        }
        let identity = Transform2D::new(Point2D::new(0.0, 0.0).expect("origem"), 0.0, false, false)
            .expect("identidade");
        let drag = ed.drag;
        let shift = |item: SymbolItem| match (ed.selection, drag) {
            (Some(sel), Some((a, b))) if sel == item => (b.x() - a.x(), b.y() - a.y()),
            _ => (0.0, 0.0),
        };
        for (i, p) in ed.primitives().iter().enumerate() {
            let (dx, dy) = shift(SymbolItem::Primitive(i));
            let shown = p.translated(dx, dy).unwrap_or_else(|_| p.clone());
            let selected = ed.selection == Some(SymbolItem::Primitive(i));
            symbol_view::paint_styled(
                &painter,
                &shown,
                ed.style(i),
                identity,
                camera,
                rect,
                Stroke::new(
                    if selected { 2.0_f32 } else { 1.2_f32 },
                    if selected { theme::ACCENT } else { theme::INK },
                ),
                &symbol_view::placeholder,
            );
        }
        for (i, port) in ed.ports().iter().enumerate() {
            let (dx, dy) = shift(SymbolItem::Port(i));
            let selected = ed.selection == Some(SymbolItem::Port(i));
            if let Some(s) = port
                .position()
                .translated(dx, dy)
                .ok()
                .and_then(|p| camera.world_to_screen(p, rect))
            {
                let color = if selected {
                    theme::ACCENT
                } else {
                    egui::Color32::from_rgb(0, 140, 70)
                };
                painter.circle_stroke(s, 4.0, Stroke::new(1.5_f32, color));
                let a = port.orientation_degrees().to_radians();
                let tip = s + egui::vec2(a.cos() as f32, -(a.sin() as f32)) * 10.0;
                painter.line_segment([s, tip], Stroke::new(1.5_f32, color));
                painter.text(
                    s + egui::vec2(6.0, -6.0),
                    egui::Align2::LEFT_BOTTOM,
                    port.identification(),
                    egui::FontId::proportional(11.0),
                    color,
                );
            }
        }
        // Tool preview.
        let cursor = response.hover_pos().and_then(world);
        let mut preview = ed.pending.clone();
        if let Some(c) = cursor
            && ed.tool != SymbolTool::Select
        {
            preview.push(c);
        }
        if let Some(p) = ed.build_tool_primitive(&preview, false) {
            symbol_view::paint_primitive(
                &painter,
                &p,
                identity,
                camera,
                rect,
                Stroke::new(1.0_f32, theme::ACCENT),
            );
        }
        for p in &ed.pending {
            if let Some(s) = camera.world_to_screen(*p, rect) {
                painter.circle_stroke(s, 3.0, Stroke::new(1.0_f32, theme::ACCENT));
            }
        }
        // Interaction.
        let tolerance = 6.0 / camera.scale();
        if ed.tool == SymbolTool::Select {
            if response.drag_started_by(PointerButton::Primary)
                && let Some(start) = ui
                    .input(|i| i.pointer.press_origin())
                    .or(response.interact_pointer_pos())
                    .and_then(world)
                && let Some(item) = ed.hit(start, tolerance)
            {
                ed.selection = Some(item);
                ed.drag = Some((start, start));
            }
            if let Some((start, _)) = ed.drag
                && let Some(now) = response.interact_pointer_pos().and_then(world)
            {
                ed.drag = Some((start, now));
            }
            if response.drag_stopped_by(PointerButton::Primary)
                && let Some((a, b)) = ed.drag.take()
                && (a != b)
            {
                ed.move_selection(b.x() - a.x(), b.y() - a.y());
            }
            if response.clicked_by(PointerButton::Primary)
                && let Some(at) = response
                    .interact_pointer_pos()
                    .and_then(|p| camera.screen_to_world(p, rect))
            {
                ed.selection = ed.hit(at, tolerance);
            }
        } else if response.double_clicked_by(PointerButton::Primary)
            && matches!(ed.tool, SymbolTool::Polyline | SymbolTool::Polygon)
        {
            // Double-click finishes only polylines and polygons; other tools interpret two rapid clicks as two points.
            ed.finish_path();
        } else if response.clicked_by(PointerButton::Primary)
            && let Some(at) = response.interact_pointer_pos().and_then(world)
        {
            ed.tool_click(at);
        }
        if response.secondary_clicked() {
            ed.cancel();
        }
        // Keyboard handling; the symbol editor has its own undo history.
        if !ui.ctx().wants_keyboard_input() {
            let (del, enter, undo, redo, esc) = ui.input_mut(|i| {
                (
                    i.consume_key(egui::Modifiers::NONE, Key::Delete),
                    i.consume_key(egui::Modifiers::NONE, Key::Enter),
                    i.consume_key(egui::Modifiers::COMMAND, Key::Z),
                    i.consume_key(egui::Modifiers::COMMAND, Key::Y),
                    i.consume_key(egui::Modifiers::NONE, Key::Escape),
                )
            });
            if del {
                ed.delete_selection();
            }
            if enter {
                ed.finish_path();
            }
            if undo {
                ed.undo();
            }
            if redo {
                ed.redo();
            }
            if esc {
                ed.cancel();
            }
        }
    }

    pub(in crate::symbol_editor) fn symbol_properties_ui(&mut self, ui: &mut Ui) {
        let ed = &mut self.symbol_editor;
        ui.heading("Símbolo");
        let Some(draft) = ed.draft.clone() else {
            return;
        };
        let mut name = draft.name.clone();
        let mut category = draft.category.clone();
        egui::Grid::new("symbol_props")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Nome");
                ui.add(egui::TextEdit::singleline(&mut name).desired_width(150.0));
                ui.end_row();
                ui.label("Categoria");
                ui.add(egui::TextEdit::singleline(&mut category).desired_width(150.0));
                ui.end_row();
                ui.label("ID");
                ui.small(
                    ed.id
                        .as_ref()
                        .map(|i| i.as_str().to_owned())
                        .unwrap_or_default(),
                );
                ui.end_row();
                ui.label("Portas");
                ui.label(draft.ports.len().to_string());
                ui.end_row();
            });
        if name != draft.name || category != draft.category {
            ed.change(|d| {
                d.name = name;
                d.category = category;
            });
        }
        ui.separator();
        ui.label("Novos textos");
        ui.horizontal(|ui| {
            ui.label("Texto");
            ui.add(egui::TextEdit::singleline(&mut ed.new_text).desired_width(140.0));
        });
        egui::ComboBox::from_id_salt("symbol_field_choice")
            .selected_text(match ed.field {
                FieldChoice::Designation => "Designação",
                FieldChoice::FunctionKind => "Função",
                FieldChoice::PortLabel => "Ponto da porta",
                FieldChoice::Reference => "Referência",
                FieldChoice::Property => "Propriedade",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut ed.field, FieldChoice::Designation, "Designação");
                ui.selectable_value(&mut ed.field, FieldChoice::FunctionKind, "Função");
                ui.selectable_value(&mut ed.field, FieldChoice::PortLabel, "Ponto da porta");
                ui.selectable_value(&mut ed.field, FieldChoice::Reference, "Referência");
                ui.selectable_value(&mut ed.field, FieldChoice::Property, "Propriedade");
            });
        if ed.field == FieldChoice::Property {
            ui.horizontal(|ui| {
                ui.label("Chave");
                ui.add(egui::TextEdit::singleline(&mut ed.field_key).desired_width(120.0));
            });
        }
        ui.separator();
        match ed.selection {
            Some(SymbolItem::Primitive(i)) if i < draft.primitives.len() => {
                let p = draft.primitives[i].clone();
                ui.strong(p.kind_label());
                let mut style = ed.style(i);
                let before = style;
                egui::ComboBox::from_id_salt("symbol_dash")
                    .selected_text(style.dash.label())
                    .show_ui(ui, |ui| {
                        for dash in LineDash::ALL {
                            ui.selectable_value(&mut style.dash, dash, dash.label());
                        }
                    });
                let mut own = style.width_mm.is_some();
                let mut width = style.width_mm.unwrap_or(0.25);
                ui.horizontal(|ui| {
                    ui.checkbox(&mut own, "Espessura própria");
                    if own {
                        ui.add(
                            egui::DragValue::new(&mut width)
                                .range(0.05..=5.0)
                                .speed(0.05)
                                .suffix(" mm"),
                        );
                    }
                });
                style.width_mm = own.then_some(width);
                if p.is_closed() {
                    ui.checkbox(&mut style.fill, "Preenchimento");
                }
                if style != before {
                    ed.change(|d| {
                        if style.is_default() {
                            d.styles.remove(&i);
                        } else {
                            d.styles.insert(i, style);
                        }
                    });
                }
                let mut edited = p.clone();
                match &mut edited {
                    SymbolPrimitive::Text { text, height, .. } => {
                        ui.add(egui::TextEdit::singleline(text).desired_width(180.0));
                        ui.add(
                            egui::DragValue::new(height)
                                .range(0.5..=50.0)
                                .speed(0.1)
                                .prefix("Altura "),
                        );
                    }
                    SymbolPrimitive::DynamicText {
                        field,
                        height,
                        rotation_degrees,
                        ..
                    } => {
                        ui.label(field.label());
                        if let SymbolTextField::PortLabel(port)
                        | SymbolTextField::DeviceProperty(port) = field
                        {
                            ui.add(egui::TextEdit::singleline(port).desired_width(120.0));
                        }
                        ui.add(
                            egui::DragValue::new(height)
                                .range(0.5..=50.0)
                                .speed(0.1)
                                .prefix("Altura "),
                        );
                        ui.add(
                            egui::DragValue::new(rotation_degrees)
                                .speed(15.0)
                                .prefix("Rotação "),
                        );
                    }
                    SymbolPrimitive::Ellipse {
                        rotation_degrees, ..
                    } => {
                        ui.add(
                            egui::DragValue::new(rotation_degrees)
                                .speed(5.0)
                                .prefix("Rotação "),
                        );
                    }
                    _ => {}
                }
                if edited != p && edited.bounds().is_ok() {
                    ed.change(|d| d.primitives[i] = edited);
                }
            }
            Some(SymbolItem::Port(i)) if i < draft.ports.len() => {
                let port = draft.ports[i].clone();
                ui.strong("Porta");
                let mut id = port.id().to_owned();
                let mut ident = port.identification().to_owned();
                let mut orientation = port.orientation_degrees();
                ui.horizontal(|ui| {
                    ui.label("ID");
                    ui.add(egui::TextEdit::singleline(&mut id).desired_width(80.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Identificação");
                    ui.add(egui::TextEdit::singleline(&mut ident).desired_width(80.0));
                });
                egui::ComboBox::from_id_salt("symbol_port_orientation")
                    .selected_text(format!("{orientation}°"))
                    .show_ui(ui, |ui| {
                        for a in [0.0, 90.0, 180.0, 270.0] {
                            ui.selectable_value(&mut orientation, a, format!("{a}°"));
                        }
                    });
                let unique = !draft
                    .ports
                    .iter()
                    .enumerate()
                    .any(|(j, p)| j != i && p.id() == id);
                if (id != port.id()
                    || ident != port.identification()
                    || orientation != port.orientation_degrees())
                    && unique
                    && let Ok(new) = SymbolPort::new(
                        id,
                        port.position(),
                        orientation,
                        ident,
                        port.role().map(str::to_owned),
                    )
                {
                    ed.change(|d| d.ports[i] = new);
                }
                if !unique {
                    ui.colored_label(theme::ACCENT, "ID de porta repetido");
                }
            }
            _ => {
                ui.weak("Selecione um elemento ou uma porta.");
            }
        }
    }
}
