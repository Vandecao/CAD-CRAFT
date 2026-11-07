// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_gui::{
    app::CadApp,
    commands::{CommandId, registry},
    preferences::RibbonTab,
};
use eframe::egui::{self, Context, Event, FullOutput, Pos2, RawInput, Rect, Shape, Vec2};

fn frame(app: &mut CadApp, ctx: &Context, w: f32, h: f32, events: Vec<Event>) -> FullOutput {
    ctx.run(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(w, h))),
            events,
            ..Default::default()
        },
        |ctx| app.show(ctx),
    )
}
fn text_shapes(output: &FullOutput) -> impl Iterator<Item = (&egui::epaint::TextShape, Rect)> {
    output.shapes.iter().filter_map(|s| {
        if let Shape::Text(t) = &s.shape {
            Some((t, s.clip_rect))
        } else {
            None
        }
    })
}
fn visible_command(output: &FullOutput, tab: RibbonTab) -> bool {
    let commands = registry();
    text_shapes(output).any(|(t, clip)| {
        t.pos.y > 65.0
            && t.pos.y < 165.0
            && clip.max.y < 185.0
            && clip.contains_rect(Rect::from_min_size(t.pos, t.galley.size()))
            && commands
                .iter()
                .any(|c| c.tab == tab && c.name == t.galley.text())
    })
}
fn luma(c: egui::Color32) -> f32 {
    let f = |v: u8| {
        let v = v as f32 / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * f(c.r()) + 0.7152 * f(c.g()) + 0.0722 * f(c.b())
}
fn contrast(a: egui::Color32, b: egui::Color32) -> f32 {
    (luma(a).max(luma(b)) + 0.05) / (luma(a).min(luma(b)) + 0.05)
}

fn text_center(output: &FullOutput, label: &str, min_y: f32, max_y: f32) -> Pos2 {
    let (t, _) = text_shapes(output)
        .find(|(t, c)| {
            t.galley.text() == label && t.pos.y >= min_y && t.pos.y < max_y && c.contains(t.pos)
        })
        .unwrap_or_else(|| panic!("missing visible {label}"));
    t.pos + t.galley.size() * 0.5
}
fn pointer(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    }
}
fn click(app: &mut CadApp, ctx: &Context, pos: Pos2) -> FullOutput {
    frame(
        app,
        ctx,
        1600.0,
        900.0,
        vec![Event::PointerMoved(pos), pointer(pos, true)],
    );
    frame(app, ctx, 1600.0, 900.0, vec![pointer(pos, false)]);
    frame(app, ctx, 1600.0, 900.0, vec![])
}
#[test]
fn real_tab_clicks_repeat_without_collapsing_and_explicit_double_click_toggles() {
    let ctx = Context::default();
    let mut app = CadApp::default();
    let mut output = frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
    output.shapes.clear();
    for _ in 0..3 {
        output = frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
    }
    for _ in 0..5 {
        for tab in RibbonTab::ALL {
            let pos = text_center(&output, tab.label(), 30.0, 65.0);
            output = click(&mut app, &ctx, pos);
            assert_eq!(app.preferences.ribbon_tab, tab);
            assert!(!app.preferences.ribbon_collapsed, "tab {tab:?}");
            assert!(visible_command(&output, tab), "no commands for {tab:?}");
        }
    }
    // Allow the previous click sequence to expire, then double-click the active tab.
    for _ in 0..60 {
        frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
    }
    let pos = text_center(&output, RibbonTab::Integrations.label(), 30.0, 65.0);
    click(&mut app, &ctx, pos);
    output = click(&mut app, &ctx, pos);
    assert!(app.preferences.ribbon_collapsed);
    let expand = text_center(&output, "Expandir", 30.0, 65.0);
    output = click(&mut app, &ctx, expand);
    assert!(!app.preferences.ribbon_collapsed);
    assert!(visible_command(&output, RibbonTab::Integrations));
}

