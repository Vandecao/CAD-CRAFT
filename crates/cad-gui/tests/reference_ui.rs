// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_craft::domain::*;
use cad_gui::{app::CadApp, commands::CommandId, editing::GraphicEdit};
use eframe::egui::{self, Context, Event, Modifiers, PointerButton, Pos2, Rect, vec2};
fn frame(app: &mut CadApp, ctx: &Context, events: Vec<Event>) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1600., 900.))),
            events,
            ..Default::default()
        },
        |ctx| app.show(ctx),
    )
}
fn click(app: &mut CadApp, ctx: &Context, at: Pos2) {
    frame(
        app,
        ctx,
        vec![
            Event::PointerMoved(at),
            Event::PointerButton {
                pos: at,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    frame(
        app,
        ctx,
        vec![Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );
}
fn setup() -> (CadApp, Context) {
    let mut app = CadApp::default();
    let ctx = Context::default();
    app.session
        .replace(cad_gui::demo::project().unwrap(), false)
        .unwrap();
    let page = app.session.project().unwrap().pages().next().unwrap().id();
    app.session.open_page(page).unwrap();
    for _ in 0..4 {
        frame(&mut app, &ctx, vec![]);
    }
    (app, ctx)
}
#[test]
fn real_marker_click_inserts_once_and_escape_cancels_without_mutation() {
    for command in [CommandId::InsertInterruption, CommandId::InsertPotential] {
        let (mut app, ctx) = setup();
        let before = app.session.snapshot().unwrap();
        app.execute(command, &ctx);
        frame(&mut app, &ctx, vec![]);
        let at = app.last_viewport.unwrap().center();
        click(&mut app, &ctx, at);
        assert_eq!(app.session.project().unwrap().markers().count(), 1);
        assert_eq!(app.session.history.undo_len(), 1);
        let id = app
            .session
            .project()
            .unwrap()
            .markers()
            .next()
            .unwrap()
            .id();
        assert_eq!(app.session.graphics.selection.primary(), Some(id));
        app.execute(CommandId::Undo, &ctx);
        assert_eq!(app.session.snapshot().unwrap(), before);
        app.execute(command, &ctx);
        frame(
            &mut app,
            &ctx,
            vec![Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
        );
        click(&mut app, &ctx, at);
        assert_eq!(app.session.snapshot().unwrap(), before);
    }
}
#[test]
fn marker_reference_link_navigates_to_real_other_page_and_updates_selection() {
    let (mut app, ctx) = setup();
    let pages: Vec<_> = app
        .session
        .project()
        .unwrap()
        .pages()
        .map(Page::id)
        .collect();
    let source = app
        .session
        .insert_electrical_marker(
            MarkerKind::Interruption,
            Text::new("ORIGEM").unwrap(),
            None,
            Point2D::new(80., 100.).unwrap(),
        )
        .unwrap();
    app.session.open_page(pages[1]).unwrap();
    let dest = app
        .session
        .insert_electrical_marker(
            MarkerKind::Interruption,
            Text::new("DESTINO_TESTE").unwrap(),
            None,
            Point2D::new(120., 80.).unwrap(),
        )
        .unwrap();
    app.graphic_edit(GraphicEdit::Marker(MarkerEdit::SetGroup {
        id: ReferenceGroupId::new(),
        mode: ReferenceMode::Star,
        members: vec![source, dest],
        continuity: false,
    }));
    app.session.navigate_reference(source).unwrap();
    app.execute(CommandId::ReferenceProperties, &ctx);
    let mut output = frame(&mut app, &ctx, vec![]);
    for _ in 0..3 {
        output = frame(&mut app, &ctx, vec![]);
    }
    let link = output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Text(t)
                if t.galley.text().starts_with("Ir: DESTINO_TESTE")
                    && s.clip_rect.contains(t.visual_bounding_rect().center()) =>
            {
                Some(t.visual_bounding_rect().center())
            }
            _ => None,
        })
        .expect("visible real destination link");
    let snapshot = app.session.snapshot().unwrap();
    let history = app.session.history.undo_len();
    click(&mut app, &ctx, link);
    assert_eq!(app.session.active, Some(pages[1]));
    assert_eq!(app.session.graphics.selection.primary(), Some(dest));
    assert_eq!(app.session.snapshot().unwrap(), snapshot);
    assert_eq!(app.session.history.undo_len(), history);
}
