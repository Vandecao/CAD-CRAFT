// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
mod physical_common;
use cad_craft::domain::*;
use physical_common::{connection, fixture, part, populated, wire};

#[test]
fn batch_creates_and_links_conductor_as_one_atomic_reversible_delta() {
    let (mut p, _, ends, points) = fixture(2);
    let edge = connection(&mut p, points[0], points[1]);
    let networks = p.networks();
    let before = p.clone();
    let w = wire(&p, "W1", &ends[0], &ends[1]);
    let id = w.id();
    let delta = p
        .edit_physical(PhysicalEdit::Batch(vec![
            PhysicalEdit::UpsertConductor(w),
            PhysicalEdit::SetConductorConnection {
                conductor: id,
                link: Some(ConductorConnection::new(edge, points[0])),
            },
        ]))
        .unwrap();
    assert_eq!(
        delta.len(),
        1,
        "create + link compose into one conductor change"
    );
    assert_eq!(p.physical_for_connection(edge).unwrap().conductors(), &[id]);
    assert_eq!(
        p.networks(),
        networks,
        "physical wiring never alters topology"
    );
    let after = p.clone();
    p.restore_physical(&delta, false).unwrap();
    assert_eq!(p, before);
    p.restore_physical(&delta, true).unwrap();
    assert_eq!(p, after);

    // A failing later step rolls back the earlier ones: no partial mutation.
    let bad = wire(&p, "W2", &ends[1], &ends[0]);
    let bad_id = bad.id();
    let snapshot = p.clone();
    assert!(
        p.edit_physical(PhysicalEdit::Batch(vec![
            PhysicalEdit::UpsertConductor(bad),
            PhysicalEdit::SetConductorConnection {
                conductor: bad_id,
                link: Some(ConductorConnection::new(edge, points[0])),
            },
        ]))
        .is_err()
    );
    assert_eq!(p, snapshot);
    assert!(
        p.edit_physical(PhysicalEdit::Batch(vec![PhysicalEdit::Batch(vec![])]))
            .is_err()
    );
    assert_eq!(p, snapshot);
}

#[test]
fn strip_numbering_is_unique_and_swaps_only_succeed_atomically() {
    let (mut p, _, ends, _) = fixture(2);
    let strip = TerminalStrip::new(p.id(), part("X1"))
        .with_terminals(vec![ends[0].terminal_id(), ends[1].terminal_id()]);
    p.edit_physical(PhysicalEdit::UpsertStrip(strip)).unwrap();
    let rename = |p: &Project, i: usize, n: &str| {
        PhysicalEdit::UpsertTerminal(
            p.terminal(ends[i].terminal_id())
                .unwrap()
                .clone()
                .with_designation(part(n)),
        )
    };
    let before = p.clone();
    assert!(
        p.edit_physical(rename(&p, 0, "T1")).is_err(),
        "duplicate number in strip"
    );
    assert_eq!(p, before);
    let swap = PhysicalEdit::Batch(vec![rename(&p, 0, "T1"), rename(&p, 1, "T0")]);
    let delta = p.edit_physical(swap).unwrap();
    let name = |p: &Project, i: usize| {
        p.terminal(ends[i].terminal_id())
            .unwrap()
            .designation()
            .as_str()
            .to_owned()
    };
    assert_eq!((name(&p, 0), name(&p, 1)), ("T1".into(), "T0".into()));
    p.restore_physical(&delta, false).unwrap();
    assert_eq!(p, before);
}

