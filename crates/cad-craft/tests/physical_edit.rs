// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

mod physical_common;
use cad_craft::domain::*;

#[test]
fn inserted_bridge_owns_explicit_connection_and_replays_without_physical_wire() {
    let (mut p, _, ends, points) = physical_common::fixture(3);
    let strip = p
        .insert_terminal_strip(TerminalStrip::new(p.id(), physical_common::part("X1")))
        .unwrap();
    p.attach_terminal(strip, ends[0].terminal_id()).unwrap();
    p.attach_terminal(strip, ends[1].terminal_id()).unwrap();
    let before = p.clone();
    let delta = p
        .edit_physical(PhysicalEdit::CreateBridge {
            strip,
            source: points[0],
            target: points[1],
            kind: TerminalBridgeKind::Inserted,
        })
        .unwrap();
    assert_eq!(p.terminal_bridges().count(), 1);
    assert!(p.are_connected(points[0], points[1]).unwrap());
    assert_eq!(p.conductors().count(), 0);
    let id = p.terminal_bridges().next().unwrap().id();
    assert!(p.remove_connection(id).is_err());
    let after = p.clone();
    p.restore_physical(&delta, false).unwrap();
    assert_eq!(p, before);
    p.restore_physical(&delta, true).unwrap();
    assert_eq!(p, after);
    let loaded: Project = serde_json::from_value(serde_json::json!(p)).unwrap();
    assert_eq!(loaded, p);
}

#[test]
fn strip_order_and_cable_cores_have_reversible_edits_with_stable_ids() {
    let (mut p, _, ends, _) = physical_common::fixture(3);
    let strip = TerminalStrip::new(p.id(), physical_common::part("X1"));
    let strip_id = strip.id();
    p.edit_physical(PhysicalEdit::UpsertStrip(strip)).unwrap();
    let ordered = p
        .terminal_strip(strip_id)
        .unwrap()
        .clone()
        .with_terminals(vec![ends[2].terminal_id(), ends[0].terminal_id()]);
    let before = p.clone();
    let delta = p.edit_physical(PhysicalEdit::UpsertStrip(ordered)).unwrap();
    assert_eq!(
        p.terminal_strip(strip_id).unwrap().terminals(),
        &[ends[2].terminal_id(), ends[0].terminal_id()]
    );
    p.restore_physical(&delta, false).unwrap();
    assert_eq!(p, before);
    p.restore_physical(&delta, true).unwrap();

    let cable = Cable::new(p.id(), physical_common::part("W1"), CableKind::Control, 2).unwrap();
    let cable_id = cable.id();
    p.edit_physical(PhysicalEdit::UpsertCable(cable)).unwrap();
    let core = CableCore::new(
        p.id(),
        cable_id,
        physical_common::part("1"),
        ConductorMarking::color(ConductorColor::Black),
    );
    let core_id = core.id();
    p.edit_physical(PhysicalEdit::UpsertCore(core)).unwrap();
    let wire = physical_common::wire(&p, "L1", &ends[0], &ends[1]);
    let wire_id = wire.id();
    p.edit_physical(PhysicalEdit::UpsertConductor(wire))
        .unwrap();
    let before = p.clone();
    let delta = p
        .edit_physical(PhysicalEdit::SetCoreConductor {
            core: core_id,
            conductor: Some(wire_id),
        })
        .unwrap();
    assert_eq!(p.cable_core(core_id).unwrap().conductor_id(), Some(wire_id));
    assert_eq!(p.cable_for_conductor(wire_id).unwrap().id(), cable_id);
    p.restore_physical(&delta, false).unwrap();
    assert_eq!(p, before);
    p.restore_physical(&delta, true).unwrap();
    assert_eq!(
        p,
        serde_json::from_value::<Project>(serde_json::json!(p)).unwrap()
    );
}

