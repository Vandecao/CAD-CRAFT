// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Opt-in real egui frame timings; generate the synthetic input with the facade performance test.
use cad_gui::app::CadApp;
use eframe::egui::{Context, Event, MouseWheelUnit, Pos2, RawInput, Rect, Vec2};
use std::time::{Duration, Instant};

fn frame(app: &mut CadApp, ctx: &Context, events: Vec<Event>) {
    let output = ctx.run(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 900.0))),
            events,
            ..Default::default()
        },
        |ctx| app.show(ctx),
    );
    let meshes = ctx.tessellate(output.shapes, output.pixels_per_point);
    assert!(
        !meshes.is_empty(),
        "the benchmark must construct drawable geometry"
    );
    std::hint::black_box(meshes);
}

fn measured_batch(mut action: impl FnMut()) -> Duration {
    // Batch short frames so coarse host clocks do not report a zero median.
    const FRAMES: u32 = 32;
    let start = Instant::now();
    for _ in 0..FRAMES {
        action();
    }
    let elapsed = start.elapsed();
    assert!(
        !elapsed.is_zero(),
        "host clock must resolve a batch of real frames"
    );
    elapsed / FRAMES
}

#[test]
fn large_synthetic_project_draw_and_zoom_stay_within_ci_budget() {
    let Some(path) = std::env::var_os("CADCRAFT_PERFORMANCE_PROJECT") else {
        return;
    };
    let bytes = std::fs::read(path).expect("generated synthetic performance project");
    let project = cad_craft::persistence::read_project_from_slice(&bytes)
        .expect("valid synthetic project")
        .project;
    assert_eq!(project.pages().count(), 200);
    assert_eq!(project.devices().count(), 5_000);
    assert_eq!(project.connections().count(), 20_000);
    let page = project.pages().next().expect("synthetic page").id();
    let ctx = Context::default();
    let mut app = CadApp::default();
    app.session.replace(project, true).expect("valid session");
    app.session.open_page(page).expect("existing page");
    for _ in 0..4 {
        frame(&mut app, &ctx, vec![]);
    }
    let viewport = app.last_viewport.expect("paper viewport");
    let mut draws = Vec::new();
    let mut zooms = Vec::new();
    let mut pans = Vec::new();
    for _ in 0..11 {
        draws.push(measured_batch(|| frame(&mut app, &ctx, vec![])));
        let mut forward = false;
        zooms.push(measured_batch(|| {
            let before = app
                .session
                .active_tab_mut()
                .expect("active page")
                .camera
                .scale();
            forward = !forward;
            frame(
                &mut app,
                &ctx,
                vec![
                    Event::PointerMoved(viewport.center()),
                    Event::MouseWheel {
                        unit: MouseWheelUnit::Point,
                        delta: Vec2::new(0.0, if forward { 20.0 } else { -20.0 }),
                        modifiers: Default::default(),
                    },
                ],
            );
            assert_ne!(
                app.session
                    .active_tab_mut()
                    .expect("active page")
                    .camera
                    .scale(),
                before,
                "wheel input must really zoom"
            );
        }));
        let anchor = cad_craft::domain::Point2D::new(0.0, 0.0).expect("finite anchor");
        let pointer = viewport.center();
        frame(
            &mut app,
            &ctx,
            vec![
                Event::PointerMoved(pointer),
                Event::PointerButton {
                    pos: pointer,
                    button: eframe::egui::PointerButton::Middle,
                    pressed: true,
                    modifiers: Default::default(),
                },
            ],
        );
        let mut forward = false;
        pans.push(measured_batch(|| {
            let before = app
                .session
                .active_tab_mut()
                .expect("active page")
                .camera
                .world_to_screen(anchor, viewport)
                .expect("finite coordinate");
            forward = !forward;
            frame(
                &mut app,
                &ctx,
                vec![Event::PointerMoved(
                    pointer
                        + if forward {
                            Vec2::new(10.0, 10.0)
                        } else {
                            Vec2::ZERO
                        },
                )],
            );
            let after = app
                .session
                .active_tab_mut()
                .expect("active page")
                .camera
                .world_to_screen(anchor, viewport)
                .expect("finite coordinate");
            assert!(after.distance(before) > 1.0, "middle drag must really pan");
        }));
        frame(
            &mut app,
            &ctx,
            vec![Event::PointerButton {
                pos: pointer,
                button: eframe::egui::PointerButton::Middle,
                pressed: false,
                modifiers: Default::default(),
            }],
        );
    }
    for (name, mut timings) in [("draw", draws), ("wheel_zoom", zooms), ("pan", pans)] {
        timings.sort();
        let median = timings[timings.len() / 2];
        eprintln!(
            "PERF gui_{name}_median={:.3} ms",
            median.as_secs_f64() * 1000.0
        );
        assert!(
            median < Duration::from_millis(160),
            "{name}: {median:?} exceeds loose CI frame budget"
        );
    }
    assert_eq!(
        app.session
            .project()
            .expect("unchanged project")
            .connections()
            .count(),
        20_000
    );
}
