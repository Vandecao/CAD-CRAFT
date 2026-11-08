// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Typed coordinates for symbols and wire corners, terminal symbols on a
//! strip, and the single-line tool, through the application session.
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

use cad_craft::domain::*;
use cad_gui::{app::CadApp, editing::GraphicEdit, interaction::DevicePlan};

fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}
fn app(kind: PageKind) -> (CadApp, PageId) {
    let mut project = Project::new("Etapa 35").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "Esquema").unwrap())
        .unwrap();
    let page = project
        .insert_page(
            Page::new(
                project.id(),
                doc,
                "Página",
                "1",
                kind,
                Size2D::new(420.0, 297.0).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let mut app = CadApp::default();
    app.session.replace(project, false).unwrap();
    app.session.open_page(page).unwrap();
    (app, page)
}
fn begin(app: &mut CadApp, key: &str, plan: DevicePlan) {
    let t = Transform2D::new(p(0., 0.), 0., false, false).unwrap();
    app.session
        .begin_device_insertion(SymbolDefinitionId::new(key).unwrap(), plan, t)
        .unwrap();
}
fn typed(app: &mut CadApp, x: f64, y: f64, relative: bool) -> Point2D {
    app.coordinates.x = x;
    app.coordinates.y = y;
    app.coordinates.relative = relative;
    app.apply_typed_point().unwrap()
}
fn new_device() -> DevicePlan {
    DevicePlan::New {
        prefix: String::new(),
    }
}

#[test]
fn typed_coordinates_place_symbols_and_wire_corners_exactly() {
    let (mut app, _) = app(PageKind::MultiLine);
    begin(&mut app, "cad-craft.coil", new_device());
    typed(&mut app, 123.0, 77.0, false);
    // Relative to the last typed point: 40 mm to the right.
    typed(&mut app, 40.0, 0.0, true);
    app.session.graphics.cancel();
    let project = app.session.project().unwrap();
    let mut origins: Vec<_> = project
        .representations()
        .map(|r| r.transform().translation())
        .collect();
    origins.sort_by(|a, b| a.x().total_cmp(&b.x()));
    assert_eq!(origins, [p(123.0, 77.0), p(163.0, 77.0)]);
    // Wire from the top port of the first coil (123, 87) to the top port
    // of the second (163, 87) through typed corners.
    app.session.begin_connection().unwrap();
    typed(&mut app, 123.0, 87.0, false);
    typed(&mut app, 0.0, 20.0, true);
    typed(&mut app, 40.0, 0.0, true);
    typed(&mut app, 163.0, 87.0, false);
    let project = app.session.project().unwrap();
    let route = project.connection_routes().next().expect("rota criada");
    assert_eq!(
        route.points(),
        [
            p(123.0, 87.0),
            p(123.0, 107.0),
            p(163.0, 107.0),
            p(163.0, 87.0)
        ]
    );
    app.graphic_edit(GraphicEdit::Undo);
    assert_eq!(
        app.session.project().unwrap().connection_routes().count(),
        0
    );
}

#[test]
fn terminal_symbols_on_a_strip_number_x1_1_to_3() {
    let (mut app, _) = app(PageKind::MultiLine);
    begin(
        &mut app,
        "cad-craft.terminal",
        DevicePlan::StripTerminal("X1".into()),
    );
    for x in [50.0, 70.0, 90.0] {
        typed(&mut app, x, 100.0, false);
    }
    let project = app.session.project().unwrap();
    let strip = project.terminal_strips().next().unwrap();
    let names: Vec<_> = strip
        .terminals()
        .iter()
        .map(|t| {
            project
                .terminal(*t)
                .unwrap()
                .designation()
                .as_str()
                .to_owned()
        })
        .collect();
    assert_eq!(names, ["1", "2", "3"]);
    app.graphic_edit(GraphicEdit::Undo);
    let project = app.session.project().unwrap();
    assert_eq!(
        project.terminal_strips().next().unwrap().terminals().len(),
        2
    );
}

#[test]
fn the_single_line_tool_links_two_symbols_without_connections() {
    let (mut app, _) = app(PageKind::SingleLine);
    begin(&mut app, "cad-craft.contact-no", new_device());
    typed(&mut app, 60.0, 150.0, false);
    typed(&mut app, 160.0, 150.0, false);
    app.session.graphics.cancel();
    let project = app.session.project().unwrap();
    let mut reps: Vec<_> = project
        .representations()
        .map(|r| (r.transform().translation().x(), r.id()))
        .collect();
    reps.sort_by(|a, b| a.0.total_cmp(&b.0));
    let (q1, k1) = (reps[0].1, reps[1].1);
    app.session.begin_single_line().unwrap();
    app.session.graphics.single_line_phases = 3;
    assert!(
        !app.session
            .single_line_click(p(60.0, 140.0), Some(q1))
            .unwrap()
    );
    assert!(!app.session.single_line_click(p(60.0, 120.0), None).unwrap());
    assert!(
        !app.session
            .single_line_click(p(160.0, 120.0), None)
            .unwrap()
    );
    assert!(
        app.session
            .single_line_click(p(160.0, 140.0), Some(k1))
            .unwrap()
    );
    let project = app.session.project().unwrap();
    let link = project.single_lines().next().expect("ligação unifilar");
    assert_eq!((link.source(), link.target(), link.phases()), (q1, k1, 3));
    assert_eq!(link.points().len(), 4);
    assert_eq!(project.connections().count(), 0);
    app.graphic_edit(GraphicEdit::Undo);
    assert_eq!(app.session.project().unwrap().single_lines().count(), 0);
}
