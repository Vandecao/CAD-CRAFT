// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Fixture helpers use validated synthetic inputs and fail immediately on setup errors.
#![allow(clippy::unwrap_used)]
use cad_craft::domain::*;
use cad_gui::{app::CadApp, commands::CommandId};
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
fn at(out: &egui::FullOutput, label: &str) -> Pos2 {
    let node = out
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, n)| n.label() == Some(label))
        .unwrap_or_else(|| panic!("widget {label} ausente"));
    let b = node.1.bounds().unwrap();
    Pos2::new(((b.x0 + b.x1) / 2.) as f32, ((b.y0 + b.y1) / 2.) as f32)
}
fn click(app: &mut CadApp, ctx: &Context, label: &str) {
    frame(app, ctx, vec![]);
    let out = frame(app, ctx, vec![]);
    let pos = at(&out, label);
    let event = |pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    frame(app, ctx, vec![Event::PointerMoved(pos), event(true)]);
    frame(app, ctx, vec![event(false)]);
    frame(app, ctx, vec![]);
}

#[test]
fn device_numbering_dialog_previews_and_undoes_from_real_click() {
    let mut project = Project::new("Número").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "D").unwrap())
        .unwrap();
    let page = project
        .insert_page(
            Page::new(
                project.id(),
                doc,
                "P",
                "1",
                PageKind::MultiLine,
                Size2D::new(297.0, 210.0).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let device = project
        .insert_device(
            Device::new(project.id(), Designation::new("K5").unwrap(), "Contator").unwrap(),
        )
        .unwrap();
    project
        .insert_representation(Representation::new(
            project.id(),
            page,
            RepresentationTarget::Device(device),
            Transform2D::new(Point2D::new(10.0, 20.0).unwrap(), 0.0, false, false).unwrap(),
        ))
        .unwrap();
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut app = CadApp::default();
    app.session.replace(project, false).unwrap();
    frame(&mut app, &ctx, vec![]);
    app.execute(CommandId::NumberDevices, &ctx);
    let out = frame(&mut app, &ctx, vec![]);
    assert!(
        out.platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, n)| n
                .label()
                .or_else(|| n.value())
                .is_some_and(|s| s.contains("-K5") && s.contains("-K1")))
    );
    click(&mut app, &ctx, "Aplicar numeração");
    assert_eq!(
        app.session
            .project()
            .unwrap()
            .device(device)
            .unwrap()
            .designation()
            .to_string(),
        "-K1"
    );
    app.execute(CommandId::Undo, &ctx);
    assert_eq!(
        app.session
            .project()
            .unwrap()
            .device(device)
            .unwrap()
            .designation()
            .to_string(),
        "-K5"
    );
}

