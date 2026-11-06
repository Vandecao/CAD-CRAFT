// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

mod physical_common;
use cad_craft::domain::*;
use serde_json::json;
fn pt(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}
fn tr(a: f64) -> Transform2D {
    Transform2D::new(pt(-10., 20.), a, false, false).unwrap()
}
fn sheet(p: &mut Project) -> PageId {
    let d = p
        .insert_document(Document::new(p.id(), "D").unwrap())
        .unwrap();
    p.insert_page(
        Page::new(
            p.id(),
            d,
            "P",
            "1",
            PageKind::MultiLine,
            Size2D::new(297., 210.).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}
fn def() -> SymbolDefinition {
    SymbolLibrary::builtin()
        .definitions()
        .next()
        .unwrap()
        .clone()
}
#[test]
fn insertion_and_undo_restore_ids_definition_registry_and_indices() {
    let mut p = Project::new("P").unwrap();
    let page = sheet(&mut p);
    let d = def();
    let before = p.clone();
    let r = Representation::new_symbol(p.id(), page, &d, None, tr(0.)).unwrap();
    let id = r.id();
    p.insert_symbol_representation(r.clone(), d.clone())
        .unwrap();
    assert_eq!(p.symbol_definition(d.id()), Some(&d));
    assert_eq!(p.representations_on_page(page).unwrap()[0].id(), id);
    assert!(p.related_representations(id).unwrap().is_empty());
    assert!(p.remove_unused_symbol_definition(d.id()).is_err());
    p.remove_representation(id).unwrap();
    assert_eq!(
        p.remove_unused_symbol_definition(d.id()).unwrap(),
        Some(d.clone())
    );
    assert_eq!(p, before);
    p.insert_symbol_representation(r, d).unwrap();
    p.validate().unwrap();
    assert_eq!(p.representation(id).unwrap().id(), id);
}
#[test]
fn insertion_failure_has_no_definition_residue_and_plain_unassociated_is_rejected() {
    let mut p = Project::new("P").unwrap();
    let page = sheet(&mut p);
    let d = def();
    let before = p.clone();
    let r = Representation::new_symbol(p.id(), PageId::new(), &d, None, tr(0.)).unwrap();
    assert!(p.insert_symbol_representation(r, d).is_err());
    assert_eq!(p, before);
    assert!(
        p.insert_representation(Representation::new(
            p.id(),
            page,
            RepresentationTarget::Unassociated,
            tr(0.)
        ))
        .is_err()
    );
    assert_eq!(p, before);
}
#[test]
fn compatible_old_snapshot_and_symbol_roundtrip_are_deterministic() {
    let mut p = Project::new("P").unwrap();
    let page = sheet(&mut p);
    let mut old = json!(p);
    old.as_object_mut().unwrap().remove("symbol_definitions");
    assert_eq!(serde_json::from_value::<Project>(old).unwrap(), p);
    let d = def();
    let r = Representation::new_symbol(p.id(), page, &d, None, tr(90.)).unwrap();
    p.insert_symbol_representation(r, d).unwrap();
    let bytes = serde_json::to_string(&p).unwrap();
    let q: Project = serde_json::from_str(&bytes).unwrap();
    assert_eq!(p, q);
    assert_eq!(bytes, serde_json::to_string(&q).unwrap());
    let mut bad = json!(p);
    bad["symbol_definitions"] = json!([]);
    assert!(serde_json::from_value::<Project>(bad).is_err());
}
#[test]
fn incompatible_definition_version_bounds_and_orientation_are_rejected_atomically() {
    let mut p = Project::new("P").unwrap();
    let page = sheet(&mut p);
    let d = SymbolDefinition::new(
        SymbolDefinitionId::new("limited").unwrap(),
        1,
        "Limited",
        "Test",
        vec![SymbolPrimitive::Line {
            start: pt(0., 0.),
            end: pt(5., 0.),
        }],
        vec![],
    )
    .unwrap()
    .with_orientations(SymbolOrientations::new(vec![0., 90.], false, false).unwrap());
    let r = Representation::new_symbol(p.id(), page, &d, None, tr(0.)).unwrap();
    let id = r.id();
    p.insert_symbol_representation(r, d.clone()).unwrap();
    let before = p.clone();
    assert!(p.transform_representations(&[(id, tr(45.))]).is_err());
    assert_eq!(p, before);
    assert!(p.set_representation_bounds(id, None).is_err());
    assert_eq!(p, before);
    let mut v = json!(d);
    v["version"] = json!(2);
    let newer: SymbolDefinition = serde_json::from_value(v).unwrap();
    let r = Representation::new_symbol(p.id(), page, &newer, None, tr(0.)).unwrap();
    assert!(p.insert_symbol_representation(r, newer).is_err());
    assert_eq!(p, before);
}
#[test]
fn graphical_ports_coincidence_and_association_preserve_actual_wiring() {
    let mut p = physical_common::populated();
    let page = sheet(&mut p);
    let before = json!(p);
    let networks = p.networks();
    let d = def();
    let target = RepresentationTarget::Device(p.devices().next().unwrap().id());
    for association in [None, None, Some(target)] {
        let r = Representation::new_symbol(p.id(), page, &d, association, tr(0.)).unwrap();
        p.insert_symbol_representation(r, d.clone()).unwrap();
    }
    assert_eq!(p.representations_for_entity(target).unwrap().len(), 1);
    let unassociated: Vec<_> = p
        .representations()
        .filter(|r| r.target() == RepresentationTarget::Unassociated)
        .collect();
    for r in unassociated {
        assert!(p.related_representations(r.id()).unwrap().is_empty());
    }
    let after = json!(p);
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
    assert_eq!(p.networks(), networks);
    p.validate().unwrap();
}
#[test]
fn tampered_snapshots_reject_symbol_bounds_and_zero_version() {
    let mut p = Project::new("P").unwrap();
    let page = sheet(&mut p);
    let d = def();
    let r = Representation::new_symbol(p.id(), page, &d, None, tr(0.)).unwrap();
    p.insert_symbol_representation(r, d).unwrap();
    let mut bad = json!(p);
    bad["representations"][0]["local_bounds"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<Project>(bad).is_err());
    let mut bad = json!(p);
    bad["symbol_definitions"][0]["version"] = json!(0);
    assert!(serde_json::from_value::<Project>(bad).is_err());
}

#[test]
fn missing_electrical_target_and_duplicate_definitions_are_rejected() {
    let mut p = Project::new("P").unwrap();
    let page = sheet(&mut p);
    let d = def();
    let before = p.clone();
    let r = Representation::new_symbol(
        p.id(),
        page,
        &d,
        Some(RepresentationTarget::Device(DeviceId::new())),
        tr(0.),
    )
    .unwrap();
    assert!(p.insert_symbol_representation(r, d.clone()).is_err());
    assert_eq!(p, before);
    let r = Representation::new_symbol(p.id(), page, &d, None, tr(0.)).unwrap();
    p.insert_symbol_representation(r, d).unwrap();
    let mut bad = json!(p);
    let duplicate = bad["symbol_definitions"][0].clone();
    bad["symbol_definitions"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert!(serde_json::from_value::<Project>(bad).is_err());
}
#[test]
fn last_symbol_disallowed_orientation_keeps_mixed_transform_batch_atomic() {
    let mut p = Project::new("P").unwrap();
    let page = sheet(&mut p);
    let a = def();
    let b = SymbolDefinition::new(
        SymbolDefinitionId::new("fixed").unwrap(),
        1,
        "Fixed",
        "Test",
        vec![SymbolPrimitive::Anchor {
            position: pt(0., 0.),
        }],
        vec![],
    )
    .unwrap()
    .with_orientations(SymbolOrientations::new(vec![0.], false, false).unwrap());
    let ra = Representation::new_symbol(p.id(), page, &a, None, tr(0.)).unwrap();
    let rb = Representation::new_symbol(p.id(), page, &b, None, tr(0.)).unwrap();
    let ia = ra.id();
    let ib = rb.id();
    p.insert_symbol_representation(ra, a).unwrap();
    p.insert_symbol_representation(rb, b).unwrap();
    let before = p.clone();
    assert!(
        p.transform_representations(&[(ia, tr(90.)), (ib, tr(90.))])
            .is_err()
    );
    assert_eq!(p, before);
}
