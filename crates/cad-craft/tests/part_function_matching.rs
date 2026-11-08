// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Assigning a part reuses the functions already drawn for the device:
//! exact matches are kept, points with other names are renamed (keeping
//! bindings), and only functions the device lacks are created.
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

use cad_craft::domain::*;

fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}
fn symbol(key: &str) -> SymbolDefinition {
    SymbolLibrary::builtin()
        .get(&SymbolDefinitionId::new(format!("cad-craft.{key}")).unwrap())
        .unwrap()
        .clone()
}
/// One power pole, typed as a main contact.
fn pole() -> SymbolDefinition {
    let base = symbol("contact-no")
        .with_identity(SymbolDefinitionId::new("usuario.polo").unwrap(), "Polo")
        .unwrap();
    let metadata = base.metadata_with_hints(SymbolFunctionKind::MainContact, "K");
    base.with_metadata(metadata)
}
fn base() -> (Project, PageId) {
    let mut project = Project::new("Peças").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "Esquema").unwrap())
        .unwrap();
    let page = project
        .insert_page(
            Page::new(
                project.id(),
                doc,
                "Força",
                "1",
                PageKind::MultiLine,
                Size2D::new(420.0, 297.0).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    (project, page)
}
/// Draw `definition` for `device` (new K1 when `None`) with `labels`.
fn draw(
    project: &mut Project,
    page: PageId,
    device: Option<DeviceId>,
    definition: SymbolDefinition,
    kind: FunctionKind,
    labels: &[&str],
    x: f64,
) -> PlacementDelta {
    let device = match device {
        Some(id) => PlacementDevice::Existing(id),
        None => PlacementDevice::New {
            designation: project.next_device_designation("K").unwrap(),
            classification: "Contator".into(),
        },
    };
    let labels: Vec<String> = labels.iter().map(|l| l.to_string()).collect();
    project
        .place_symbol_with_point_labels(
            SymbolPlacement {
                page,
                definition,
                transform: Transform2D::new(p(x, 100.0), 0.0, false, false).unwrap(),
                device,
                function_kind: kind,
            },
            &labels,
        )
        .unwrap()
}
fn function(kind: FunctionKind, role: FunctionRole, points: [&str; 2]) -> PartFunction {
    PartFunction::new(kind, role, points).unwrap()
}
fn contactor(extra: Option<[&str; 2]>) -> Part {
    use FunctionKind::*;
    use FunctionRole::*;
    let mut functions = vec![
        function(Coil, Main, ["A1", "A2"]),
        function(MainContact, Auxiliary, ["1", "2"]),
        function(MainContact, Auxiliary, ["3", "4"]),
        function(MainContact, Auxiliary, ["5", "6"]),
        function(AuxiliaryContact, Auxiliary, ["13", "14"]),
        function(AuxiliaryContact, Auxiliary, ["21", "22"]),
    ];
    if let Some(points) = extra {
        functions.push(function(AuxiliaryContact, Auxiliary, points));
    }
    Part::new("ACME.K3", "ACME", "K3", "Contator", "Contator tripolar")
        .unwrap()
        .with_functions(functions)
        .unwrap()
}
/// K1 drawn with coil, three poles and the two auxiliary contacts.
fn drawn_k1() -> (Project, DeviceId, PlacementDelta) {
    let (mut project, page) = base();
    let coil = draw(
        &mut project,
        page,
        None,
        symbol("coil"),
        FunctionKind::Coil,
        &["A1", "A2"],
        40.,
    );
    let k1 = coil.device_id();
    for (i, labels) in [["1", "2"], ["3", "4"], ["5", "6"]].iter().enumerate() {
        draw(
            &mut project,
            page,
            Some(k1),
            pole(),
            FunctionKind::MainContact,
            labels,
            80. + 20. * i as f64,
        );
    }
    for (key, labels, x) in [
        ("contact-no", ["13", "14"], 160.),
        ("contact-nc", ["21", "22"], 180.),
    ] {
        draw(
            &mut project,
            page,
            Some(k1),
            symbol(key),
            FunctionKind::AuxiliaryContact,
            &labels,
            x,
        );
    }
    (project, k1, coil)
}
fn unrepresented(project: &Project) -> Vec<String> {
    project
        .check(CheckScope::Project)
        .messages
        .into_iter()
        .filter(|m| m.rule == "VER-006")
        .map(|m| m.text)
        .collect()
}

#[test]
fn a_fully_drawn_contactor_gets_its_part_without_new_functions() {
    let (mut project, k1, _) = drawn_k1();
    let part = contactor(None);
    let preview = project.part_assignment_preview(k1, &part);
    assert!(
        preview
            .iter()
            .all(|r| matches!(r.decision, PartFunctionMatch::Existing(_))),
        "{preview:?}"
    );
    let before = project.functions().count();
    project.assign_part(k1, part, 1).unwrap();
    assert_eq!(project.functions().count(), before, "nenhuma função nova");
    assert!(
        unrepresented(&project).is_empty(),
        "{:?}",
        unrepresented(&project)
    );
}

#[test]
fn coil_points_named_1_2_are_renamed_to_a1_a2_and_undo_restores_them() {
    let (mut project, page) = base();
    let coil = draw(
        &mut project,
        page,
        None,
        symbol("coil"),
        FunctionKind::Coil,
        &["1", "2"],
        40.,
    );
    let k1 = coil.device_id();
    let part = Part::new("ACME.R", "ACME", "R", "Relé", "Relé")
        .unwrap()
        .with_functions(vec![function(
            FunctionKind::Coil,
            FunctionRole::Main,
            ["A1", "A2"],
        )])
        .unwrap();
    let preview = project.part_assignment_preview(k1, &part);
    let PartFunctionMatch::Rename { function, points } = &preview[0].decision else {
        panic!("renomear esperado: {preview:?}");
    };
    assert_eq!(*function, coil.function_id());
    let names: Vec<_> = points.iter().map(|(_, n)| n.as_str()).collect();
    assert_eq!(names, ["A1", "A2"]);
    let bindings: Vec<_> = symbol("coil")
        .ports()
        .iter()
        .map(|port| {
            let r = PortRef::new(coil.representation_id(), port.id()).unwrap();
            project.port_binding(&r).unwrap()
        })
        .collect();
    let before = project.clone();
    let delta = project.assign_part(k1, part, 1).unwrap();
    project.validate().unwrap();
    assert_eq!(project.functions().count(), 1, "sem bobina duplicada");
    // Same points (IDs and bindings), new names shown on the sheet.
    for (port, point) in symbol("coil").ports().iter().zip(&bindings) {
        let r = PortRef::new(coil.representation_id(), port.id()).unwrap();
        assert_eq!(project.port_binding(&r), Some(*point));
    }
    assert_eq!(
        project
            .symbol_text(
                coil.representation_id(),
                &SymbolTextField::PortLabel("2".into())
            )
            .as_deref(),
        Some("A1")
    );
    project.restore_part_assignment(&delta, false).unwrap();
    assert_eq!(project, before);
    project.restore_part_assignment(&delta, true).unwrap();
    assert_eq!(project.functions().count(), 1);
}

#[test]
fn a_part_function_the_device_lacks_is_new_and_reported_as_unrepresented() {
    let (mut project, k1, _) = drawn_k1();
    let part = contactor(Some(["33", "34"]));
    let preview = project.part_assignment_preview(k1, &part);
    let new: Vec<_> = preview
        .iter()
        .filter(|r| r.decision == PartFunctionMatch::New)
        .map(|r| part.functions()[r.index].points()[0].as_str())
        .collect();
    assert_eq!(new, ["33"]);
    let before = project.functions().count();
    project.assign_part(k1, part, 1).unwrap();
    assert_eq!(project.functions().count(), before + 1);
    let messages = unrepresented(&project);
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(messages[0].contains("Contato auxiliar"), "{}", messages[0]);
}