#[test]
fn deletions_with_dependencies_are_rejected_without_mutation() {
    let mut p = populated();
    let strip = p.terminal_strips().next().unwrap().id();
    let cable = p.cables().next().unwrap().id();
    let assigned = p.cable_cores().find_map(CableCore::conductor_id).unwrap();
    let terminal = p.terminals().next().unwrap().id();
    let before = p.clone();
    for edit in [
        PhysicalEdit::DeleteCable(cable),
        PhysicalEdit::DeleteConductor(assigned),
        PhysicalEdit::DeleteTerminal(terminal),
    ] {
        assert!(p.edit_physical(edit).is_err());
        assert_eq!(p, before);
    }
    let members = p.terminal_strip(strip).unwrap().terminals().to_vec();
    let points: Vec<_> = p
        .connection_points()
        .filter(|cp| cp.terminal_id().is_some_and(|t| members.contains(&t)))
        .map(ConnectionPoint::id)
        .step_by(2)
        .take(2)
        .collect();
    p.edit_physical(PhysicalEdit::CreateBridge {
        strip,
        source: points[0],
        target: points[1],
        kind: TerminalBridgeKind::Fixed,
    })
    .unwrap();
    let with_bridge = p.clone();
    assert!(p.edit_physical(PhysicalEdit::DeleteStrip(strip)).is_err());
    let emptied = p
        .terminal_strip(strip)
        .unwrap()
        .clone()
        .with_terminals(vec![]);
    assert!(p.edit_physical(PhysicalEdit::UpsertStrip(emptied)).is_err());
    assert_eq!(p, with_bridge);
    let bridge = p.terminal_bridges().next().unwrap().id();
    let w = p.conductors().next().unwrap().id();
    assert!(
        p.edit_physical(PhysicalEdit::SetConductorConnection {
            conductor: w,
            link: Some(ConductorConnection::new(bridge, points[0])),
        })
        .is_err(),
        "a bridge never receives an external conductor"
    );
    assert_eq!(p, with_bridge);
}

#[test]
fn technical_lists_are_derived_from_current_core_state() {
    let mut p = populated();
    let lists = p.physical_lists();
    assert_eq!(lists.strips.len(), 1);
    assert_eq!(lists.strips[0].terminals.len(), 4);
    assert_eq!(
        lists.terminals.iter().filter_map(|t| t.order).count(),
        4,
        "every strip member has an explicit 1-based order"
    );
    assert_eq!(lists.cables.len(), 1);
    assert_eq!(
        (lists.cables[0].registered_cores, lists.cables[0].used_cores),
        (4, 3)
    );
    assert_eq!(
        lists.cores.iter().filter(|c| c.conductor.is_none()).count(),
        1
    );
    assert!(
        lists
            .cores
            .iter()
            .filter(|c| c.conductor.is_some())
            .all(|c| c.source.is_some() && c.target.is_some())
    );
    assert_eq!(lists.conductors.len(), 3);
    assert!(
        lists
            .conductors
            .iter()
            .all(|w| w.cable.is_some() && w.connection.is_some())
    );
    assert_eq!(lists.assignments.len(), 3);
    assert!(
        lists
            .assignments
            .iter()
            .all(|a| a.conductors.len() == 1 && !a.bridge)
    );
    assert!(
        p.terminal_label(lists.terminals[0].terminal)
            .starts_with("X1:")
    );

    let w = lists.conductors[0].conductor;
    p.edit_physical(PhysicalEdit::SetConductorConnection {
        conductor: w,
        link: None,
    })
    .unwrap();
    let updated = p.physical_lists();
    assert_eq!(
        updated
            .conductors
            .iter()
            .filter(|w| w.connection.is_none())
            .count(),
        1
    );
    assert_eq!(
        updated
            .assignments
            .iter()
            .filter(|a| a.conductors.is_empty())
            .count(),
        1
    );
    // Without graphic representations there is no occurrence to navigate to.
    assert_eq!(
        p.physical_occurrence(RepresentationTarget::Conductor(w)),
        None
    );
}

#[test]
fn bridge_and_cable_description_snapshots_stay_compatible_and_validated() {
    let mut p = populated();
    let strip = p.terminal_strips().next().unwrap().id();
    let points: Vec<_> = p
        .connection_points()
        .map(ConnectionPoint::id)
        .step_by(2)
        .take(2)
        .collect();
    p.edit_physical(PhysicalEdit::CreateBridge {
        strip,
        source: points[0],
        target: points[1],
        kind: TerminalBridgeKind::Inserted,
    })
    .unwrap();
    let json = serde_json::to_value(&p).unwrap();
    let loaded: Project = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(loaded, p);
    assert_eq!(loaded.physical_lists(), p.physical_lists());

    // Earlier snapshots: no bridge registry and no cable description field.
    let mut old = serde_json::to_value(populated()).unwrap();
    old.as_object_mut().unwrap().remove("terminal_bridges");
    for cable in old["cables"].as_array_mut().unwrap() {
        cable.as_object_mut().unwrap().remove("description");
    }
    let legacy: Project = serde_json::from_value(old).unwrap();
    assert_eq!(legacy.terminal_bridges().count(), 0);
    assert!(legacy.cables().all(|c| c.description().is_none()));

    // A bridge pointing to a missing strip cannot replace a valid project.
    let mut broken = json;
    broken["terminal_bridges"][0]["strip_id"] =
        serde_json::to_value(TerminalStripId::new()).unwrap();
    assert!(serde_json::from_value::<Project>(broken).is_err());
}

