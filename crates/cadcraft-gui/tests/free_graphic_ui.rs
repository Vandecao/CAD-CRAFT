// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Free graphics driven by real egui events: tools, selection, movement, text editing, deletion, keyboard copy/paste and save/reopen.
// Fixture helpers use validated synthetic inputs and fail immediately on setup errors.
#![allow(clippy::unwrap_used)]
use cad_craft::domain::*;
use cad_gui::{app::CadApp, commands::CommandId, project_file::ScriptedPicker};
use eframe::egui::{self, Context, Event, Key, Modifiers, PointerButton, Pos2, Rect, vec2};

struct Ui {
    app: CadApp,
    ctx: Context,
    time: f64,
    page: PageId,
}
impl Ui {
    fn new() -> Self {
        let mut app = CadApp::default();
        app.session
            .replace(cad_gui::demo::project().unwrap(), true)
            .unwrap();
        let page = app.session.project().unwrap().pages().next().unwrap().id();
        app.session.open_page(page).unwrap();
        let mut ui = Self {
            app,
            ctx: Context::default(),
            time: 0.0,
            page,
        };
        for _ in 0..3 {
            ui.frame(vec![]);
        }
        ui
    }
    fn frame(&mut self, events: Vec<Event>) {
        self.time += 0.05;
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1600.0, 900.0))),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        let app = &mut self.app;
        let _ = self.ctx.run(input, |ctx| app.show(ctx));
    }
    fn button(&self, pressed: bool, pos: Pos2) -> Event {
        Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        }
    }
    fn screen(&self, x: f64, y: f64) -> Pos2 {
        self.app.session.tabs[0]
            .camera
            .world_to_screen(Point2D::new(x, y).unwrap(), self.app.last_viewport.unwrap())
            .unwrap()
    }
    fn click(&mut self, x: f64, y: f64) {
        let pos = self.screen(x, y);
        let (down, up) = (self.button(true, pos), self.button(false, pos));
        self.frame(vec![Event::PointerMoved(pos), down]);
        self.frame(vec![up]);
    }
    fn double_click(&mut self, x: f64, y: f64) {
        let pos = self.screen(x, y);
        let (down, up) = (self.button(true, pos), self.button(false, pos));
        self.frame(vec![Event::PointerMoved(pos), down.clone()]);
        self.frame(vec![up.clone()]);
        self.frame(vec![down]);
        self.frame(vec![up]);
        self.frame(vec![]);
    }
    fn drag(&mut self, from: (f64, f64), to: (f64, f64)) {
        let (a, b) = (self.screen(from.0, from.1), self.screen(to.0, to.1));
        self.frame(vec![Event::PointerMoved(a), self.button(true, a)]);
        for i in 1..=6 {
            let t = i as f32 / 6.0;
            self.frame(vec![Event::PointerMoved(a + (b - a) * t)]);
        }
        let up = self.button(false, b);
        self.frame(vec![up]);
        self.frame(vec![]);
    }
    fn key(&mut self, key: Key, modifiers: Modifiers) {
        let ev = |pressed| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers,
        };
        self.frame(vec![ev(true)]);
        self.frame(vec![ev(false)]);
    }
    fn free(&self) -> Vec<Representation> {
        self.app
            .session
            .project()
            .unwrap()
            .representations_on_page(self.page)
            .unwrap()
            .into_iter()
            .filter(|r| r.graphic().is_some())
            .cloned()
            .collect()
    }
    fn select_tool(&mut self) {
        self.app.execute(CommandId::Select, &self.ctx.clone());
        self.frame(vec![]);
    }
}

#[test]
fn real_clicks_preview_then_insert_line_and_text() {
    let mut ui = Ui::new();
    ui.app.execute(CommandId::InsertLine, &ui.ctx.clone());
    let before = ui.app.session.snapshot().unwrap();
    ui.click(20.0, 20.0);
    let second = ui.screen(44.0, 36.0);
    ui.frame(vec![Event::PointerMoved(second)]);
    assert_eq!(
        ui.app.session.snapshot().unwrap(),
        before,
        "prévia não altera"
    );
    ui.click(44.0, 36.0);
    assert_eq!(ui.free().len(), 1);
    ui.app.execute(CommandId::InsertFreeText, &ui.ctx.clone());
    ui.click(20.0, 20.0);
    assert_eq!(ui.free().len(), 2);
    ui.app.session.project().unwrap().validate().unwrap();
}

