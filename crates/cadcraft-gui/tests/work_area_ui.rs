// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Work area like EPLAN, with real egui frames: one background colour,
//! the frame as the limit, a thin outline for pages without a frame, cursor
//! guide lines, negative coordinates outside the frame and "Ajustar folha à
//! moldura" with one Ctrl+Z.
// Fixture helpers use validated synthetic inputs and fail immediately on setup errors.
#![allow(clippy::unwrap_used)]
use cad_craft::domain::*;
use cad_gui::{app::CadApp, editing::GraphicEdit, theme};
use eframe::egui::{
    self, Color32, Context, Event, Key, Modifiers, PointerButton, Pos2, Rect, Shape,
    accesskit::Role, epaint::RectShape, vec2,
};

const SCREEN: Rect = Rect {
    min: Pos2::ZERO,
    max: Pos2::new(1600.0, 1000.0),
};

/// Project with an A3 page (factory A3 frame) and a 500 × 250 page.
fn app(with_frame: bool) -> (CadApp, Context, PageId, PageId) {
    let mut project = if with_frame {
        cad_gui::state::with_factory_frames(Project::new("Área").unwrap()).unwrap()
    } else {
        Project::new("Área").unwrap()
    };
    let doc = project
        .insert_document(Document::new(project.id(), "Esquema").unwrap())
        .unwrap();
    let mut page = |n: &str, size: Size2D| {
        project
            .insert_page(Page::new(project.id(), doc, n, n, PageKind::MultiLine, size).unwrap())
            .unwrap()
    };
    let a3 = page("1", PaperFormat::A3.size(true));
    let wide = page("2", Size2D::new(500.0, 250.0).unwrap());
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut app = CadApp::default();
    app.session.replace(project, false).unwrap();
    app.session.open_page(a3).unwrap();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    (app, ctx, a3, wide)
}
fn frame(app: &mut CadApp, ctx: &Context, events: Vec<Event>) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(SCREEN),
            events,
            ..Default::default()
        },
        |ctx| app.show(ctx),
    )
}
fn work_area(app: &CadApp) -> Rect {
    app.last_viewport.unwrap()
}
fn rects(out: &egui::FullOutput) -> Vec<RectShape> {
    let mut found = vec![];
    for clipped in &out.shapes {
        if let Shape::Rect(r) = &clipped.shape {
            found.push(r.clone());
        }
    }
    found
}
/// Line segments of the given colour (guides are 1 px line segments).
fn lines(out: &egui::FullOutput, color: Color32) -> Vec<[Pos2; 2]> {
    let mut found = vec![];
    for clipped in &out.shapes {
        if let Shape::LineSegment { points, stroke } = &clipped.shape
            && stroke.color == color
        {
            found.push(*points);
        }
    }
    found
}
fn hover(app: &mut CadApp, ctx: &Context, at: Pos2) -> egui::FullOutput {
    frame(app, ctx, vec![Event::PointerMoved(at)]);
    frame(app, ctx, vec![Event::PointerMoved(at)])
}
fn labels(out: &egui::FullOutput) -> Vec<String> {
    out.platform_output
        .accesskit_update
        .as_ref()
        .map(|u| {
            u.nodes
                .iter()
                .filter_map(|(_, n)| n.label().or_else(|| n.value()).map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn the_work_area_has_one_colour_and_no_paper_rectangle() {
    let (mut app, ctx, _, _) = app(true);
    let out = frame(&mut app, &ctx, vec![]);
    let area = work_area(&app);
    let found = rects(&out);
    assert!(
        found
            .iter()
            .any(|r| r.fill == theme::WORK_AREA && r.rect.contains_rect(area.shrink(1.0))),
        "fundo único da área de trabalho"
    );
    assert!(
        !found
            .iter()
            .any(|r| r.fill == theme::CANVAS && r.rect.intersects(area.shrink(2.0))),
        "nenhuma parte cinza na área de trabalho"
    );
}

#[test]
fn cursor_guides_cross_the_work_area_and_hide_outside_or_when_off() {
    let (mut app, ctx, _, _) = app(true);
    app.preferences.snap_enabled = false;
    let area = work_area(&app);
    let at = area.center() + vec2(37.0, -21.0);
    let out = hover(&mut app, &ctx, at);
    let guides = lines(&out, theme::CURSOR_GUIDE);
    assert!(
        guides.iter().any(|[a, b]| (a.y - at.y).abs() < 0.6
            && (b.y - at.y).abs() < 0.6
            && (a.x - area.left()).abs() < 1.0
            && (b.x - area.right()).abs() < 1.0),
        "linha horizontal de ponta a ponta: {guides:?}"
    );
    assert!(
        guides.iter().any(|[a, b]| (a.x - at.x).abs() < 0.6
            && (a.y - area.top()).abs() < 1.0
            && (b.y - area.bottom()).abs() < 1.0),
        "linha vertical de ponta a ponta"
    );
    // Outside the work area (over the ribbon) there are no guides.
    let out = hover(&mut app, &ctx, Pos2::new(area.center().x, 20.0));
    assert!(lines(&out, theme::CURSOR_GUIDE).is_empty());
    // Switched off from the View command, none either.
    app.execute(cad_gui::commands::CommandId::ToggleCursorGuides, &ctx);
    assert!(!app.preferences.cursor_guides);
    let out = hover(&mut app, &ctx, at);
    assert!(lines(&out, theme::CURSOR_GUIDE).is_empty());
}

#[test]
fn guides_follow_the_snapped_point_when_snap_is_on() {
    let (mut app, ctx, a3, _) = app(true);
    app.preferences.snap_enabled = true;
    app.preferences.grid_mm = 5.0;
    let area = work_area(&app);
    let at = area.center() + vec2(3.3, 2.7);
    let out = hover(&mut app, &ctx, at);
    let camera = app
        .session
        .tabs
        .iter()
        .find(|t| t.page == a3)
        .unwrap()
        .camera;
    let world = camera.screen_to_world(at, area).unwrap();
    let snapped = cad_gui::camera::snap(world, 5.0, true);
    let screen = camera.world_to_screen(snapped, area).unwrap();
    let guides = lines(&out, theme::CURSOR_GUIDE);
    assert!(guides.iter().any(|[a, _]| (a.y - screen.y).abs() < 0.6));
    assert!(guides.iter().any(|[a, _]| (a.x - screen.x).abs() < 0.6));
}

#[test]
fn coordinates_continue_below_the_frame_as_negative_values() {
    let (mut app, ctx, a3, _) = app(true);
    let area = work_area(&app);
    let camera = app
        .session
        .tabs
        .iter()
        .find(|t| t.page == a3)
        .unwrap()
        .camera;
    let below = camera
        .world_to_screen(Point2D::new(100.0, -20.0).unwrap(), area)
        .unwrap();
    assert!(
        area.contains(below),
        "abaixo da moldura ainda é área de trabalho"
    );
    let out = hover(&mut app, &ctx, below);
    assert!(
        labels(&out)
            .iter()
            .any(|l| l.contains("Y: -20.00 mm") || l.contains("Y: -19.9")),
        "{:?}",
        labels(&out)
            .into_iter()
            .filter(|l| l.contains("Y:"))
            .collect::<Vec<_>>()
    );
}

#[test]
fn a_page_without_frame_is_only_a_thin_outline() {
    let (mut app, ctx, _, _) = app(false);
    let out = frame(&mut app, &ctx, vec![]);
    assert!(
        !rects(&out)
            .iter()
            .any(|r| r.fill == theme::PAPER && r.rect.width() < work_area(&app).width() - 2.0),
        "sem folha branca destacada"
    );
    assert!(
        !lines(&out, theme::GRID).is_empty(),
        "contorno fino da folha"
    );
}

#[test]
fn fit_sheet_to_frame_button_resizes_the_page_and_ctrl_z_restores_it() {
    let (mut app, ctx, _, wide) = app(true);
    app.session.open_page(wide).unwrap();
    app.session.select(cad_gui::state::Selection::Page(wide));
    let mut out = frame(&mut app, &ctx, vec![]);
    for _ in 0..3 {
        out = frame(&mut app, &ctx, vec![]);
    }
    let button = out
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, n)| n.label() == Some("Ajustar folha à moldura") && n.role() == Role::Button)
        .and_then(|(_, n)| n.bounds())
        .map(|b| Pos2::new(((b.x0 + b.x1) / 2.) as f32, ((b.y0 + b.y1) / 2.) as f32))
        .expect("botão Ajustar folha à moldura no painel de propriedades");
    let click = |pressed| Event::PointerButton {
        pos: button,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    frame(&mut app, &ctx, vec![Event::PointerMoved(button)]);
    frame(&mut app, &ctx, vec![click(true)]);
    frame(&mut app, &ctx, vec![click(false)]);
    frame(&mut app, &ctx, vec![]);
    let size = |app: &CadApp| app.session.project().unwrap().page(wide).unwrap().size();
    assert_eq!(size(&app), PaperFormat::A3.size(true));
    frame(
        &mut app,
        &ctx,
        vec![Event::Key {
            key: Key::Z,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::CTRL,
        }],
    );
    frame(&mut app, &ctx, vec![]);
    assert_eq!(size(&app), Size2D::new(500.0, 250.0).unwrap());
}

#[test]
fn choosing_a_frame_for_a_page_adopts_its_size_in_one_undo_step() {
    let (mut app, _ctx, _, wide) = app(true);
    let a4 = app
        .session
        .project()
        .unwrap()
        .frames()
        .find(|f| f.name().as_str() == "CAD-CRAFT A4 retrato")
        .unwrap()
        .id();
    // The same batch the properties panel sends.
    app.graphic_edit(GraphicEdit::Frame(FrameEdit::Batch(vec![
        FrameEdit::SetPageFrame {
            page: wide,
            frame: Some(a4),
        },
        FrameEdit::FitPageToFrame { page: wide },
    ])));
    let page = |app: &CadApp| app.session.project().unwrap().page(wide).unwrap().clone();
    assert_eq!(page(&app).size(), PaperFormat::A4.size(false));
    assert_eq!(page(&app).frame(), Some(a4));
    app.graphic_edit(GraphicEdit::Undo);
    assert_eq!(page(&app).size(), Size2D::new(500.0, 250.0).unwrap());
    assert_eq!(page(&app).frame(), None);
}

#[test]
fn fitting_a_page_without_any_frame_is_refused() {
    let (mut app, _ctx, a3, _) = app(false);
    let before = app.session.project().unwrap().page(a3).unwrap().size();
    app.graphic_edit(GraphicEdit::Frame(FrameEdit::FitPageToFrame { page: a3 }));
    assert_eq!(
        app.session.project().unwrap().page(a3).unwrap().size(),
        before
    );
}
