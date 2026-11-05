// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_craft::domain::*;
use cad_gui::{
    app::CadApp,
    camera::Camera,
    commands::{CommandId, registry},
    editing::GraphicEdit,
};
use eframe::egui::{self, Context, Event, Key, Modifiers, PointerButton, Pos2, Rect, vec2};

fn frame(app: &mut CadApp, ctx: &Context, events: Vec<Event>) -> egui::FullOutput {
    let modifiers = events
        .iter()
        .rev()
        .find_map(|e| match e {
            Event::PointerButton { modifiers, .. } | Event::Key { modifiers, .. } => {
                Some(*modifiers)
            }
            _ => None,
        })
        .unwrap_or(Modifiers::NONE);
    ctx.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1600.0, 900.0))),
            events,
            modifiers,
            ..Default::default()
        },
        |ctx| app.show(ctx),
    )
}
fn setup() -> (CadApp, Context) {
    let mut app = CadApp::default();
    let ctx = Context::default();
    app.session
        .replace(cad_gui::demo::project().unwrap(), true)
        .unwrap();
    let page = app.session.project().unwrap().pages().next().unwrap().id();
    app.session.open_page(page).unwrap();
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    (app, ctx)
}
#[test]
fn ribbon_library_is_available_and_opens_existing_insertion_panel() {
    let (mut app, ctx) = setup();
    let command = registry()
        .into_iter()
        .find(|c| c.name == "Biblioteca de símbolos")
        .unwrap();
    assert!(command.available(&app.session));
    let before = app.session.snapshot().unwrap();
    app.execute(command.id, &ctx);
    let output = frame(&mut app, &ctx, vec![]);
    assert!(output.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Text(t) if t.galley.job.text.contains("Contato normalmente aberto"))));
    assert_eq!(app.session.snapshot().unwrap(), before);
}

