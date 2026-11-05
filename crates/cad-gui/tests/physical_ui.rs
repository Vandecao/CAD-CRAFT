// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_craft::domain::*;
use cad_gui::{
    app::CadApp,
    commands::{CommandId, registry},
    editing::GraphicEdit,
    preferences::RibbonTab,
    state::Session,
};
use eframe::egui::{self, Context, Event, Modifiers, PointerButton, Pos2, Rect, vec2};

fn frame(app: &mut CadApp, ctx: &Context, events: Vec<Event>) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1600., 900.))),
            events,
            ..Default::default()
        },
        |ctx| app.show(ctx),
    )
}
fn press(at: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}
fn click(app: &mut CadApp, ctx: &Context, at: Pos2) -> egui::FullOutput {
    frame(app, ctx, vec![Event::PointerMoved(at), press(at, true)]);
    frame(app, ctx, vec![press(at, false)])
}
/// Accessible nodes (label, center, role) published by egui for this frame.
fn nodes(output: &egui::FullOutput) -> Vec<(String, Pos2, egui::accesskit::Role)> {
    let Some(update) = &output.platform_output.accesskit_update else {
        return vec![];
    };
    update
        .nodes
        .iter()
        .filter_map(|(_, node)| {
            let b = node.bounds()?;
            Some((
                node.label().unwrap_or_default().to_owned(),
                Pos2::new(((b.x0 + b.x1) / 2.) as f32, ((b.y0 + b.y1) / 2.) as f32),
                node.role(),
            ))
        })
        .collect()
}
fn find(output: &egui::FullOutput, label: &str) -> Pos2 {
    nodes(output)
        .into_iter()
        .find(|(l, _, _)| l == label)
        .map(|(_, p, _)| p)
        .unwrap_or_else(|| panic!("widget {label:?} not rendered"))
}

#[test]
fn ribbon_registers_physical_engineering_commands_on_existing_tabs() {
    let commands = registry();
    let tab_of = |id: CommandId| commands.iter().find(|c| c.id == id).map(|c| c.tab);
    assert_eq!(
        tab_of(CommandId::TerminalStripEditor),
        Some(RibbonTab::Devices)
    );
    assert_eq!(
        tab_of(CommandId::WiringEditor),
        Some(RibbonTab::Connections)
    );
    assert_eq!(tab_of(CommandId::CableEditor), Some(RibbonTab::Cables));
    assert_eq!(tab_of(CommandId::TechnicalLists), Some(RibbonTab::Cables));
    assert!(RibbonTab::ALL.contains(&RibbonTab::Cables));
}

#[test]
fn real_egui_clicks_create_a_strip_through_history_and_undo_restores() {
    let mut app = CadApp::default();
    let ctx = Context::default();
    ctx.enable_accesskit();
    app.session
        .replace(cad_gui::demo::project().unwrap(), false)
        .unwrap();
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    let before = app.session.snapshot().unwrap();
    app.execute(CommandId::TerminalStripEditor, &ctx);
    frame(&mut app, &ctx, vec![]);
    let output = frame(&mut app, &ctx, vec![]);
    let button = find(&output, "Criar régua");
    // The text field of the same row, left of the button.
    let field = nodes(&output)
        .into_iter()
        .filter(|(_, p, role)| {
            *role == egui::accesskit::Role::TextInput
                && (p.y - button.y).abs() < 6.
                && p.x < button.x
        })
        .max_by(|a, b| a.1.x.total_cmp(&b.1.x))
        .expect("strip name field")
        .1;
    click(&mut app, &ctx, field);
    frame(&mut app, &ctx, vec![Event::Text("X7".into())]);
    let output = click(&mut app, &ctx, button);
    let project = app.session.project().unwrap();
    let strip = project
        .terminal_strips()
        .find(|s| s.designation().as_str() == "X7")
        .expect("strip created by the real click");
    let id = strip.id();
    assert!(
        nodes(&output)
            .iter()
            .all(|(l, _, _)| !l.contains("em implementação"))
    );
    app.session.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert!(app.session.project().unwrap().terminal_strip(id).is_none());
    assert_eq!(app.session.snapshot().unwrap(), before);
    app.session.dispatch_graphic(GraphicEdit::Redo).unwrap();
    assert!(app.session.project().unwrap().terminal_strip(id).is_some());

    // Conductors and lists tabs render real controls, not placeholders.
    for tab in [CommandId::WiringEditor, CommandId::TechnicalLists] {
        app.execute(tab, &ctx);
        frame(&mut app, &ctx, vec![]);
        let output = frame(&mut app, &ctx, vec![]);
        let labels: Vec<_> = nodes(&output).into_iter().map(|n| n.0).collect();
        let expected = if tab == CommandId::WiringEditor {
            "Criar condutor"
        } else {
            "Localizar seleção"
        };
        assert!(labels.iter().any(|l| l == expected), "{expected} missing");
    }
}