#[test]
fn backstage_and_documents_preserve_ribbon_camera_selection_and_panel_sizes() {
    let ctx = Context::default();
    let mut app = CadApp::default();
    app.execute(CommandId::Demo, &ctx);
    let ids: Vec<_> = app
        .session
        .project()
        .unwrap()
        .pages()
        .take(3)
        .map(|p| p.id())
        .collect();
    for id in &ids {
        app.session.open_page(*id).unwrap();
    }
    app.preferences.ribbon_tab = RibbonTab::Connections;
    for _ in 0..4 {
        frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
    }
    for collapsed in [false, true] {
        app.preferences.ribbon_collapsed = collapsed;
        for _ in 0..12 {
            frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
        }
        let before = app.preferences.clone();
        let selection = app.session.selection;
        let cameras: Vec<_> = app
            .session
            .tabs
            .iter()
            .map(|t| (t.page, t.camera))
            .collect();
        let output = frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
        let file = text_center(&output, "Arquivo", 30.0, 65.0);
        let output = click(&mut app, &ctx, file);
        let back = text_center(&output, "Voltar", 30.0, 150.0);
        let output = click(&mut app, &ctx, back);
        assert_eq!(app.preferences, before);
        assert_eq!(app.session.selection, selection);
        assert_eq!(
            app.session
                .tabs
                .iter()
                .map(|t| (t.page, t.camera))
                .collect::<Vec<_>>(),
            cameras
        );
        assert_eq!(visible_command(&output, RibbonTab::Connections), !collapsed);
    }
    app.preferences.ribbon_collapsed = false;
    for id in ids {
        app.session.open_page(id).unwrap();
        let output = frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
        assert!(
            visible_command(&output, RibbonTab::Connections),
            "commands={:?}",
            text_shapes(&output)
                .filter(|(t, _)| t.pos.y > 55.0 && t.pos.y < 165.0)
                .map(|(t, c)| (t.galley.text(), t.pos, c))
                .collect::<Vec<_>>()
        );
        app.session.close_tab(id);
    }
    let saved = app.preferences.to_json().unwrap();
    let mut reopened = CadApp::default();
    reopened.preferences = cad_gui::preferences::Preferences::from_json(&saved).unwrap();
    let new_ctx = Context::default();
    frame(&mut reopened, &new_ctx, 1600.0, 900.0, vec![]);
    let output = frame(&mut reopened, &new_ctx, 1600.0, 900.0, vec![]);
    assert!(visible_command(&output, RibbonTab::Connections));
}

#[test]
fn grid_toggle_and_dpi_preserve_snap_and_world_spacing() {
    for dpi in [1.0, 1.25, 1.5, 2.0] {
        let ctx = Context::default();
        ctx.set_pixels_per_point(dpi);
        let mut app = CadApp::default();
        app.execute(CommandId::Demo, &ctx);
        let page = app.session.project().unwrap().pages().next().unwrap().id();
        app.session.open_page(page).unwrap();
        app.preferences.snap_enabled = true;
        app.preferences.grid_mm = 2.5;
        for _ in 0..12 {
            frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
        }
        // Zoom in so the 2.5 mm dots are at least MIN_GRID_PX apart at every
        // DPI (farther out the grid hides, as in EPLAN).
        let view = app.last_viewport.unwrap();
        let scale = app.session.tabs[0].camera.scale();
        let needed = cad_gui::camera::MIN_GRID_PX * 2.0 / (2.5 * scale);
        if needed > 1.0 {
            app.session.tabs[0]
                .camera
                .zoom_at(needed, view.center(), view);
        }
        frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
        let before = app.last_viewport;
        let camera = app.session.tabs[0].camera;
        let output = frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
        assert!(output.shapes.iter().any(|s|matches!(&s.shape,Shape::Circle(c) if c.radius==0.8 && before.unwrap().contains(c.center))));
        app.execute(CommandId::ToggleGrid, &ctx);
        let output = frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
        assert!(!output.shapes.iter().any(|s|matches!(&s.shape,Shape::Circle(c) if c.radius==0.8 && before.unwrap().contains(c.center))));
        assert!(app.preferences.snap_enabled);
        assert_eq!(app.preferences.grid_mm, 2.5);
        assert_eq!(app.last_viewport, before);
        assert_eq!(app.session.tabs[0].camera, camera);
    }
}

