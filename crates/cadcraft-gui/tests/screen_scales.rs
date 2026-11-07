// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Resolution and interface zoom regressions; start with the dialog matrix.
#![allow(clippy::unwrap_used)]

use cad_gui::{app::CadApp, commands::CommandId};
use eframe::egui::{self, Context, Id, Pos2, Rect, Vec2};

fn frame(app: &mut CadApp, ctx: &Context, size: Vec2) -> egui::FullOutput {
    frame_events(app, ctx, size, vec![])
}

fn frame_events(
    app: &mut CadApp,
    ctx: &Context,
    size: Vec2,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size / ctx.zoom_factor())),
            events,
            ..Default::default()
        },
        |ctx| app.show(ctx),
    )
}

fn visible_text(output: &egui::FullOutput, label: &str) -> Option<Rect> {
    output.shapes.iter().find_map(|shape| {
        let egui::Shape::Text(text) = &shape.shape else {
            return None;
        };
        let bounds = Rect::from_min_size(text.pos, text.galley.size());
        (text.galley.text() == label && shape.clip_rect.expand(1.).contains_rect(bounds))
            .then_some(bounds)
    })
}

#[test]
fn overflowing_ribbon_tabs_scroll_until_last_tab_is_visible() {
    let ctx = Context::default();
    ctx.enable_accesskit();
    ctx.set_zoom_factor(2.);
    let mut app = CadApp::default();
    let size = egui::vec2(1280., 720.);
    let mut out = frame(&mut app, &ctx, size);
    for _ in 0..5 {
        out = frame(&mut app, &ctx, size);
    }
    assert!(
        visible_text(&out, "Integrações").is_none(),
        "test requires overflowing tabs"
    );
    let y = visible_text(&out, "Arquivo")
        .expect("file button remains visible")
        .center()
        .y;
    frame_events(
        &mut app,
        &ctx,
        size,
        vec![egui::Event::PointerMoved(Pos2::new(300., y))],
    );
    out = frame_events(
        &mut app,
        &ctx,
        size,
        vec![egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(-2000., 0.),
            modifiers: egui::Modifiers::NONE,
        }],
    );
    for _ in 0..30 {
        out = frame(&mut app, &ctx, size);
    }
    let tab = visible_text(&out, "Integrações")
        .expect("horizontal scroll reveals last tab without clipping");
    assert!(ctx.content_rect().contains_rect(tab));
    assert!(
        out.platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.label() == Some("Integrações"))
    );
}

#[test]
fn insertion_header_keeps_assistant_tab_visible_on_small_logical_screen() {
    let ctx = Context::default();
    ctx.set_zoom_factor(2.);
    let mut app = CadApp::default();
    let size = egui::vec2(1280., 720.);
    let mut out = frame(&mut app, &ctx, size);
    for _ in 0..5 {
        out = frame(&mut app, &ctx, size);
    }
    assert!(
        out.shapes.iter().any(|shape| {
            let egui::Shape::Text(text) = &shape.shape else {
                return false;
            };
            let bounds = Rect::from_min_size(text.pos, text.galley.size());
            text.galley.text() == "✦ Assistente"
                && bounds.top() < 250.
                && shape.clip_rect.contains_rect(bounds)
        }),
        "insertion header assistant tab must be visible, rather than clipped outside its panel"
    );
}

fn wheel(app: &mut CadApp, ctx: &Context, size: Vec2, at: Pos2, delta: Vec2) -> egui::FullOutput {
    frame_events(app, ctx, size, vec![egui::Event::PointerMoved(at)]);
    let mut out = frame_events(
        app,
        ctx,
        size,
        vec![egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    for _ in 0..30 {
        out = frame(app, ctx, size);
    }
    out
}

#[test]
fn wide_part_dialog_bottom_export_button_is_reachable_at_every_scale() {
    for size in [
        egui::vec2(1280., 720.),
        egui::vec2(1920., 1080.),
        egui::vec2(3840., 2160.),
    ] {
        for zoom in [1., 1.25, 1.5, 2.] {
            let ctx = Context::default();
            ctx.enable_accesskit();
            ctx.set_zoom_factor(zoom);
            let mut app = CadApp::default();
            app.session
                .replace(cad_gui::demo::project().unwrap(), true)
                .unwrap();
            for _ in 0..3 {
                frame(&mut app, &ctx, size);
            }
            app.execute(CommandId::PartEditor, &ctx);
            for _ in 0..5 {
                frame(&mut app, &ctx, size);
            }
            let rect = ctx
                .memory(|m| m.area_rect(Id::new("Peças e dispositivos")))
                .unwrap();
            let out = wheel(
                &mut app,
                &ctx,
                size,
                rect.center(),
                egui::vec2(-10000., -10000.),
            );
            let button = visible_text(&out, "Exportar lista CSV")
                .unwrap_or_else(|| panic!("bottom export button clipped at {size:?} zoom {zoom}"));
            assert!(rect.contains_rect(button));
            assert!(
                out.platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(_, node)| node.label() == Some("Exportar lista CSV")
                        && node.role() == egui::accesskit::Role::Button)
            );
        }
    }
}

#[test]
fn insertion_panel_scroll_reaches_last_category_on_small_screen() {
    let ctx = Context::default();
    ctx.enable_accesskit();
    ctx.set_zoom_factor(2.);
    let size = egui::vec2(1280., 720.);
    let mut app = CadApp::default();
    let mut out = frame(&mut app, &ctx, size);
    for _ in 0..5 {
        out = frame(&mut app, &ctx, size);
    }
    let x = visible_text(&out, "Centro de inserção")
        .expect("insertion tab visible")
        .center()
        .x;
    let at = Pos2::new(x, app.last_viewport.unwrap().bottom() - 10.);
    for _ in 0..30 {
        out = wheel(&mut app, &ctx, size, at, egui::vec2(0., -40.));
        if visible_text(&out, "Catálogos").is_some() {
            break;
        }
    }
    let category = visible_text(&out, "Catálogos").unwrap_or_else(|| {
        panic!(
            "panel scroll must reveal last category completely; pointer={at:?}; text={:?}",
            out.shapes
                .iter()
                .filter_map(|shape| match &shape.shape {
                    egui::Shape::Text(text) if text.pos.x > 380. && text.pos.x < 500. => Some((
                        text.galley.text(),
                        text.pos,
                        text.galley.size(),
                        shape.clip_rect
                    )),
                    _ => None,
                })
                .collect::<Vec<_>>()
        )
    });
    assert!(ctx.content_rect().contains_rect(category));
    assert!(
        out.platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.label() == Some("Catálogos"))
    );
}

