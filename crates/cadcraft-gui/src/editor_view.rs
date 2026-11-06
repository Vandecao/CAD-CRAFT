// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! egui input arbitration and context menus. Geometry and history are independent.
use crate::{
    app::{CadApp, Level},
    camera::Camera,
    commands::CommandId,
    editing::GraphicEdit,
    hit_test::hit_candidates,
    interaction::{Gesture, Tool},
};
use cad_craft::domain::*;
use eframe::egui::{self, CursorIcon, PointerButton, Rect, Response, Stroke, StrokeKind, Ui};
impl CadApp {
    pub(crate) fn interact_editor(
        &mut self,
        ui: &mut Ui,
        response: &Response,
        page: PageId,
        camera: Camera,
        rect: Rect,
    ) {
        let blocked = self.dialogs_state.dialog.is_some()
            || self.dialogs_state.backstage.is_some()
            || ui.input(|i| {
                !i.focused
                    || i.events
                        .iter()
                        .any(|e| matches!(e, egui::Event::WindowFocused(false)))
            });
        let middle = ui.input(|i| i.pointer.button_down(PointerButton::Middle));
        if blocked || middle {
            self.session.graphics.cancel();
            self.session.graphics.hover = None;
            return;
        }
        let cursor = ui.input(|i| i.pointer.interact_pos());
        let ctrl = ui.input(|i| i.modifiers.ctrl);
        if matches!(
            self.session.graphics.tool,
            Tool::InsertGraphic(_) | Tool::InsertText | Tool::Paste | Tool::InsertMacro
        ) {
            if response.secondary_clicked() {
                self.session.graphics.cancel();
                return;
            }
            if response.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
                if let Some(world) = cursor.and_then(|c| camera.screen_to_world(c, rect)) {
                    let position = crate::camera::snap(
                        world,
                        self.preferences.grid_mm,
                        self.preferences.snap_enabled,
                    );
                    let tool = self.session.graphics.tool;
                    let faint = Stroke::new(1.5_f32, crate::theme::ACCENT.linear_multiply(0.65));
                    // Paint previews above the sheet after this interaction, clipped to the editor area.
                    let overlay = ui
                        .ctx()
                        .layer_painter(egui::LayerId::new(
                            egui::Order::Foreground,
                            egui::Id::new("editor_preview"),
                        ))
                        .with_clip_rect(rect);
                    match tool {
                        Tool::InsertText => {
                            if let Ok(graphic) = PageGraphic::new(
                                SymbolPrimitive::Text {
                                    origin: Point2D::new(0.0, 0.0).expect("origin"),
                                    text: self.free_content.free_text.clone(),
                                    height: self.free_content.free_height,
                                },
                                self.free_content.free_stroke,
                                self.free_content.free_align,
                            ) {
                                let transform =
                                    Transform2D::new(position, 0.0, false, false).expect("cursor");
                                if let Ok(primitive) = graphic.drawable() {
                                    crate::symbol_view::paint_primitive(
                                        &overlay, &primitive, transform, camera, rect, faint,
                                    );
                                }
                                if response.clicked_by(PointerButton::Primary) {
                                    if let Err(e) = self.session.insert_graphic(graphic, transform)
                                    {
                                        self.log(Level::Error, e.to_string());
                                    }
                                    self.session.graphics.tool = Tool::InsertText;
                                }
                            }
                        }
                        Tool::InsertGraphic(kind) => {
                            if let Some(start) = self.session.graphics.graphic_start {
                                let local_start = Point2D::new(0.0, 0.0).expect("origin");
                                if let Ok(local_end) =
                                    Point2D::new(position.x() - start.x(), position.y() - start.y())
                                    && let Ok(graphic) = PageGraphic::from_drag(
                                        kind,
                                        local_start,
                                        local_end,
                                        self.free_content.free_stroke,
                                    )
                                {
                                    let transform =
                                        Transform2D::new(start, 0.0, false, false).expect("cursor");
                                    if let Ok(primitive) = graphic.drawable() {
                                        crate::symbol_view::paint_primitive(
                                            &overlay, &primitive, transform, camera, rect, faint,
                                        );
                                    }
                                    if response.clicked_by(PointerButton::Primary) {
                                        if let Err(e) =
                                            self.session.insert_graphic(graphic, transform)
                                        {
                                            self.log(Level::Error, e.to_string());
                                        }
                                        self.session.graphics.tool = Tool::InsertGraphic(kind);
                                        self.session.graphics.graphic_start = None;
                                    }
                                }
                            } else if response.clicked_by(PointerButton::Primary) {
                                self.session.graphics.graphic_start = Some(position);
                            }
                        }
                        Tool::Paste => {
                            if let Ok(preview) = self.session.clipboard_preview(position) {
                                for r in preview {
                                    if let Some(g) = r.graphic()
                                        && let Ok(primitive) = g.drawable()
                                    {
                                        crate::symbol_view::paint_primitive(
                                            &overlay,
                                            &primitive,
                                            r.transform(),
                                            camera,
                                            rect,
                                            faint,
                                        );
                                    } else if let Some(d) = r.symbol().and_then(|id| {
                                        self.session.project().and_then(|p| p.symbol_definition(id))
                                    }) {
                                        crate::symbol_view::paint_symbol(
                                            &overlay,
                                            d,
                                            r.transform(),
                                            camera,
                                            rect,
                                            faint,
                                        );
                                    }
                                }
                            }
                            if response.clicked_by(PointerButton::Primary) {
                                if let Err(e) = self.session.confirm_paste(position) {
                                    self.log(Level::Error, e.to_string());
                                }
                                self.session.graphics.tool = Tool::Paste;
                            }
                        }
                        Tool::InsertMacro => {
                            let Some(draft) = self.session.graphics.macro_draft.clone() else {
                                self.session.graphics.cancel();
                                return;
                            };
                            let Ok(transform) = draft.transform(position) else {
                                return;
                            };
                            paint_macro_preview(
                                &overlay,
                                &draft.definition,
                                transform,
                                camera,
                                rect,
                                faint,
                            );
                            if response.clicked_by(PointerButton::Primary) {
                                match self.session.insert_macro(&draft.definition, transform) {
                                    Ok(report) => {
                                        let summary = report.summary();
                                        self.log(
                                            if summary.is_empty() {
                                                Level::Info
                                            } else {
                                                Level::Warning
                                            },
                                            format!(
                                                "Macro \"{}\" inserida. {summary} Clique para repetir; Esc encerra.",
                                                draft.definition.name().as_str()
                                            ),
                                        );
                                    }
                                    Err(e) => self.log(Level::Error, e.to_string()),
                                }
                                self.session.graphics.tool = Tool::InsertMacro;
                                self.session.graphics.macro_draft = Some(draft);
                            }
                        }
                        _ => {}
                    }
                }
            }
            return;
        }
        if matches!(
            self.session.graphics.tool,
            Tool::InsertInterruption | Tool::InsertPotential
        ) {
            if response.secondary_clicked() {
                self.session.graphics.cancel();
                return;
            }
            if response.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
                if let Some(world) = cursor.and_then(|c| camera.screen_to_world(c, rect)) {
                    let position = crate::camera::snap(
                        world,
                        self.preferences.grid_mm,
                        self.preferences.snap_enabled,
                    );
                    let potential = self.session.graphics.tool == Tool::InsertPotential;
                    if let Ok(definition) = crate::references::marker_definition(potential) {
                        crate::symbol_view::paint_symbol(
                            ui.painter(),
                            &definition,
                            Transform2D::new(position, 0., false, false).expect("finite snap"),
                            camera,
                            rect,
                            Stroke::new(1.5_f32, crate::theme::ACCENT),
                        );
                    }
                    if response.clicked_by(PointerButton::Primary) {
                        let kind = if potential {
                            MarkerKind::Potential {
                                kind: None,
                                description: String::new(),
                            }
                        } else {
                            MarkerKind::Interruption
                        };
                        let name = Text::new(if potential {
                            "Potencial"
                        } else {
                            "Interrupção"
                        })
                        .expect("static label");
                        match self
                            .session
                            .insert_electrical_marker(kind, name, None, position)
                        {
                            Ok(_) => self.open_reference_properties(),
                            Err(e) => self.log(Level::Error, e.to_string()),
                        }
                    }
                }
            }
            return;
        }
        if self.session.graphics.insertion.is_some() {
            self.session.graphics.hover = None;
            if response.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
                if let Some(world) = cursor.and_then(|c| camera.screen_to_world(c, rect)) {
                    if let Err(e) = self.session.graphics.update_insertion(
                        world,
                        self.preferences.grid_mm,
                        self.preferences.snap_enabled,
                    ) {
                        self.log(Level::Error, e.to_string());
                    }
                    if response.clicked_by(PointerButton::Primary) {
                        match self.session.confirm_symbol_insertion() {
                            Ok(_) => self.log(
                                Level::Info,
                                "Símbolo inserido. Clique para repetir; Esc encerra.",
                            ),
                            Err(e) => self.log(Level::Error, e.to_string()),
                        }
                    }
                }
            } else if let Some(draft) = &mut self.session.graphics.insertion {
                draft.positioned = false;
            }
            if response.secondary_clicked() {
                self.session.graphics.cancel();
            }
            return;
        }
        if self.interact_connections(ui, response, page, camera, rect) {
            return;
        }
        let hovered = response
            .hovered()
            .then_some(cursor)
            .flatten()
            .and_then(|cursor| {
                self.session.project().and_then(|p| {
                    hit_candidates(p, page, camera, rect, cursor)
                        .first()
                        .copied()
                })
            });
        self.session.graphics.hover = hovered;
        if response.drag_started_by(PointerButton::Primary)
            && let Some(start) = ui.input(|i| i.pointer.press_origin())
            && let Some(world) = camera.screen_to_world(start, rect)
        {
            let hit = self.session.project().and_then(|p| {
                hit_candidates(p, page, camera, rect, start)
                    .first()
                    .copied()
            });
            if let Some(id) = hit {
                if !ctrl {
                    // Temporarily take only transient state, never clone the Project.
                    let mut interaction = std::mem::take(&mut self.session.graphics);
                    let result = interaction.begin_move(
                        self.session.project().expect("active page"),
                        page,
                        id,
                        world,
                    );
                    self.session.graphics = interaction;
                    self.session.sync_graphic_selection();
                    if let Err(e) = result {
                        self.log(Level::Error, e.to_string());
                    }
                }
            } else {
                self.session.graphics.begin_rectangle(world, ctrl);
            }
        }
        if self.session.graphics.is_active()
            && let Some(world) = cursor.and_then(|c| camera.screen_to_world(c, rect))
        {
            self.session.graphics.update_rectangle(world);
            if let Err(e) = self.session.graphics.update_move(
                world,
                self.preferences.grid_mm,
                self.preferences.snap_enabled,
            ) {
                self.log(Level::Error, e.to_string());
            }
        }
        if response.drag_stopped_by(PointerButton::Primary) {
            if matches!(self.session.graphics.gesture, Some(Gesture::Move(_))) {
                if let Some(batch) = self.session.graphics.finish_move() {
                    self.graphic_edit(GraphicEdit::Set(batch));
                }
            } else {
                let mut interaction = std::mem::take(&mut self.session.graphics);
                interaction.finish_rectangle(self.session.project().expect("active page"), page);
                self.session.graphics = interaction;
                self.session.sync_graphic_selection();
            }
        }
        if response.clicked_by(PointerButton::Primary) {
            self.session.graphics.click(hovered, ctrl);
            self.session.sync_graphic_selection();
        }
        // Double-clicking free text opens its text editor.
        if response.double_clicked_by(PointerButton::Primary)
            && let Some(id) = hovered
            && let Some(SymbolPrimitive::Text { text, .. }) = self
                .session
                .project()
                .and_then(|p| p.representation(id))
                .and_then(|r| r.graphic())
                .map(|g| g.primitive().clone())
        {
            self.dialogs_state.dialog = Some(crate::dialogs::Dialog::EditFreeText { id, text });
        }
        if response.secondary_clicked() {
            self.chrome.context_object = hovered;
            self.session.graphics.cancel();
            if let Some(id) = hovered
                && !self.session.graphics.selection.ids().contains(&id)
            {
                self.session.select_graphics([id]);
            }
        }
        if response.hovered() {
            let cursor = if matches!(self.session.graphics.gesture, Some(Gesture::Move(_)))
                || self.session.graphics.tool == Tool::Move
            {
                CursorIcon::Move
            } else if hovered.is_some() {
                CursorIcon::PointingHand
            } else {
                CursorIcon::Default
            };
            ui.ctx().set_cursor_icon(cursor);
        }
        // Menu position is stable while open, independent of current hover.
        let menu_object = self.chrome.context_object;
        response.context_menu(|ui| {
            let commands: &[CommandId] = if menu_object.is_some_and(|id| {
                self.session
                    .project()
                    .is_some_and(|p| p.connection_route(id).is_some())
            }) {
                &[
                    CommandId::Select,
                    CommandId::ShowProperties,
                    CommandId::EditConnection,
                    CommandId::AddConnectionVertex,
                    CommandId::RemoveConnectionVertex,
                    CommandId::DeleteConnection,
                    CommandId::HighlightNetwork,
                    CommandId::Copy,
                    CommandId::MacroCreate,
                    CommandId::Undo,
                    CommandId::Redo,
                ]
            } else if menu_object.is_some() {
                &[
                    CommandId::Select,
                    CommandId::ShowProperties,
                    CommandId::Move,
                    CommandId::Rotate90,
                    CommandId::Rotate180,
                    CommandId::Rotate270,
                    CommandId::MirrorHorizontal,
                    CommandId::MirrorVertical,
                    CommandId::LocateReference,
                    CommandId::Copy,
                    CommandId::Cut,
                    CommandId::DeleteFree,
                    CommandId::MacroCreate,
                    CommandId::Undo,
                    CommandId::Redo,
                ]
            } else {
                &[
                    CommandId::FitPage,
                    CommandId::FitContent,
                    CommandId::ToggleGrid,
                    CommandId::ToggleSnap,
                    CommandId::PageProperties,
                    CommandId::Paste,
                ]
            };
            for &id in commands {
                if self.command_button(ui, id, false, false) {
                    self.execute(id, ui.ctx());
                    ui.close();
                }
            }
            if menu_object.is_none() {
                ui.separator();
                ui.label("Grade / snap (mm)");
                ui.add(
                    egui::DragValue::new(&mut self.preferences.grid_mm)
                        .range(0.1..=1000.0)
                        .speed(0.1),
                );
            }
        });
    }
    pub(crate) fn paint_selection_window(
        &self,
        painter: &egui::Painter,
        camera: Camera,
        rect: Rect,
    ) {
        if let Some(Gesture::Rectangle { start, current, .. }) = self.session.graphics.gesture
            && let (Some(a), Some(b)) = (
                camera.world_to_screen(start, rect),
                camera.world_to_screen(current, rect),
            )
        {
            let crossing = current.x() < start.x();
            let color = if crossing {
                egui::Color32::from_rgb(40, 180, 120)
            } else {
                crate::theme::ACCENT
            };
            let area = Rect::from_two_pos(a, b).intersect(rect);
            painter.rect(
                area,
                0,
                color.linear_multiply(0.12),
                Stroke::new(1.0_f32, color),
                StrokeKind::Inside,
            );
            painter.text(
                area.left_top() + egui::vec2(4.0, 4.0),
                egui::Align2::LEFT_TOP,
                if crossing {
                    "Cruzamento"
                } else {
                    "Contenção"
                },
                egui::FontId::proportional(11.0),
                color,
            );
        }
    }
}