#[test]
fn cable_metadata_and_capacity_edits_validate_before_replacing_state() {
    let (mut p, _, _, _) = physical_common::fixture(1);
    let cable = Cable::new(p.id(), physical_common::part("W1"), CableKind::Control, 2).unwrap();
    let id = cable.id();
    p.edit_physical(PhysicalEdit::UpsertCable(cable)).unwrap();
    let edited = p
        .cable(id)
        .unwrap()
        .clone()
        .with_designation(physical_common::part("W2"))
        .with_description(Text::new("Comando").unwrap())
        .with_manufacturer(Text::new("Fabricante").unwrap())
        .with_catalog_code(Text::new("COD-42").unwrap())
        .with_nominal_core_count(3)
        .unwrap();
    let before = p.clone();
    let delta = p.edit_physical(PhysicalEdit::UpsertCable(edited)).unwrap();
    assert_eq!(p.cable(id).unwrap().designation().as_str(), "W2");
    assert_eq!(
        p.cable(id).unwrap().description().unwrap().as_str(),
        "Comando"
    );
    assert_eq!(p.cable(id).unwrap().nominal_core_count(), 3);
    p.restore_physical(&delta, false).unwrap();
    assert_eq!(p, before);
    p.restore_physical(&delta, true).unwrap();
    let core = CableCore::new(
        p.id(),
        id,
        physical_common::part("1"),
        ConductorMarking::color(ConductorColor::Black),
    );
    p.edit_physical(PhysicalEdit::UpsertCore(core)).unwrap();
    p.edit_physical(PhysicalEdit::UpsertCore(CableCore::new(
        p.id(),
        id,
        physical_common::part("2"),
        ConductorMarking::color(ConductorColor::Blue),
    )))
    .unwrap();
    let before = p.clone();
    assert!(
        p.cable(id)
            .unwrap()
            .clone()
            .with_nominal_core_count(0)
            .is_err()
    );
    assert!(
        p.edit_physical(PhysicalEdit::UpsertCable(
            p.cable(id)
                .unwrap()
                .clone()
                .with_nominal_core_count(1)
                .unwrap(),
        ))
        .is_err()
    );
    assert_eq!(p, before);
}

#[test]
fn terminal_core_and_wire_edits_preserve_identity_and_reject_incompatible_assignment() {
    let (mut p, _, ends, _) = physical_common::fixture(3);
    let terminal_id = ends[0].terminal_id();
    p.edit_physical(PhysicalEdit::UpsertTerminal(
        p.terminal(terminal_id)
            .unwrap()
            .clone()
            .with_designation(physical_common::part("1")),
    ))
    .unwrap();
    assert_eq!(p.terminal(terminal_id).unwrap().designation().as_str(), "1");
    let cable = physical_common::cable(&mut p, 2);
    let core = CableCore::new(
        p.id(),
        cable,
        physical_common::part("1"),
        ConductorMarking::color(ConductorColor::Blue),
    )
    .with_cross_section(CrossSectionMm2::new(2.5).unwrap());
    let core_id = core.id();
    p.edit_physical(PhysicalEdit::UpsertCore(core)).unwrap();
    let edited = p
        .cable_core(core_id)
        .unwrap()
        .clone()
        .with_identification(physical_common::part("L1"))
        .with_marking(ConductorMarking::color(ConductorColor::Red));
    p.edit_physical(PhysicalEdit::UpsertCore(edited)).unwrap();
    let wire = physical_common::wire(&p, "F1", &ends[0], &ends[1]);
    let wire_id = wire.id();
    p.edit_physical(PhysicalEdit::UpsertConductor(
        wire.with_designation(physical_common::part("F2")),
    ))
    .unwrap();
    let before = p.clone();
    assert!(
        p.edit_physical(PhysicalEdit::SetCoreConductor {
            core: core_id,
            conductor: Some(wire_id),
        })
        .is_err()
    );
    assert_eq!(p, before);
    assert_eq!(p.networks(), before.networks());
}
