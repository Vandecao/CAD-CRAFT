// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

use cad_craft::{domain::*, error::DomainError};

fn device(project: &Project, tag: &str) -> Device {
    Device::new(project.id(), Designation::new(tag).unwrap(), "contactor").unwrap()
}

#[test]
fn project_documents_and_pages_are_queryable_without_duplicate_storage() {
    assert!(Project::new("  ").is_err());
    let mut project = Project::new("Linha de produção").unwrap();
    let id = project.id();
    project.set_description("Painel de comando");
    project.set_metadata("author", "Engenharia").unwrap();
    let mut document = Document::new(id, "Comando").unwrap();
    document.set_metadata("revision", "A").unwrap();
    let doc_id = project.insert_document(document).unwrap();
    for (number, kind) in [
        ("1", PageKind::MultiLine),
        ("2", PageKind::SingleLine),
        ("3", PageKind::Overview),
        ("4", PageKind::PanelLayout),
        ("5", PageKind::TechnicalDocumentation),
    ] {
        let mut page = Page::new(
            id,
            doc_id,
            "Folha",
            number,
            kind,
            Size2D::new(420.0, 297.0).unwrap(),
        )
        .unwrap();
        page.set_metadata("revision", "B").unwrap();
        project.insert_page(page).unwrap();
    }
    assert_eq!(project.id(), id);
    assert_eq!(project.documents().count(), 1);
    assert_eq!(project.pages_in_document(doc_id).count(), 5);
    assert_eq!(project.document(doc_id).unwrap().name().as_str(), "Comando");
    assert!(project.page(PageId::new()).is_none());
    project.validate().unwrap();
}