#[test]
fn thousands_of_physical_records_derive_lists_and_batch_without_quadratic_cost() {
    let (mut p, device, _, _) = fixture(0);
    let strip = TerminalStrip::new(p.id(), part("X9"));
    let strip_id = strip.id();
    let mut edits = vec![PhysicalEdit::UpsertStrip(strip.clone())];
    let mut order = Vec::new();
    for i in 0..2_000 {
        let t = Terminal::new(p.id(), TerminalOwner::Device(device), part(format!("{i}")))
            .with_positions(vec![part("A")])
            .unwrap();
        order.push(t.id());
        edits.push(PhysicalEdit::UpsertTerminal(t));
    }
    edits.push(PhysicalEdit::UpsertStrip(
        strip.with_terminals(order.clone()),
    ));
    let cable = Cable::new(p.id(), part("W9"), CableKind::Control, 2_000).unwrap();
    let cable_id = cable.id();
    edits.push(PhysicalEdit::UpsertCable(cable));
    for i in 0..2_000 {
        edits.push(PhysicalEdit::UpsertCore(CableCore::new(
            p.id(),
            cable_id,
            part(format!("{i}")),
            ConductorMarking::color(ConductorColor::Black),
        )));
    }
    let start = std::time::Instant::now();
    let delta = p.edit_physical(PhysicalEdit::Batch(edits)).unwrap();
    let batch = start.elapsed();
    assert_eq!(delta.len(), 1 + 2_000 + 1 + 2_000);
    let start = std::time::Instant::now();
    let lists = p.physical_lists();
    let derive = start.elapsed();
    assert_eq!(lists.terminals.len(), 2_000);
    assert_eq!(lists.cores.len(), 2_000);
    assert_eq!(lists.strips[0].terminals, order);
    assert_eq!(
        lists.terminals.iter().filter_map(|t| t.order).max(),
        Some(2_000)
    );
    assert!(p.terminal_strip(strip_id).is_some());
    // Generous debug-build bounds; a quadratic scan of 4 000 rows exceeds them.
    assert!(batch.as_secs_f64() < 5.0, "batch took {batch:?}");
    assert!(derive.as_secs_f64() < 2.0, "lists took {derive:?}");
    p.restore_physical(&delta, false).unwrap();
    assert_eq!(p.terminals().count(), 0);
}

#[test]
fn bridge_upsert_only_edits_existing_bridges_and_never_adopts_connections() {
    let mut p = populated();
    let strip = p.terminal_strips().next().unwrap().id();
    let points: Vec<_> = p
        .connection_points()
        .map(ConnectionPoint::id)
        .step_by(2)
        .take(2)
        .collect();
    p.edit_physical(PhysicalEdit::CreateBridge {
        strip,
        source: points[0],
        target: points[1],
        kind: TerminalBridgeKind::Inserted,
    })
    .unwrap();
    let bridge = p.terminal_bridges().next().unwrap().clone();
    let delta = p
        .edit_physical(PhysicalEdit::UpsertBridge(
            bridge.clone().with_kind(TerminalBridgeKind::Fixed),
        ))
        .unwrap();
    assert_eq!(
        p.terminal_bridge(bridge.id()).unwrap().kind(),
        TerminalBridgeKind::Fixed
    );
    p.restore_physical(&delta, false).unwrap();
    assert_eq!(
        p.terminal_bridge(bridge.id()).unwrap().kind(),
        TerminalBridgeKind::Inserted
    );

    // A plain logical connection cannot be turned into a bridge via Upsert.
    let plain = p
        .connections()
        .find(|c| p.terminal_bridge(c.id()).is_none())
        .unwrap()
        .id();
    let mut forged = serde_json::to_value(&bridge).unwrap();
    forged["connection_id"] = serde_json::to_value(plain).unwrap();
    let forged: TerminalBridge = serde_json::from_value(forged).unwrap();
    let before = p.clone();
    assert!(p.edit_physical(PhysicalEdit::UpsertBridge(forged)).is_err());
    assert_eq!(p, before);
}
