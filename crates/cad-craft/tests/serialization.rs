// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

use cad_craft::domain::*;
use serde_json::{Value, json};

fn complete_project() -> Project {
    let mut p = Project::new("Comando elétrico").unwrap();
    p.set_description("Contator com funções separadas");
    p.set_metadata("author", "Engenharia").unwrap();
    let d = p
        .insert_device(Device::new(p.id(), Designation::new("K1").unwrap(), "contactor").unwrap())
        .unwrap();
    let f = p
        .insert_function(Function::new(
            p.id(),
            d,
            Designation::new("K1")
                .unwrap()
                .with_location_path(vec![DesignationPart::new("P1").unwrap()])
                .with_functional_path(vec![DesignationPart::new("Drive").unwrap()]),
            FunctionKind::Coil,
        ))
        .unwrap();
    p.insert_function(Function::new(
        p.id(),
        d,
        Designation::new("K1").unwrap(),
        FunctionKind::AuxiliaryContact,
    ))
    .unwrap();
    let c = p
        .insert_component(
            Component::new(p.id(), Designation::new("K1").unwrap(), "contactor")
                .unwrap()
                .with_catalog("Fabricante exemplo", "CAT-123")
                .unwrap()
                .with_assignment(Some(d), Some(f)),
        )
        .unwrap();
    let t = p
        .insert_terminal(Terminal::new(
            p.id(),
            TerminalOwner::Component(c),
            DesignationPart::new("A1").unwrap(),
        ))
        .unwrap();
    let cp = p
        .insert_connection_point(
            ConnectionPoint::new(p.id(), f, DesignationPart::new("1").unwrap()).with_terminal(t),
        )
        .unwrap();
    let mut doc = Document::new(p.id(), "Comando").unwrap();
    doc.set_metadata("revision", "A").unwrap();
    let doc = p.insert_document(doc).unwrap();
    let mut page = Page::new(
        p.id(),
        doc,
        "Bobina",
        "1",
        PageKind::MultiLine,
        Size2D::new(420.0, 297.0).unwrap(),
    )
    .unwrap();
    page.set_metadata("revision", "B").unwrap();
    let page = p.insert_page(page).unwrap();
    p.add_page_element(page, PageElement::Function(f)).unwrap();
    p.add_page_element(page, PageElement::Component(c)).unwrap();
    let targets = [
        EntityRef::Project(p.id()),
        EntityRef::Document(doc),
        EntityRef::Page(page),
        EntityRef::Device(d),
        EntityRef::Function(f),
        EntityRef::Component(c),
        EntityRef::Terminal(t),
        EntityRef::ConnectionPoint(cp),
    ];
    let mut props = TechnicalProperties::default();
    props.set_known(KnownProperty::RatedVoltage, 230.0).unwrap();
    props
        .set_custom(
            "supply",
            PropertyValue::Enumeration(EnumValue::new("supply", ["AC", "DC"], "AC").unwrap()),
        )
        .unwrap();
    props
        .set_custom("note", PropertyValue::Text("símbolo futuro".into()))
        .unwrap();
    props
        .set_custom("poles", PropertyValue::Integer(3))
        .unwrap();
    props
        .set_custom(
            "offset",
            PropertyValue::Decimal(Decimal::new(-1.25).unwrap()),
        )
        .unwrap();
    props
        .set_custom("enabled", PropertyValue::Boolean(true))
        .unwrap();
    for (i, target) in targets.iter().enumerate() {
        props
            .set_custom(format!("ref_{i}"), PropertyValue::Reference(*target))
            .unwrap();
    }
    for target in targets {
        p.set_properties(target, props.clone()).unwrap();
    }
    p
}

#[test]
fn complete_project_round_trip_preserves_identity_relationships_and_properties() {
    let p = complete_project();
    let bytes = serde_json::to_vec(&p).unwrap();
    let loaded: Project = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(loaded, p);
    loaded.validate().unwrap();
    assert_eq!(loaded.devices().count(), 1);
    assert_eq!(loaded.functions().count(), 2);
    assert_eq!(
        loaded.pages().next().unwrap().size(),
        Size2D::new(420.0, 297.0).unwrap()
    );
    assert_eq!(
        loaded
            .components()
            .next()
            .unwrap()
            .catalog_code()
            .unwrap()
            .as_str(),
        "CAT-123"
    );
}

#[test]
fn import_resolves_forward_and_cyclic_property_references_after_collecting_entities() {
    let p = complete_project();
    let mut data = serde_json::to_value(&p).unwrap();
    data["functions"].as_array_mut().unwrap().reverse();
    let loaded: Project = serde_json::from_value(data).unwrap();
    assert_eq!(loaded, p);
}

#[test]
fn duplicate_ids_in_any_registry_are_rejected_during_import() {
    let data = serde_json::to_value(complete_project()).unwrap();
    for key in [
        "documents",
        "pages",
        "devices",
        "functions",
        "components",
        "terminals",
        "connection_points",
    ] {
        let mut damaged = data.clone();
        let entries = damaged[key].as_array_mut().unwrap();
        entries.push(entries[0].clone());
        assert!(
            serde_json::from_value::<Project>(damaged).is_err(),
            "accepted duplicate in {key}"
        );
    }
}

