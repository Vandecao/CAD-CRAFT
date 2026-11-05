// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
mod physical_common;
use cad_craft::domain::*;
use physical_common::*;
use serde_json::{Value, json};

#[test]
fn full_round_trip_preserves_ids_associations_unused_cores_and_rebuilds_indexes() {
    let p = populated();
    let bytes = serde_json::to_vec(&p).unwrap();
    let mut loaded: Project = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(loaded, p);
    assert_eq!(serde_json::to_vec(&loaded).unwrap(), bytes);
    loaded.validate().unwrap();
    let core = loaded
        .cable_cores()
        .find(|c| c.conductor_id().is_some())
        .unwrap()
        .id();
    let w = loaded.unassign_core_conductor(core).unwrap().unwrap();
    loaded.assign_core_conductor(core, w).unwrap();
    let link = loaded.unassign_conductor_connection(w).unwrap().unwrap();
    loaded.assign_conductor_connection(w, link).unwrap();
    assert_eq!(loaded, p);
    let data = json!(loaded);
    assert!(data.get("physical_index").is_none());
}

#[test]
fn earlier_stage_snapshots_without_physical_fields_remain_readable() {
    let (p, _, _, points) = fixture(2);
    let mut stage2 = p.clone();
    connection(&mut stage2, points[0], points[1]);
    for project in [p, stage2] {
        let mut data = json!(project);
        for key in ["conductors", "cables", "cable_cores", "terminal_strips"] {
            data.as_object_mut().unwrap().remove(key);
        }
        for terminal in data["terminals"].as_array_mut().unwrap() {
            terminal.as_object_mut().unwrap().remove("positions");
        }
        if project.connections().count() == 0 {
            data.as_object_mut().unwrap().remove("connections");
        }
        let loaded: Project = serde_json::from_value(data).unwrap();
        assert_eq!(loaded.networks(), project.networks());
        assert_eq!(loaded.conductors().count(), 0);
        assert_eq!(loaded.terminals().next().unwrap().positions().count(), 0);
        loaded.validate().unwrap();
    }
}

#[test]
fn physical_snapshot_rejects_duplicate_ids_foreign_projects_and_null_registries() {
    let data = json!(populated());
    for key in ["conductors", "cables", "cable_cores", "terminal_strips"] {
        let mut duplicate = data.clone();
        let first = duplicate[key][0].clone();
        duplicate[key].as_array_mut().unwrap().push(first);
        assert!(
            serde_json::from_value::<Project>(duplicate).is_err(),
            "duplicate {key}"
        );
        let mut foreign = data.clone();
        foreign[key][0]["project_id"] = json!(ProjectId::new());
        assert!(
            serde_json::from_value::<Project>(foreign).is_err(),
            "foreign {key}"
        );
        let mut null = data.clone();
        null[key] = Value::Null;
        assert!(
            serde_json::from_value::<Project>(null).is_err(),
            "null {key}"
        );
    }
}

#[test]
fn physical_snapshot_rejects_dangling_and_incompatible_relations() {
    let data = json!(populated());
    let mutations = [
        ("/conductors/0/source/terminal_id", json!(TerminalId::new())),
        ("/conductors/0/source/position", json!("missing")),
        (
            "/conductors/0/connection/connection_id",
            json!(ConnectionId::new()),
        ),
        (
            "/conductors/0/connection/source_point",
            json!(ConnectionPointId::new()),
        ),
        (
            "/conductors/0/connection/source_point",
            data["connection_points"][3]["id"].clone(),
        ),
        ("/cable_cores/0/cable_id", json!(CableId::new())),
        ("/cable_cores/0/conductor_id", json!(ConductorId::new())),
        ("/cable_cores/0/cross_section", json!(2.5)),
        ("/cable_cores/0/marking/color", json!("Blue")),
        ("/terminal_strips/0/terminals/0", json!(TerminalId::new())),
        ("/cables/0/nominal_core_count", json!(2)),
    ];
    for (path, value) in mutations {
        let mut damaged = data.clone();
        *damaged.pointer_mut(path).unwrap() = value;
        assert!(
            serde_json::from_value::<Project>(damaged).is_err(),
            "accepted {path}"
        );
    }
}