#[test]
fn expanded_ribbon_never_silently_hides_commands_on_short_windows() {
    let ctx = Context::default();
    let mut app = CadApp::default();
    for height in [900.0, 470.0, 360.0, 900.0] {
        for tab in RibbonTab::ALL {
            app.preferences.ribbon_tab = tab;
            for _ in 0..2 {
                frame(&mut app, &ctx, 1280.0, height, vec![]);
            }
            let output = frame(&mut app, &ctx, 1280.0, height, vec![]);
            assert!(!app.preferences.ribbon_collapsed);
            assert!(
                visible_command(&output, tab),
                "Expanded {tab:?} has no visible command at height {height}"
            );
        }
    }
}
#[test]
fn welcome_actions_have_readable_text_on_their_actual_button_fill() {
    let ctx = Context::default();
    let mut app = CadApp::default();
    frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
    let output = frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
    for label in [
        "Criar sessão em memória",
        "Abrir demonstração CAD-CRAFT",
        "Importar snapshot técnico",
    ] {
        let (text, _) = text_shapes(&output)
            .find(|(t, c)| t.galley.text() == label && t.pos.y > 200.0 && c.contains(t.pos))
            .unwrap();
        let center = text.pos + text.galley.size() * 0.5;
        let fill = output
            .shapes
            .iter()
            .rev()
            .find_map(|s| match &s.shape {
                Shape::Rect(r) if r.rect.contains(center) && r.rect.width() < 400.0 => Some(r.fill),
                _ => None,
            })
            .unwrap();
        let color = text.galley.job.sections[0].format.color;
        assert!(
            contrast(color, fill) >= 4.5,
            "{label}: {color:?}/{fill:?}, contrast {}",
            contrast(color, fill)
        );
    }
}
#[test]
fn grid_is_painted_as_points_without_continuous_grid_lines() {
    let ctx = Context::default();
    let mut app = CadApp::default();
    app.execute(CommandId::Demo, &ctx);
    let page = app.session.project().unwrap().pages().next().unwrap().id();
    app.session.open_page(page).unwrap();
    for _ in 0..2 {
        frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
    }
    let output = frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
    let viewport = app.last_viewport.unwrap();
    let dots=output.shapes.iter().filter(|s|matches!(&s.shape,Shape::Circle(c) if c.radius<=1.0 && c.fill!=egui::Color32::TRANSPARENT && viewport.contains(c.center))).count();
    assert!(dots > 100, "expected dots, got {dots}");
    assert!(!output.shapes.iter().any(|s| match &s.shape {
        Shape::LineSegment { points, stroke } =>
            stroke.width < 0.5
                && (points[0].distance(points[1]) >= viewport.width().min(viewport.height()) - 1.0),
        _ => false,
    }));
}
#[cfg(target_os = "windows")]
#[test]
fn startup_requests_dark_native_decoration_without_removing_window_controls() {
    let ctx = Context::default();
    let mut app = CadApp::default();
    let output = frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
    let commands = &output.viewport_output[&egui::ViewportId::ROOT].commands;
    assert!(
        commands
            .iter()
            .any(|c| matches!(c, egui::ViewportCommand::SetTheme(egui::SystemTheme::Dark)))
    );
    assert!(
        !commands
            .iter()
            .any(|c| matches!(c, egui::ViewportCommand::Decorations(false)))
    );
}