#[test]
fn rectangle_circle_arc_select_move_and_delete_with_real_events() {
    let mut ui = Ui::new();
    for (command, a, b) in [
        (CommandId::InsertRectangle, (20.0, 20.0), (52.0, 40.0)),
        (CommandId::InsertCircle, (100.0, 40.0), (112.0, 40.0)),
        (CommandId::InsertArc, (160.0, 40.0), (172.0, 40.0)),
    ] {
        ui.app.execute(command, &ui.ctx.clone());
        ui.click(a.0, a.1);
        ui.click(b.0, b.1);
    }
    let kinds: Vec<_> = ui
        .free()
        .iter()
        .map(|r| match r.graphic().unwrap().primitive() {
            SymbolPrimitive::Rectangle { .. } => "ret",
            SymbolPrimitive::Circle { .. } => "cir",
            SymbolPrimitive::Arc { .. } => "arc",
            _ => "outro",
        })
        .collect();
    let mut sorted = kinds.clone();
    sorted.sort();
    assert_eq!(sorted, ["arc", "cir", "ret"]);
    let rect = ui
        .free()
        .into_iter()
        .find(|r| {
            matches!(
                r.graphic().unwrap().primitive(),
                SymbolPrimitive::Rectangle { .. }
            )
        })
        .unwrap();
    // Select by the border; the empty center is not selectable.
    ui.select_tool();
    ui.click(36.0, 30.0);
    assert!(ui.app.session.graphics.selection.ids().is_empty());
    ui.click(20.0, 32.0);
    assert_eq!(ui.app.session.graphics.selection.primary(), Some(rect.id()));
    // Move by dragging the border 12 mm right on the 4 mm grid.
    ui.drag((20.0, 32.0), (32.0, 32.0));
    let moved = ui
        .app
        .session
        .project()
        .unwrap()
        .representation(rect.id())
        .unwrap()
        .transform()
        .translation();
    let start = rect.transform().translation();
    assert!(
        ((moved.x() - start.x()) - 12.0).abs() < 1e-3 && (moved.y() - start.y()).abs() < 1e-3,
        "{start:?} → {moved:?}"
    );
    // Delete through the keyboard and undo.
    let before_delete = ui.app.session.snapshot().unwrap();
    ui.key(Key::Delete, Modifiers::NONE);
    assert_eq!(ui.free().len(), 2);
    ui.key(Key::Z, Modifiers::CTRL);
    assert_eq!(ui.app.session.snapshot().unwrap(), before_delete);
}

#[test]
fn double_click_edits_text_and_keyboard_copy_paste_and_duplicate() {
    let mut ui = Ui::new();
    ui.app.execute(CommandId::InsertFreeText, &ui.ctx.clone());
    ui.click(100.0, 100.0);
    let text = ui.free()[0].clone();
    let bounds = text.bounds().unwrap();
    let inside = (
        bounds.origin().x() + bounds.size().width() / 2.0,
        bounds.origin().y() + bounds.size().height() / 2.0,
    );
    ui.select_tool();
    ui.double_click(inside.0, inside.1);
    ui.key(Key::A, Modifiers::COMMAND);
    ui.frame(vec![Event::Text("Nota de campo".into())]);
    ui.key(Key::Enter, Modifiers::NONE);
    let edited = ui.free()[0].clone();
    assert_eq!(edited.id(), text.id());
    assert!(matches!(
        edited.graphic().unwrap().primitive(),
        SymbolPrimitive::Text { text, .. } if text == "Nota de campo"
    ));
    // Ctrl+C, Ctrl+V and a destination click; Esc ends pasting.
    ui.app.session.select_graphics([text.id()]);
    ui.key(Key::C, Modifiers::CTRL);
    ui.key(Key::V, Modifiers::CTRL);
    ui.click(200.0, 120.0);
    ui.click(240.0, 120.0);
    ui.key(Key::Escape, Modifiers::NONE);
    assert_eq!(ui.free().len(), 3, "duas colagens seguidas");
    // Ctrl+D duplicates the selection.
    ui.app.session.select_graphics([text.id()]);
    ui.key(Key::D, Modifiers::CTRL);
    assert_eq!(ui.free().len(), 4);
    ui.app.session.project().unwrap().validate().unwrap();
}