#[test]
fn physical_snapshot_rejects_duplicate_associations_and_parallel_identifications() {
    let data = json!(populated());
    for (path, value) in [
        (
            "/cable_cores/1/identification",
            data["cable_cores"][0]["identification"].clone(),
        ),
        (
            "/cable_cores/1/conductor_id",
            data["cable_cores"][0]["conductor_id"].clone(),
        ),
        (
            "/terminal_strips/0/terminals/1",
            data["terminal_strips"][0]["terminals"][0].clone(),
        ),
    ] {
        let mut damaged = data.clone();
        *damaged.pointer_mut(path).unwrap() = value;
        assert!(
            serde_json::from_value::<Project>(damaged).is_err(),
            "accepted {path}"
        );
    }
    let mut damaged = data.clone();
    let mut second = damaged["terminal_strips"][0].clone();
    second["id"] = json!(TerminalStripId::new());
    damaged["terminal_strips"]
        .as_array_mut()
        .unwrap()
        .push(second);
    assert!(serde_json::from_value::<Project>(damaged).is_err());
    let mut damaged = data.clone();
    let mut duplicate = data["conductors"][0].clone();
    duplicate["id"] = json!(ConductorId::new());
    duplicate["source"] = data["conductors"][0]["target"].clone();
    duplicate["target"] = data["conductors"][0]["source"].clone();
    duplicate["connection"] = Value::Null;
    damaged["conductors"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert!(serde_json::from_value::<Project>(damaged).is_err());
}

#[test]
fn malformed_physical_values_and_unknown_fields_cannot_bypass_validation() {
    let data = json!(populated());
    for (path, value) in [
        ("/conductors/0/cross_section", json!(0)),
        ("/conductors/0/length", json!(-1)),
        ("/conductors/0/designation", json!(" ")),
        (
            "/conductors/0/marking",
            json!({"color":null, "identification":null}),
        ),
        (
            "/conductors/0/target",
            data["conductors"][0]["source"].clone(),
        ),
        ("/cables/0/nominal_core_count", json!(0)),
        ("/cable_cores/0/cross_section", json!(-1)),
        ("/terminal_strips/0/designation", json!("")),
    ] {
        let mut damaged = data.clone();
        *damaged.pointer_mut(path).unwrap() = value;
        assert!(
            serde_json::from_value::<Project>(damaged).is_err(),
            "accepted {path}"
        );
    }
    for key in ["conductors", "cables", "cable_cores", "terminal_strips"] {
        let mut damaged = data.clone();
        damaged[key][0]["unknown"] = json!(true);
        assert!(serde_json::from_value::<Project>(damaged).is_err());
    }
}

#[test]
fn physical_registry_order_is_canonical_but_strip_order_is_preserved() {
    let p = populated();
    let bytes = serde_json::to_vec(&p).unwrap();
    let mut data = json!(p);
    for key in [
        "conductors",
        "cables",
        "cable_cores",
        "terminal_strips",
        "terminals",
    ] {
        data[key].as_array_mut().unwrap().reverse();
    }
    let loaded: Project = serde_json::from_value(data).unwrap();
    assert_eq!(loaded, p);
    assert_eq!(serde_json::to_vec(&loaded).unwrap(), bytes);
}

#[test]
fn physical_properties_resolve_forward_cycles_but_reject_dangling_references() {
    let mut p = populated();
    let cable = p.cables().next().unwrap().id();
    let w = p.conductors().next().unwrap().id();
    for (target, other) in [
        (EntityRef::Cable(cable), EntityRef::Conductor(w)),
        (EntityRef::Conductor(w), EntityRef::Cable(cable)),
    ] {
        let mut props = TechnicalProperties::default();
        props
            .set_custom("related", PropertyValue::Reference(other))
            .unwrap();
        p.set_properties(target, props).unwrap();
    }
    assert_eq!(serde_json::from_value::<Project>(json!(p)).unwrap(), p);
    for key in ["conductors", "cables", "cable_cores", "terminal_strips"] {
        let mut data = json!(p);
        data[key][0]["properties"]["custom"]["missing"] =
            json!({"Reference":{"Cable":CableId::new()}});
        assert!(serde_json::from_value::<Project>(data).is_err());
    }
    for key in ["conductors", "cable_cores"] {
        let mut data = json!(p);
        data[key][0]["properties"]["known"]["CrossSection"] = json!(1.5);
        assert!(serde_json::from_value::<Project>(data).is_err());
    }
}