#[test]
fn contactor_has_multiple_functions_across_pages_and_physical_terminal() {
    let mut project = Project::new("P").unwrap();
    let d = project.insert_device(device(&project, "K1")).unwrap();
    let coil = project
        .insert_function(Function::new(
            project.id(),
            d,
            Designation::new("K1").unwrap(),
            FunctionKind::Coil,
        ))
        .unwrap();
    let aux = project
        .insert_function(Function::new(
            project.id(),
            d,
            Designation::new("K1").unwrap(),
            FunctionKind::AuxiliaryContact,
        ))
        .unwrap();
    assert_eq!(project.functions_for_device(d).count(), 2);
    let component = Component::new(project.id(), Designation::new("K1").unwrap(), "contactor")
        .unwrap()
        .with_assignment(Some(d), Some(coil))
        .with_catalog("Example", "ABC-01")
        .unwrap();
    let c = project.insert_component(component).unwrap();
    let terminal = Terminal::new(
        project.id(),
        TerminalOwner::Component(c),
        DesignationPart::new("A1").unwrap(),
    );
    let t = project.insert_terminal(terminal).unwrap();
    let cp = project
        .insert_connection_point(
            ConnectionPoint::new(project.id(), coil, DesignationPart::new("1").unwrap())
                .with_terminal(t),
        )
        .unwrap();
    assert_eq!(project.connection_point(cp).unwrap().terminal_id(), Some(t));
    let doc = project
        .insert_document(Document::new(project.id(), "D").unwrap())
        .unwrap();
    for f in [coil, aux] {
        let page = project
            .insert_page(
                Page::new(
                    project.id(),
                    doc,
                    "P",
                    "1",
                    PageKind::MultiLine,
                    Size2D::new(1.0, 1.0).unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        project
            .add_page_element(page, PageElement::Function(f))
            .unwrap();
        project
            .add_page_element(page, PageElement::Component(c))
            .unwrap();
        assert_eq!(project.page(page).unwrap().elements().count(), 2);
    }
    project.validate().unwrap();
}

#[test]
fn duplicate_and_foreign_entities_are_rejected_without_mutation() {
    let mut project = Project::new("A").unwrap();
    let d = device(&project, "K1");
    let id = project.insert_device(d.clone()).unwrap();
    let before = project.clone();
    assert_eq!(
        project.insert_device(d),
        Err(DomainError::DuplicateEntity(EntityRef::Device(id)))
    );
    let other = Project::new("B").unwrap();
    assert!(matches!(
        project.insert_device(device(&other, "K2")),
        Err(DomainError::ForeignProject { .. })
    ));
    assert_eq!(project, before);
}

#[test]
fn missing_parent_references_are_rejected_for_every_entity_kind() {
    let mut project = Project::new("A").unwrap();
    let before = project.clone();
    assert!(
        project
            .insert_function(Function::new(
                project.id(),
                DeviceId::new(),
                Designation::new("K1").unwrap(),
                FunctionKind::Coil
            ))
            .is_err()
    );
    assert!(
        project
            .insert_page(
                Page::new(
                    project.id(),
                    DocumentId::new(),
                    "P",
                    "1",
                    PageKind::Overview,
                    Size2D::new(1.0, 1.0).unwrap()
                )
                .unwrap()
            )
            .is_err()
    );
    assert!(
        project
            .insert_component(
                Component::new(project.id(), Designation::new("K1").unwrap(), "test")
                    .unwrap()
                    .with_assignment(Some(DeviceId::new()), None)
            )
            .is_err()
    );
    assert!(
        project
            .insert_terminal(Terminal::new(
                project.id(),
                TerminalOwner::Component(ComponentId::new()),
                DesignationPart::new("1").unwrap()
            ))
            .is_err()
    );
    assert!(
        project
            .insert_terminal(Terminal::new(
                project.id(),
                TerminalOwner::Device(DeviceId::new()),
                DesignationPart::new("1").unwrap()
            ))
            .is_err()
    );
    assert!(
        project
            .insert_connection_point(ConnectionPoint::new(
                project.id(),
                FunctionId::new(),
                DesignationPart::new("1").unwrap()
            ))
            .is_err()
    );
    assert_eq!(project, before);
}

#[test]
fn incompatible_component_function_and_terminal_relations_are_rejected() {
    let mut project = Project::new("A").unwrap();
    let d1 = project.insert_device(device(&project, "K1")).unwrap();
    let d2 = project.insert_device(device(&project, "K2")).unwrap();
    let f1 = project
        .insert_function(Function::new(
            project.id(),
            d1,
            Designation::new("K1").unwrap(),
            FunctionKind::Coil,
        ))
        .unwrap();
    let f2 = project
        .insert_function(Function::new(
            project.id(),
            d1,
            Designation::new("K1").unwrap(),
            FunctionKind::MainContact,
        ))
        .unwrap();
    let make_c = || Component::new(project.id(), Designation::new("K1").unwrap(), "test").unwrap();
    let wrong_device = make_c().with_assignment(Some(d2), Some(f1));
    let no_device = make_c().with_assignment(None, Some(f1));
    assert!(project.insert_component(wrong_device).is_err());
    assert!(project.insert_component(no_device).is_err());
    let t = project
        .insert_terminal(Terminal::new(
            project.id(),
            TerminalOwner::Device(d2),
            DesignationPart::new("1").unwrap(),
        ))
        .unwrap();
    assert!(
        project
            .insert_connection_point(
                ConnectionPoint::new(project.id(), f1, DesignationPart::new("1").unwrap())
                    .with_terminal(t)
            )
            .is_err()
    );
    let c = project
        .insert_component(
            Component::new(project.id(), Designation::new("K1").unwrap(), "test")
                .unwrap()
                .with_assignment(Some(d1), Some(f1)),
        )
        .unwrap();
    let t = project
        .insert_terminal(Terminal::new(
            project.id(),
            TerminalOwner::Component(c),
            DesignationPart::new("1").unwrap(),
        ))
        .unwrap();
    assert!(
        project
            .insert_connection_point(
                ConnectionPoint::new(project.id(), f2, DesignationPart::new("1").unwrap())
                    .with_terminal(t)
            )
            .is_err()
    );
    project.validate().unwrap();
}

#[test]
fn properties_and_page_references_are_validated_before_mutating_project() {
    let mut project = Project::new("A").unwrap();
    let d = project.insert_device(device(&project, "K1")).unwrap();
    let mut props = TechnicalProperties::default();
    props
        .set_custom("device", PropertyValue::Reference(EntityRef::Device(d)))
        .unwrap();
    project
        .set_properties(EntityRef::Device(d), props.clone())
        .unwrap();
    let before = project.clone();
    props
        .set_custom(
            "missing",
            PropertyValue::Reference(EntityRef::Page(PageId::new())),
        )
        .unwrap();
    assert!(project.set_properties(EntityRef::Device(d), props).is_err());
    assert!(
        project
            .set_properties(
                EntityRef::Device(DeviceId::new()),
                TechnicalProperties::default()
            )
            .is_err()
    );
    assert_eq!(project, before);
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
                Size2D::new(1.0, 1.0).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let c = project
        .insert_component(
            Component::new(project.id(), Designation::new("K1").unwrap(), "test").unwrap(),
        )
        .unwrap();
    project
        .add_page_element(page, PageElement::Component(c))
        .unwrap();
    let before = project.clone();
    assert!(
        project
            .add_page_element(page, PageElement::Component(c))
            .is_err()
    );
    assert!(
        project
            .add_page_element(page, PageElement::Function(FunctionId::new()))
            .is_err()
    );
    assert!(
        project
            .add_page_element(PageId::new(), PageElement::Component(c))
            .is_err()
    );
    assert_eq!(project, before);
}

#[test]
fn detached_entity_properties_are_checked_on_insertion() {
    let mut project = Project::new("A").unwrap();
    let mut props = TechnicalProperties::default();
    props
        .set_custom(
            "foreign",
            PropertyValue::Reference(EntityRef::Project(ProjectId::new())),
        )
        .unwrap();
    assert!(
        project
            .insert_device(device(&project, "K1").with_properties(props))
            .is_err()
    );
    assert_eq!(project.devices().count(), 0);
}
