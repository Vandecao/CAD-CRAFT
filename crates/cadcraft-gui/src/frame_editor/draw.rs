// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! draw support for the frame editor; start at the parent module.

use super::*;

impl CadApp {
    pub(crate) fn show_frame_editor(&mut self, ui: &mut Ui) {
        let Some(frame) = self.editor_frame() else {
            self.close_frame_editor();
            return;
        };
        ui.horizontal_wrapped(|ui| {
            ui.strong(format!(
                "Moldura: {} · v{}",
                frame.name().as_str(),
                frame.version()
            ));
            ui.separator();
            for tool in FrameTool::ALL {
                if ui
                    .selectable_label(self.frame_editor.tool == tool, tool.label())
                    .clicked()
                {
                    self.set_frame_tool(tool);
                }
            }
            ui.separator();
            let any = !self.frame_editor.selection.is_empty();
            if ui
                .add_enabled(any, egui::Button::new("Duplicar seleção"))
                .clicked()
            {
                self.editor_transform_selection(true, 5.0, -5.0, false);
            }
            if ui
                .add_enabled(any, egui::Button::new("Excluir seleção"))
                .clicked()
            {
                self.editor_delete_selection();
            }
            if ui
                .add_enabled(any, egui::Button::new("Alinhar à grade"))
                .clicked()
            {
                self.editor_transform_selection(false, 0.0, 0.0, true);
            }
            if ui.button("Ajustar à tela").clicked() {
                self.frame_editor.needs_fit = true;
            }
            if ui.button("Fechar editor").clicked() {
                self.close_frame_editor();
            }
        });
        if !self.frame_editor.is_open() {
            return;
        }
        ui.small(self.frame_editor.tool.hint());
        egui::SidePanel::right("frame_editor_properties")
            .resizable(false)
            .exact_width(270.0)
            .frame(egui::Frame::new().fill(theme::PANEL).inner_margin(6))
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| self.frame_properties_ui(ui, &frame));
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::TITLE))
            .show_inside(ui, |ui| self.frame_canvas(ui, &frame));
    }

    pub(in crate::frame_editor) fn frame_canvas(&mut self, ui: &mut Ui, frame: &FrameTemplate) {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        self.frame_editor.viewport = Some(rect);
        let sheet = Rect2D::new(pt(0.0, 0.0).expect("origin"), frame.sheet()).expect("sheet");
        if self.frame_editor.needs_fit && rect.width() > 40.0 && rect.height() > 40.0 {
            self.frame_editor.camera.fit(sheet, rect.shrink(24.0));
            self.frame_editor.needs_fit = false;
        }
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0
                && let Some(cursor) = response.hover_pos()
            {
                self.frame_editor
                    .camera
                    .zoom_at((scroll as f64 / 200.0).exp(), cursor, rect);
            }
        }
        if response.dragged_by(PointerButton::Middle) {
            self.frame_editor.camera.pan(response.drag_delta());
        }
        let camera = self.frame_editor.camera;
        let grid = self.preferences.grid_mm;
        let snapping = self.preferences.snap_enabled;
        let world = |p: egui::Pos2| {
            camera
                .screen_to_world(p, rect)
                .map(|w| snap(w, grid, snapping))
        };
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0, theme::TITLE);
        if let Some(paper) = camera.clipped_rect(sheet, rect) {
            painter.rect_filled(paper, 0, theme::PAPER);
        }
        if self.preferences.grid_visible {
            for p in grid_points(&camera, rect, grid) {
                if camera
                    .screen_to_world(p, rect)
                    .is_some_and(|w| sheet.contains(w))
                {
                    painter.circle_filled(p, 0.8, theme::GRID_DOT);
                }
            }
        }
        // Usable area and zone divisions serve as editor guides.
        let u = frame.usable();
        if let Some(r) = camera.clipped_rect(u, rect) {
            painter.rect_stroke(
                r,
                0,
                Stroke::new(1.0_f32, theme::GRID),
                egui::StrokeKind::Inside,
            );
        }
        let Some(project) = self.session.project() else {
            return;
        };
        let (marks, rest): (Vec<_>, Vec<_>) = frame
            .elements()
            .iter()
            .enumerate()
            .partition(|(_, e)| matches!(e, FrameElement::Text { style, .. } | FrameElement::Special { style, .. } if style.watermark));
        let drag = self.frame_editor.drag;
        for (i, e) in marks.into_iter().chain(rest) {
            let selected = self.frame_editor.selection.contains(&i);
            let shown = match (selected, drag) {
                (true, Some((a, b))) => e
                    .translated(b.x() - a.x(), b.y() - a.y())
                    .unwrap_or(e.clone()),
                _ => e.clone(),
            };
            frame_view::paint_element(
                &painter,
                project,
                None,
                &shown,
                camera,
                rect,
                selected.then_some(theme::ACCENT),
            );
        }
        frame_view::paint_zone_labels(&painter, frame, camera, rect);
        // Pending points and preview.
        let cursor = response.hover_pos().and_then(world);
        let mut preview = self.frame_editor.pending.clone();
        if let Some(c) = cursor
            && self.frame_editor.tool != FrameTool::Select
        {
            preview.push(c);
        }
        if let Some(element) = self.build_tool_element(&preview, true) {
            frame_view::paint_element(
                &painter,
                project,
                None,
                &element,
                camera,
                rect,
                Some(theme::ACCENT),
            );
        }
        for p in &self.frame_editor.pending {
            if let Some(s) = camera.world_to_screen(*p, rect) {
                painter.circle_stroke(s, 3.0, Stroke::new(1.0_f32, theme::ACCENT));
            }
        }
        // Interaction.
        let ctrl = ui.input(|i| i.modifiers.command);
        let tool = self.frame_editor.tool;
        if tool == FrameTool::Select {
            // Dragging starts at the pointer press, rather than where egui detects motion.
            if response.drag_started_by(PointerButton::Primary)
                && let Some(start) = ui
                    .input(|i| i.pointer.press_origin())
                    .or(response.interact_pointer_pos())
                    .and_then(world)
            {
                let hit = self.hit(frame, start);
                if let Some(i) = hit {
                    if !self.frame_editor.selection.contains(&i) {
                        if !ctrl {
                            self.frame_editor.selection.clear();
                        }
                        self.frame_editor.selection.insert(i);
                    }
                    self.frame_editor.drag = Some((start, start));
                }
            }
            if let Some((start, _)) = self.frame_editor.drag
                && let Some(now) = response.interact_pointer_pos().and_then(world)
            {
                self.frame_editor.drag = Some((start, now));
            }
            if response.drag_stopped_by(PointerButton::Primary)
                && let Some((a, b)) = self.frame_editor.drag.take()
                && (a.x() != b.x() || a.y() != b.y())
            {
                self.editor_transform_selection(false, b.x() - a.x(), b.y() - a.y(), false);
            }
            if response.clicked_by(PointerButton::Primary)
                && let Some(p) = response.interact_pointer_pos().and_then(world)
            {
                match self.hit(frame, p) {
                    Some(i) if ctrl => {
                        if !self.frame_editor.selection.remove(&i) {
                            self.frame_editor.selection.insert(i);
                        }
                    }
                    Some(i) => self.frame_editor.selection = BTreeSet::from([i]),
                    None if !ctrl => self.frame_editor.selection.clear(),
                    None => {}
                }
            }
        } else if response.clicked_by(PointerButton::Primary)
            && let Some(p) = response.interact_pointer_pos().and_then(world)
        {
            self.frame_editor.pending.push(p);
            if self.frame_editor.pending.len() >= tool.clicks() {
                let points = std::mem::take(&mut self.frame_editor.pending);
                self.place_tool_element(&points);
            }
        }
        if !ui.ctx().wants_keyboard_input() && response.hovered() {
            if ui.input(|i| i.key_pressed(Key::Delete)) {
                self.editor_delete_selection();
            }
            if ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, Key::D)) {
                self.editor_transform_selection(true, 5.0, -5.0, false);
            }
        }
    }
}
impl CadApp {
    pub(in crate::frame_editor) fn sync_forms(&mut self, frame: &FrameTemplate) {
        let key = (
            frame.id(),
            frame.version(),
            self.frame_editor
                .selection
                .iter()
                .copied()
                .collect::<Vec<_>>(),
        );
        if self.frame_editor.form_key.as_ref() == Some(&key) {
            return;
        }
        self.frame_editor
            .selection
            .retain(|i| *i < frame.elements().len());
        let u = frame.usable();
        self.frame_editor.frame_form = FrameForm {
            name: frame.name().as_str().to_owned(),
            description: frame.description().to_owned(),
            sheet: [frame.sheet().width(), frame.sheet().height()],
            usable: [
                u.origin().x(),
                u.origin().y(),
                u.size().width(),
                u.size().height(),
            ],
            axes: [
                AxisForm::from(&frame.zones().columns),
                AxisForm::from(&frame.zones().rows),
            ],
        };
        if let [i] = key.2.as_slice()
            && let Some(e) = frame.elements().get(*i)
        {
            self.frame_editor.element_form = element_form(e);
        }
        self.frame_editor.form_key = Some(key);
    }

