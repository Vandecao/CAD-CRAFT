// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Regressions for native window commands and idle scheduling; start with settled frames.
use cad_gui::app::CadApp;
use eframe::egui::{Context, Pos2, RawInput, Rect, Vec2, ViewportCommand, ViewportId};

fn frame(app: &mut CadApp, ctx: &Context) -> eframe::egui::FullOutput {
    ctx.run(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 900.0))),
            ..Default::default()
        },
        |ctx| app.show(ctx),
    )
}

#[test]
fn unchanged_title_does_not_keep_requesting_native_redraws() {
    let ctx = Context::default();
    let mut app = CadApp::default();
    let first = frame(&mut app, &ctx);
    assert!(first.viewport_output[&ViewportId::ROOT].commands.iter().any(
        |command| matches!(command, ViewportCommand::Title(title) if title == "CAD-CRAFT — Nenhum projeto aberto")
    ));
    for _ in 0..8 {
        frame(&mut app, &ctx);
    }
    let settled = frame(&mut app, &ctx);
    assert!(
        !settled.viewport_output[&ViewportId::ROOT]
            .commands
            .iter()
            .any(|command| matches!(command, ViewportCommand::Title(_))),
        "an unchanged native title must not request another frame"
    );
    assert!(
        settled.viewport_output[&ViewportId::ROOT].repaint_delay
            > std::time::Duration::from_secs(1),
        "a clean idle window must not continuously request frames"
    );
    app.session
        .replace(cad_gui::demo::project().expect("valid demo"), true)
        .expect("valid session");
    let changed = frame(&mut app, &ctx);
    assert!(changed.viewport_output[&ViewportId::ROOT].commands.iter().any(
        |command| matches!(command, ViewportCommand::Title(title) if title.contains(app.session.project().expect("project").name().as_str()))
    ));
    let settled = frame(&mut app, &ctx);
    assert!(
        !settled.viewport_output[&ViewportId::ROOT]
            .commands
            .iter()
            .any(|command| matches!(command, ViewportCommand::Title(_)))
    );
    app.session.dirty = true;
    let changed = frame(&mut app, &ctx);
    assert!(
        changed.viewport_output[&ViewportId::ROOT]
            .commands
            .iter()
            .any(
                |command| matches!(command, ViewportCommand::Title(title) if title.ends_with(" *"))
            )
    );
}

#[test]
fn dirty_idle_project_wakes_for_its_next_recovery_checkpoint() {
    let ctx = Context::default();
    let mut app = CadApp::default();
    app.session
        .replace(cad_gui::demo::project().expect("valid demo"), true)
        .expect("valid session");
    app.session.dirty = true;
    // The future checkpoint must wake the app even when no input arrives.
    // No recovery file is written: this test stays before the first deadline.
    app.files.recovery = Some(cad_gui::project_file::Recovery::new(
        std::env::temp_dir().join(format!("cadcraft-idle-checkpoint-{}", std::process::id())),
    ));
    app.files.autosave_interval = std::time::Duration::from_secs(60);
    app.files.last_autosave = Some(std::time::Instant::now());
    for _ in 0..10 {
        frame(&mut app, &ctx);
    }
    let output = frame(&mut app, &ctx);
    let delay = output.viewport_output[&ViewportId::ROOT].repaint_delay;
    assert!(
        delay <= std::time::Duration::from_secs(60),
        "checkpoint must wake before its deadline: {delay:?}"
    );
    assert!(
        delay > std::time::Duration::from_secs(1),
        "idle checkpoint must not poll every frame: {delay:?}"
    );
}
