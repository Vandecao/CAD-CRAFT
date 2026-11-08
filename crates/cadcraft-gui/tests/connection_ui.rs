// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Fixture helpers use validated synthetic inputs and fail immediately on setup errors.
#![allow(clippy::unwrap_used)]
use cad_craft::domain::*;
use cad_gui::{
    app::CadApp, camera::Camera, commands::CommandId, connection_demo, editing::GraphicEdit,
    interaction::Gesture,
};
use eframe::egui::{self, Context, Event, Key, Modifiers, PointerButton, Pos2, Rect, vec2};

fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}
fn frame(app: &mut CadApp, ctx: &Context, events: Vec<Event>) -> egui::FullOutput {
    let modifiers = events
        .iter()
        .rev()
        .find_map(|e| match e {
            Event::PointerButton { modifiers, .. } | Event::Key { modifiers, .. } => {
                Some(*modifiers)
            }
            _ => None,
        })
        .unwrap_or(Modifiers::NONE);
    frame_modifiers(app, ctx, events, modifiers)
}
fn frame_modifiers(
    app: &mut CadApp,
    ctx: &Context,
    events: Vec<Event>,
    modifiers: Modifiers,
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1600., 900.))),
            events,
            modifiers,
            ..Default::default()
        },
        |ctx| app.show(ctx),
    )
}

#[test]
fn junction_marker_at_bound_port_is_selectable_by_real_click() {
    let (mut app, ctx) = setup(false, 2., 1.);
    let ids: Vec<_> = app
        .session
        .project()
        .unwrap()
        .connection_routes()
        .map(ConnectionRoute::id)
        .collect();
    let vertical = app
        .session
        .project()
        .unwrap()
        .connection_routes()
        .find(|r| r.points().first() == Some(&p(160., 60.)))
        .unwrap()
        .id();
    app.graphic_edit(GraphicEdit::Drawing(ConnectionDrawingEdit::SetRoute {
        id: vertical,
        points: vec![
            p(160., 60.),
            p(80., 60.),
            p(80., 100.),
            p(160., 100.),
            p(160., 220.),
        ],
    }));
    app.graphic_edit(GraphicEdit::Drawing(
        ConnectionDrawingEdit::CreateJunction {
            page: app.session.active.unwrap(),
            position: p(80., 100.),
            routes: ids,
        },
    ));
    let id = app
        .session
        .project()
        .unwrap()
        .connection_junctions()
        .next()
        .expect("junction fixture")
        .id();
    let before = app.session.snapshot().unwrap();
    click_world(&mut app, &ctx, p(80., 100.));
    assert_eq!(app.session.graphics.junction_selection, Some(id));
    assert_eq!(app.session.snapshot().unwrap(), before);
}

#[test]
fn symbol_stroke_over_route_keeps_selected_symbol_drag_precedence() {
    let (mut app, ctx) = setup(false, 2., 1.);
    let symbol = app
        .session
        .project()
        .unwrap()
        .representations()
        .find(|r| r.symbol().is_some() && r.transform().translation() == p(80., 110.))
        .unwrap()
        .id();
    app.session.select_graphics([symbol]);
    let before = app.session.snapshot().unwrap();
    let at = screen(&mut app, p(80., 116.));
    let target = screen(&mut app, p(92., 116.));
    frame(
        &mut app,
        &ctx,
        vec![
            Event::PointerMoved(at),
            button(at, PointerButton::Primary, true),
        ],
    );
    frame(&mut app, &ctx, vec![Event::PointerMoved(target)]);
    assert!(
        app.session.graphics.route_drag.is_none(),
        "route behind the selected symbol must not capture its drag"
    );
    assert!(matches!(
        app.session.graphics.gesture,
        Some(Gesture::Move(_))
    ));
    assert_eq!(app.session.graphics.selection.primary(), Some(symbol));
    assert_eq!(app.session.snapshot().unwrap(), before);
    frame(
        &mut app,
        &ctx,
        vec![button(target, PointerButton::Primary, false)],
    );
    assert_eq!(app.session.history.undo_len(), 1);
    assert_ne!(
        app.session
            .project()
            .unwrap()
            .representation(symbol)
            .unwrap()
            .transform()
            .translation(),
        p(80., 110.)
    );
    app.execute(CommandId::Undo, &ctx);
    assert_eq!(app.session.snapshot().unwrap(), before);
}

