// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Insertion with the session: a user symbol typed as a KT coil creates KT1
//! with a coil function, and contacts on an existing device take the next
//! free IEC point numbers, or the labels typed in the insertion centre for
//! the first click only.
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

use cad_craft::domain::*;
use cad_gui::{app::CadApp, interaction::DevicePlan};

fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}
fn app() -> (CadApp, PageId) {
    let mut project = Project::new("Pontos").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "Esquema").unwrap())
        .unwrap();
    let page = project
        .insert_page(
            Page::new(
                project.id(),
                doc,
                "Comando",
                "1",
                PageKind::MultiLine,
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
/// Insert `id` at `x` with `plan` through the insertion tool; return the
/// labels of the points of the new function.
fn insert(app: &mut CadApp, id: &str, plan: DevicePlan, xs: &[f64]) -> Vec<Vec<String>> {
    let id = SymbolDefinitionId::new(id).unwrap();
    let t = Transform2D::new(p(0., 0.), 0., false, false).unwrap();
    app.session.begin_device_insertion(id, plan, t).unwrap();
    let mut created = vec![];
    for &x in xs {
        let draft = app.session.graphics.insertion.as_mut().unwrap();
        draft.transform = Transform2D::new(p(x, 100.), 0., false, false).unwrap();
        draft.positioned = true;
        let rep = app.session.confirm_symbol_insertion().unwrap();
        let project = app.session.project().unwrap();
        let RepresentationTarget::Function(f) = project.representation(rep).unwrap().target()
        else {
            panic!("função esperada");
        };
        let mut labels: Vec<String> = project
            .connection_points()
            .filter(|pt| pt.function_id() == f)
            .map(|pt| pt.designation().as_str().to_owned())
            .collect();
        labels.sort();
        created.push(labels);
    }
    app.session.graphics.cancel();
    created
}

#[test]
fn a_user_symbol_typed_as_kt_coil_creates_kt1_with_a_coil() {
    let (mut app, _) = app();
    let coil = SymbolLibrary::builtin()
        .get(&SymbolDefinitionId::new("cad-craft.supply").unwrap())
        .unwrap()
        .with_identity(
            SymbolDefinitionId::new("usuario.bobina-temporizada").unwrap(),
            "Bobina temporizada",
        )
        .unwrap();
    let metadata = coil.metadata_with_hints(SymbolFunctionKind::Coil, "KT");
    app.session
        .symbol_library
        .push(coil.with_metadata(metadata));
    insert(
        &mut app,
        "usuario.bobina-temporizada",
        DevicePlan::New {
            prefix: String::new(),
        },
        &[60.],
    );
    let project = app.session.project().unwrap();
    let device = project.devices().next().unwrap();
    assert_eq!(device.designation().to_string(), "-KT1");
    let function = project.functions().next().unwrap();
    assert_eq!(function.kind(), &FunctionKind::Coil);
}

#[test]
fn contacts_on_an_existing_device_follow_the_suggestion_or_the_typed_labels() {
    let (mut app, _) = app();
    insert(
        &mut app,
        "cad-craft.coil",
        DevicePlan::New {
            prefix: String::new(),
        },
        &[40.],
    );
    let k1 = app
        .session
        .project()
        .unwrap()
        .devices()
        .next()
        .unwrap()
        .id();
    // Two clicks with the suggestion: 13-14, then 23-24.
    let auto = insert(
        &mut app,
        "cad-craft.contact-no",
        DevicePlan::Existing(k1),
        &[80., 120.],
    );
    assert_eq!(auto, [["13", "14"], ["23", "24"]]);
    // Typed labels apply to the first click; the next one is suggested again.
    let typed = insert(
        &mut app,
        "cad-craft.contact-no",
        DevicePlan::ExistingWithPoints(k1, vec!["43".into(), "44".into()]),
        &[160., 200.],
    );
    assert_eq!(typed, [["43", "44"], ["33", "34"]]);
    let report = app.session.project().unwrap().check(CheckScope::Project);
    assert!(!report.messages.iter().any(|m| m.rule == "VER-021"));
}

#[test]
fn updating_project_symbols_brings_the_factory_v2_drawing() {
    let (mut app, page) = app();
    let new = SymbolLibrary::builtin()
        .get(&SymbolDefinitionId::new("cad-craft.contact-no").unwrap())
        .unwrap()
        .clone();
    // Version 1 as stored by projects made before the factory symbols v2.
    let old = SymbolDefinition::new(
        new.id().clone(),
        1,
        new.name(),
        new.category(),
        vec![SymbolPrimitive::Line {
            start: p(0., -10.),
            end: p(0., 10.),
        }],
        vec![
            SymbolPort::new("1", p(0., -10.), 270., "1", None).unwrap(),
            SymbolPort::new("2", p(0., 10.), 90., "2", None).unwrap(),
        ],
    )
    .unwrap();
    let mut project = app.session.project().unwrap().clone();
    project
        .place_symbol(SymbolPlacement {
            page,
            definition: old,
            transform: Transform2D::new(p(60., 100.), 0., false, false).unwrap(),
            device: PlacementDevice::New {
                designation: project.next_device_designation("K").unwrap(),
                classification: "Relé".into(),
            },
            function_kind: FunctionKind::AuxiliaryContact,
        })
        .unwrap();
    app.session.replace(project, false).unwrap();
    let version = |app: &CadApp| {
        app.session
            .project()
            .unwrap()
            .symbol_definition(new.id())
            .unwrap()
            .version()
    };
    assert_eq!(version(&app), 1, "o projeto antigo mantém a versão 1");
    let ctx = eframe::egui::Context::default();
    app.execute(cad_gui::commands::CommandId::SymbolUpdateProject, &ctx);
    assert_eq!(version(&app), 2);
}
