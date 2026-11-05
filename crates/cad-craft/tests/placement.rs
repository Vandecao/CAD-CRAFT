// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 12: inserção de símbolo com cadastro elétrico e Undo documental.
use cad_craft::domain::*;

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
fn base() -> (Project, PageId) {
    let mut project = Project::new("Colocação").unwrap();
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
    (project, page)
}
fn new_device(project: &Project, page: PageId, key: &str, prefix: &str, x: f64) -> SymbolPlacement {
    SymbolPlacement {
        page,
        definition: symbol(key),
        transform: at(x, 100.0),
        device: PlacementDevice::New {
            designation: project.next_device_designation(prefix).unwrap(),
            classification: "Relé".into(),
        },
        function_kind: FunctionKind::Coil,
    }
}

#[test]
fn new_device_placement_registers_device_function_points_and_bindings() {
    let (mut project, page) = base();
    let delta = project
        .place_symbol(new_device(&project, page, "coil", "K", 100.0))
        .unwrap();
    assert!(delta.created_device());
    let device = project.device(delta.device_id()).unwrap();
    assert_eq!(device.designation().to_string(), "-K1");
    let function = project.function(delta.function_id()).unwrap();
    assert_eq!(function.role(), FunctionRole::Main);
    assert_eq!(function.device_id(), device.id());
    let rep = project.representation(delta.representation_id()).unwrap();
    assert_eq!(rep.target(), RepresentationTarget::Function(function.id()));
    let ports = symbol("coil").ports().len();
    assert_eq!(delta.point_ids().len(), ports);
    for port in symbol("coil").ports() {
        let bound = project
            .port_binding(&PortRef::new(rep.id(), port.id()).unwrap())
            .unwrap();
        assert!(delta.point_ids().contains(&bound));
    }
    assert_eq!(
        project.next_device_designation("K").unwrap().to_string(),
        "-K2"
    );
    project.validate().unwrap();
}

#[test]
fn placed_devices_can_be_connected_and_networks_are_derived() {
    let (mut project, page) = base();
    let a = project
        .place_symbol(new_device(&project, page, "contact-no", "K", 100.0))
        .unwrap();
    let b = project
        .place_symbol(SymbolPlacement {
            transform: at(100.0, 150.0),
            ..new_device(&project, page, "coil", "K", 100.0)
        })
        .unwrap();
    let source = PortRef::new(a.representation_id(), "2").unwrap();
    let target = PortRef::new(b.representation_id(), "1").unwrap();
    let start = project.port_position(&source).unwrap();
    let end = project.port_position(&target).unwrap();
    project
        .edit_connection_drawing(ConnectionDrawingEdit::CreateRoute {
            source,
            target,
            points: vec![start, end],
        })
        .unwrap();
    let a2 = project.port_binding(&PortRef::new(a.representation_id(), "2").unwrap());
    let b1 = project.port_binding(&PortRef::new(b.representation_id(), "1").unwrap());
    assert!(project.are_connected(a2.unwrap(), b1.unwrap()).unwrap());
    assert_eq!(project.devices().count(), 2);
}

#[test]
fn undo_and_redo_of_placement_restore_exact_state_with_same_ids() {
    let (mut project, page) = base();
    let before = project.clone();
    let delta = project
        .place_symbol(new_device(&project, page, "motor", "M", 100.0))
        .unwrap();
    let after = project.clone();
    project.restore_placement(&delta, false).unwrap();
    assert_eq!(project, before);
    assert!(project.symbol_definitions().next().is_none());
    project.restore_placement(&delta, true).unwrap();
    assert_eq!(project, after);
    // Repeated replay in the same direction is refused without changes.
    assert!(project.restore_placement(&delta, true).is_err());
    assert_eq!(project, after);
}

#[test]
fn undo_is_refused_while_another_entity_references_the_placement() {
    let (mut project, page) = base();
    let a = project
        .place_symbol(new_device(&project, page, "contact-no", "K", 100.0))
        .unwrap();
    let b = project
        .place_symbol(new_device(&project, page, "coil", "K", 200.0))
        .unwrap();
    let source = PortRef::new(a.representation_id(), "1").unwrap();
    let target = PortRef::new(b.representation_id(), "1").unwrap();
    let s = project.port_position(&source).unwrap();
    let t = project.port_position(&target).unwrap();
    project
        .edit_connection_drawing(ConnectionDrawingEdit::CreateRoute {
            source,
            target,
            points: vec![s, p(s.x(), 60.0), p(t.x(), 60.0), t],
        })
        .unwrap();
    let snapshot = project.clone();
    assert!(project.restore_placement(&a, false).is_err());
    assert_eq!(project, snapshot);
    // A property citing the new device also blocks removal and is rolled back.
    let (mut project, page) = base();
    let c = project
        .place_symbol(new_device(&project, page, "coil", "K", 100.0))
        .unwrap();
    let mut props = TechnicalProperties::default();
    props
        .set_custom(
            "ref",
            PropertyValue::Reference(EntityRef::Device(c.device_id())),
        )
        .unwrap();
    project
        .set_properties(EntityRef::Project(project.id()), props)
        .unwrap();
    let snapshot = project.clone();
    assert!(project.restore_placement(&c, false).is_err());
    assert_eq!(project, snapshot);
}