#[test]
fn ctrl_dragging_route_never_edits_geometry_or_history() {
    let (mut app, ctx) = setup(false, 2., 1.);
    let before = app.session.snapshot().unwrap();
    let at = screen(&mut app, p(120., 140.));
    let target = screen(&mut app, p(120., 152.));
    frame_modifiers(
        &mut app,
        &ctx,
        vec![
            Event::PointerMoved(at),
            Event::PointerButton {
                pos: at,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::CTRL,
            },
        ],
        Modifiers::CTRL,
    );
    frame_modifiers(
        &mut app,
        &ctx,
        vec![Event::PointerMoved(target)],
        Modifiers::CTRL,
    );
    frame_modifiers(
        &mut app,
        &ctx,
        vec![Event::PointerButton {
            pos: target,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::CTRL,
        }],
        Modifiers::CTRL,
    );
    assert!(
        app.session.snapshot().unwrap() == before,
        "Ctrl drag must preserve additive-selection behavior"
    );
    assert_eq!(app.session.history.undo_len(), 0);
    assert!(app.session.graphics.route_drag.is_none());
}

#[test]
fn window_selected_symbols_and_route_move_together_in_one_transaction() {
    let (mut app, ctx) = setup(false, 2., 1.);
    let route = app
        .session
        .project()
        .unwrap()
        .connection_routes()
        .find(|r| r.points().contains(&p(80., 140.)))
        .unwrap()
        .id();
    let symbols: Vec<_> = app
        .session
        .project()
        .unwrap()
        .representations()
        .filter(|r| {
            r.symbol().is_some()
                && [p(80., 110.), p(240., 190.)].contains(&r.transform().translation())
        })
        .map(|r| (r.id(), r.transform().translation()))
        .collect();
    let begin = screen(&mut app, p(60., 210.));
    let end = screen(&mut app, p(260., 80.));
    frame(
        &mut app,
        &ctx,
        vec![
            Event::PointerMoved(begin),
            button(begin, PointerButton::Primary, true),
        ],
    );
    frame(&mut app, &ctx, vec![Event::PointerMoved(end)]);
    frame(
        &mut app,
        &ctx,
        vec![button(end, PointerButton::Primary, false)],
    );
    assert_eq!(
        app.session.graphics.selection.ids().len(),
        3,
        "window must select two symbols and their whole route"
    );
    assert!(app.session.graphics.selection.ids().contains(&route));
    let before = app.session.snapshot().unwrap();
    let start = screen(&mut app, p(80., 116.));
    let target = screen(&mut app, p(92., 124.));
    frame(
        &mut app,
        &ctx,
        vec![
            Event::PointerMoved(start),
            button(start, PointerButton::Primary, true),
        ],
    );
    frame(&mut app, &ctx, vec![Event::PointerMoved(target)]);
    assert_eq!(app.session.snapshot().unwrap(), before);
    frame(
        &mut app,
        &ctx,
        vec![button(target, PointerButton::Primary, false)],
    );
    assert_eq!(
        app.session.history.undo_len(),
        1,
        "group movement should be one committed transaction"
    );
    for (id, position) in symbols {
        assert_eq!(
            app.session
                .project()
                .unwrap()
                .representation(id)
                .unwrap()
                .transform()
                .translation(),
            if position == p(80., 110.) {
                p(92., 120.)
            } else {
                p(252., 200.)
            }
        );
    }
    let expected = [p(92., 110.), p(92., 150.), p(252., 150.), p(252., 190.)];
    assert_eq!(
        app.session
            .project()
            .unwrap()
            .connection_route(route)
            .unwrap()
            .points(),
        expected
    );
    app.execute(CommandId::Undo, &ctx);
    assert_eq!(app.session.snapshot().unwrap(), before);
}
fn button(pos: Pos2, button: PointerButton, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button,
        pressed,
        modifiers: Modifiers::NONE,
    }
}
fn click(app: &mut CadApp, ctx: &Context, pos: Pos2, which: PointerButton) {
    frame(
        app,
        ctx,
        vec![Event::PointerMoved(pos), button(pos, which, true)],
    );
    frame(app, ctx, vec![button(pos, which, false)]);
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
fn setup(blank: bool, scale: f64, dpi: f32) -> (CadApp, Context) {
    let mut project = connection_demo::project().unwrap();
    if blank {
        let ids: Vec<_> = project
            .connection_routes()
            .map(ConnectionRoute::id)
            .collect();
        for id in ids {
            project
                .edit_connection_drawing(ConnectionDrawingEdit::DeleteRoute { id })
                .unwrap();
        }
    }
    let page = project.pages().next().expect("fixture page").id();
    let mut app = CadApp::default();
    let ctx = Context::default();
    ctx.set_pixels_per_point(dpi);
    app.session.replace(project, false).unwrap();
    app.session.open_page(page).unwrap();
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    app.preferences.grid_visible = false;
    app.preferences.grid_mm = 4.;
    app.preferences.snap_enabled = true;
    let tab = app.session.active_tab_mut().unwrap();
    tab.camera = Camera::new([160., 140.], scale).unwrap();
    tab.needs_fit = false;
    frame(&mut app, &ctx, vec![]);
    (app, ctx)
}
fn screen(app: &mut CadApp, point: Point2D) -> Pos2 {
    let viewport = app.last_viewport.unwrap();
    app.session
        .active_tab_mut()
        .unwrap()
        .camera
        .world_to_screen(point, viewport)
        .unwrap()
}
fn click_world(app: &mut CadApp, ctx: &Context, point: Point2D) {
    let at = screen(app, point);
    click(app, ctx, at, PointerButton::Primary);
}
fn texts(output: &egui::FullOutput) -> Vec<String> {
    output
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            egui::Shape::Text(t) => Some(t.galley.job.text.clone()),
            _ => None,
        })
        .collect()
}
fn text_center(output: &egui::FullOutput, value: &str) -> Pos2 {
    output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Text(t)
                if t.galley.job.text == value
                    && s.clip_rect.contains(t.visual_bounding_rect().center()) =>
            {
                Some(t.visual_bounding_rect().center())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("visible text: {value}"))
}

