// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
mod common;
use cad_craft::domain::*;
use common::{connect, fixture};
use serde_json::json;

#[test]
fn connection_round_trip_normalizes_reversed_endpoints_without_changing_id() {
    let (p, ids) = fixture(2);
    let c = Connection::new(p.id(), ids[0], ids[1]).unwrap();
    let mut value = serde_json::to_value(&c).unwrap();
    value["source"] = json!(c.endpoints().1);
    value["target"] = json!(c.endpoints().0);
    assert_eq!(serde_json::from_value::<Connection>(value).unwrap(), c);
}

#[test]
fn project_snapshot_preserves_connections_and_rebuilds_mutable_index() {
    let (mut p, ids) = fixture(5);
    let edge = connect(&mut p, ids[0], ids[1]);
    connect(&mut p, ids[2], ids[3]);
    let bytes = serde_json::to_vec(&p).unwrap();
    let mut loaded: Project = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(loaded, p);
    assert_eq!(serde_json::to_vec(&loaded).unwrap(), bytes);
    loaded.validate().unwrap();
    let c = loaded.remove_connection(edge).unwrap();
    assert_eq!(loaded.insert_connection(c).unwrap(), edge);
    assert!(
        loaded
            .insert_connection(Connection::new(loaded.id(), ids[1], ids[0]).unwrap())
            .is_err()
    );
    assert_eq!(loaded, p);
    let value = serde_json::to_value(p).unwrap();
    assert!(value.get("networks").is_none());
    assert!(value.get("graph").is_none());
}

#[test]
fn earlier_snapshots_without_connections_load_as_empty_graphs() {
    let (p, _) = fixture(2);
    let mut data = serde_json::to_value(&p).unwrap();
    data.as_object_mut().unwrap().remove("connections");
    assert_eq!(serde_json::from_value::<Project>(data).unwrap(), p);
}

#[test]
fn malformed_connections_cannot_bypass_local_or_project_validation() {
    let (mut p, ids) = fixture(2);
    connect(&mut p, ids[0], ids[1]);
    let data = serde_json::to_value(&p).unwrap();
    for (field, value) in [
        ("id", json!("00000000-0000-0000-0000-000000000000")),
        ("project_id", json!(ProjectId::new())),
        ("source", json!(ConnectionPointId::new())),
        ("target", json!(ConnectionPointId::new())),
        ("target", data["connections"][0]["source"].clone()),
        ("unknown", json!(true)),
    ] {
        let mut damaged = data.clone();
        damaged["connections"][0][field] = value;
        assert!(
            serde_json::from_value::<Project>(damaged).is_err(),
            "accepted {field}"
        );
    }
    for field in ["id", "project_id", "source", "target"] {
        let mut damaged = data.clone();
        damaged["connections"][0]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(serde_json::from_value::<Project>(damaged).is_err());
    }
    let mut damaged = data.clone();
    damaged["connections"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<Project>(damaged).is_err());
    let mut local = data["connections"][0].clone();
    local["target"] = local["source"].clone();
    assert!(serde_json::from_value::<Connection>(local).is_err());
}

#[test]
fn duplicate_snapshot_ids_and_reversed_pairs_are_rejected() {
    let (mut p, ids) = fixture(3);
    connect(&mut p, ids[0], ids[1]);
    let data = serde_json::to_value(p).unwrap();
    for new_id in [false, true] {
        let mut damaged = data.clone();
        let mut duplicate = damaged["connections"][0].clone();
        if new_id {
            duplicate["id"] = json!(ConnectionId::new());
            duplicate["source"] = data["connections"][0]["target"].clone();
            duplicate["target"] = data["connections"][0]["source"].clone();
        }
        damaged["connections"]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
        assert!(serde_json::from_value::<Project>(damaged).is_err());
    }
}

#[test]
fn snapshot_input_order_does_not_change_canonical_output() {
    let (mut p, ids) = fixture(4);
    for pair in ids.windows(2) {
        connect(&mut p, pair[0], pair[1]);
    }
    let bytes = serde_json::to_vec(&p).unwrap();
    let mut data = serde_json::to_value(&p).unwrap();
    for key in ["connections", "connection_points", "functions", "devices"] {
        data[key].as_array_mut().unwrap().reverse();
    }
    let loaded: Project = serde_json::from_value(data).unwrap();
    assert_eq!(loaded, p);
    assert_eq!(serde_json::to_vec(&loaded).unwrap(), bytes);
}