fn button(pos: Pos2, button: PointerButton, pressed: bool, modifiers: Modifiers) -> Event {
    Event::PointerButton {
        pos,
        button,
        pressed,
        modifiers,
    }
}
fn click(app: &mut CadApp, ctx: &Context, pos: Pos2, modifiers: Modifiers) {
    frame(
        app,
        ctx,
        vec![
            Event::PointerMoved(pos),
            button(pos, PointerButton::Primary, true, modifiers),
        ],
    );
    frame(
        app,
        ctx,
        vec![button(pos, PointerButton::Primary, false, modifiers)],
    );
}
fn key(key: Key) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}
fn begin(app: &mut CadApp) {
    app.session
        .begin_symbol_insertion(
            SymbolDefinitionId::new("cad-craft.contact-no").unwrap(),
            None,
            Transform2D::new(Point2D::new(0.0, 0.0).unwrap(), 0.0, false, false).unwrap(),
        )
        .unwrap();
}
fn text_position(output: &egui::FullOutput, value: &str) -> Pos2 {
    output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Text(t)
                if t.galley.job.text == value
                    && s.clip_rect.contains(t.visual_bounding_rect().center()) =>
            {
                Some(t.visual_bounding_rect().center())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("visible text: {value}"))
}
#[test]
fn library_row_and_insert_button_start_tool_without_editing_project() {
    let (mut app, ctx) = setup();
    app.execute(CommandId::SymbolLibrary, &ctx);
    let output = frame(&mut app, &ctx, vec![]);
    let row = text_position(&output, "Contato normalmente aberto");
    let before = app.session.snapshot().unwrap();
    click(&mut app, &ctx, row, Modifiers::NONE);
    let output = frame(&mut app, &ctx, vec![]);
    let insert = text_position(&output, "Inserir na página");
    click(&mut app, &ctx, insert, Modifiers::NONE);
    assert!(app.session.graphics.insertion.is_some());
    assert_eq!(app.session.snapshot().unwrap(), before);
    assert_eq!(app.session.history.undo_len(), 0);
}
#[test]
fn real_preview_click_repeat_and_escape_snap_with_hidden_grid() {
    let (mut app, ctx) = setup();
    app.preferences.grid_visible = false;
    app.preferences.grid_mm = 4.0;
    app.preferences.snap_enabled = true;
    let tab = app.session.active_tab_mut().unwrap();
    tab.camera = Camera::new([0.0, 0.0], 5.0).unwrap();
    tab.needs_fit = false;
    let camera = tab.camera;
    begin(&mut app);
    let before = app.session.snapshot().unwrap();
    let viewport = app.last_viewport.unwrap();
    let at = camera
        .world_to_screen(Point2D::new(-5.2, 6.2).unwrap(), viewport)
        .unwrap();
    frame(&mut app, &ctx, vec![Event::PointerMoved(at)]);
    assert_eq!(app.session.snapshot().unwrap(), before);
    assert_eq!(app.session.history.undo_len(), 0);
    assert!(app.session.graphics.insertion.as_ref().unwrap().positioned);
    click(&mut app, &ctx, at, Modifiers::NONE);
    let first = app.session.graphics.selection.primary().unwrap();
    assert_eq!(
        app.session
            .project()
            .unwrap()
            .representation(first)
            .unwrap()
            .transform()
            .translation(),
        Point2D::new(-4.0, 8.0).unwrap()
    );
    click(&mut app, &ctx, at + vec2(120.0, 0.0), Modifiers::NONE);
    let second = app.session.graphics.selection.primary().unwrap();
    assert_ne!(first, second);
    assert_eq!(app.session.history.undo_len(), 2);
    frame(&mut app, &ctx, vec![key(Key::Escape)]);
    assert!(app.session.graphics.insertion.is_none());
    assert_eq!(app.session.active_tab_mut().unwrap().camera, camera);
    app.execute(CommandId::Undo, &ctx);
    app.execute(CommandId::Undo, &ctx);
    assert_eq!(app.session.snapshot().unwrap(), before);
    app.execute(CommandId::Redo, &ctx);
    app.execute(CommandId::Redo, &ctx);
    assert!(
        app.session
            .project()
            .unwrap()
            .representation(first)
            .is_some()
    );
    assert!(
        app.session
            .project()
            .unwrap()
            .representation(second)
            .is_some()
    );
}
#[test]
fn real_cancel_focus_middle_and_escape_never_insert() {
    for cancel in 0..3 {
        let (mut app, ctx) = setup();
        let before = app.session.snapshot().unwrap();
        begin(&mut app);
        let at = app.last_viewport.unwrap().center();
        frame(&mut app, &ctx, vec![Event::PointerMoved(at)]);
        let event = match cancel {
            0 => key(Key::Escape),
            1 => Event::WindowFocused(false),
            _ => button(at, PointerButton::Middle, true, Modifiers::NONE),
        };
        frame(&mut app, &ctx, vec![event]);
        assert!(app.session.graphics.insertion.is_none());
        assert_eq!(app.session.snapshot().unwrap(), before);
        assert_eq!(app.session.history.undo_len(), 0);
    }
}
#[test]
fn inserted_symbols_use_existing_multiselect_drag_commands_and_properties() {
    let (mut app, ctx) = setup();
    app.preferences.grid_mm = 4.0;
    let tab = app.session.active_tab_mut().unwrap();
    tab.camera = Camera::new([0.0, 0.0], 4.0).unwrap();
    tab.needs_fit = false;
    let camera = tab.camera;
    begin(&mut app);
    let viewport = app.last_viewport.unwrap();
    let a = viewport.center() + vec2(-80.0, 0.0);
    let b = viewport.center() + vec2(80.0, 0.0);
    click(&mut app, &ctx, a, Modifiers::NONE);
    let first = app.session.graphics.selection.primary().unwrap();
    click(&mut app, &ctx, b, Modifiers::NONE);
    let second = app.session.graphics.selection.primary().unwrap();
    frame(&mut app, &ctx, vec![key(Key::Escape)]);
    click(&mut app, &ctx, a, Modifiers::NONE);
    let output = frame(&mut app, &ctx, vec![]);
    assert!(output.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.job.text=="Contato normalmente aberto")));
    click(&mut app, &ctx, b, Modifiers::CTRL);
    assert_eq!(app.session.graphics.selection.ids().len(), 2);
    let before = app.session.snapshot().unwrap();
    frame(
        &mut app,
        &ctx,
        vec![button(a, PointerButton::Primary, true, Modifiers::NONE)],
    );
    frame(
        &mut app,
        &ctx,
        vec![Event::PointerMoved(a + vec2(48.0, -32.0))],
    );
    assert_eq!(app.session.snapshot().unwrap(), before);
    frame(
        &mut app,
        &ctx,
        vec![button(
            a + vec2(48.0, -32.0),
            PointerButton::Primary,
            false,
            Modifiers::NONE,
        )],
    );
    assert_eq!(app.session.history.undo_len(), 3);
    let p = app.session.project().unwrap();
    assert_eq!(
        p.representation(first).unwrap().transform().translation(),
        Point2D::new(-8.0, 8.0).unwrap()
    );
    assert_eq!(
        p.representation(second).unwrap().transform().translation(),
        Point2D::new(32.0, 8.0).unwrap()
    );
    app.execute(CommandId::Rotate90, &ctx);
    app.execute(CommandId::MirrorHorizontal, &ctx);
    for id in [first, second] {
        let t = app
            .session
            .project()
            .unwrap()
            .representation(id)
            .unwrap()
            .transform();
        assert_eq!(t.rotation_degrees(), 270.0);
        assert!(t.mirror_x());
    }
    app.graphic_edit(GraphicEdit::Undo);
    app.graphic_edit(GraphicEdit::Undo);
    app.graphic_edit(GraphicEdit::Undo);
    assert_eq!(app.session.snapshot().unwrap(), before);
    assert_eq!(app.session.active_tab_mut().unwrap().camera, camera);
}

#[test]
fn compact_library_can_scroll_to_symbol_rows_when_controls_exceed_panel_height() {
    let (mut app, ctx) = setup();
    ctx.set_zoom_factor(1.25);
    app.execute(CommandId::SymbolLibrary, &ctx);
    let small_frame = |app: &mut CadApp, events| {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1280.0, 720.0))),
                events,
                ..Default::default()
            },
            |ctx| app.show(ctx),
        )
    };
    let mut output = small_frame(&mut app, vec![]);
    for _ in 0..3 {
        output = small_frame(&mut app, vec![]);
    }
    let at = text_position(&output, "Orientação e associação") + vec2(20.0, 25.0);
    for _ in 0..5 {
        output = small_frame(
            &mut app,
            vec![
                Event::PointerMoved(at),
                Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: vec2(0.0, -70.0),
                    modifiers: Modifiers::NONE,
                },
            ],
        );
    }
    let visible=output.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.job.text=="Contato normalmente aberto" && s.clip_rect.contains_rect(t.visual_bounding_rect())));
    assert!(
        visible,
        "scroll must expose a complete selectable symbol row in the compact panel"
    );
}