#[test]
fn connection_and_page_dialogs_apply_from_real_clicks() {
    let mut project = Project::new("Números").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "D").unwrap())
        .unwrap();
    let first = project
        .insert_page(
            Page::new(
                project.id(),
                doc,
                "P1",
                "8",
                PageKind::MultiLine,
                Size2D::new(297.0, 210.0).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let second = project
        .insert_page(
            Page::new(
                project.id(),
                doc,
                "P2",
                "9",
                PageKind::MultiLine,
                Size2D::new(297.0, 210.0).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let device = project
        .insert_device(Device::new(project.id(), Designation::new("K1").unwrap(), "K").unwrap())
        .unwrap();
    let function = project
        .insert_function(Function::new(
            project.id(),
            device,
            Designation::new("K1").unwrap(),
            FunctionKind::Coil,
        ))
        .unwrap();
    let a = project
        .insert_connection_point(ConnectionPoint::new(
            project.id(),
            function,
            DesignationPart::new("A1").unwrap(),
        ))
        .unwrap();
    let b = project
        .insert_connection_point(ConnectionPoint::new(
            project.id(),
            function,
            DesignationPart::new("A2").unwrap(),
        ))
        .unwrap();
    let wire = project
        .insert_connection(Connection::new(project.id(), a, b).unwrap())
        .unwrap();
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut app = CadApp::default();
    app.session.replace(project, false).unwrap();
    assert_eq!(
        app.session
            .project()
            .unwrap()
            .preview_connection_numbering()
            .unwrap()
            .rows
            .len(),
        1
    );
    frame(&mut app, &ctx, vec![]);
    app.execute(CommandId::NumberConnections, &ctx);
    click(&mut app, &ctx, "Aplicar numeração");
    assert_eq!(
        app.session
            .project()
            .unwrap()
            .connection(wire)
            .unwrap()
            .number()
            .map(Text::as_str),
        Some("1"),
        "undo={}, preview={:?}",
        app.session.history.undo_len(),
        app.session
            .project()
            .unwrap()
            .preview_connection_numbering()
            .unwrap()
    );
    app.execute(CommandId::Undo, &ctx);
    assert!(
        app.session
            .project()
            .unwrap()
            .connection(wire)
            .unwrap()
            .number()
            .is_none()
    );
    app.session.select(cad_gui::state::Selection::Document(doc));
    app.execute(CommandId::NumberPages, &ctx);
    click(&mut app, &ctx, "Aplicar numeração");
    assert_eq!(
        app.session
            .project()
            .unwrap()
            .page(first)
            .unwrap()
            .number()
            .as_str(),
        "1"
    );
    assert_eq!(
        app.session
            .project()
            .unwrap()
            .page(second)
            .unwrap()
            .number()
            .as_str(),
        "2"
    );
    app.execute(CommandId::Undo, &ctx);
    assert_eq!(
        app.session
            .project()
            .unwrap()
            .page(first)
            .unwrap()
            .number()
            .as_str(),
        "8"
    );
}

fn labels(out: &egui::FullOutput) -> Vec<String> {
    out.platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, n)| n.label().or_else(|| n.value()).map(str::to_owned))
        .collect()
}

fn three_pages() -> (Project, [PageId; 3]) {
    let mut project = Project::new("Numeração").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "D").unwrap())
        .unwrap();
    let pages = ["1", "2", "3"].map(|number| {
        project
            .insert_page(
                Page::new(
                    project.id(),
                    doc,
                    format!("P{number}"),
                    number,
                    PageKind::MultiLine,
                    Size2D::new(297.0, 210.0).unwrap(),
                )
                .unwrap(),
            )
            .unwrap()
    });
    (project, pages)
}

fn placed(project: &mut Project, name: &str, page: PageId) -> DeviceId {
    let device = project
        .insert_device(
            Device::new(project.id(), Designation::new(name).unwrap(), "Contator").unwrap(),
        )
        .unwrap();
    project
        .insert_representation(Representation::new(
            project.id(),
            page,
            RepresentationTarget::Device(device),
            Transform2D::new(Point2D::new(50.0, 100.0).unwrap(), 0.0, false, false).unwrap(),
        ))
        .unwrap();
    device
}

fn designation(app: &CadApp, id: DeviceId) -> String {
    let p = app.session.project().unwrap();
    p.device(id).unwrap().designation().to_string()
}