fn wired_session() -> (Session, ConnectionId, ConductorId) {
    let mut p = Project::new("Fiação").unwrap();
    let device = p
        .insert_device(Device::new(p.id(), Designation::new("K1").unwrap(), "relé").unwrap())
        .unwrap();
    let function = p
        .insert_function(Function::new(
            p.id(),
            device,
            Designation::new("K1").unwrap(),
            FunctionKind::Coil,
        ))
        .unwrap();
    let mut points = vec![];
    let mut ends = vec![];
    for name in ["A1", "A2"] {
        let t = p
            .insert_terminal(
                Terminal::new(
                    p.id(),
                    TerminalOwner::Device(device),
                    DesignationPart::new(name).unwrap(),
                )
                .with_positions(vec![DesignationPart::new("a").unwrap()])
                .unwrap(),
            )
            .unwrap();
        ends.push(PhysicalTermination::new(
            t,
            DesignationPart::new("a").unwrap(),
        ));
        points.push(
            p.insert_connection_point(
                ConnectionPoint::new(p.id(), function, DesignationPart::new(name).unwrap())
                    .with_terminal(t),
            )
            .unwrap(),
        );
    }
    let edge = p
        .insert_connection(Connection::new(p.id(), points[0], points[1]).unwrap())
        .unwrap();
    let wire = Conductor::new(
        p.id(),
        DesignationPart::new("W1").unwrap(),
        ends[0].clone(),
        ends[1].clone(),
        CrossSectionMm2::new(1.5).unwrap(),
        ConductorMarking::color(ConductorColor::Black),
    )
    .unwrap();
    let id = wire.id();
    let mut session = Session::default();
    session.replace(p, true).unwrap();
    session
        .dispatch_physical(PhysicalEdit::Batch(vec![
            PhysicalEdit::UpsertConductor(wire),
            PhysicalEdit::SetConductorConnection {
                conductor: id,
                link: Some(ConductorConnection::new(edge, points[0])),
            },
        ]))
        .unwrap();
    (session, edge, id)
}

#[test]
fn conductor_association_is_one_history_step_and_keeps_networks() {
    let (mut session, edge, wire) = wired_session();
    let project = session.project().unwrap();
    let networks = project.networks();
    assert_eq!(
        project.physical_for_connection(edge).unwrap().conductors(),
        &[wire]
    );
    let assigned = session.snapshot().unwrap();
    session.dispatch_graphic(GraphicEdit::Undo).unwrap();
    let project = session.project().unwrap();
    assert!(
        project.conductor(wire).is_none(),
        "create + link undone together"
    );
    assert_eq!(project.networks(), networks);
    session.dispatch_graphic(GraphicEdit::Redo).unwrap();
    assert_eq!(session.snapshot().unwrap(), assigned);

    // Unlinking keeps the conductor and the logical connection intact.
    session
        .dispatch_physical(PhysicalEdit::SetConductorConnection {
            conductor: wire,
            link: None,
        })
        .unwrap();
    let project = session.project().unwrap();
    assert!(project.conductor(wire).is_some());
    assert!(project.connection(edge).is_some());
    assert!(
        project
            .physical_for_connection(edge)
            .unwrap()
            .conductors()
            .is_empty()
    );
    // Invalid edits are rejected without touching history or state.
    let snapshot = session.snapshot().unwrap();
    assert!(
        session
            .dispatch_physical(PhysicalEdit::DeleteCable(CableId::new()))
            .is_err()
    );
    assert_eq!(session.snapshot().unwrap(), snapshot);
}