#[test]
fn dialogs_fit_every_resolution_and_interface_zoom() {
    let mut failures = Vec::new();
    for size in [
        egui::vec2(1280., 720.),
        egui::vec2(1920., 1080.),
        egui::vec2(3840., 2160.),
    ] {
        for zoom in [1., 1.25, 1.5, 2.] {
            for (command, id) in [
                (CommandId::ExportPdf, "pdf_window"),
                (CommandId::Print, "pdf_window"),
                (CommandId::ExportDxf, "pdf_window"),
                (CommandId::ExportDwg, "pdf_window"),
                (CommandId::NumberDevices, "numbering_window"),
                (CommandId::NumberConnections, "numbering_window"),
                (CommandId::NumberPages, "numbering_window"),
                (CommandId::PartEditor, "Peças e dispositivos"),
                (CommandId::MacroNavigator, "Macros"),
                (CommandId::TerminalStripEditor, "Engenharia física"),
                (CommandId::GenerateReport, "Gerar relatório"),
                (CommandId::ExportList, "Exportar lista"),
                (CommandId::NewPage, "session_dialog"),
                (CommandId::NewDocument, "session_dialog"),
                (CommandId::NewProject, "session_dialog"),
                (CommandId::ImportEdz, "Importar .edz"),
                (CommandId::CheckProject, "checks_window"),
            ] {
                let ctx = Context::default();
                ctx.enable_accesskit();
                let mut app = CadApp::default();
                app.session
                    .replace(cad_gui::demo::project().unwrap(), true)
                    .unwrap();
                let page = app.session.project().unwrap().pages().next().unwrap().id();
                app.session.open_page(page).unwrap();
                frame(&mut app, &ctx, size);
                ctx.set_zoom_factor(zoom);
                frame(&mut app, &ctx, size);
                app.execute(command, &ctx);
                let mut out = frame(&mut app, &ctx, size);
                for _ in 0..4 {
                    out = frame(&mut app, &ctx, size);
                }
                assert!(
                    out.platform_output
                        .accesskit_update
                        .as_ref()
                        .unwrap()
                        .nodes
                        .len()
                        > 10
                );
                let rect = ctx
                    .memory(|m| m.area_rect(Id::new(id)))
                    .unwrap_or_else(|| panic!("missing {command:?} area {id}"));
                let screen = ctx.content_rect().expand(1.);
                if !screen.contains_rect(rect) {
                    failures.push(format!(
                        "{size:?} zoom {zoom} {command:?}: {rect:?} outside {screen:?}"
                    ));
                }
                if id == "session_dialog" {
                    for label in ["Confirmar", "Cancelar"] {
                        let button = visible_text(&out, label).unwrap_or_else(|| {
                            panic!("{command:?} {size:?} zoom {zoom}: {label} clipped")
                        });
                        assert!(screen.contains_rect(button));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn panels_leave_visible_paper_at_every_resolution_and_interface_zoom() {
    for size in [
        egui::vec2(1280., 720.),
        egui::vec2(1920., 1080.),
        egui::vec2(3840., 2160.),
    ] {
        for zoom in [1., 1.25, 1.5, 2.] {
            let ctx = Context::default();
            ctx.enable_accesskit();
            let mut app = CadApp::default();
            ctx.set_zoom_factor(zoom);
            for _ in 0..5 {
                frame(&mut app, &ctx, size);
            }
            let paper = app.last_viewport.unwrap();
            assert!(
                paper.width() >= 140. && paper.height() >= 65.,
                "{size:?} zoom {zoom}: {paper:?}"
            );
            assert!(ctx.content_rect().contains_rect(paper));
        }
    }
}
