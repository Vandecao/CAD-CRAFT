// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Reproducible inspection harness. Uses the production UI and requests its
//! rendered framebuffer before swap; no project or preference writes.
use cad_gui::{app::CadApp, commands::CommandId};
use eframe::egui;
struct Preview {
    app: CadApp,
    mode: String,
    frame: u32,
}
impl eframe::App for Preview {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.app.show(ctx);
        if self.frame == 2 && self.mode != "welcome" && self.mode != "zoom-out" {
            self.app.execute(CommandId::FitPage, ctx);
        }
        if self.frame == 3 && ["editor", "group", "preview"].contains(&self.mode.as_str()) {
            let page = self.app.session.active.expect("QA page");
            let ids: Vec<_> = self
                .app
                .session
                .project()
                .expect("the QA demo has an open project")
                .representations_on_page(page)
                .expect("the QA page belongs to the demo project")
                .iter()
                .map(|r| r.id())
                .collect();
            self.app.preferences.ribbon_tab = cad_gui::preferences::RibbonTab::Edit;
            self.app.session.select_graphics(if self.mode == "editor" {
                vec![ids[0]]
            } else {
                ids.clone()
            });
            if self.mode == "editor" {
                self.app
                    .graphic_edit(cad_gui::editing::GraphicEdit::Rotate(90.0));
                self.app
                    .graphic_edit(cad_gui::editing::GraphicEdit::MirrorHorizontal);
            }
            if self.mode == "preview" {
                let p = self
                    .app
                    .session
                    .project()
                    .expect("the QA demo has an open project");
                let anchor = p
                    .representation(ids[0])
                    .expect("the selected QA representation exists")
                    .transform()
                    .translation();
                let mut interaction = std::mem::take(&mut self.app.session.graphics);
                interaction
                    .begin_move(
                        self.app
                            .session
                            .project()
                            .expect("the QA demo has an open project"),
                        page,
                        ids[0],
                        anchor,
                    )
                    .expect("the selected QA representation can start a move");
                interaction
                    .update_move(
                        anchor
                            .translated(40.0, 16.0)
                            .expect("the bounded QA translation remains finite"),
                        4.0,
                        true,
                    )
                    .expect("the bounded QA preview move is valid");
                self.app.session.graphics = interaction;
            }
        }
        if self.frame == 5 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        }
        for event in ctx.input(|i| i.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event {
                let path = std::env::var("CAD_CRAFT_QA_IMAGE").expect("Set CAD_CRAFT_QA_IMAGE");
                let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                image::save_buffer(
                    path,
                    &bytes,
                    image.size[0] as u32,
                    image.size[1] as u32,
                    image::ColorType::Rgba8,
                )
                .expect("write QA image");
                // The QA project is intentionally disposable. Edited scenarios
                // must not wait at the production unsaved-project confirmation.
                self.app.session.dirty = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        if self.frame == 0 {
            match self.mode.as_str() {
                "symbols" => {
                    self.app.preferences.ribbon_tab = cad_gui::preferences::RibbonTab::Insert;
                    self.app.execute(CommandId::SymbolLibrary, ctx);
                }
                "zoom-out" => {
                    for _ in 0..7 {
                        self.app.execute(CommandId::ZoomOut, ctx);
                    }
                }
                "backstage" => self.app.execute(CommandId::Settings, ctx),
                _ => {}
            }
        }
        self.frame += 1;
        ctx.request_repaint();
    }
    fn persist_egui_memory(&self) -> bool {
        false
    }
}
fn main() -> eframe::Result {
    let args: Vec<_> = std::env::args().collect();
    let mode = args.get(1).cloned().unwrap_or_else(|| "demo".into());
    let width = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1600.0);
    let height = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(900.0);
    let dpi = args
        .get(4)
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(1.0);
    let mut app = CadApp::default();
    if mode != "welcome" {
        app.session
            .replace(
                if mode == "connections" {
                    cad_gui::connection_demo::project().expect("valid connection demo")
                } else {
                    cad_gui::demo::project().expect("valid demo")
                },
                true,
            )
            .expect("valid demo");
        let id = app
            .session
            .project()
            .expect("demo")
            .pages()
            .next()
            .expect("demo page")
            .id();
        app.session.open_page(id).expect("demo page");
        if mode == "connections" {
            app.preferences.ribbon_tab = cad_gui::preferences::RibbonTab::Connections;
            let route = app
                .session
                .project()
                .expect("the connection QA demo has an open project")
                .connection_routes()
                .next()
                .map(|r| r.id());
            if let Some(id) = route {
                app.session.select_graphics([id]);
                app.session.highlight_network();
            }
        }
        if mode == "symbols" {
            use cad_craft::domain::*;
            let mut selected = None;
            for (i, d) in SymbolLibrary::builtin().definitions().enumerate() {
                app.session
                    .dispatch_graphic(cad_gui::editing::GraphicEdit::InsertSymbol {
                        definition: d.id().clone(),
                        target: None,
                        transform: Transform2D::new(
                            Point2D::new(
                                45.0 + (i % 3) as f64 * 85.0,
                                45.0 + (i / 3) as f64 * 42.0,
                            )
                            .expect("the bounded QA grid coordinates are finite"),
                            (i % 4) as f64 * 90.0,
                            i % 3 == 0,
                            false,
                        )
                        .expect("the QA orthogonal transform is valid"),
                    })
                    .expect("the builtin QA symbol can be inserted");
                if d.id().as_str() == "cad-craft.motor" {
                    selected = app.session.graphics.selection.primary();
                }
            }
            if let Some(id) = selected {
                app.session.select_graphics([id]);
            }
        }
    }
    eframe::run_native(
        "CAD-CRAFT • Inspeção de renderização",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([width, height]),
            persist_window: false,
            ..Default::default()
        },
        Box::new(move |cc| {
            cc.egui_ctx.set_zoom_factor(dpi);
            Ok(Box::new(Preview {
                app,
                mode,
                frame: 0,
            }))
        }),
    )
}