#[test]
fn foreign_project_ids_in_any_entity_are_rejected_during_import() {
    let data = serde_json::to_value(complete_project()).unwrap();
    for key in [
        "documents",
        "pages",
        "devices",
        "functions",
        "components",
        "terminals",
        "connection_points",
    ] {
        let mut damaged = data.clone();
        damaged[key][0]["project_id"] = json!(ProjectId::new());
        assert!(
            serde_json::from_value::<Project>(damaged).is_err(),
            "accepted foreign project in {key}"
        );
    }
}

#[test]
fn missing_structural_references_are_rejected_during_import() {
    let data = serde_json::to_value(complete_project()).unwrap();
    let mutations = [
        ("pages", "document_id", json!(DocumentId::new())),
        ("functions", "device_id", json!(DeviceId::new())),
        ("components", "device_id", json!(DeviceId::new())),
        ("components", "function_id", json!(FunctionId::new())),
        ("terminals", "owner", json!({"Device": DeviceId::new()})),
        (
            "terminals",
            "owner",
            json!({"Component": ComponentId::new()}),
        ),
        ("connection_points", "function_id", json!(FunctionId::new())),
        ("connection_points", "terminal_id", json!(TerminalId::new())),
    ];
    for (key, field, value) in mutations {
        let mut damaged = data.clone();
        damaged[key][0][field] = value;
        assert!(
            serde_json::from_value::<Project>(damaged).is_err(),
            "accepted {key}.{field}"
        );
    }
}

#[test]
fn invalid_page_elements_and_duplicate_references_are_rejected() {
    let data = serde_json::to_value(complete_project()).unwrap();
    for element in [
        json!({"Component": ComponentId::new()}),
        json!({"Function": FunctionId::new()}),
        data["pages"][0]["elements"][0].clone(),
    ] {
        let mut damaged = data.clone();
        damaged["pages"][0]["elements"]
            .as_array_mut()
            .unwrap()
            .push(element);
        assert!(serde_json::from_value::<Project>(damaged).is_err());
    }
}

#[test]
fn dangling_property_references_are_rejected_on_all_entity_types() {
    let data = serde_json::to_value(complete_project()).unwrap();
    for key in [
        "documents",
        "pages",
        "devices",
        "functions",
        "components",
        "terminals",
        "connection_points",
    ] {
        let mut damaged = data.clone();
        damaged[key][0]["properties"]["custom"]["bad"] =
            json!({"Reference": {"Device": DeviceId::new()}});
        assert!(serde_json::from_value::<Project>(damaged).is_err());
    }
    let mut damaged = data;
    damaged["properties"]["custom"]["bad"] = json!({"Reference": {"Project": ProjectId::new()}});
    assert!(serde_json::from_value::<Project>(damaged).is_err());
}

#[test]
fn malformed_values_unknown_fields_and_missing_required_fields_are_rejected() {
    let data = serde_json::to_value(complete_project()).unwrap();
    for (pointer, value) in [
        ("/id", json!("00000000-0000-0000-0000-000000000000")),
        ("/name", json!(" ")),
        ("/pages/0/size/width", json!(-1.0)),
        ("/pages/0/number", json!("")),
        ("/devices/0/designation/device", json!("-K1")),
        ("/devices/0/classification", json!("")),
        ("/components/0/manufacturer", json!("")),
        ("/terminals/0/designation", json!("A 1")),
    ] {
        let mut damaged = data.clone();
        *damaged.pointer_mut(pointer).unwrap() = value;
        assert!(
            serde_json::from_value::<Project>(damaged).is_err(),
            "accepted {pointer}"
        );
    }
    let mut unknown = data.clone();
    unknown["unexpected"] = Value::Bool(true);
    assert!(serde_json::from_value::<Project>(unknown).is_err());
    let mut missing = data;
    missing.as_object_mut().unwrap().remove("devices");
    assert!(serde_json::from_value::<Project>(missing).is_err());
}

#[test]
fn incompatible_relations_cannot_be_imported_even_when_ids_exist() {
    let mut data = serde_json::to_value(complete_project()).unwrap();
    data["components"][0]["device_id"] = Value::Null;
    assert!(serde_json::from_value::<Project>(data).is_err());
    let mut data = serde_json::to_value(complete_project()).unwrap();
    let point_function = data["connection_points"][0]["function_id"].clone();
    let other = data["functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["id"] != point_function)
        .unwrap()["id"]
        .clone();
    data["connection_points"][0]["function_id"] = other;
    assert!(serde_json::from_value::<Project>(data).is_err());
}

#[test]
fn isolated_entity_round_trips_keep_valid_local_values() {
    let p = complete_project();
    macro_rules! round_trip {
        ($value:expr, $ty:ty) => {{
            let entity = $value;
            assert_eq!(
                &serde_json::from_value::<$ty>(serde_json::to_value(entity).unwrap()).unwrap(),
                entity
            );
        }};
    }
    round_trip!(p.documents().next().unwrap(), Document);
    round_trip!(p.pages().next().unwrap(), Page);
    round_trip!(p.devices().next().unwrap(), Device);
    round_trip!(p.functions().next().unwrap(), Function);
    round_trip!(p.components().next().unwrap(), Component);
    round_trip!(p.terminals().next().unwrap(), Terminal);
    round_trip!(p.connection_points().next().unwrap(), ConnectionPoint);
}
