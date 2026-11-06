// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Application frames, shortcuts and command widgets; start with CadApp::show.
use super::*;

impl CadApp {
    pub fn show(&mut self, ctx: &Context) {
        // Reload the symbol library when its directory changes.
        let folder = self.preferences.symbol_library.trim().to_owned();
        if self.chrome.symbol_library_seen.as_deref() != Some(folder.as_str()) {
            self.chrome.symbol_library_seen = Some(folder.clone());
            if !folder.is_empty() {
                self.reload_symbol_library();
            }
        }
        let system_theme = ctx.input(|i| i.raw.system_theme);
        if !self.chrome.theme_ready {
            theme::apply(ctx);
            self.chrome.theme_ready = true;
        } else if system_theme != self.chrome.last_system_theme
            || ctx.input(|i| {
                i.events
                    .iter()
                    .any(|e| matches!(e, egui::Event::WindowFocused(true)))
            })
        {
            // winit 0.30.13 may reset the caption on WM_SETTINGCHANGE without
            // emitting ThemeChanged. Reassert on focus as well, never every frame.
            theme::request_native_dark(ctx);
        }
        self.chrome.last_system_theme = system_theme;
        self.preferences.sanitize();
        if self.dialogs_state.dialog.is_some()
            || self.dialogs_state.backstage.is_some()
            || ctx.input(|i| {
                !i.focused
                    || i.events
                        .iter()
                        .any(|e| matches!(e, egui::Event::WindowFocused(false)))
            })
        {
            self.session.graphics.cancel();
            self.session.graphics.hover = None;
        }
        if self.navigation.index_revision != self.session.revision {
            self.navigation.index = self
                .session
                .project()
                .map(PageIndex::build)
                .unwrap_or_default();
            self.navigation.index_revision = self.session.revision;
            self.navigator.invalidate();
        }
        if ctx.input(|i| i.viewport().close_requested())
            && self.session.dirty
            && !self.chrome.exit_confirmed
        {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.dialogs_state.dialog = Some(Dialog::Confirm(PendingAction::Exit));
        }
        self.shortcuts(ctx);
        self.autosave();
        let title = if let Some(p) = self.session.project() {
            format!(
                "CAD-CRAFT — {}{}{}{}",
                p.name().as_str(),
                self.session
                    .file
                    .as_ref()
                    .map(|f| format!(
                        " [{}{}]",
                        f.file_name(),
                        if f.read_only { ", somente leitura" } else { "" }
                    ))
                    .unwrap_or_default(),
                self.session
                    .active
                    .and_then(|id| p.page(id))
                    .map(|p| format!(" — {}", p.number().as_str()))
                    .unwrap_or_default(),
                if self.session.dirty { " *" } else { "" }
            )
        } else {
            "CAD-CRAFT — Nenhum projeto aberto".into()
        };
        ctx.send_viewport_cmd(ViewportCommand::Title(title));
        self.top_bar(ctx);
        if self.dialogs_state.backstage.is_some() {
            self.show_backstage(ctx);
        } else {
            self.show_ribbon(ctx);
            self.global_status(ctx);
            self.show_messages(ctx);
            // Shared width budget leaves at least 240 logical points for the CAD area.
            let screen = ctx.content_rect();
            let visible = [PanelId::Navigator, PanelId::Insertion, PanelId::Properties]
                .into_iter()
                .filter(|p| self.preferences.panel(*p).visible)
                .count()
                .max(1) as f32;
            let max_width = ((screen.width() - 240.0) / visible).max(65.0);
            for panel in [PanelId::Properties, PanelId::Insertion, PanelId::Navigator] {
                if !self.preferences.panel(panel).visible {
                    continue;
                }
                let pref = self.preferences.panel(panel);
                let collapsed = pref.collapsed;
                let width = if collapsed {
                    30.0
                } else {
                    pref.width.min(max_width)
                };
                let id = egui::Id::new(("side", panel, self.chrome.layout_generation));
                let side = if panel == PanelId::Navigator {
                    egui::SidePanel::left(id)
                } else {
                    egui::SidePanel::right(id)
                };
                let output = side
                    .resizable(!collapsed)
                    .min_width(if collapsed { 30.0 } else { 65.0 })
                    .max_width(if collapsed { 30.0 } else { max_width })
                    .default_width(width)
                    .frame(egui::Frame::new().fill(theme::PANEL).inner_margin(4))
                    .show(ctx, |ui| {
                        if collapsed {
                            if ui
                                .button("»")
                                .on_hover_text(format!("Expandir {}", panel.label()))
                                .clicked()
                            {
                                self.preferences.panel_mut(panel).collapsed = false;
                            }
                            return;
                        }
                        match panel {
                            PanelId::Navigator => self.show_navigator(ui),
                            PanelId::Insertion => self.show_insertion(ui),
                            PanelId::Properties => self.show_properties(ui),
                            _ => {}
                        }
                    });
                if !collapsed {
                    self.preferences.panel_mut(panel).width = output.response.rect.width();
                }
            }
            egui::CentralPanel::default()
                .frame(egui::Frame::new().fill(theme::TITLE).inner_margin(0))
                .show(ctx, |ui| {
                    if self.frame_editor.is_open() {
                        self.show_frame_editor(ui);
                    } else if self.symbol_editor.is_open() {
                        self.show_symbol_editor(ui);
                    } else {
                        self.show_documents(ui);
                    }
                });
        }
        self.show_dialog(ctx);
        let mut numbering = std::mem::take(&mut self.dialogs_state.numbering);
        numbering.show(&mut self.session, ctx);
        self.dialogs_state.numbering = numbering;
        self.reports.show(&mut self.session, &mut self.files, ctx);
        self.poll_qet(ctx);
        self.poll_assistant(ctx);
        self.poll_edz(ctx);
        self.show_edz(ctx);
        self.show_checks(ctx);
        self.show_pdf_window(ctx);
        let mut physical_editor = std::mem::take(&mut self.panels_state.physical_editor);
        physical_editor.show(&mut self.session, ctx);
        let locate = physical_editor.take_locate();
        self.panels_state.physical_editor = physical_editor;
        let mut part_editor = std::mem::take(&mut self.part_editor);
        part_editor.show(&mut self.session, &self.preferences, ctx);
        let part_locate = part_editor.take_locate();
        self.part_editor = part_editor;
        let mut macro_library = std::mem::take(&mut self.macro_library);
        macro_library.show(&mut self.session, &self.preferences, ctx);
        let macro_insert = macro_library.take_insert();
        self.macro_library = macro_library;
        if let Some(definition) = macro_insert {
            self.start_macro_insertion(definition);
        }
        if let Some(id) = part_locate {
            self.navigate_to_reference(id);
        }
        if let Some(id) = locate {
            self.navigate_to_reference(id);
        }
    }
    fn shortcuts(&mut self, ctx: &Context) {
        // Navigate validation messages.
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, Key::F8)) {
            self.step_check(-1);
        } else if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::F8)) {
            self.step_check(1);
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, Key::F1)) {
            self.preferences.ribbon_collapsed = !self.preferences.ribbon_collapsed;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::F11)) {
            self.chrome.fullscreen = !self.chrome.fullscreen;
            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(self.chrome.fullscreen));
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape)) {
            if self.frame_editor.is_open()
                && self.dialogs_state.dialog.is_none()
                && self.frame_editor.cancel()
            {
                return;
            }
            if self.symbol_editor.is_open() && self.dialogs_state.dialog.is_none() {
                self.symbol_editor.cancel();
                return;
            }
            if self.session.graphics.is_active()
                || self.session.graphics.tool == crate::interaction::Tool::Move
            {
                self.session.graphics.cancel();
            } else if self.dialogs_state.dialog.is_some() {
                self.dialogs_state.dialog = None;
            } else if !self.chrome.command_query.is_empty() {
                self.chrome.command_query.clear();
            } else if self.dialogs_state.backstage.is_some() {
                self.dialogs_state.backstage = None;
            } else {
                self.session
                    .select(self.session.active.map_or(Selection::None, Selection::Page));
            }
            return;
        }
        if self.dialogs_state.dialog.is_some()
            || self.dialogs_state.backstage.is_some()
            || ctx.wants_keyboard_input()
        {
            return;
        }
        // The symbol editor handles Ctrl+Z/Ctrl+Y/Delete on its own canvas.
        if self.symbol_editor.is_open() {
            return;
        }
        if self.frame_editor.is_open() {
            for (key, command) in [(Key::Z, CommandId::Undo), (Key::Y, CommandId::Redo)] {
                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, key)) {
                    self.execute(command, ctx);
                }
            }
            return;
        }
        // Ctrl+Shift+S before Ctrl+S: consume_key matches modifiers exactly.
        for (modifiers, key, command) in [
            (egui::Modifiers::CTRL, Key::Z, CommandId::Undo),
            (egui::Modifiers::CTRL, Key::Y, CommandId::Redo),
            (
                egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                Key::S,
                CommandId::SaveProjectAs,
            ),
            (egui::Modifiers::CTRL, Key::S, CommandId::SaveProject),
            (egui::Modifiers::CTRL, Key::O, CommandId::OpenProject),
            (egui::Modifiers::CTRL, Key::C, CommandId::Copy),
            (egui::Modifiers::CTRL, Key::X, CommandId::Cut),
            (egui::Modifiers::CTRL, Key::V, CommandId::Paste),
            (egui::Modifiers::CTRL, Key::D, CommandId::Duplicate),
            (egui::Modifiers::NONE, Key::Delete, CommandId::DeleteFree),
        ] {
            if ctx.input_mut(|i| i.consume_key(modifiers, key)) {
                self.execute(command, ctx);
            }
        }
        // On Windows/winit, Ctrl+C/X/V arrive as clipboard events (Copy/Cut/Paste), rather than keys. Paste requires text in the system clipboard; see copy_marker.
        let clipboard_commands: Vec<CommandId> = ctx.input_mut(|i| {
            let mut found = Vec::new();
            i.events.retain(|e| {
                let command = match e {
                    egui::Event::Copy => CommandId::Copy,
                    egui::Event::Cut => CommandId::Cut,
                    egui::Event::Paste(_) => CommandId::Paste,
                    _ => return true,
                };
                found.push(command);
                false
            });
            found
        });
        for command in clipboard_commands {
            self.execute(command, ctx);
        }
        for (key, dir) in [(Key::PageUp, -1isize), (Key::PageDown, 1)] {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, key))
                && !self.navigation.index.rows.is_empty()
            {
                let index = self
                    .navigation
                    .index
                    .rows
                    .iter()
                    .position(|r| Some(r.id) == self.session.active)
                    .unwrap_or(0);
                let next = index
                    .saturating_add_signed(dir)
                    .min(self.navigation.index.rows.len() - 1);
                let id = self.navigation.index.rows[next].id;
                self.open_page(id);
            }
        }
    }
    /// Whether a modal dialog is open; also used by GUI tests.
    pub(crate) fn toggle_panel(&mut self, id: PanelId) {
        let p = self.preferences.panel_mut(id);
        p.visible = !p.visible;
        if p.visible {
            p.collapsed = false;
        }
    }
    pub(crate) fn command_button(
        &self,
        ui: &mut Ui,
        id: CommandId,
        large: bool,
        icon_only: bool,
    ) -> bool {
        let Some(c) = self.chrome.commands.iter().find(|c| c.id == id) else {
            return false;
        };
        let tooltip = format!(
            "{}\n{}{}",
            c.name,
            c.description,
            c.reason(&self.session)
                .map(|s| format!("\n{s}"))
                .unwrap_or_default()
        );
        crate::icons::button(
            ui,
            c.icon,
            if icon_only { "" } else { c.name },
            &tooltip,
            c.available(&self.session),
            self.assistant.highlight == Some(id),
            large,
        )
        .clicked()
    }
}