#[test]
fn free_content_survives_save_and_reopen_through_the_interface() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("livre.cadcraft");
    let mut ui = Ui::new();
    ui.app.execute(CommandId::InsertRectangle, &ui.ctx.clone());
    ui.click(20.0, 20.0);
    ui.click(52.0, 40.0);
    ui.app.execute(CommandId::InsertFreeText, &ui.ctx.clone());
    ui.click(100.0, 100.0);
    let saved = ui.free();
    assert_eq!(saved.len(), 2);
    ui.app.files.picker = Box::new(ScriptedPicker {
        save: [path.clone()].into(),
        ..Default::default()
    });
    ui.app.execute(CommandId::SaveProjectAs, &ui.ctx.clone());
    ui.frame(vec![]);
    assert!(path.exists());
    drop(ui);

    let mut other = Ui::new();
    other.app.files.picker = Box::new(ScriptedPicker {
        open: [path.clone()].into(),
        ..Default::default()
    });
    other.app.session.dirty = false;
    other
        .app
        .execute(CommandId::OpenProject, &other.ctx.clone());
    for _ in 0..3 {
        other.frame(vec![]);
    }
    let project = other.app.session.project().unwrap();
    for r in &saved {
        assert_eq!(
            project.representation(r.id()),
            Some(r),
            "conteúdo, camada e ID"
        );
    }
}

/// On Windows, winit emits Copy/Cut/Paste for Ctrl+C/X/V without key events; shortcuts must handle these events.
#[test]
fn clipboard_events_from_the_os_drive_copy_cut_and_paste() {
    let mut ui = Ui::new();
    ui.app.execute(CommandId::InsertFreeText, &ui.ctx.clone());
    ui.click(100.0, 100.0);
    ui.select_tool();
    let text = ui.free()[0].id();
    ui.app.session.select_graphics([text]);
    ui.frame(vec![Event::Copy]);
    assert!(ui.app.session.clipboard.is_some(), "Copy copia a seleção");
    ui.frame(vec![Event::Paste("CAD-CRAFT".into())]);
    ui.click(200.0, 120.0);
    ui.key(Key::Escape, Modifiers::NONE);
    assert_eq!(ui.free().len(), 2, "Paste inicia a colagem");
    ui.app.session.select_graphics([text]);
    ui.frame(vec![Event::Cut]);
    assert!(
        ui.app
            .session
            .project()
            .unwrap()
            .representation(text)
            .is_none()
    );
    assert!(ui.app.session.clipboard.as_ref().unwrap().is_cut());
}

#[test]
fn os_paste_after_page_tree_focus_places_copied_graphic_on_active_page() {
    let mut ui = Ui::new();
    ui.app.execute(CommandId::InsertFreeText, &ui.ctx.clone());
    ui.click(100., 100.);
    ui.select_tool();
    let original = ui.free()[0].id();
    ui.app.session.select_graphics([original]);
    ui.frame(vec![Event::Copy]);
    assert!(ui.app.session.clipboard.is_some());
    let tree = egui::Id::new("page_tree");
    ui.ctx.memory_mut(|memory| memory.request_focus(tree));
    ui.frame(vec![]);
    assert_eq!(ui.ctx.memory(|memory| memory.focused()), Some(tree));
    let before = ui.app.session.snapshot().unwrap();
    ui.frame(vec![Event::Paste("CAD-CRAFT".into())]);
    assert_eq!(
        ui.app.session.graphics.tool,
        cad_gui::interaction::Tool::Paste
    );
    assert_eq!(ui.app.session.snapshot().unwrap(), before);
    ui.click(200., 120.);
    ui.key(Key::Escape, Modifiers::NONE);
    let graphics = ui.free();
    assert_eq!(graphics.len(), 2);
    assert!(graphics.iter().all(|graphic| graphic.page_id() == ui.page));
    assert!(graphics.iter().any(|graphic| graphic.id() != original));
    ui.key(Key::Z, Modifiers::CTRL);
    assert_eq!(ui.app.session.snapshot().unwrap(), before);
}