#[test]
fn demo_has_bound_ports_and_two_crossing_electrically_separate_routes() {
    let project = connection_demo::project().unwrap();
    assert_eq!(project.pages().count(), 1);
    assert_eq!(project.devices().count(), 4);
    assert_eq!(project.functions().count(), 4);
    assert_eq!(project.connection_routes().count(), 2);
    assert_eq!(project.connection_junctions().count(), 0);
    assert_eq!(project.conductors().count(), 0);
    let symbols: Vec<_> = project
        .representations()
        .filter(|r| r.symbol().is_some())
        .collect();
    assert_eq!(symbols.len(), 4);
    for symbol in symbols {
        let definition = project.symbol_definition(symbol.symbol().unwrap()).unwrap();
        for port in definition.ports() {
            assert!(
                project
                    .port_binding(&PortRef::new(symbol.id(), port.id()).unwrap())
                    .is_some()
            );
        }
    }
    let routes: Vec<_> = project.connection_routes().collect();
    assert!(routes.iter().all(|r| r.contains(p(160., 140.))));
    let first = project
        .network_for_point(project.port_binding(routes[0].source()).unwrap())
        .unwrap();
    assert!(!first.connections().contains(&routes[1].connection_id()));
    project.validate().unwrap();
}

#[test]
fn newly_drawn_automatic_elbow_follows_endpoint_without_becoming_a_waypoint() {
    let (mut app, ctx) = setup(true, 2., 1.);
    app.execute(CommandId::DrawConnection, &ctx);
    click_world(&mut app, &ctx, p(80., 100.));
    click_world(&mut app, &ctx, p(240., 180.));
    let route = app
        .session
        .project()
        .unwrap()
        .connection_routes()
        .next()
        .unwrap()
        .clone();
    assert_eq!(
        route.points(),
        &[p(80., 100.), p(240., 100.), p(240., 180.)]
    );
    app.session
        .select_graphics([route.target().representation_id()]);
    app.graphic_edit(GraphicEdit::Translate(40., 0.));
    assert_eq!(
        app.session
            .project()
            .unwrap()
            .connection_route(route.id())
            .unwrap()
            .points(),
        &[p(80., 100.), p(280., 100.), p(280., 180.)]
    );
    app.graphic_edit(GraphicEdit::Translate(-40., 0.));
    assert_eq!(
        app.session
            .project()
            .unwrap()
            .connection_route(route.id())
            .unwrap()
            .points(),
        route.points()
    );
}

