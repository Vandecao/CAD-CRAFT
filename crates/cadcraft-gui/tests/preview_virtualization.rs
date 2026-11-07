// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Thumbnail virtualization regressions; start with visible_rows_remain_bounded_and_last_page_is_reachable.
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

use cad_craft::domain::*;
use cad_gui::{app::CadApp, preferences::PanelId};
use eframe::egui::{self, Context, Event, FullOutput, Pos2, Rect, Vec2};

fn frame(app: &mut CadApp, ctx: &Context, events: Vec<Event>) -> FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600., 900.))),
            events,
            ..Default::default()
        },
        |ctx| app.show(ctx),
    )
}

fn thumbnails(output: &FullOutput) -> Vec<(String, Rect)> {
    output
        .shapes
        .iter()
        .filter_map(|shape| {
            let egui::Shape::Text(text) = &shape.shape else {
                return None;
            };
            let label = text.galley.text();
            label.contains(" · Thumbnail ").then(|| {
                (
                    label.to_owned(),
                    Rect::from_min_size(text.pos, text.galley.size()),
                )
            })
        })
        .collect()
}

#[test]
fn visible_rows_remain_bounded_and_last_page_is_reachable() {
    let mut project = Project::new("Thumbnail virtualization").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "D").unwrap())
        .unwrap();
    for n in 1..=200 {
        project
            .insert_page(
                Page::new(
                    project.id(),
                    doc,
                    format!("Thumbnail {n}"),
                    n.to_string(),
                    PageKind::MultiLine,
                    PaperFormat::A3.size(true),
                )
                .unwrap(),
            )
            .unwrap();
    }
    let mut app = CadApp::default();
    app.session.replace(project, true).unwrap();
    let ctx = Context::default();
    let mut output = frame(&mut app, &ctx, vec![]);
    for _ in 0..5 {
        output = frame(&mut app, &ctx, vec![]);
    }
    let labels = thumbnails(&output);
    eprintln!("initial thumbnail labels: {} {labels:?}", labels.len());
    assert!(!labels.is_empty());
    assert!(
        labels.len() <= 8,
        "200-page project must render only the small visible thumbnail range"
    );
    let pointer = labels[0].1.center() - Vec2::new(0., 30.);
    frame(&mut app, &ctx, vec![Event::PointerMoved(pointer)]);
    output = frame(
        &mut app,
        &ctx,
        vec![Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: Vec2::new(0., -100_000.),
            modifiers: egui::Modifiers::NONE,
        }],
    );
    for _ in 0..40 {
        output = frame(&mut app, &ctx, vec![]);
    }
    let labels = thumbnails(&output);
    eprintln!("last thumbnail labels: {} {labels:?}", labels.len());
    assert!(labels.len() <= 8);
    assert!(
        labels.iter().any(|(text, _)| text == "200 · Thumbnail 200"),
        "wheel scroll must reach the final page"
    );
}

#[test]
fn measure_preview_frame_cost_when_requested() {
    let Some(path) = std::env::var_os("CADCRAFT_PERFORMANCE_PROJECT") else {
        return;
    };
    let project = cad_craft::persistence::read_project_from_slice(&std::fs::read(path).unwrap())
        .unwrap()
        .project;
    let page = project.pages().next().unwrap().id();
    for operation in ["references", "bounds", "display_list"] {
        use cad_craft::display::PageDisplayList;
        let start = std::time::Instant::now();
        for _ in 0..32 {
            match operation {
                "references" => {
                    std::hint::black_box(project.sheet_references(page));
                }
                "bounds" => {
                    std::hint::black_box(project.document_bounds(page).unwrap());
                }
                _ => {
                    std::hint::black_box(project.page_display_list(page).unwrap());
                }
            }
        }
        eprintln!(
            "PERF model_{operation}_ms={:.3}",
            start.elapsed().as_secs_f64() * 1000. / 32.
        );
    }
    let mut app = CadApp::default();
    app.session.replace(project, true).unwrap();
    app.session.open_page(page).unwrap();
    let ctx = Context::default();
    let original = app.preferences.clone();
    for mode in [
        "all",
        "no_preview",
        "no_grid",
        "no_properties",
        "no_insertion",
        "no_navigator",
        "only_paper",
    ] {
        app.preferences = original.clone();
        for (name, panel) in [
            ("no_preview", PanelId::Preview),
            ("no_properties", PanelId::Properties),
            ("no_insertion", PanelId::Insertion),
            ("no_navigator", PanelId::Navigator),
        ] {
            if mode == name {
                app.preferences.panel_mut(panel).visible = false;
            }
        }
        if mode == "no_grid" || mode == "only_paper" {
            app.preferences.grid_visible = false;
        }
        if mode == "only_paper" {
            for panel in PanelId::ALL {
                app.preferences.panel_mut(panel).visible = false;
            }
        }
        for _ in 0..5 {
            frame(&mut app, &ctx, vec![]);
        }
        let mut ui_time = std::time::Duration::ZERO;
        let mut mesh_time = std::time::Duration::ZERO;
        let mut shapes = 0;
        for _ in 0..32 {
            let start = std::time::Instant::now();
            let output = frame(&mut app, &ctx, vec![]);
            ui_time += start.elapsed();
            shapes = output.shapes.len();
            let start = std::time::Instant::now();
            std::hint::black_box(ctx.tessellate(output.shapes, output.pixels_per_point));
            mesh_time += start.elapsed();
        }
        eprintln!(
            "PERF mode={mode} ui_ms={:.3} mesh_ms={:.3} shapes={shapes}",
            ui_time.as_secs_f64() * 1000. / 32.,
            mesh_time.as_secs_f64() * 1000. / 32.
        );
    }
}