impl CadApp {
    /// Replace only free-text content in one transaction, preserving height, stroke and alignment.
    pub(crate) fn edit_free_text(
        &mut self,
        id: RepresentationId,
        text: &str,
    ) -> Result<bool, String> {
        let graphic = self
            .session
            .project()
            .and_then(|p| p.representation(id))
            .and_then(|r| r.graphic())
            .cloned()
            .ok_or_else(|| "texto livre inexistente".to_owned())?;
        let SymbolPrimitive::Text { origin, height, .. } = graphic.primitive().clone() else {
            return Err("o item não é um texto livre".to_owned());
        };
        let primitive = SymbolPrimitive::Text {
            origin,
            text: text.to_owned(),
            height,
        };
        PageGraphic::new(primitive, graphic.stroke_mm(), graphic.alignment())
            .and_then(|g| g.with_style(graphic.style()))
            .and_then(|g| self.session.edit_free_graphic(id, g))
            .map_err(|e| e.to_string())
    }
}

/// Paint the macro being positioned, including symbols, free graphics and wires; used by insertion and thumbnails.
pub(crate) fn paint_macro_preview(
    painter: &egui::Painter,
    definition: &MacroDefinition,
    transform: Transform2D,
    camera: Camera,
    rect: Rect,
    stroke: Stroke,
) {
    let content = definition.content();
    let Ok((reps, routes)) = content.preview(definition.base(), transform) else {
        return;
    };
    for r in &reps {
        if let Some(g) = r.graphic()
            && let Ok(primitive) = g.drawable()
        {
            crate::symbol_view::paint_primitive(
                painter,
                &primitive,
                r.transform(),
                camera,
                rect,
                stroke,
            );
        } else if let Some(d) = r.symbol().and_then(|id| content.definition(id)) {
            crate::symbol_view::paint_symbol(painter, d, r.transform(), camera, rect, stroke);
        }
    }
    for points in &routes {
        for pair in points.windows(2) {
            if let Some(segment) = camera.clipped_segment(pair[0], pair[1], rect) {
                painter.line_segment(segment, stroke);
            }
        }
    }
}
