// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 19: macros — captura, inserção com IDs e designações novos, fios,
//! junções, Undo/Redo exato, outro projeto, macro de página e `.cadmacro`.
use cad_craft::domain::*;
use cad_craft::persistence;

fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}
fn at(x: f64, y: f64) -> Transform2D {
    Transform2D::new(p(x, y), 0.0, false, false).unwrap()
}
fn symbol(key: &str) -> SymbolDefinition {
    SymbolLibrary::builtin()
        .get(&SymbolDefinitionId::new(format!("cad-craft.{key}")).unwrap())
        .unwrap()
        .clone()
}
fn project_with_pages(n: usize) -> (Project, DocumentId, Vec<PageId>) {
    let mut project = Project::new("Macros").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "Esquema").unwrap())
        .unwrap();
    let pages = (0..n)
        .map(|i| {
            project
                .insert_page(
                    Page::new(
                        project.id(),
                        doc,
                        format!("Página {i}"),
                        format!("{}", i + 1),
                        PageKind::MultiLine,
                        Size2D::new(420.0, 297.0).unwrap(),
                    )
                    .unwrap(),
                )
                .unwrap()
        })
        .collect();
    (project, doc, pages)
}

#[test]
fn insertion_uses_project_page_column_rule_when_enabled() {
    let (mut project, _, pages) = project_with_pages(2);
    let source = place(
        &mut project,
        pages[0],
        "coil",
        PlacementDevice::New {
            designation: Designation::new("K5").unwrap(),
            classification: "K".into(),
        },
        FunctionKind::Coil,
        100.0,
        120.0,
    );
    let (content, _) = project.capture_macro(&[source]).unwrap();
    let definition = MacroDefinition::new(
        "TESTE.NUMERO",
        "Número",
        MacroKind::Window,
        p(100.0, 120.0),
        content,
    )
    .unwrap();
    let mut scheme = project.numbering_scheme().clone();
    scheme.use_on_insert = true;
    scheme.device_format = DeviceNumberFormat::PageColumn;
    project.set_numbering_scheme(scheme).unwrap();
    project
        .insert_macro(&definition, pages[1], at(200.0, 100.0))
        .unwrap();
    assert!(
        project
            .devices()
            .any(|d| d.designation().to_string() == "-K2.1")
    );
}
fn place(
    project: &mut Project,
    page: PageId,
    key: &str,
    device: PlacementDevice,
    kind: FunctionKind,
    x: f64,
    y: f64,
) -> RepresentationId {
    project
        .place_symbol(SymbolPlacement {
            page,
            definition: symbol(key),
            transform: at(x, y),
            device,
            function_kind: kind,
        })
        .unwrap()
        .representation_id()
}
fn new_device(project: &Project, prefix: &str) -> PlacementDevice {
    PlacementDevice::New {
        designation: project.next_device_designation(prefix).unwrap(),
        classification: prefix.into(),
    }
}
fn port(r: RepresentationId, id: &str) -> PortRef {
    PortRef::new(r, id).unwrap()
}
fn route(project: &mut Project, a: PortRef, b: PortRef, points: Vec<Point2D>) -> RepresentationId {
    project
        .edit_connection_drawing(ConnectionDrawingEdit::CreateRoute {
            source: a.clone(),
            target: b.clone(),
            points,
        })
        .unwrap();
    project
        .connection_routes()
        .find(|r| r.source() == &a && r.target() == &b)
        .unwrap()
        .id()
}

struct Circuit {
    q1: RepresentationId,
    f1: RepresentationId,
    k1: RepresentationId,
    k1_aux: RepresentationId,
    r1: RepresentationId,
}

