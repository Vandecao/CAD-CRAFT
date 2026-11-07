// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Keyboard activation must not submit a dialog created by the same Enter event.
#![allow(clippy::unwrap_used)]

use cad_gui::app::CadApp;
use eframe::egui::{self, Context, Event, Key, Modifiers, Pos2, Rect, vec2};

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

fn key(app: &mut CadApp, ctx: &Context, key: Key) -> egui::FullOutput {
    frame(
        app,
        ctx,
        vec![Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    );
    frame(
        app,
        ctx,
        vec![Event::Key {
            key,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    )
}

fn search_result(app: &mut CadApp, ctx: &Context, query: &str) {
    for _ in 0..3 {
        frame(app, ctx, vec![]);
    }
    ctx.memory_mut(|memory| memory.request_focus(egui::Id::new("command_search")));
    frame(app, ctx, vec![]);
    frame(app, ctx, vec![Event::Text(query.into())]);
    frame(app, ctx, vec![]);
    let mut focused_label = None;
    for _ in 0..12 {
        let out = key(app, ctx, Key::Tab);
        let tree = out.platform_output.accesskit_update.as_ref().unwrap();
        focused_label = tree
            .nodes
            .iter()
            .find(|(id, _)| *id == tree.focus)
            .and_then(|(_, node)| node.label())
            .map(str::to_owned);
        if focused_label
            .as_deref()
            .is_some_and(|label| label.starts_with(query))
        {
            return;
        }
    }
    panic!("Tab must focus the real command result: {focused_label:?}");
}

fn app() -> (CadApp, Context) {
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut app = CadApp::default();
    app.session
        .replace(cad_gui::demo::project().unwrap(), true)
        .unwrap();
    app.session.dirty = true;
    (app, ctx)
}

#[test]
fn enter_opening_close_confirmation_does_not_discard_dirty_project() {
    let (mut app, ctx) = app();
    let before = app.session.snapshot().unwrap();
    search_result(&mut app, &ctx, "Fechar sessão");
    key(&mut app, &ctx, Key::Enter);
    assert!(
        app.session.project().is_some(),
        "opening confirmation must preserve the dirty project"
    );
    assert_eq!(
        app.session.snapshot().unwrap(),
        before,
        "opening confirmation must not discard the project"
    );
    assert!(app.session.dirty);
    let out = frame(&mut app, &ctx, vec![]);
    assert!(
        out.platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.label().or_else(|| node.value()) == Some("Alterações não salvas")),
        "dialog content: {:?}",out.shapes.iter().filter_map(|shape|match &shape.shape { egui::Shape::Text(text) if text.galley.text().contains("salv") => Some(text.galley.text()), _ => None }).collect::<Vec<_>>()
    );
    key(&mut app, &ctx, Key::Enter);
    assert!(
        app.session.project().is_none(),
        "a second explicit Enter may confirm discard"
    );
}

#[test]
fn escape_cancels_keyboard_opened_close_confirmation_without_discarding() {
    let (mut app, ctx) = app();
    let before = app.session.snapshot().unwrap();
    search_result(&mut app, &ctx, "Fechar sessão");
    key(&mut app, &ctx, Key::Enter);
    key(&mut app, &ctx, Key::Escape);
    assert_eq!(app.session.snapshot().unwrap(), before);
    assert!(app.session.dirty);
    let out = frame(&mut app, &ctx, vec![]);
    assert!(
        !out.platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.label().or_else(|| node.value()) == Some("Alterações não salvas"))
    );
}

#[test]
fn enter_opening_new_document_does_not_validate_empty_name() {
    let (mut app, ctx) = app();
    let before = app.session.snapshot().unwrap();
    search_result(&mut app, &ctx, "Novo documento");
    key(&mut app, &ctx, Key::Enter);
    let out = frame(&mut app, &ctx, vec![]);
    let labels: Vec<_> = out
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().or_else(|| node.value()))
        .collect();
    assert!(labels.contains(&"Novo documento"));
    assert!(labels.contains(&"Nome do documento"));
    let validation_errors: Vec<_> = out
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.text().contains("texto obrigatório vazio") => {
                Some(text.galley.text())
            }
            _ => None,
        })
        .collect();
    assert!(
        validation_errors.is_empty(),
        "opening a form must not attempt to submit its empty name: {validation_errors:?}"
    );
    assert_eq!(app.session.snapshot().unwrap(), before);
}
