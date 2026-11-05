// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
mod physical_common;
use cad_craft::{domain::*, error::DomainError};
use physical_common::*;
use serde_json::json;

#[test]
fn detached_strip_with_late_conflict_never_claims_earlier_free_terminal() {
    let (mut p, _, ends, _) = fixture(2);
    let occupied = p
        .insert_terminal_strip(TerminalStrip::new(p.id(), part("X1")))
        .unwrap();
    p.attach_terminal(occupied, ends[1].terminal_id()).unwrap();
    let mut data = json!(TerminalStrip::new(p.id(), part("X2")));
    data["terminals"] = json!([ends[0].terminal_id(), ends[1].terminal_id()]);
    let detached: TerminalStrip = serde_json::from_value(data).unwrap();
    let before = p.clone();
    assert!(p.insert_terminal_strip(detached.clone()).is_err());
    assert_eq!(p, before);
    assert_eq!(p.strip_for_terminal(ends[0].terminal_id()), None);
    let mut valid = json!(detached);
    valid["terminals"] = json!([ends[0].terminal_id()]);
    let id = p
        .insert_terminal_strip(serde_json::from_value(valid).unwrap())
        .unwrap();
    assert_eq!(
        p.strip_for_terminal(ends[0].terminal_id()).unwrap().id(),
        id
    );
    p.validate().unwrap();
}

#[test]
fn detached_core_cannot_take_conductor_from_another_cable_or_reserve_capacity_on_failure() {
    let (mut p, _, ends, _) = fixture(2);
    let first = cable(&mut p, 1);
    let second = cable(&mut p, 1);
    let original = core(&mut p, first, "1");
    let wire = p
        .insert_conductor(wire(&p, "W1", &ends[0], &ends[1]))
        .unwrap();
    p.assign_core_conductor(original, wire).unwrap();
    let mut data = json!(CableCore::new(
        p.id(),
        second,
        part("1"),
        ConductorMarking::identified(part("1"))
    ));
    data["conductor_id"] = json!(wire);
    let detached: CableCore = serde_json::from_value(data).unwrap();
    let before = p.clone();
    assert!(p.insert_cable_core(detached.clone()).is_err());
    assert_eq!(p, before);
    assert_eq!(p.cores_for_cable(second).count(), 0);
    p.unassign_core_conductor(original).unwrap();
    let id = p.insert_cable_core(detached).unwrap();
    assert_eq!(p.core_for_conductor(wire).unwrap().id(), id);
    assert_eq!(p.cable_for_conductor(wire).unwrap().id(), second);
    p.validate().unwrap();
}

#[test]
fn detached_conductor_link_is_checked_before_insertion_and_indexed_when_valid() {
    let (mut p, _, ends, points) = fixture(2);
    let edge = connection(&mut p, points[0], points[1]);
    let networks = p.networks();
    let mut data = json!(wire(&p, "W1", &ends[0], &ends[1]));
    data["connection"] = json!(ConductorConnection::new(edge, points[1]));
    let before = p.clone();
    assert!(
        p.insert_conductor(serde_json::from_value(data.clone()).unwrap())
            .is_err()
    );
    assert_eq!(p, before);
    data["connection"] = json!(ConductorConnection::new(edge, points[0]));
    let id = p
        .insert_conductor(serde_json::from_value(data).unwrap())
        .unwrap();
    assert_eq!(p.physical_for_connection(edge).unwrap().conductors(), &[id]);
    assert_eq!(
        p.remove_connection(edge),
        Err(DomainError::ConnectionInUse(edge))
    );
    assert_eq!(p.networks(), networks);
    p.unassign_conductor_connection(id).unwrap();
    p.remove_connection(edge).unwrap();
    p.validate().unwrap();
}