/// Renumber -K9/-K5/-K2 on three pages to -K1/-K2/-K3 in page order; auxiliary functions and parts follow, with exact undo/redo.
#[test]
fn three_pages_are_numbered_in_page_order_and_undo_redo_are_exact() {
    let (mut project, [p1, p2, p3]) = three_pages();
    let k9 = placed(&mut project, "K9", p1);
    let k5 = placed(&mut project, "K5", p2);
    let k2 = placed(&mut project, "K2", p3);
    let contact = project
        .insert_function(
            Function::new(
                project.id(),
                k9,
                Designation::new("K9").unwrap(),
                FunctionKind::AuxiliaryContact,
            )
            .with_role(FunctionRole::Auxiliary),
        )
        .unwrap();
    project
        .assign_part(
            k9,
            Part::new("ACME.K1", "ACME", "K1", "Contator", "Contator").unwrap(),
            1,
        )
        .unwrap();
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut app = CadApp::default();
    app.session.replace(project, false).unwrap();
    let original = app.session.project().unwrap().clone();
    frame(&mut app, &ctx, vec![]);
    app.execute(CommandId::NumberDevices, &ctx);
    let out = frame(&mut app, &ctx, vec![]);
    let shown = labels(&out);
    for row in ["-K9 → -K1", "-K5 → -K2", "-K2 → -K3"] {
        assert!(shown.iter().any(|l| l == row), "{row}: {shown:?}");
    }
    click(&mut app, &ctx, "Aplicar numeração");
    assert_eq!(
        [k9, k5, k2].map(|id| designation(&app, id)),
        ["-K1", "-K2", "-K3"]
    );
    let p = app.session.project().unwrap();
    assert_eq!(
        p.function(contact).unwrap().designation().to_string(),
        "-K1"
    );
    assert!(p.bom_rows().iter().any(|r| r.device_designation == "-K1"));
    let numbered = p.clone();
    app.execute(CommandId::Undo, &ctx);
    assert_eq!(app.session.project().unwrap(), &original);
    app.execute(CommandId::Redo, &ctx);
    assert_eq!(app.session.project().unwrap(), &numbered);
}

/// Conflict with a locked designation appears and disables Apply without changes.
#[test]
fn locked_conflict_is_shown_and_apply_does_nothing() {
    let (mut project, [p1, p2, _]) = three_pages();
    let other = placed(&mut project, "K2", p1);
    let locked = placed(&mut project, "K1", p2);
    project.set_designation_locked(locked, true).unwrap();
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut app = CadApp::default();
    app.session.replace(project, false).unwrap();
    let before = app.session.project().unwrap().clone();
    frame(&mut app, &ctx, vec![]);
    app.execute(CommandId::NumberDevices, &ctx);
    let out = frame(&mut app, &ctx, vec![]);
    let shown = labels(&out);
    assert!(
        shown
            .iter()
            .any(|l| l.starts_with("-K2 → -K1") && l.contains("conflito")),
        "{shown:?}"
    );
    assert!(shown.iter().any(|l| l.starts_with("Há conflitos")));
    click(&mut app, &ctx, "Aplicar numeração");
    assert_eq!(app.session.project().unwrap(), &before);
    assert_eq!(designation(&app, other), "-K2");
    assert_eq!(app.session.history.undo_len(), 0);
}