#[cfg(target_os = "windows")]
#[test]
fn native_dark_theme_is_reasserted_on_focus_and_system_theme_changes_only() {
    let ctx = Context::default();
    let mut app = CadApp::default();
    frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
    let output = frame(
        &mut app,
        &ctx,
        1600.0,
        900.0,
        vec![Event::WindowFocused(true)],
    );
    let requested = |out: &FullOutput| {
        out.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .iter()
            .any(|c| matches!(c, egui::ViewportCommand::SetTheme(egui::SystemTheme::Dark)))
    };
    assert!(requested(&output));
    assert!(!requested(&frame(&mut app, &ctx, 1600.0, 900.0, vec![])));
    for (system_theme, expected) in [
        (Some(egui::Theme::Light), true),
        (Some(egui::Theme::Light), false),
        (Some(egui::Theme::Dark), true),
    ] {
        let output = ctx.run(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 900.0))),
                system_theme,
                ..Default::default()
            },
            |ctx| app.show(ctx),
        );
        assert_eq!(requested(&output), expected);
    }
}

#[test]
fn horizontal_ribbon_scroll_reaches_last_group_and_is_isolated_per_tab() {
    let ctx = Context::default();
    let mut app = CadApp::default();
    for _ in 0..3 {
        frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
    }
    let pos = egui::pos2(700.0, 105.0);
    for _ in 0..15 {
        frame(
            &mut app,
            &ctx,
            1600.0,
            900.0,
            vec![
                Event::PointerMoved(pos),
                Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(-400.0, 0.0),
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
    }
    let output = frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
    text_center(&output, "Importar snapshot técnico", 65.0, 165.0);
    let insert = text_center(&output, RibbonTab::Insert.label(), 30.0, 65.0);
    let output = click(&mut app, &ctx, insert);
    assert!(visible_command(&output, RibbonTab::Insert));
    let home = text_center(&output, RibbonTab::Home.label(), 30.0, 65.0);
    let output = click(&mut app, &ctx, home);
    text_center(&output, "Importar snapshot técnico", 65.0, 165.0);
    assert!(!app.preferences.ribbon_collapsed);
}

#[test]
fn ribbon_and_welcome_actions_remain_accessible_across_resolution_and_dpi_matrix() {
    for (w, h) in [
        (1280.0, 720.0),
        (1366.0, 768.0),
        (1600.0, 900.0),
        (1920.0, 1080.0),
        (1920.0, 1200.0),
        (2560.0, 1440.0),
    ] {
        for dpi in [1.0, 1.25, 1.5, 2.0] {
            let ctx = Context::default();
            ctx.set_pixels_per_point(dpi);
            let mut app = CadApp::default();
            for tab in RibbonTab::ALL {
                app.preferences.ribbon_tab = tab;
                let output = frame(&mut app, &ctx, w / dpi, h / dpi, vec![]);
                assert!(visible_command(&output, tab), "{w}x{h}/{dpi}/{tab:?}");
                assert!(app.last_viewport.unwrap().top() > 165.0);
            }
            let labels = [
                "Criar sessão em memória",
                "Abrir demonstração CAD-CRAFT",
                "Importar snapshot técnico",
            ];
            let mut seen = [false; 3];
            for _ in 0..60 {
                let viewport = app.last_viewport.unwrap();
                let output = frame(
                    &mut app,
                    &ctx,
                    w / dpi,
                    h / dpi,
                    vec![
                        Event::PointerMoved(viewport.center()),
                        Event::MouseWheel {
                            unit: egui::MouseWheelUnit::Point,
                            delta: egui::vec2(0.0, -12.0),
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                );
                for (i, label) in labels.iter().enumerate() {
                    for (t, clip) in text_shapes(&output).filter(|(t, _)| t.galley.text() == *label)
                    {
                        let bounds = Rect::from_min_size(t.pos, t.galley.size());
                        if viewport.contains_rect(bounds) && clip.contains_rect(bounds) {
                            seen[i] = true;
                            assert!((bounds.center().x - viewport.center().x).abs() < 12.0);
                        }
                    }
                }
                if seen.iter().all(|v| *v) {
                    break;
                }
            }
            assert!(seen.iter().all(|v| *v), "welcome {w}x{h}/{dpi}: {seen:?}");
        }
    }
}