fn physical_window(ctx: &Context) -> Rect {
    ctx.memory(|m| m.area_rect(egui::Id::new("Engenharia física")))
        .expect("physical engineering window rendered")
}
fn assert_same_size(a: Rect, b: Rect, what: &str) {
    assert!(
        (a.height() - b.height()).abs() < 0.5 && (a.width() - b.width()).abs() < 0.5,
        "{what}: {a:?} became {b:?}"
    );
}
/// GUI-PHY-02: everything the window paints must be visible only inside its
/// frame, and the frame background must span the whole window. The window's
/// shapes start at its background rect (the window is the topmost layer
/// here); each later shape's visible part is its bounds clipped by its clip
/// rect, so scrolled-away rows are correctly ignored.
/// The window frame is painted as a filled rect (grouped with its shadow).
fn is_background(shape: &egui::Shape, window: Rect) -> bool {
    match shape {
        egui::Shape::Rect(r) => {
            r.fill.a() > 0
                && (r.rect.min - window.min).length() < 1.5
                && (r.rect.max - window.max).length() < 1.5
        }
        egui::Shape::Vec(shapes) => shapes.iter().any(|s| is_background(s, window)),
        _ => false,
    }
}
fn assert_contained(output: &egui::FullOutput, ctx: &Context, what: &str) {
    let window = physical_window(ctx);
    let start = output
        .shapes
        .iter()
        .position(|c| is_background(&c.shape, window))
        .unwrap_or_else(|| panic!("{what}: window background spanning {window:?} not painted"));
    let bounds = window.expand(1.0);
    let mut painted = 0;
    // Skip the background itself: its shadow legitimately extends outside.
    for clipped in &output.shapes[start + 1..] {
        let visible = clipped
            .shape
            .visual_bounding_rect()
            .intersect(clipped.clip_rect);
        if !visible.is_positive() {
            continue;
        }
        painted += 1;
        assert!(
            bounds.contains_rect(visible),
            "{what}: content painted at {visible:?} outside window {window:?}"
        );
    }
    assert!(painted > 20, "{what}: window body not painted ({painted})");
}
fn settle(app: &mut CadApp, ctx: &Context, frames: usize) -> egui::FullOutput {
    let mut output = frame(app, ctx, vec![]);
    for _ in 1..frames {
        output = frame(app, ctx, vec![]);
    }
    output
}
fn editor() -> (CadApp, Context) {
    let mut app = CadApp::default();
    let ctx = Context::default();
    ctx.enable_accesskit();
    app.session
        .replace(cad_gui::demo::project().unwrap(), false)
        .unwrap();
    frame(&mut app, &ctx, vec![]);
    (app, ctx)
}
/// Opens the editor with the feedback line visible (an invalid empty strip
/// name), the condition that made the window grow 28 px per frame.
fn editor_with_feedback() -> (CadApp, Context) {
    let (mut app, ctx) = editor();
    app.execute(CommandId::TerminalStripEditor, &ctx);
    let output = settle(&mut app, &ctx, 2);
    let button = find(&output, "Criar régua");
    click(&mut app, &ctx, button);
    (app, ctx)
}
const TABS: [CommandId; 5] = [
    CommandId::TechnicalLists,
    CommandId::TerminalStripEditor,
    CommandId::ConnectorEditor,
    CommandId::CableEditor,
    CommandId::WiringEditor,
];

/// GUI-PHY-01/02: with or without the feedback line, no tab grows the window
/// by itself, the window keeps its default size, stays on screen and paints
/// its controls only inside its background.
#[test]
fn physical_window_size_is_stable_and_bounded_on_every_tab() {
    for with_feedback in [false, true] {
        let (mut app, ctx) = if with_feedback {
            editor_with_feedback()
        } else {
            editor()
        };
        for command in TABS {
            let what = format!("{command:?} feedback={with_feedback}");
            app.execute(command, &ctx);
            settle(&mut app, &ctx, 3);
            let settled = physical_window(&ctx);
            assert!(
                settled.height() > 500. && settled.width() > 800.,
                "{what}: window collapsed to {settled:?}"
            );
            let output = settle(&mut app, &ctx, 40);
            let later = physical_window(&ctx);
            assert_same_size(settled, later, &format!("{what} over 40 frames"));
            assert!(
                later.bottom() <= 900.5 && later.top() >= -0.5,
                "{what}: {later:?}"
            );
            assert_contained(&output, &ctx, &what);
        }
    }
}

/// GUI-PHY-01/02: a size chosen by dragging the corner is kept after release,
/// over many frames and across tab switches, smaller or larger than before,
/// and the content stays inside the window at every size.
#[test]
fn manual_resize_of_physical_window_is_preserved_and_bounded() {
    let (mut app, ctx) = editor_with_feedback();
    app.execute(CommandId::TechnicalLists, &ctx);
    settle(&mut app, &ctx, 3);
    for delta in [vec2(-260., -230.), vec2(-150., -120.), vec2(330., 290.)] {
        let before = physical_window(&ctx);
        let grab = before.right_bottom() - vec2(3., 3.);
        let target = grab + delta;
        frame(&mut app, &ctx, vec![Event::PointerMoved(grab)]);
        frame(&mut app, &ctx, vec![press(grab, true)]);
        for step in 1..=4 {
            let at = grab + delta * (step as f32 / 4.);
            frame(&mut app, &ctx, vec![Event::PointerMoved(at)]);
        }
        frame(&mut app, &ctx, vec![press(target, false)]);
        frame(
            &mut app,
            &ctx,
            vec![Event::PointerMoved(Pos2::new(5., 450.))],
        );
        let resized = physical_window(&ctx);
        assert!(
            (resized.height() - (before.height() + delta.y)).abs() < 8.
                && (resized.width() - (before.width() + delta.x)).abs() < 8.,
            "drag {delta:?}: {before:?} -> {resized:?}"
        );
        let output = settle(&mut app, &ctx, 40);
        assert_same_size(resized, physical_window(&ctx), "after release");
        assert_contained(&output, &ctx, &format!("lists after drag {delta:?}"));
        for command in TABS {
            app.execute(command, &ctx);
            let output = settle(&mut app, &ctx, 5);
            let what = format!("{command:?} after drag {delta:?}");
            assert_same_size(resized, physical_window(&ctx), &what);
            assert_contained(&output, &ctx, &what);
        }
        app.execute(CommandId::TechnicalLists, &ctx);
        settle(&mut app, &ctx, 2);
    }
}
