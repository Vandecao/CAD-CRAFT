// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_gui::{
    app::CadApp,
    preferences::{PanelId, RibbonTab},
};
use cad_gui::{commands::CommandId, state::Selection};
use eframe::egui::{Context, Event, Key, Modifiers, Pos2, RawInput, Rect, Vec2};
use eframe::egui::{MouseWheelUnit, PointerButton};
fn frame(app: &mut CadApp, ctx: &Context, w: f32, h: f32, events: Vec<Event>) {
    let _ = ctx.run(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(w, h))),
            events,
            ..Default::default()
        },
        |ctx| app.show(ctx),
    );
}
#[test]
fn viewport_uses_central_space_at_every_zoom() {
    let ctx = Context::default();
    let mut app = CadApp::default();
    app.session
        .replace(cad_gui::demo::project().unwrap(), true)
        .unwrap();
    let id = app.session.project().unwrap().pages().next().unwrap().id();
    app.session.open_page(id).unwrap();
    frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
    frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
    let original = app.last_viewport.unwrap();
    for factor in [0.01, 0.1, 100.0, 10.0] {
        app.session.tabs[0]
            .camera
            .zoom_at(factor, original.center(), original);
        frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
        assert_eq!(app.last_viewport.unwrap(), original);
    }
}
#[test]
fn hiding_side_panels_releases_space_to_viewport() {
    let ctx = Context::default();
    let mut app = CadApp::default();
    frame(&mut app, &ctx, 1280.0, 720.0, vec![]);
    frame(&mut app, &ctx, 1280.0, 720.0, vec![]);
    let initial = app.last_viewport.unwrap();
    for p in [PanelId::Navigator, PanelId::Insertion, PanelId::Properties] {
        app.preferences.panel_mut(p).visible = false;
    }
    frame(&mut app, &ctx, 1280.0, 720.0, vec![]);
    assert!(app.last_viewport.unwrap().width() > initial.width() + 200.0);
}
#[test]
fn layouts_remain_usable_across_resolutions_and_dpi() {
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
            frame(&mut app, &ctx, w / dpi, h / dpi, vec![]);
            let r = app.last_viewport.unwrap();
            assert!(r.width() > 140.0, "{w}/{dpi}: {r:?}");
            assert!(r.height() > 65.0, "{h}/{dpi}: {r:?}");
        }
    }
}
#[test]
fn all_ribbon_tabs_can_render_without_affecting_project() {
    let ctx = Context::default();
    let mut app = CadApp::default();
    app.session
        .replace(cad_gui::demo::project().unwrap(), true)
        .unwrap();
    let before = app.session.snapshot().unwrap();
    for tab in RibbonTab::ALL {
        app.preferences.ribbon_tab = tab;
        frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
    }
    assert_eq!(before, app.session.snapshot().unwrap());
}
#[test]
fn ctrl_f1_collapses_ribbon_and_releases_vertical_space() {
    let ctx = Context::default();
    let mut app = CadApp::default();
    frame(&mut app, &ctx, 1280.0, 720.0, vec![]);
    frame(&mut app, &ctx, 1280.0, 720.0, vec![]);
    let h = app.last_viewport.unwrap().height();
    frame(
        &mut app,
        &ctx,
        1280.0,
        720.0,
        vec![Event::Key {
            key: Key::F1,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::CTRL,
        }],
    );
    assert!(app.preferences.ribbon_collapsed);
    assert!(app.last_viewport.unwrap().height() > h + 40.0);
}