/// Partida direta: -Q1 → -F1 → bobina -K1, contato auxiliar de -K1 ligado
/// por um fio em T com junção; -K1 com peça atribuída.
fn circuit(project: &mut Project, page: PageId) -> Circuit {
    let device = new_device(project, "Q");
    let q1 = place(
        project,
        page,
        "switch",
        device,
        FunctionKind::MainContact,
        100.0,
        200.0,
    );
    let device = new_device(project, "F");
    let f1 = place(
        project,
        page,
        "contact-nc",
        device,
        FunctionKind::AuxiliaryContact,
        100.0,
        160.0,
    );
    let device = new_device(project, "K");
    let k1 = place(
        project,
        page,
        "coil",
        device,
        FunctionKind::Coil,
        100.0,
        120.0,
    );
    let k1_device = match project.representation(k1).unwrap().target() {
        RepresentationTarget::Function(f) => project.function(f).unwrap().device_id(),
        _ => unreachable!(),
    };
    let k1_aux = place(
        project,
        page,
        "contact-no",
        PlacementDevice::Existing(k1_device),
        FunctionKind::AuxiliaryContact,
        140.0,
        160.0,
    );
    route(
        project,
        port(q1, "1"),
        port(f1, "2"),
        vec![p(100.0, 190.0), p(100.0, 170.0)],
    );
    let r1 = route(
        project,
        port(f1, "1"),
        port(k1, "2"),
        vec![p(100.0, 150.0), p(100.0, 130.0)],
    );
    let r2 = route(
        project,
        port(k1_aux, "1"),
        port(k1, "2"),
        vec![
            p(140.0, 150.0),
            p(140.0, 140.0),
            p(100.0, 140.0),
            p(100.0, 130.0),
        ],
    );
    project
        .edit_connection_drawing(ConnectionDrawingEdit::CreateJunction {
            page,
            position: p(100.0, 140.0),
            routes: vec![r1, r2],
        })
        .unwrap();
    let part = Part::new("ACME.K", "ACME", "K", "Contator", "Contator").unwrap();
    project.assign_part(k1_device, part, 1).unwrap();
    project.validate().unwrap();
    Circuit {
        q1,
        f1,
        k1,
        k1_aux,
        r1,
    }
}
fn page_ids(project: &Project, page: PageId) -> Vec<RepresentationId> {
    project
        .representations()
        .filter(|r| r.page_id() == page)
        .map(Representation::id)
        .collect()
}
fn designations(project: &Project) -> Vec<String> {
    let mut names: Vec<_> = project
        .devices()
        .map(|d| d.designation().to_string())
        .collect();
    names.sort();
    names
}
fn snapshot(project: &Project) -> String {
    serde_json::to_string(project).unwrap()
}
fn window_macro(project: &Project, page: PageId) -> MacroDefinition {
    let (content, report) = project.capture_macro(&page_ids(project, page)).unwrap();
    assert!(report.is_clean(), "{report:?}");
    MacroDefinition::new(
        "PARTIDA.DIRETA",
        "Partida direta",
        MacroKind::Window,
        p(60.0, 100.0),
        content,
    )
    .unwrap()
    .with_category("Motores")
    .unwrap()
}

#[test]
fn capture_takes_devices_functions_wires_junction_and_parts() {
    let (mut project, _, pages) = project_with_pages(1);
    circuit(&mut project, pages[0]);
    let definition = window_macro(&project, pages[0]);
    let content = definition.content();
    assert_eq!(content.devices().len(), 3);
    assert_eq!(content.functions().len(), 4);
    assert_eq!(content.routes().len(), 3);
    assert_eq!(content.junctions().len(), 1);
    assert_eq!(content.parts().len(), 1);
    assert_eq!(content.definitions().len(), 4);
    assert_eq!(definition.category(), "Motores");
}

#[test]
fn inserting_twice_creates_next_designations_and_equivalent_network() {
    let (mut project, _, pages) = project_with_pages(2);
    circuit(&mut project, pages[0]);
    let definition = window_macro(&project, pages[0]);
    let (first, report) = project
        .insert_macro(&definition, pages[1], at(200.0, 100.0))
        .unwrap();
    assert!(report.is_clean(), "{report:?}");
    let (second, _) = project
        .insert_macro(&definition, pages[1], at(300.0, 100.0))
        .unwrap();
    assert_eq!(
        designations(&project),
        [
            "-F1", "-F2", "-F3", "-K1", "-K2", "-K3", "-Q1", "-Q2", "-Q3"
        ]
    );
    assert_eq!(first.route_ids().len(), 3);
    assert_eq!(second.device_ids().len(), 3);
    assert_eq!(project.connection_junctions().count(), 3);
    // Mesma rede: o ponto 1 de cada -Q liga ao ponto 2 de cada -F, e -F.1,
    // -K.2 e o contato auxiliar ficam no mesmo potencial pela junção.
    for delta in [&first, &second] {
        let reps: Vec<&Representation> = delta
            .representation_ids()
            .iter()
            .map(|id| project.representation(*id).unwrap())
            .collect();
        let point = |key: &str, port_id: &str| {
            let r = reps
                .iter()
                .find(|r| {
                    r.symbol()
                        .is_some_and(|s| s.as_str() == format!("cad-craft.{key}"))
                })
                .unwrap();
            project.port_binding(&port(r.id(), port_id)).unwrap()
        };
        assert!(
            project
                .are_connected(point("switch", "1"), point("contact-nc", "2"))
                .unwrap()
        );
        assert!(
            project
                .are_connected(point("contact-nc", "1"), point("contact-no", "1"))
                .unwrap()
        );
        assert!(
            !project
                .are_connected(point("switch", "1"), point("coil", "1"))
                .unwrap()
        );
    }
    // Nada da macro aponta para a página ou os dispositivos de origem.
    let k = project
        .devices()
        .find(|d| d.designation().to_string() == "-K2")
        .unwrap()
        .id();
    assert_eq!(
        project.functions().filter(|f| f.device_id() == k).count(),
        2
    );
    assert_eq!(
        project
            .components()
            .filter(|c| c.device_id() == Some(k))
            .count(),
        1
    );
    project.validate().unwrap();
}