#[test]
fn real_connected_drags_return_to_initial_route_with_hidden_grid_and_snap_on_or_off() {
    for enabled in [true, false] {
        for vertical in [true, false] {
            let (mut app, ctx) = setup(true, 2., 1.);
            app.preferences.snap_enabled = enabled;
            let target = app
                .session
                .project()
                .unwrap()
                .representations()
                .find(|r| r.transform().translation() == p(240., 190.))
                .unwrap()
                .id();
            // Align the authored anchor to the logical grid before the cycle.
            app.graphic_edit(GraphicEdit::Set(vec![(
                target,
                Transform2D::new(p(240., 192.), 0., false, false).unwrap(),
            )]));
            app.execute(CommandId::DrawConnection, &ctx);
            click_world(&mut app, &ctx, p(80., 100.));
            click_world(&mut app, &ctx, p(240., 182.));
            assert_eq!(
                app.session.project().unwrap().connection_routes().count(),
                1
            );
            app.execute(CommandId::Select, &ctx);
            app.session.select_graphics([target]);
            // Settle the inspector layout before mapping document positions to
            // pointer pixels. Layout changes are not part of a route resize.
            for _ in 0..5 {
                frame(&mut app, &ctx, vec![]);
            }
            let initial = app.session.snapshot().unwrap();
            for offset in [40., 20., 0.] {
                for _ in 0..3 {
                    frame(&mut app, &ctx, vec![]);
                }
                let old = app
                    .session
                    .project()
                    .unwrap()
                    .representation(target)
                    .unwrap()
                    .transform()
                    .translation();
                let desired = if vertical {
                    p(240., 192. + offset)
                } else {
                    p(240. + offset, 192.)
                };
                let start = screen(&mut app, old);
                let end = screen(&mut app, desired);
                let before = app.session.snapshot().unwrap();
                frame(
                    &mut app,
                    &ctx,
                    vec![
                        Event::PointerMoved(start),
                        button(start, PointerButton::Primary, true),
                    ],
                );
                frame(&mut app, &ctx, vec![Event::PointerMoved(end)]);
                assert_eq!(app.session.snapshot().unwrap(), before);
                frame(
                    &mut app,
                    &ctx,
                    vec![button(end, PointerButton::Primary, false)],
                );
                assert_eq!(
                    app.session
                        .project()
                        .unwrap()
                        .representation(target)
                        .unwrap()
                        .transform()
                        .translation(),
                    desired
                );
                let after = app.session.snapshot().unwrap();
                app.execute(CommandId::Undo, &ctx);
                assert_eq!(app.session.snapshot().unwrap(), before);
                app.execute(CommandId::Redo, &ctx);
                assert_eq!(app.session.snapshot().unwrap(), after);
            }
            assert_eq!(app.session.snapshot().unwrap(), initial);
        }
    }
}

#[test]
fn real_draw_preview_waypoint_confirmation_and_undo_work_at_two_scales() {
    for (scale, dpi) in [(1.5, 1.), (2.5, 1.5)] {
        let (mut app, ctx) = setup(true, scale, dpi);
        let before = app.session.snapshot().unwrap();
        app.execute(CommandId::DrawConnection, &ctx);
        assert!(app.session.graphics.connection.is_some());
        click_world(&mut app, &ctx, p(80., 100.));
        let cursor = screen(&mut app, p(119.1, 141.1));
        frame(&mut app, &ctx, vec![Event::PointerMoved(cursor)]);
        assert_eq!(app.session.snapshot().unwrap(), before);
        assert_eq!(app.session.history.undo_len(), 0);
        assert_eq!(
            app.session
                .graphics
                .connection
                .as_ref()
                .unwrap()
                .preview
                .last(),
            Some(&p(120., 140.))
        );
        click_world(&mut app, &ctx, p(120., 140.));
        click_world(&mut app, &ctx, p(240., 180.));
        let route = app
            .session
            .project()
            .unwrap()
            .connection_routes()
            .next()
            .expect("confirmed route");
        assert_eq!(route.points().first(), Some(&p(80., 100.)));
        assert_eq!(route.points().last(), Some(&p(240., 180.)));
        assert!(route.points().contains(&p(120., 140.)));
        let id = route.id();
        let after = app.session.snapshot().unwrap();
        assert_eq!(app.session.history.undo_len(), 1);
        app.execute(CommandId::Undo, &ctx);
        assert_eq!(app.session.snapshot().unwrap(), before);
        app.execute(CommandId::Redo, &ctx);
        assert_eq!(app.session.snapshot().unwrap(), after);
        assert!(
            app.session
                .project()
                .unwrap()
                .connection_route(id)
                .is_some()
        );
    }
}