fn demo() -> (CadApp, Context) {
    let ctx = Context::default();
    let mut app = CadApp::default();
    app.session
        .replace(cad_gui::demo::project().unwrap(), true)
        .unwrap();
    let page = app
        .session
        .project()
        .unwrap()
        .pages()
        .find(|p| p.name().as_str() == "Visão geral")
        .unwrap()
        .id();
    app.session.open_page(page).unwrap();
    for _ in 0..3 {
        frame(&mut app, &ctx, 1600.0, 900.0, vec![]);
    }
    (app, ctx)
}
fn input(app: &mut CadApp, ctx: &Context, events: Vec<Event>) {
    frame(app, ctx, 1600.0, 900.0, events);
}
fn button(pos: Pos2, button: PointerButton, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button,
        pressed,
        modifiers: Modifiers::NONE,
    }
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
#[test]
fn wheel_zoom_is_anchored_to_pointer_without_resizing_viewport() {
    let (mut app, ctx) = demo();
    let viewport = app.last_viewport.unwrap();
    let pointer = viewport.center() + Vec2::new(73.0, -45.0);
    input(&mut app, &ctx, vec![Event::PointerMoved(pointer)]);
    let old = app.session.active_tab_mut().unwrap().camera;
    let anchor = old.screen_to_world(pointer, viewport).unwrap();
    input(
        &mut app,
        &ctx,
        vec![Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            delta: Vec2::new(0.0, 80.0),
            modifiers: Modifiers::NONE,
        }],
    );
    let camera = app.session.active_tab_mut().unwrap().camera;
    let after = camera.screen_to_world(pointer, viewport).unwrap();
    assert!(camera.scale() > old.scale());
    assert!((after.x() - anchor.x()).abs() < 1e-9 && (after.y() - anchor.y()).abs() < 1e-9);
    assert_eq!(app.last_viewport, Some(viewport));
}
#[test]
fn middle_drag_pans_paper_and_preserves_project_and_zoom() {
    let (mut app, ctx) = demo();
    let viewport = app.last_viewport.unwrap();
    let start = viewport.center();
    let old = app.session.active_tab_mut().unwrap().camera;
    let anchor = old.screen_to_world(start, viewport).unwrap();
    let project = app.session.snapshot().unwrap();
    input(&mut app, &ctx, vec![Event::PointerMoved(start)]);
    input(
        &mut app,
        &ctx,
        vec![button(start, PointerButton::Middle, true)],
    );
    let end = start + Vec2::new(65.0, 37.0);
    input(&mut app, &ctx, vec![Event::PointerMoved(end)]);
    input(
        &mut app,
        &ctx,
        vec![button(end, PointerButton::Middle, false)],
    );
    let camera = app.session.active_tab_mut().unwrap().camera;
    assert_eq!(camera.scale(), old.scale());
    assert!(
        camera
            .world_to_screen(anchor, viewport)
            .unwrap()
            .distance(end)
            < 0.01
    );
    assert_eq!(project, app.session.snapshot().unwrap());
}
#[test]
fn click_selects_existing_representation_then_empty_paper_selects_page() {
    let (mut app, ctx) = demo();
    let page = app.session.active.unwrap();
    let rep = app
        .session
        .project()
        .unwrap()
        .representations_on_page(page)
        .unwrap()
        .into_iter()
        .find(|r| r.local_bounds().is_some())
        .unwrap();
    let id = rep.id();
    let point = rep
        .transform()
        .apply(cad_craft::domain::Point2D::new(25.0, 14.0).unwrap())
        .unwrap();
    let viewport = app.last_viewport.unwrap();
    let camera = app.session.active_tab_mut().unwrap().camera;
    for (pos, expected) in [
        (
            camera.world_to_screen(point, viewport).unwrap(),
            Selection::Representation(id),
        ),
        (viewport.center(), Selection::Page(page)),
    ] {
        input(&mut app, &ctx, vec![Event::PointerMoved(pos)]);
        input(
            &mut app,
            &ctx,
            vec![button(pos, PointerButton::Primary, true)],
        );
        input(
            &mut app,
            &ctx,
            vec![button(pos, PointerButton::Primary, false)],
        );
        assert_eq!(app.session.selection, expected);
    }
}
#[test]
fn page_shortcut_respects_text_focus_modal_and_backstage() {
    let (mut app, ctx) = demo();
    let first = app.session.active;
    input(&mut app, &ctx, vec![key(Key::PageDown, Modifiers::CTRL)]);
    assert_ne!(app.session.active, first);
    let active = app.session.active;
    ctx.memory_mut(|m| m.request_focus(eframe::egui::Id::new("command_search")));
    input(&mut app, &ctx, vec![]);
    input(
        &mut app,
        &ctx,
        vec![
            key(Key::PageDown, Modifiers::CTRL),
            Event::Text("página".into()),
        ],
    );
    assert_eq!(app.session.active, active);
    input(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
    ctx.memory_mut(|m| m.surrender_focus(eframe::egui::Id::new("command_search")));
    app.execute(CommandId::NewProject, &ctx);
    input(&mut app, &ctx, vec![key(Key::PageDown, Modifiers::CTRL)]);
    assert_eq!(app.session.active, active);
    input(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
    app.execute(CommandId::Settings, &ctx);
    input(&mut app, &ctx, vec![key(Key::PageDown, Modifiers::CTRL)]);
    assert_eq!(app.session.active, active);
    input(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
    input(&mut app, &ctx, vec![key(Key::PageDown, Modifiers::CTRL)]);
    assert_ne!(app.session.active, active);
}
#[test]
fn divider_drag_releases_space_and_restore_layout_recovers_default() {
    let (mut app, ctx) = demo();
    let before = app.last_viewport.unwrap();
    let start = Pos2::new(before.left() - 1.0, before.center().y);
    input(&mut app, &ctx, vec![Event::PointerMoved(start)]);
    input(
        &mut app,
        &ctx,
        vec![button(start, PointerButton::Primary, true)],
    );
    let end = start - Vec2::new(70.0, 0.0);
    input(&mut app, &ctx, vec![Event::PointerMoved(end)]);
    input(
        &mut app,
        &ctx,
        vec![button(end, PointerButton::Primary, false)],
    );
    assert!(app.last_viewport.unwrap().width() > before.width() + 40.0);
    app.execute(CommandId::RestoreLayout, &ctx);
    input(&mut app, &ctx, vec![]);
    assert!((app.last_viewport.unwrap().width() - before.width()).abs() < 2.0);
}

#[test]
fn tree_paint_is_clipped_above_thumbnail_panel_at_small_and_normal_sizes() {
    for (w, h) in [(1600.0, 900.0), (960.0, 540.0)] {
        let (mut app, ctx) = demo();
        for _ in 0..3 {
            frame(&mut app, &ctx, w, h, vec![]);
        }
        let output = ctx.run(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(w, h))),
                ..Default::default()
            },
            |ctx| app.show(ctx),
        );
        let preview_top = output
            .shapes
            .iter()
            .filter_map(|s| match &s.shape {
                eframe::egui::Shape::Text(t)
                    if t.galley.text().to_lowercase().contains("pré-visualização") =>
                {
                    Some(t.pos.y)
                }
                _ => None,
            })
            .reduce(f32::min)
            .expect("preview title or compact bar");
        let mut tree_rows = 0;
        for shape in &output.shapes {
            if let eframe::egui::Shape::Text(text) = &shape.shape
                && (text.galley.text().starts_with('=') || text.galley.text().starts_with('+'))
            {
                tree_rows += 1;
                assert!(
                    shape.clip_rect.bottom() <= preview_top,
                    "{w}×{h}: {:?} extends under preview at {preview_top}",
                    shape.clip_rect
                );
            }
        }
        assert!(
            tree_rows > 0,
            "at least one tree group is visible at {w}×{h}"
        );
    }
}