#[test]
fn existing_device_receives_auxiliary_function_and_keeps_device_on_undo() {
    let (mut project, page) = base();
    let coil = project
        .place_symbol(new_device(&project, page, "coil", "K", 100.0))
        .unwrap();
    let before = project.clone();
    let contact = project
        .place_symbol(SymbolPlacement {
            page,
            definition: symbol("contact-no"),
            transform: at(200.0, 100.0),
            device: PlacementDevice::Existing(coil.device_id()),
            function_kind: FunctionKind::AuxiliaryContact,
        })
        .unwrap();
    assert!(!contact.created_device());
    let function = project.function(contact.function_id()).unwrap();
    assert_eq!(function.role(), FunctionRole::Auxiliary);
    assert_eq!(
        project.device_references(coil.device_id()).unwrap().len(),
        2
    );
    project.restore_placement(&contact, false).unwrap();
    assert_eq!(project, before);
}

#[test]
fn failed_placement_leaves_project_unchanged() {
    let (mut project, page) = base();
    project
        .place_symbol(new_device(&project, page, "coil", "K", 100.0))
        .unwrap();
    let snapshot = project.clone();
    let duplicate = SymbolPlacement {
        device: PlacementDevice::New {
            designation: Designation::new("K1").unwrap(),
            classification: "Relé".into(),
        },
        ..new_device(&project, page, "coil", "K", 200.0)
    };
    assert!(project.place_symbol(duplicate).is_err());
    assert_eq!(project, snapshot);
    let missing = SymbolPlacement {
        page: PageId::new(),
        ..new_device(&project, page, "coil", "K", 200.0)
    };
    assert!(project.place_symbol(missing).is_err());
    assert_eq!(project, snapshot);
}

#[test]
fn designation_counter_skips_used_numbers() {
    let (mut project, _) = base();
    for name in ["K1", "K3", "M1"] {
        project
            .insert_device(Device::new(project.id(), Designation::new(name).unwrap(), "x").unwrap())
            .unwrap();
    }
    assert_eq!(
        project.next_device_designation("K").unwrap().to_string(),
        "-K2"
    );
    assert_eq!(
        project.next_device_designation(" M ").unwrap().to_string(),
        "-M2"
    );
    assert!(project.next_device_designation("").is_ok());
    assert!(project.next_device_designation("K/").is_err());
}

#[test]
fn document_edits_undo_and_redo_in_reverse_order() {
    let (mut project, first) = base();
    let start = project.clone();
    let doc = Document::new(project.id(), "Anexos").unwrap();
    let doc_id = doc.id();
    let page = Page::new(
        project.id(),
        doc_id,
        "Notas",
        "1",
        PageKind::TechnicalDocumentation,
        Size2D::new(210.0, 297.0).unwrap(),
    )
    .unwrap();
    let page_b = Page::new(
        project.id(),
        project.page(first).unwrap().document_id(),
        "Força",
        "2",
        PageKind::MultiLine,
        Size2D::new(420.0, 297.0).unwrap(),
    )
    .unwrap();
    let main_doc = project.page(first).unwrap().document_id();
    let mut deltas = vec![
        project
            .edit_document(DocumentEdit::InsertDocument(doc))
            .unwrap(),
        project
            .edit_document(DocumentEdit::InsertPage(Box::new(page.clone())))
            .unwrap(),
        project
            .edit_document(DocumentEdit::InsertPage(Box::new(page_b.clone())))
            .unwrap(),
    ];
    deltas.push(
        project
            .edit_document(DocumentEdit::ReorderPages {
                document: main_doc,
                order: vec![page_b.id(), first],
            })
            .unwrap(),
    );
    deltas.push(
        project
            .edit_document(DocumentEdit::RemovePage(page.id()))
            .unwrap(),
    );
    let end = project.clone();
    assert_eq!(
        project.ordered_pages(main_doc).unwrap()[0].id(),
        page_b.id()
    );
    for delta in deltas.iter().rev() {
        project.restore_document_edit(delta, false).unwrap();
    }
    assert_eq!(project, start);
    for delta in &deltas {
        project.restore_document_edit(delta, true).unwrap();
    }
    assert_eq!(project, end);
    // Out-of-order replay is refused.
    assert!(project.restore_document_edit(&deltas[1], false).is_err());
    assert_eq!(project, end);
}

#[test]
fn remove_document_requires_no_pages() {
    let (mut project, page) = base();
    let doc = project.page(page).unwrap().document_id();
    assert!(project.remove_document(doc).is_err());
    project.remove_page(page).unwrap();
    project.remove_document(doc).unwrap();
    assert_eq!(project.documents().count(), 0);
}