#[test]
fn real_escape_focus_middle_and_modal_cancel_without_document_changes() {
    for cancel in 0..4 {
        let (mut app, ctx) = setup(true, 2., 1.);
        let before = app.session.snapshot().unwrap();
        app.execute(CommandId::DrawConnection, &ctx);
        click_world(&mut app, &ctx, p(80., 100.));
        let at = screen(&mut app, p(120., 140.));
        frame(&mut app, &ctx, vec![Event::PointerMoved(at)]);
        match cancel {
            0 => {
                frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
            }
            1 => {
                frame(&mut app, &ctx, vec![Event::WindowFocused(false)]);
            }
            2 => {
                frame(
                    &mut app,
                    &ctx,
                    vec![button(at, PointerButton::Middle, true)],
                );
            }
            _ => {
                app.execute(CommandId::NewProject, &ctx);
                frame(&mut app, &ctx, vec![]);
            }
        }
        assert!(
            app.session.graphics.connection.is_none(),
            "cancel case {cancel}"
        );
        assert_eq!(app.session.snapshot().unwrap(), before);
        assert_eq!(app.session.history.undo_len(), 0);
    }
}

#[test]
fn unbound_graphic_port_is_rejected_by_real_mouse_input() {
    let (mut app, ctx) = setup(true, 2., 1.);
    let port = app
        .session
        .project()
        .unwrap()
        .port_bindings()
        .find(|(port, _)| {
            app.session.project().unwrap().port_position(port).unwrap() == p(80., 100.)
        })
        .unwrap()
        .0
        .clone();
    let mut project = app.session.project().unwrap().clone();
    project
        .edit_connection_drawing(ConnectionDrawingEdit::BindPort { port, point: None })
        .unwrap();
    app.session.replace(project, false).unwrap();
    let page = app.session.project().unwrap().pages().next().unwrap().id();
    app.session.open_page(page).unwrap();
    frame(&mut app, &ctx, vec![]);
    let before = app.session.snapshot().unwrap();
    app.execute(CommandId::DrawConnection, &ctx);
    click_world(&mut app, &ctx, p(80., 100.));
    assert!(
        app.session
            .graphics
            .connection
            .as_ref()
            .unwrap()
            .source
            .is_none()
    );
    assert_eq!(app.session.snapshot().unwrap(), before);
    assert_eq!(app.session.history.undo_len(), 0);
}

#[test]
fn real_trace_selection_and_context_expose_connection_commands() {
    let (mut app, ctx) = setup(false, 2., 1.);
    click_world(&mut app, &ctx, p(120., 120.));
    assert!(
        app.session.graphics.selection.primary().is_none(),
        "empty route AABB interior must not select the trace"
    );
    click_world(&mut app, &ctx, p(120., 140.));
    let id = app
        .session
        .graphics
        .selection
        .primary()
        .expect("route selected");
    assert!(
        app.session
            .project()
            .unwrap()
            .connection_route(id)
            .is_some()
    );
    let at = screen(&mut app, p(120., 140.));
    click(&mut app, &ctx, at, PointerButton::Secondary);
    let output = frame(&mut app, &ctx, vec![]);
    let text = texts(&output);
    for expected in [
        "Editar rota",
        "Adicionar vértice",
        "Excluir",
        "Destacar rede",
    ] {
        assert!(
            text.iter().any(|t| t.contains(expected)),
            "missing {expected}; text={text:?}"
        );
    }
}

