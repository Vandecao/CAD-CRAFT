// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

mod physical_common;
use cad_craft::domain::*;
use physical_common::*;
use serde_json::json;

fn page(p: &mut Project) -> PageId {
    let doc = p
        .insert_document(Document::new(p.id(), "D").unwrap())
        .unwrap();
    p.insert_page(
        Page::new(
            p.id(),
            doc,
            "F",
            "1",
            PageKind::MultiLine,
            Size2D::new(297.0, 210.0).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn all_electrical_targets_roundtrip_and_document_edits_preserve_wiring() {
    let mut p = populated();
    let page = page(&mut p);
    let component = p
        .insert_component(Component::new(p.id(), Designation::new("K2").unwrap(), "test").unwrap())
        .unwrap();
    let targets = [
        RepresentationTarget::Device(p.devices().next().unwrap().id()),
        RepresentationTarget::Function(p.functions().next().unwrap().id()),
        RepresentationTarget::Component(component),
        RepresentationTarget::Terminal(p.terminals().next().unwrap().id()),
        RepresentationTarget::ConnectionPoint(p.connection_points().next().unwrap().id()),
        RepresentationTarget::Connection(p.connections().next().unwrap().id()),
        RepresentationTarget::Conductor(p.conductors().next().unwrap().id()),
        RepresentationTarget::Cable(p.cables().next().unwrap().id()),
        RepresentationTarget::CableCore(p.cable_cores().next().unwrap().id()),
        RepresentationTarget::TerminalStrip(p.terminal_strips().next().unwrap().id()),
    ];
    let before = json!(p);
    let networks = p.networks();
    let transform = Transform2D::new(Point2D::new(0.0, 0.0).unwrap(), 0.0, false, false).unwrap();
    for target in targets {
        p.insert_representation(Representation::new(p.id(), page, target, transform))
            .unwrap();
    }
    let mut loaded: Project = serde_json::from_value(json!(p)).unwrap();
    assert_eq!(loaded, p);
    for target in targets {
        let id = loaded.representations_for_entity(target).unwrap()[0].id();
        loaded.remove_representation(id).unwrap();
    }
    loaded.remove_page(page).unwrap();
    loaded.validate().unwrap();
    let after = json!(loaded);
    for key in [
        "devices",
        "functions",
        "components",
        "terminals",
        "connection_points",
        "connections",
        "conductors",
        "cables",
        "cable_cores",
        "terminal_strips",
    ] {
        assert_eq!(before[key], after[key], "{key}");
    }
    assert_eq!(loaded.networks(), networks);
}

#[test]
fn each_property_owner_blocks_page_deletion() {
    let mut p = populated();
    let sheet = page(&mut p);
    let other = page(&mut p);
    let component = p
        .insert_component(Component::new(p.id(), Designation::new("K2").unwrap(), "test").unwrap())
        .unwrap();
    let owners = [
        EntityRef::Project(p.id()),
        EntityRef::Document(p.documents().next().unwrap().id()),
        EntityRef::Page(other),
        EntityRef::Device(p.devices().next().unwrap().id()),
        EntityRef::Function(p.functions().next().unwrap().id()),
        EntityRef::Component(component),
        EntityRef::Terminal(p.terminals().next().unwrap().id()),
        EntityRef::ConnectionPoint(p.connection_points().next().unwrap().id()),
        EntityRef::Conductor(p.conductors().next().unwrap().id()),
        EntityRef::Cable(p.cables().next().unwrap().id()),
        EntityRef::CableCore(p.cable_cores().next().unwrap().id()),
        EntityRef::TerminalStrip(p.terminal_strips().next().unwrap().id()),
    ];
    for owner in owners {
        let mut props = TechnicalProperties::default();
        props
            .set_custom("page", PropertyValue::Reference(EntityRef::Page(sheet)))
            .unwrap();
        p.set_properties(owner, props).unwrap();
        let before = p.clone();
        assert!(p.remove_page(sheet).is_err(), "{owner:?}");
        assert_eq!(p, before);
        p.set_properties(owner, TechnicalProperties::default())
            .unwrap();
    }
    p.remove_page(sheet).unwrap();
    p.validate().unwrap();
}

#[test]
fn every_missing_electrical_target_is_rejected() {
    let mut p = Project::new("P").unwrap();
    let page = page(&mut p);
    let targets = [
        RepresentationTarget::Device(DeviceId::new()),
        RepresentationTarget::Function(FunctionId::new()),
        RepresentationTarget::Component(ComponentId::new()),
        RepresentationTarget::Terminal(TerminalId::new()),
        RepresentationTarget::ConnectionPoint(ConnectionPointId::new()),
        RepresentationTarget::Connection(ConnectionId::new()),
        RepresentationTarget::Conductor(ConductorId::new()),
        RepresentationTarget::Cable(CableId::new()),
        RepresentationTarget::CableCore(CableCoreId::new()),
        RepresentationTarget::TerminalStrip(TerminalStripId::new()),
    ];
    let before = p.clone();
    for target in targets {
        let transform =
            Transform2D::new(Point2D::new(0.0, 0.0).unwrap(), 0.0, false, false).unwrap();
        assert!(
            p.insert_representation(Representation::new(p.id(), page, target, transform))
                .is_err()
        );
        assert!(p.representations_for_entity(target).is_err());
        assert_eq!(p, before);
    }
}

#[test]
fn extreme_union_returns_error_without_invalidating_individual_representations() {
    let mut p = populated();
    let page = page(&mut p);
    let target = RepresentationTarget::Device(p.devices().next().unwrap().id());
    for x in [-f64::MAX, f64::MAX] {
        let transform = Transform2D::new(Point2D::new(x, 0.0).unwrap(), 0.0, false, false).unwrap();
        p.insert_representation(Representation::new(p.id(), page, target, transform))
            .unwrap();
    }
    assert!(p.document_bounds(page).is_err());
    p.validate().unwrap();
}