#[test]
fn os_clipboard_events_with_text_focus_preserve_graphic_selection_and_project() {
    let mut ui = Ui::new();
    ui.app.execute(CommandId::InsertFreeText, &ui.ctx.clone());
    ui.click(100., 100.);
    ui.select_tool();
    let graphic = ui.free()[0].id();
    ui.app.session.select_graphics([graphic]);
    ui.frame(vec![Event::Copy]);
    let before = ui.app.session.snapshot().unwrap();
    let search = egui::Id::new("command_search");
    ui.ctx.memory_mut(|memory| memory.request_focus(search));
    ui.frame(vec![]);
    assert_eq!(ui.ctx.memory(|memory| memory.focused()), Some(search));
    for event in [Event::Copy, Event::Cut, Event::Paste("CAD-CRAFT".into())] {
        ui.frame(vec![event]);
        assert_eq!(ui.app.session.snapshot().unwrap(), before);
        assert_eq!(
            ui.app.session.graphics.tool,
            cad_gui::interaction::Tool::Select
        );
        assert_eq!(ui.app.session.graphics.selection.primary(), Some(graphic));
    }
}

/// Create a free ellipse through real clicks; editing preserves its style.
#[test]
fn ellipse_tool_and_style_survive_edits() {
    let mut ui = Ui::new();
    ui.app.execute(CommandId::InsertEllipse, &ui.ctx.clone());
    ui.click(20.0, 20.0);
    ui.click(60.0, 40.0);
    let e = ui.free()[0].clone();
    assert!(matches!(
        e.graphic().unwrap().primitive(),
        SymbolPrimitive::Ellipse { .. }
    ));
    let styled = e
        .graphic()
        .unwrap()
        .clone()
        .with_style(PrimitiveStyle {
            dash: LineDash::Dashed,
            width_mm: None,
            fill: true,
        })
        .unwrap();
    ui.app.session.edit_free_graphic(e.id(), styled).unwrap();
    // Text edited by double-click keeps its own style.
    ui.app.execute(CommandId::InsertFreeText, &ui.ctx.clone());
    ui.click(100.0, 100.0);
    let text = ui
        .free()
        .into_iter()
        .find(|r| {
            matches!(
                r.graphic().unwrap().primitive(),
                SymbolPrimitive::Text { .. }
            )
        })
        .unwrap();
    let dotted = text
        .graphic()
        .unwrap()
        .clone()
        .with_style(PrimitiveStyle {
            dash: LineDash::Dotted,
            ..Default::default()
        })
        .unwrap();
    ui.app.session.edit_free_graphic(text.id(), dotted).unwrap();
    ui.select_tool();
    let b = text.bounds().unwrap();
    ui.double_click(
        b.origin().x() + b.size().width() / 2.0,
        b.origin().y() + b.size().height() / 2.0,
    );
    ui.key(Key::A, Modifiers::COMMAND);
    ui.frame(vec![Event::Text("Outro".into())]);
    ui.key(Key::Enter, Modifiers::NONE);
    let p = ui.app.session.project().unwrap();
    assert_eq!(
        p.representation(text.id())
            .unwrap()
            .graphic()
            .unwrap()
            .style()
            .dash,
        LineDash::Dotted
    );
    assert!(
        p.representation(e.id())
            .unwrap()
            .graphic()
            .unwrap()
            .style()
            .fill
    );
}