#[test]
fn ctrl_z_and_y_inside_property_text_never_replay_connection_history() {
    for shortcut in [Key::Z, Key::Y] {
        let (mut app, ctx) = setup(true, 2., 1.);
        app.execute(CommandId::DrawConnection, &ctx);
        click_world(&mut app, &ctx, p(80., 100.));
        click_world(&mut app, &ctx, p(240., 180.));
        frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
        if shortcut == Key::Y {
            app.execute(CommandId::Undo, &ctx);
        }
        let before = app.session.snapshot().unwrap();
        let undo = app.session.history.undo_len();
        let redo = app.session.history.can_redo();
        app.execute(CommandId::ShowProperties, &ctx);
        let output = frame(&mut app, &ctx, vec![]);
        let at = text_center(&output, "Procurar propriedade");
        click(&mut app, &ctx, at, PointerButton::Primary);
        frame(&mut app, &ctx, vec![Event::Text("conexão".into())]);
        assert!(ctx.wants_keyboard_input());
        frame(&mut app, &ctx, vec![key(shortcut, Modifiers::CTRL)]);
        assert_eq!(
            app.session.snapshot().unwrap(),
            before,
            "shortcut {shortcut:?}"
        );
        assert_eq!(app.session.history.undo_len(), undo);
        assert_eq!(app.session.history.can_redo(), redo);
    }
}

#[test]
fn real_segment_drag_has_transient_preview_and_one_transaction() {
    let (mut app, ctx) = setup(false, 2., 1.);
    let id = app
        .session
        .project()
        .unwrap()
        .connection_routes()
        .find(|r| r.points().contains(&p(80., 140.)))
        .unwrap()
        .id();
    let before = app.session.snapshot().unwrap();
    let original = app
        .session
        .project()
        .unwrap()
        .connection_route(id)
        .unwrap()
        .points()
        .to_vec();
    let a = screen(&mut app, p(120., 140.));
    let b = screen(&mut app, p(120., 152.));
    frame(
        &mut app,
        &ctx,
        vec![
            Event::PointerMoved(a),
            button(a, PointerButton::Primary, true),
        ],
    );
    frame(&mut app, &ctx, vec![Event::PointerMoved(b)]);
    assert_eq!(app.session.snapshot().unwrap(), before);
    assert_eq!(app.session.history.undo_len(), 0);
    assert!(app.session.graphics.route_drag.is_some());
    frame(
        &mut app,
        &ctx,
        vec![button(b, PointerButton::Primary, false)],
    );
    let edited = app
        .session
        .project()
        .unwrap()
        .connection_route(id)
        .unwrap()
        .points();
    assert_ne!(edited, original);
    assert_eq!(edited.first(), original.first());
    assert_eq!(edited.last(), original.last());
    assert_eq!(app.session.history.undo_len(), 1);
    app.execute(CommandId::Undo, &ctx);
    assert_eq!(app.session.snapshot().unwrap(), before);
}

#[test]
fn a_corner_clicked_near_a_wire_lands_on_the_grid_along_that_wire() {
    // The star-delta test left a corner at y = 75.947 mm: the raw projection
    // of the cursor onto a nearby wire. With snap on it must stay on the grid.
    let (mut app, ctx) = setup(false, 2., 1.);
    app.execute(CommandId::DrawConnection, &ctx);
    click_world(&mut app, &ctx, p(80., 100.));
    click_world(&mut app, &ctx, p(160.3, 121.3));
    click_world(&mut app, &ctx, p(240., 200.));
    let project = app.session.project().unwrap();
    let route = project
        .connection_routes()
        .find(|r| r.points().contains(&p(160., 120.)))
        .unwrap_or_else(|| {
            panic!(
                "{:?}",
                project
                    .connection_routes()
                    .map(|r| r.points().to_vec())
                    .collect::<Vec<_>>()
            )
        });
    assert!(
        route
            .points()
            .iter()
            .all(|q| (q.x() / 4.).fract() == 0. && (q.y() / 4.).fract() == 0.),
        "{:?}",
        route.points()
    );
}