#[test]
fn undo_removes_whole_insertion_and_redo_restores_identical_state() {
    let (mut project, _, pages) = project_with_pages(2);
    circuit(&mut project, pages[0]);
    let definition = window_macro(&project, pages[0]);
    let before = snapshot(&project);
    let (delta, _) = project
        .insert_macro(&definition, pages[1], at(200.0, 100.0))
        .unwrap();
    let after = snapshot(&project);
    project.restore_macro_insert(&delta, false).unwrap();
    assert_eq!(snapshot(&project), before);
    project.restore_macro_insert(&delta, true).unwrap();
    assert_eq!(snapshot(&project), after);
    // Repetir o refazer é recusado sem alterar nada.
    assert!(project.restore_macro_insert(&delta, true).is_err());
    assert_eq!(snapshot(&project), after);
}

#[test]
fn rotation_and_mirroring_keep_wires_on_ports() {
    let (mut project, _, pages) = project_with_pages(2);
    circuit(&mut project, pages[0]);
    let definition = window_macro(&project, pages[0]);
    for (x, rotation, mirror) in [
        (100.0, 90.0, false),
        (200.0, 180.0, false),
        (300.0, 270.0, false),
        (100.0, 0.0, true),
    ] {
        let y = if mirror { 50.0 } else { 200.0 };
        let (delta, _) = project
            .insert_macro(
                &definition,
                pages[1],
                Transform2D::new(p(x, y), rotation, mirror, false).unwrap(),
            )
            .unwrap();
        for id in delta.route_ids() {
            let r = project.connection_route(id).unwrap();
            assert_eq!(
                r.points().first(),
                Some(&project.port_position(r.source()).unwrap())
            );
            assert_eq!(
                r.points().last(),
                Some(&project.port_position(r.target()).unwrap())
            );
        }
    }
    project.validate().unwrap();
}

#[test]
fn route_with_end_outside_selection_is_dropped_and_reported() {
    let (mut project, _, pages) = project_with_pages(1);
    let c = circuit(&mut project, pages[0]);
    let (content, report) = project.capture_macro(&[c.q1, c.f1, c.r1]).unwrap();
    assert_eq!(report.dropped_routes, 1);
    assert_eq!(content.routes().len(), 1);
    assert_eq!(content.junctions().len(), 0);
    let (_, report) = project
        .capture_macro(&[c.f1, c.k1, c.k1_aux, c.r1])
        .unwrap();
    assert_eq!(report.dropped_routes, 0);
    assert!(!report.summary().contains("junção") || report.dropped_junctions > 0);
}

#[test]
fn inserting_into_other_project_adds_symbols_and_parts_and_survives_save() {
    let (mut source, _, pages) = project_with_pages(1);
    circuit(&mut source, pages[0]);
    let definition = window_macro(&source, pages[0]);
    let (mut target, _, target_pages) = project_with_pages(1);
    assert_eq!(target.symbol_definitions().count(), 0);
    target
        .insert_macro(&definition, target_pages[0], at(150.0, 150.0))
        .unwrap();
    assert_eq!(target.symbol_definitions().count(), 4);
    assert_eq!(target.parts().count(), 1);
    assert_eq!(designations(&target), ["-F1", "-K1", "-Q1"]);
    let bytes = persistence::write_project_to_vec(&target).unwrap();
    let reopened = persistence::read_project_from_slice(&bytes)
        .unwrap()
        .project;
    assert_eq!(snapshot(&reopened), snapshot(&target));
}

#[test]
fn different_symbol_version_keeps_project_version_and_warns() {
    let (mut source, _, pages) = project_with_pages(1);
    circuit(&mut source, pages[0]);
    let definition = window_macro(&source, pages[0]);
    let (mut target, _, target_pages) = project_with_pages(1);
    let coil = symbol("coil");
    let newer = SymbolDefinition::new(
        coil.id().clone(),
        2,
        "Bobina v2",
        coil.category(),
        coil.primitives().to_vec(),
        coil.ports().to_vec(),
    )
    .unwrap();
    target.add_symbol_definition(newer.clone()).unwrap();
    let (_, report) = target
        .insert_macro(&definition, target_pages[0], at(150.0, 150.0))
        .unwrap();
    assert_eq!(report.kept_symbols, vec![coil.id().clone()]);
    assert_eq!(target.symbol_definition(coil.id()), Some(&newer));
    assert!(report.summary().contains("cad-craft.coil"));
}

