// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Properties inspector with real egui frames: typing X and pressing Enter
//! moves the selected symbol without the separate "Aplicar transformação"
//! button, and one Ctrl+Z undoes it.
// Fixture helpers use validated synthetic inputs and fail immediately on setup errors.
#![allow(clippy::unwrap_used)]
use cad_gui::{app::CadApp, state::Selection};
use eframe::egui::{
    self, Context, Event, Key, Modifiers, PointerButton, Pos2, Rect, accesskit::Role,
};

fn frame(app: &mut CadApp, ctx: &Context, events: Vec<Event>) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(1600., 1000.))),
            events,
            ..Default::default()
        },
        |ctx| app.show(ctx),
    )
}
fn key(key: Key, modifiers: Modifiers) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}
/// Vertical center of the spin field showing `value`, if laid out.
fn spin_bounds(out: &egui::FullOutput, value: f64) -> Option<f64> {
    out.platform_output
        .accesskit_update
        .as_ref()?
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == Role::SpinButton && n.numeric_value() == Some(value))
        .find_map(|(_, n)| n.bounds())
        .map(|b| (b.y0 + b.y1) / 2.)
}
/// Center of the spin field showing `value` (the inspector lists X first).
fn spin_with_value(out: &egui::FullOutput, value: f64) -> Pos2 {
    let nodes = &out.platform_output.accesskit_update.as_ref().unwrap().nodes;
    nodes
        .iter()
        .filter(|(_, n)| n.role() == Role::SpinButton)
        .filter(|(_, n)| n.numeric_value() == Some(value))
        .find_map(|(_, n)| n.bounds())
        .map(|b| Pos2::new(((b.x0 + b.x1) / 2.) as f32, ((b.y0 + b.y1) / 2.) as f32))
        .unwrap_or_else(|| {
            panic!(
                "campo com {value}: {:?}",
                nodes
                    .iter()
                    .filter(|(_, n)| n.role() == Role::SpinButton)
                    .map(|(_, n)| (n.numeric_value(), n.bounds().is_some()))
                    .collect::<Vec<_>>()
            )
        })
}

#[test]
fn typing_x_and_enter_moves_the_symbol_in_one_undo_step() {
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut app = CadApp::default();
    let mut project = cad_gui::connection_demo::project().unwrap();
    // Without wires, so the move is a plain transform.
    let routes: Vec<_> = project
        .connection_routes()
        .map(cad_craft::domain::ConnectionRoute::id)
        .collect();
    for id in routes {
        project
            .edit_connection_drawing(cad_craft::domain::ConnectionDrawingEdit::DeleteRoute { id })
            .unwrap();
    }
    let page = project.pages().next().unwrap().id();
    let rep = project
        .representations()
        .find(|r| r.page_id() == page && r.symbol().is_some())
        .unwrap()
        .id();
    app.session.replace(project, false).unwrap();
    app.session.open_page(page).unwrap();
    app.session.select(Selection::Representation(rep));
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    // The transform rows sit below the port list: scroll the properties
    // panel a little at a time until the X field is clear of the bottom panels.
    let before_x = app
        .session
        .project()
        .unwrap()
        .representation(rep)
        .unwrap()
        .transform()
        .translation()
        .x();
    let mut out = frame(&mut app, &ctx, vec![]);
    for _ in 0..40 {
        let visible = spin_bounds(&out, before_x).is_some_and(|y| y < 650.);
        if visible {
            break;
        }
        frame(
            &mut app,
            &ctx,
            vec![
                Event::PointerMoved(Pos2::new(1500., 400.)),
                Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -40.),
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        for _ in 0..5 {
            out = frame(&mut app, &ctx, vec![]);
        }
    }
    let x = |app: &CadApp| {
        app.session
            .project()
            .unwrap()
            .representation(rep)
            .unwrap()
            .transform()
            .translation()
            .x()
    };
    let before = x(&app);
    let at = spin_with_value(&out, before);
    let click = |pressed| Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    frame(&mut app, &ctx, vec![Event::PointerMoved(at), click(true)]);
    frame(&mut app, &ctx, vec![click(false)]);
    frame(
        &mut app,
        &ctx,
        vec![key(Key::A, Modifiers::CTRL), Event::Text("123".into())],
    );
    frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
    let last = frame(&mut app, &ctx, vec![]);
    let texts: Vec<String> = last
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, n)| n.label().or(n.value()).map(str::to_owned))
        .filter(|t| t.contains("123") || t.contains("INFO") || t.contains("ERRO"))
        .collect();
    assert_eq!(x(&app), 123.0, "Enter aplica o X digitado: {texts:?}");
    app.graphic_edit(cad_gui::editing::GraphicEdit::Undo);
    assert_eq!(x(&app), before);
}