    pub(in crate::frame_editor) fn frame_properties_ui(
        &mut self,
        ui: &mut Ui,
        frame: &FrameTemplate,
    ) {
        self.sync_forms(frame);
        let selection: Vec<usize> = self.frame_editor.selection.iter().copied().collect();
        if let [i] = selection.as_slice()
            && let Some(e) = frame.elements().get(*i).cloned()
        {
            ui.heading(e.kind_label());
            let mut apply = false;
            element_form_ui(ui, &e, &mut self.frame_editor.element_form, &mut apply);
            if apply {
                let image_aspect = e
                    .asset()
                    .and_then(|a| self.session.project()?.asset(a))
                    .map(ImageAsset::aspect);
                match apply_element_form(&e, &self.frame_editor.element_form, image_aspect) {
                    Ok(new) => {
                        let mut draft = frame.draft();
                        draft.elements[*i] = new;
                        self.commit_frame_draft(draft, vec![]);
                    }
                    Err(e) => self.log(Level::Error, e),
                }
            }
            return;
        }
        if selection.len() > 1 {
            ui.label(format!("{} elementos selecionados.", selection.len()));
            ui.weak("Use Duplicar, Excluir ou Alinhar à grade, ou arraste para mover.");
            ui.separator();
        }
        ui.heading("Moldura");
        let form = &mut self.frame_editor.frame_form;
        let mut apply = false;
        egui::Grid::new("frame_form").num_columns(2).show(ui, |ui| {
            ui.label("Nome");
            ui.add(egui::TextEdit::singleline(&mut form.name).desired_width(140.0));
            ui.end_row();
            ui.label("Descrição");
            ui.add(
                egui::TextEdit::multiline(&mut form.description)
                    .desired_width(140.0)
                    .desired_rows(3),
            );
            ui.end_row();
            ui.label("Formato");
            let current = Size2D::new(form.sheet[0], form.sheet[1])
                .ok()
                .and_then(PaperFormat::matching)
                .map(|(f, l)| format!("{} {}", f.label(), if l { "paisagem" } else { "retrato" }))
                .unwrap_or_else(|| "Personalizado".into());
            egui::ComboBox::from_id_salt("frame_sheet_format")
                .width(120.0)
                .selected_text(current)
                .show_ui(ui, |ui| {
                    for f in PaperFormat::ALL {
                        for landscape in [true, false] {
                            let label = format!(
                                "{} {}",
                                f.label(),
                                if landscape { "paisagem" } else { "retrato" }
                            );
                            if ui.selectable_label(false, label).clicked() {
                                let s = f.size(landscape);
                                form.sheet = [s.width(), s.height()];
                            }
                        }
                    }
                });
            ui.end_row();
            for (label, value) in [("Largura da folha", 0usize), ("Altura da folha", 1)] {
                ui.label(label);
                ui.add(
                    egui::DragValue::new(&mut form.sheet[value])
                        .speed(1.0)
                        .suffix(" mm"),
                );
                ui.end_row();
            }
            for (label, value) in [
                ("Área útil X", 0usize),
                ("Área útil Y", 1),
                ("Área útil largura", 2),
                ("Área útil altura", 3),
            ] {
                ui.label(label);
                ui.add(
                    egui::DragValue::new(&mut form.usable[value])
                        .speed(0.5)
                        .suffix(" mm"),
                );
                ui.end_row();
            }
        });
        for (axis, title, start_side, end_side) in [
            (0usize, "Colunas", "Rótulos em cima", "Rótulos embaixo"),
            (1, "Linhas", "Rótulos à esquerda", "Rótulos à direita"),
        ] {
            ui.separator();
            ui.strong(title);
            let a = &mut form.axes[axis];
            egui::Grid::new(("frame_axis", axis))
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Quantidade");
                    ui.add(egui::DragValue::new(&mut a.count).range(0..=200));
                    ui.end_row();
                    ui.label("Tamanho uniforme");
                    ui.add(egui::DragValue::new(&mut a.size).speed(0.5).suffix(" mm"));
                    ui.end_row();
                    ui.label("Individuais (;)");
                    ui.add(egui::TextEdit::singleline(&mut a.custom).desired_width(140.0));
                    ui.end_row();
                    ui.label("Rótulo");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut a.letters, false, "1, 2, 3");
                        ui.selectable_value(&mut a.letters, true, "A, B, C");
                    });
                    ui.end_row();
                    ui.label("Início");
                    ui.add(egui::DragValue::new(&mut a.start).range(0..=999));
                    ui.end_row();
                });
            ui.checkbox(&mut a.labels_start, start_side);
            ui.checkbox(&mut a.labels_end, end_side);
        }
        ui.add_space(6.0);
        if ui.button("Aplicar moldura").clicked() {
            apply = true;
        }
        if apply {
            match build_frame_draft(frame, &self.frame_editor.frame_form) {
                Ok(draft) => {
                    self.commit_frame_draft(draft, vec![]);
                }
                Err(e) => self.log(Level::Error, e),
            }
        }
    }

    // File commands.
}