#[test]
fn page_macro_creates_page_with_content_and_undo_removes_it() {
    let (mut project, doc, pages) = project_with_pages(1);
    circuit(&mut project, pages[0]);
    let (content, _) = project.capture_page_macro(pages[0]).unwrap();
    let definition = MacroDefinition::new(
        "PAGINA.PARTIDA",
        "Página de partida",
        MacroKind::Page {
            name: Text::new("Partida").unwrap(),
            kind: PageKind::MultiLine,
            size: Size2D::new(420.0, 297.0).unwrap(),
        },
        p(0.0, 0.0),
        content,
    )
    .unwrap();
    let before = snapshot(&project);
    let (delta, _) = project
        .insert_page_macro(&definition, doc, "2", None)
        .unwrap();
    let page = delta.created_page().unwrap();
    assert_eq!(project.page(page).unwrap().name().as_str(), "Partida");
    assert_eq!(
        project
            .representations()
            .filter(|r| r.page_id() == page)
            .count(),
        page_ids(&project, pages[0]).len()
    );
    project.restore_macro_insert(&delta, false).unwrap();
    assert_eq!(snapshot(&project), before);
    project.restore_macro_insert(&delta, true).unwrap();
    assert!(project.page(page).is_some());
}

#[test]
fn cadmacro_round_trips_and_rejects_invalid_files() {
    let (mut project, _, pages) = project_with_pages(1);
    circuit(&mut project, pages[0]);
    let definition = window_macro(&project, pages[0])
        .with_description("Partida direta com contator")
        .unwrap();
    let bytes = persistence::write_macro_package(&definition).unwrap();
    let read = persistence::read_macro_package(&bytes).unwrap();
    assert_eq!(read, definition);
    // Não é ZIP / truncado.
    assert!(persistence::read_macro_package(b"not a zip").is_err());
    assert!(persistence::read_macro_package(&bytes[..bytes.len() / 2]).is_err());
    // Outro tipo de pacote.
    let part = Part::new("X.Y", "X", "Y", "Peça", "Classe").unwrap();
    let part_bytes = persistence::write_part_package(&part).unwrap();
    assert!(persistence::read_macro_package(&part_bytes).is_err());
    // Referência quebrada: um vínculo aponta para ponto inexistente.
    let mut value = serde_json::to_value(&definition).unwrap();
    let bindings = value["content"]["bindings"].as_array_mut().unwrap();
    bindings[0]["point"] = serde_json::Value::String(ConnectionPointId::new().to_string());
    assert!(
        serde_json::from_value::<MacroDefinition>(value.clone())
            .unwrap()
            .validate()
            .is_err()
    );
    // Versão zero.
    let mut zero = serde_json::to_value(&definition).unwrap();
    zero["version"] = 0.into();
    assert!(serde_json::from_value::<MacroDefinition>(zero).is_err());
}

#[test]
fn transform_composition_matches_sequential_application() {
    let inner = Transform2D::new(p(10.0, 5.0), 90.0, true, false).unwrap();
    for outer in [
        Transform2D::new(p(3.0, -7.0), 0.0, false, false).unwrap(),
        Transform2D::new(p(3.0, -7.0), 90.0, false, false).unwrap(),
        Transform2D::new(p(3.0, -7.0), 270.0, true, false).unwrap(),
        Transform2D::new(p(3.0, -7.0), 180.0, false, true).unwrap(),
        Transform2D::new(p(3.0, -7.0), 0.0, true, true).unwrap(),
    ] {
        let composed = inner.then(outer).unwrap();
        for q in [p(0.0, 0.0), p(2.5, -1.0), p(-4.0, 6.0)] {
            assert_eq!(
                composed.apply(q).unwrap(),
                outer.apply(inner.apply(q).unwrap()).unwrap(),
                "{outer:?}"
            );
        }
        assert!(!(composed.mirror_x() && composed.mirror_y()));
    }
}

#[test]
fn selection_on_two_pages_or_empty_is_refused() {
    let (mut project, _, pages) = project_with_pages(2);
    let c = circuit(&mut project, pages[0]);
    let device = new_device(&project, "K");
    let other = place(
        &mut project,
        pages[1],
        "coil",
        device,
        FunctionKind::Coil,
        50.0,
        50.0,
    );
    assert!(project.capture_macro(&[]).is_err());
    assert!(project.capture_macro(&[c.k1, other]).is_err());
}
