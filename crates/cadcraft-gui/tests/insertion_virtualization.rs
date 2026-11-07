// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Large insertion lists render visible rows while preserving scrolling and selection.
// Synthetic fixture helpers fail immediately on invalid setup.
#![allow(clippy::unwrap_used)]
use cad_craft::domain::*;
use cad_gui::{app::CadApp, preferences::PanelId, state::Selection};
use eframe::egui::{self, Context, Event, FullOutput, Pos2, Rect, Vec2};

fn frame(app: &mut CadApp, ctx: &Context, events: Vec<Event>) -> FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600., 900.))),
            time: Some(ctx.input(|input| input.time) + 1.0 / 60.0),
            events,
            ..Default::default()
        },
        |ctx| app.show(ctx),
    )
}

fn rows(output: &FullOutput) -> Vec<(String, Rect)> {
    output
        .shapes
        .iter()
        .filter_map(|shape| {
            let egui::Shape::Text(text) = &shape.shape else {
                return None;
            };
            text.galley.text().contains(" · virtual row").then(|| {
                (
                    text.galley.text().to_owned(),
                    Rect::from_min_size(text.pos, text.galley.size()),
                )
            })
        })
        .collect()
}

#[test]
fn device_rows_are_bounded_and_scrolling_can_select_the_last_device() {
    let mut project = Project::new("Insertion list").unwrap();
    for n in 1..=500 {
        project
            .insert_device(
                Device::new(
                    project.id(),
                    Designation::new(format!("K{n:04}")).unwrap(),
                    "virtual row",
                )
                .unwrap(),
            )
            .unwrap();
    }
    let last = project.devices().last().unwrap();
    let expected = format!("{} · virtual row", last.designation());
    let last_id = last.id();
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut style = (*ctx.style()).clone();
    style.animation_time = 0.;
    ctx.set_style(style);
    let mut app = CadApp::default();
    app.session.replace(project, false).unwrap();
    for panel in [PanelId::Navigator, PanelId::Properties] {
        app.preferences.panel_mut(panel).visible = false;
    }
    let mut output = frame(&mut app, &ctx, vec![]);
    for _ in 0..8 {
        output = frame(&mut app, &ctx, vec![]);
    }
    let initial = rows(&output);
    assert!(!initial.is_empty());
    assert!(
        initial.len() < 50,
        "only visible insertion rows should be built, got {}",
        initial.len()
    );
    let accessible_rows = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| {
            node.label()
                .is_some_and(|label| label.contains(" · virtual row"))
        })
        .count();
    assert!(
        accessible_rows < 50,
        "offscreen device widgets must be virtualized too, got {accessible_rows}"
    );
    let pointer = initial[0].1.center();
    frame(
        &mut app,
        &ctx,
        vec![
            Event::PointerMoved(pointer),
            Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: Vec2::new(0., -100_000.),
                modifiers: Default::default(),
            },
        ],
    );
    for _ in 0..40 {
        output = frame(&mut app, &ctx, vec![]);
    }
    let last_rows = rows(&output);
    assert!(last_rows.len() < 50);
    let target = last_rows
        .iter()
        .find(|(label, _)| *label == expected)
        .expect("last device is reachable")
        .1
        .center();
    frame(&mut app, &ctx, vec![Event::PointerMoved(target)]);
    frame(&mut app, &ctx, vec![]);
    for pressed in [true, false] {
        frame(
            &mut app,
            &ctx,
            vec![
                Event::PointerMoved(target),
                Event::PointerButton {
                    pos: target,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
    assert_eq!(
        app.session.selection,
        Selection::Electrical(RepresentationTarget::Device(last_id))
    );
}
