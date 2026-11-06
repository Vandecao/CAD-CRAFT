// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

mod document_common;
use cad_craft::domain::*;
use document_common::*;
use serde_json::{Value, json};

#[test]
fn full_transform_visual_and_bounds_survive_snapshot_and_reject_overflow() {
    let (mut p, a, _, d, _) = fixture();
    let transform =
        Transform2D::new(Point2D::new(-50.0, 20.0).unwrap(), 135.0, true, true).unwrap();
    let local = Rect2D::new(
        Point2D::new(-2.0, -3.0).unwrap(),
        Size2D::new(5.0, 7.0).unwrap(),
    )
    .unwrap();
    let r = Representation::new(p.id(), a, RepresentationTarget::Device(d), transform)
        .with_local_bounds(local)
        .unwrap()
        .with_visual(VisualProperties {
            visible: false,
            layer: -4,
            label: Some(Text::new("Rótulo").unwrap()),
        });
    p.insert_representation(r).unwrap();
    let data = json!(p);
    let restored: Project = serde_json::from_value(data.clone()).unwrap();
    assert_eq!(restored, p);
    let mut bad = data;
    bad["representations"][0]["transform"] = json!({"translation":{"x":f64::MAX,"y":0},"rotation_degrees":0,"mirror_x":false,"mirror_y":false});
    bad["representations"][0]["local_bounds"] =
        json!({"origin":{"x":f64::MAX,"y":0},"size":{"width":0,"height":0}});
    assert!(serde_json::from_value::<Project>(bad.clone()).is_err());
    assert!(serde_json::from_value::<Representation>(bad["representations"][0].clone()).is_err());
}

#[test]
fn old_project_with_no_documents_stays_empty_and_null_registry_is_rejected() {
    let mut v = json!(Project::new("old").unwrap());
    v.as_object_mut().unwrap().remove("representations");
    v.as_object_mut().unwrap().remove("identification");
    let p: Project = serde_json::from_value(v.clone()).unwrap();
    assert_eq!(p.documents().count(), 0);
    assert_eq!(p.pages().count(), 0);
    assert_eq!(p.representations().count(), 0);
    v["representations"] = Value::Null;
    assert!(serde_json::from_value::<Project>(v).is_err());
}

fn populated() -> Project {
    let (mut p, a, b, d, c) = fixture();
    p.set_identification(Some(Text::new("P4").unwrap()));
    p.set_page_structure(
        a,
        DocumentStructure::new(vec![DesignationPart::new("Drive").unwrap()], vec![]),
    )
    .unwrap();
    p.reorder_pages(p.page(a).unwrap().document_id(), &[b, a])
        .unwrap();
    p.insert_representation(rep(&p, a, RepresentationTarget::Device(d)))
        .unwrap();
    p.insert_representation(rep(&p, b, RepresentationTarget::Connection(c)))
        .unwrap();
    p
}

#[test]
fn roundtrip_preserves_ids_order_structure_and_derived_queries() {
    let p = populated();
    let bytes = serde_json::to_string(&p).unwrap();
    let restored: Project = serde_json::from_str(&bytes).unwrap();
    assert_eq!(p, restored);
    restored.validate().unwrap();
    assert_eq!(serde_json::to_string(&restored).unwrap(), bytes);
    let mut value = serde_json::to_value(&p).unwrap();
    for key in ["pages", "representations", "documents"] {
        value[key].as_array_mut().unwrap().reverse();
    }
    assert_eq!(serde_json::from_value::<Project>(value).unwrap(), p);
}

#[test]
fn legacy_snapshots_do_not_invent_documentary_content() {
    let (p, _, _, _, _) = fixture();
    let mut v = serde_json::to_value(&p).unwrap();
    v.as_object_mut().unwrap().remove("representations");
    v.as_object_mut().unwrap().remove("identification");
    for page in v["pages"].as_array_mut().unwrap() {
        page.as_object_mut().unwrap().remove("order");
        page.as_object_mut().unwrap().remove("structure");
    }
    for schema_level in [3, 2, 1] {
        if schema_level == 2 {
            for key in ["conductors", "cables", "cable_cores", "terminal_strips"] {
                v.as_object_mut().unwrap().remove(key);
            }
        }
        if schema_level == 1 {
            v.as_object_mut().unwrap().remove("connections");
        }
        let restored: Project = serde_json::from_value(v.clone()).unwrap();
        assert!(restored.identification().is_none());
        assert_eq!(restored.representations().count(), 0);
        assert!(restored.pages().all(|p| p.order().is_none()
            && p.structure().functional_path().is_empty()
            && p.structure().location_path().is_empty()));
        assert_eq!(restored.pages().count(), 2);
        restored.validate().unwrap();
    }
}

#[test]
fn corrupt_representation_snapshots_are_rejected() {
    let good = serde_json::to_value(populated()).unwrap();
    let mut cases: Vec<Value> = vec![];
    let mut v = good.clone();
    let duplicate = v["representations"][0].clone();
    v["representations"].as_array_mut().unwrap().push(duplicate);
    cases.push(v);
    let mut v = good.clone();
    v["representations"][0]["page_id"] = json!(PageId::new());
    cases.push(v);
    let mut v = good.clone();
    v["representations"][0]["project_id"] = json!(ProjectId::new());
    cases.push(v);
    let mut v = good.clone();
    v["representations"][0]["target"] = json!({"Connection":ConnectionId::new()});
    cases.push(v);
    let mut v = good.clone();
    v["representations"][0]["transform"]["rotation_degrees"] = json!("NaN");
    cases.push(v);
    let mut v = good.clone();
    v["representations"][0]["id"] = json!("00000000-0000-0000-0000-000000000000");
    cases.push(v);
    let mut v = good.clone();
    v["pages"][0]["structure"]["functional_path"] = json!(["bad/path"]);
    cases.push(v);
    let mut v = good;
    v["representations"][0]["unexpected"] = json!(true);
    cases.push(v);
    for (i, v) in cases.into_iter().enumerate() {
        assert!(serde_json::from_value::<Project>(v).is_err(), "case {i}");
    }
}