/// The mirror lists represented contacts with type, points and page-column references, and unrepresented contacts without references; clicking navigates.
#[test]
fn contact_mirror_lists_contacts_and_navigates_on_click() {
    let (mut project, [p1, p2, _]) = three_pages();
    let k1 = project
        .insert_device(
            Device::new(project.id(), Designation::new("K1").unwrap(), "Contator").unwrap(),
        )
        .unwrap();
    let function = |project: &mut Project, kind, role, points: [&str; 2]| {
        let id = project
            .insert_function(
                Function::new(project.id(), k1, Designation::new("K1").unwrap(), kind)
                    .with_role(role),
            )
            .unwrap();
        for label in points {
            project
                .insert_connection_point(ConnectionPoint::new(
                    project.id(),
                    id,
                    DesignationPart::new(label).unwrap(),
                ))
                .unwrap();
        }
        id
    };
    let coil = function(
        &mut project,
        FunctionKind::Coil,
        FunctionRole::Main,
        ["A1", "A2"],
    );
    let no = function(
        &mut project,
        FunctionKind::AuxiliaryContact,
        FunctionRole::Auxiliary,
        ["13", "14"],
    );
    function(
        &mut project,
        FunctionKind::AuxiliaryContact,
        FunctionRole::Auxiliary,
        ["21", "22"],
    );
    let at = |x, y| Transform2D::new(Point2D::new(x, y).unwrap(), 0.0, false, false).unwrap();
    project
        .insert_representation(Representation::new(
            project.id(),
            p1,
            RepresentationTarget::Function(coil),
            at(50.0, 100.0),
        ))
        .unwrap();
    let contact = project
        .insert_representation(Representation::new(
            project.id(),
            p2,
            RepresentationTarget::Function(no),
            at(80.0, 120.0),
        ))
        .unwrap();
    let reference = project.function_references(no).unwrap()[0]
        .reference
        .clone();
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut app = CadApp::default();
    app.session.replace(project, false).unwrap();
    app.session.select(cad_gui::state::Selection::Electrical(
        RepresentationTarget::Device(k1),
    ));
    tall(&mut app, &ctx, vec![]);
    let out = tall(&mut app, &ctx, vec![]);
    let shown = labels(&out);
    for expected in [
        "Espelho de contatos",
        "NA 13-14",
        "NF 21-22",
        "Não representado",
    ] {
        assert!(shown.iter().any(|l| l == expected), "{expected}: {shown:?}");
    }
    // The same text appears in the occurrences and contacts section; the mirror follows.
    let link = out
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, n)| n.label() == Some(reference.as_str()))
        .filter_map(|(_, n)| n.bounds())
        .max_by(|a, b| a.y0.total_cmp(&b.y0))
        .expect("link do espelho");
    let pos = Pos2::new(
        ((link.x0 + link.x1) / 2.) as f32,
        ((link.y0 + link.y1) / 2.) as f32,
    );
    let event = |pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    tall(&mut app, &ctx, vec![Event::PointerMoved(pos), event(true)]);
    tall(&mut app, &ctx, vec![event(false)]);
    tall(&mut app, &ctx, vec![]);
    eprintln!(
        "DBG2 sel={:?} active={:?} tabs={}",
        app.session.selection,
        app.session.active,
        app.session.tabs.len()
    );
    assert_eq!(app.session.active, Some(p2));
    assert_eq!(
        app.session.selection,
        cad_gui::state::Selection::Representation(contact)
    );
}

/// Manual wire-number editing is one exactly reversible transaction; the network stays unchanged.
#[test]
fn manual_wire_number_is_one_undoable_transaction() {
    let mut project = Project::new("Fio").unwrap();
    let device = project
        .insert_device(Device::new(project.id(), Designation::new("K1").unwrap(), "K").unwrap())
        .unwrap();
    let function = project
        .insert_function(Function::new(
            project.id(),
            device,
            Designation::new("K1").unwrap(),
            FunctionKind::Coil,
        ))
        .unwrap();
    let [a, b] = ["A1", "A2"].map(|label| {
        project
            .insert_connection_point(ConnectionPoint::new(
                project.id(),
                function,
                DesignationPart::new(label).unwrap(),
            ))
            .unwrap()
    });
    let wire = project
        .insert_connection(Connection::new(project.id(), a, b).unwrap())
        .unwrap();
    let ctx = Context::default();
    let mut app = CadApp::default();
    app.session.replace(project, false).unwrap();
    let networks = app.session.project().unwrap().networks();
    assert!(
        app.session
            .set_connection_number(wire, Some(Text::new("L1").unwrap()), true)
            .unwrap()
    );
    let number = |app: &CadApp| {
        let c = app
            .session
            .project()
            .unwrap()
            .connection(wire)
            .unwrap()
            .clone();
        (c.number().map(|n| n.as_str().to_owned()), c.number_locked())
    };
    assert_eq!(number(&app), (Some("L1".into()), true));
    assert_eq!(app.session.project().unwrap().networks(), networks);
    app.execute(CommandId::Undo, &ctx);
    assert_eq!(number(&app), (None, false));
    app.execute(CommandId::Redo, &ctx);
    assert_eq!(number(&app), (Some("L1".into()), true));
}

/// On a tall screen, the mirror at the end of the properties panel must be visible to receive a click.
fn tall(app: &mut CadApp, ctx: &Context, events: Vec<Event>) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1600., 1600.))),
            events,
            ..Default::default()
        },
        |ctx| app.show(ctx),
    )
}
