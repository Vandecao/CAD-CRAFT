// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 11 — especializações de bornes: regras verificáveis por tipo, sem
//! criação implícita de continuidade, redes ou terra.
mod physical_common;
use cad_craft::domain::*;
use physical_common::part;
use std::collections::BTreeMap;

fn sides(pairs: &[(&str, TerminalSide)]) -> BTreeMap<DesignationPart, TerminalSide> {
    pairs.iter().map(|(p, s)| (part(*p), *s)).collect()
}
fn upsert(p: &mut Project, t: Terminal) -> Result<PhysicalDelta, cad_craft::error::DomainError> {
    p.edit_physical(PhysicalEdit::UpsertTerminal(t))
}

#[test]
fn kinds_require_their_declared_properties() {
    let (mut p, _, ends, _) = physical_common::fixture(1);
    let id = ends[0].terminal_id();
    let base = p.terminal(id).unwrap().clone(); // positions A, B
    let snapshot = p.clone();
    let invalid = [
        // Feed-through without sides / with a single side.
        base.clone().with_kind(TerminalKind::FeedThrough),
        base.clone()
            .with_kind(TerminalKind::FeedThrough)
            .with_sides(sides(&[("A", TerminalSide::A), ("B", TerminalSide::A)])),
        // Side on a position that does not exist.
        base.clone().with_sides(sides(&[("Z", TerminalSide::A)])),
        // Disconnect without state; state on a non-disconnect terminal.
        base.clone()
            .with_kind(TerminalKind::Disconnect)
            .with_sides(sides(&[("A", TerminalSide::A), ("B", TerminalSide::B)])),
        base.clone()
            .with_disconnect_state(Some(DisconnectState::Open)),
        // Distribution with a single position.
        base.clone()
            .with_positions(vec![part("A")])
            .unwrap()
            .with_kind(TerminalKind::Distribution),
        // PE without any connection position.
        base.clone()
            .with_positions(vec![])
            .unwrap()
            .with_kind(TerminalKind::ProtectiveEarth),
        // Levels on a non-multilevel terminal.
        base.clone().with_levels(vec![
            TerminalLevel::new(part("1"), vec![part("A")]).unwrap(),
            TerminalLevel::new(part("2"), vec![part("B")]).unwrap(),
        ]),
    ];
    for terminal in invalid {
        assert!(upsert(&mut p, terminal.clone()).is_err(), "{terminal:?}");
        assert_eq!(p, snapshot);
    }
    let valid = [
        base.clone()
            .with_kind(TerminalKind::FeedThrough)
            .with_sides(sides(&[("A", TerminalSide::A), ("B", TerminalSide::B)])),
        base.clone()
            .with_kind(TerminalKind::Disconnect)
            .with_sides(sides(&[("A", TerminalSide::A), ("B", TerminalSide::B)]))
            .with_disconnect_state(Some(DisconnectState::Open)),
        base.clone().with_kind(TerminalKind::Distribution),
        base.clone().with_kind(TerminalKind::ProtectiveEarth),
        base.clone(),
    ];
    for terminal in valid {
        upsert(&mut p, terminal.clone()).unwrap();
        assert_eq!(p.terminal(id).unwrap(), &terminal);
        assert_eq!(p.terminal(id).unwrap().id(), id, "identity is stable");
    }
}

#[test]
fn protective_earth_classification_creates_nothing_electrical() {
    let (mut p, _, ends, points) = physical_common::fixture(2);
    let networks = p.networks();
    let connections = p.connections().count();
    let id = ends[0].terminal_id();
    let pe = p
        .terminal(id)
        .unwrap()
        .clone()
        .with_kind(TerminalKind::ProtectiveEarth)
        .with_designation(part("PE"));
    upsert(&mut p, pe).unwrap();
    assert_eq!(p.networks(), networks, "no fictitious earth network");
    assert_eq!(p.connections().count(), connections);
    assert!(!p.are_connected(points[0], points[1]).unwrap());
    let row = p
        .physical_lists()
        .terminals
        .into_iter()
        .find(|t| t.terminal == id)
        .unwrap();
    assert_eq!(row.kind, TerminalKind::ProtectiveEarth);
}

#[test]
fn protective_earth_rejects_declared_non_pe_conductors_and_mixed_bridges() {
    let (mut p, _, ends, points) = physical_common::fixture(3);
    let earth = ends[0].terminal_id();
    let pe = p
        .terminal(earth)
        .unwrap()
        .clone()
        .with_kind(TerminalKind::ProtectiveEarth);
    upsert(&mut p, pe).unwrap();
    let snapshot = p.clone();
    let power =
        physical_common::wire(&p, "W1", &ends[0], &ends[1]).with_function(ConductorFunction::Power);
    assert!(
        p.edit_physical(PhysicalEdit::UpsertConductor(power))
            .is_err()
    );
    assert_eq!(p, snapshot);
    for function in [
        ConductorFunction::ProtectiveEarth,
        ConductorFunction::Unspecified,
    ] {
        let wire = physical_common::wire(&p, "W2", &ends[0], &ends[1]).with_function(function);
        let mut q = p.clone();
        q.edit_physical(PhysicalEdit::UpsertConductor(wire))
            .unwrap();
    }
    // Bridges: PE ↔ non-PE rejected, PE ↔ PE allowed.
    let strip = TerminalStrip::new(p.id(), part("X1"))
        .with_terminals(ends.iter().map(PhysicalTermination::terminal_id).collect());
    let strip_id = strip.id();
    p.edit_physical(PhysicalEdit::UpsertStrip(strip)).unwrap();
    let snapshot = p.clone();
    let bridge = |a, b| PhysicalEdit::CreateBridge {
        strip: strip_id,
        source: a,
        target: b,
        kind: TerminalBridgeKind::Fixed,
    };
    assert!(p.edit_physical(bridge(points[0], points[1])).is_err());
    assert_eq!(p, snapshot);
    let pe2 = p
        .terminal(ends[1].terminal_id())
        .unwrap()
        .clone()
        .with_kind(TerminalKind::ProtectiveEarth);
    p.edit_physical(PhysicalEdit::Batch(vec![
        PhysicalEdit::UpsertTerminal(pe2),
        bridge(points[0], points[1]),
    ]))
    .unwrap();
    assert!(
        p.are_connected(points[0], points[1]).unwrap(),
        "explicit PE bridge"
    );
    // A bridged terminal cannot silently stop being PE.
    let snapshot = p.clone();
    let generic = p
        .terminal(ends[1].terminal_id())
        .unwrap()
        .clone()
        .with_kind(TerminalKind::Generic);
    assert!(upsert(&mut p, generic).is_err());
    assert_eq!(p, snapshot);
}

/// One device with a two-level terminal (levels 1: a/b, 2: c/d) and four points.
fn multilevel() -> (
    Project,
    TerminalId,
    [TerminalLevelId; 2],
    Vec<ConnectionPointId>,
) {
    let (mut p, device, _, _) = physical_common::fixture(0);
    let function = p.functions_for_device(device).next().unwrap().id();
    let l1 = TerminalLevel::new(part("1"), vec![part("a"), part("b")]).unwrap();
    let l2 = TerminalLevel::new(part("2"), vec![part("c"), part("d")]).unwrap();
    let levels = [l1.id(), l2.id()];
    let terminal = Terminal::new(p.id(), TerminalOwner::Device(device), part("5"))
        .with_kind(TerminalKind::MultiLevel)
        .with_levels(vec![l1, l2]);
    let id = terminal.id();
    p.edit_physical(PhysicalEdit::UpsertTerminal(terminal))
        .unwrap();
    let points = (0..4)
        .map(|i| {
            p.insert_connection_point(ConnectionPoint::new(
                p.id(),
                function,
                part(format!("Q{i}")),
            ))
            .unwrap()
        })
        .collect();
    (p, id, levels, points)
}

#[test]
fn multilevel_levels_are_identified_and_validated() {
    let (mut p, id, levels, _) = multilevel();
    let t = p.terminal(id).unwrap().clone();
    assert_eq!(t.levels().len(), 2);
    assert_eq!(
        t.positions().count(),
        4,
        "positions are the union of levels"
    );
    assert_eq!(t.level_of_position(&part("c")).unwrap().id(), levels[1]);
    let row = p
        .physical_lists()
        .terminals
        .into_iter()
        .find(|r| r.terminal == id)
        .unwrap();
    assert_eq!(row.levels, vec!["1 (a/b)", "2 (c/d)"]);
    let snapshot = p.clone();
    let l = |name: &str, positions: &[&str]| {
        TerminalLevel::new(part(name), positions.iter().map(|p| part(*p)).collect()).unwrap()
    };
    let invalid = [
        // A single level.
        t.clone().with_levels(vec![l("1", &["a"])]),
        // Duplicate level name.
        t.clone().with_levels(vec![l("1", &["a"]), l("1", &["b"])]),
        // Shared position.
        t.clone()
            .with_levels(vec![l("1", &["a"]), l("2", &["a", "b"])]),
        // Empty level.
        t.clone().with_levels(vec![l("1", &["a"]), l("2", &[])]),
        // Positions diverging from levels.
        t.clone()
            .with_positions(vec![part("a"), part("z")])
            .unwrap(),
    ];
    for terminal in invalid {
        assert!(upsert(&mut p, terminal.clone()).is_err(), "{terminal:?}");
        assert_eq!(p, snapshot);
    }
    // Renaming a level keeps its identity.
    let renamed: Vec<_> = t
        .levels()
        .iter()
        .map(|level| {
            let name = if level.id() == levels[0] { "N1" } else { "N2" };
            level.clone().with_designation(part(name))
        })
        .collect();
    upsert(&mut p, t.clone().with_levels(renamed)).unwrap();
    let after = p.terminal(id).unwrap();
    assert_eq!(after.level(levels[0]).unwrap().designation().as_str(), "N1");
    assert_eq!(after.level(levels[1]).unwrap().designation().as_str(), "N2");
}

#[test]
fn multilevel_levels_stay_electrically_independent() {
    let (mut p, id, levels, points) = multilevel();
    // A point on a multilevel terminal must name an existing level.
    let snapshot = p.clone();
    for level in [None, Some(TerminalLevelId::new())] {
        assert!(
            p.edit_physical(PhysicalEdit::SetPointTerminal {
                point: points[0],
                terminal: Some(id),
                level,
            })
            .is_err()
        );
        assert_eq!(p, snapshot);
    }
    let set = |p: &mut Project, point, level| {
        p.edit_physical(PhysicalEdit::SetPointTerminal {
            point,
            terminal: Some(id),
            level: Some(level),
        })
        .unwrap()
    };
    set(&mut p, points[0], levels[0]);
    set(&mut p, points[1], levels[1]);
    assert!(
        !p.are_connected(points[0], points[1]).unwrap(),
        "levels of one terminal are isolated"
    );
    // A conductor linked through the level-1 point must land on a level-1 position.
    let (mut q, other) = (p.clone(), points[2]);
    let other_terminal = Terminal::new(q.id(), q.terminal(id).unwrap().owner(), part("6"))
        .with_positions(vec![part("x")])
        .unwrap();
    let other_id = other_terminal.id();
    q.edit_physical(PhysicalEdit::Batch(vec![
        PhysicalEdit::UpsertTerminal(other_terminal),
        PhysicalEdit::SetPointTerminal {
            point: other,
            terminal: Some(other_id),
            level: None,
        },
    ]))
    .unwrap();
    let edge = physical_common::connection(&mut q, points[0], other);
    let far = PhysicalTermination::new(other_id, part("x"));
    for (position, ok) in [("c", false), ("a", true)] {
        let near = PhysicalTermination::new(id, part(position));
        let wire = physical_common::wire(&q, format!("W{position}"), &near, &far);
        let wire_id = wire.id();
        let state = q.clone();
        let result = q.edit_physical(PhysicalEdit::Batch(vec![
            PhysicalEdit::UpsertConductor(wire),
            PhysicalEdit::SetConductorConnection {
                conductor: wire_id,
                link: Some(ConductorConnection::new(edge, points[0])),
            },
        ]));
        assert_eq!(result.is_ok(), ok, "position {position}");
        if !ok {
            assert_eq!(q, state);
        }
    }
    // Explicit continuity between levels only through a validated bridge.
    let strip = TerminalStrip::new(p.id(), part("X1")).with_terminals(vec![id]);
    let strip_id = strip.id();
    p.edit_physical(PhysicalEdit::Batch(vec![
        PhysicalEdit::UpsertStrip(strip),
        PhysicalEdit::CreateBridge {
            strip: strip_id,
            source: points[0],
            target: points[1],
            kind: TerminalBridgeKind::Inserted,
        },
    ]))
    .unwrap();
    assert!(p.are_connected(points[0], points[1]).unwrap());
}

#[test]
fn removing_a_level_in_use_or_changing_kind_is_rejected_atomically() {
    let (mut p, id, levels, points) = multilevel();
    p.edit_physical(PhysicalEdit::SetPointTerminal {
        point: points[0],
        terminal: Some(id),
        level: Some(levels[1]),
    })
    .unwrap();
    let snapshot = p.clone();
    let t = p.terminal(id).unwrap().clone();
    let kept = t.level(levels[0]).unwrap().clone();
    let third = TerminalLevel::new(part("3"), vec![part("e")]).unwrap();
    // Dropping level 2 leaves the point with a dangling level.
    assert!(upsert(&mut p, t.clone().with_levels(vec![kept, third])).is_err());
    // Becoming generic leaves a level reference on the point.
    let generic = t
        .clone()
        .with_levels(vec![])
        .with_kind(TerminalKind::Generic);
    assert!(upsert(&mut p, generic).is_err());
    assert_eq!(p, snapshot);
}

#[test]
fn specializations_round_trip_with_undo_and_old_snapshots_stay_valid() {
    let (mut p, id, levels, points) = multilevel();
    p.edit_physical(PhysicalEdit::SetPointTerminal {
        point: points[3],
        terminal: Some(id),
        level: Some(levels[0]),
    })
    .unwrap();
    let before = p.clone();
    let terminal = p.terminal(id).unwrap().clone();
    let delta = p
        .edit_physical(PhysicalEdit::UpsertTerminal(
            terminal.with_sides(sides(&[("a", TerminalSide::A), ("c", TerminalSide::B)])),
        ))
        .unwrap();
    let after = p.clone();
    p.restore_physical(&delta, false).unwrap();
    assert_eq!(p, before);
    p.restore_physical(&delta, true).unwrap();
    assert_eq!(p, after);
    let json = serde_json::to_value(&p).unwrap();
    let loaded: Project = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(loaded, p);
    assert_eq!(
        loaded.connection_point(points[3]).unwrap().terminal_level(),
        Some(levels[0])
    );
    // Inconsistent snapshots are rejected.
    let terminal_index = json["terminals"]
        .as_array()
        .unwrap()
        .iter()
        .position(|t| t["id"] == serde_json::json!(id))
        .unwrap();
    let mut bad = json.clone();
    bad["terminals"][terminal_index]["kind"] = serde_json::json!("Generic");
    assert!(serde_json::from_value::<Project>(bad).is_err());
    let mut bad = json;
    bad["terminals"][terminal_index]["levels"][1]["positions"] = serde_json::json!(["a"]);
    assert!(serde_json::from_value::<Project>(bad).is_err());
    // Snapshot from before Etapa 11 (no specialization fields) loads as generic.
    let old = physical_common::populated();
    let loaded: Project = serde_json::from_value(serde_json::to_value(&old).unwrap()).unwrap();
    assert!(
        loaded
            .terminals()
            .all(|t| t.kind() == TerminalKind::Generic)
    );
}

#[test]
fn renumbering_specialized_terminals_on_a_strip_keeps_kinds_and_ids() {
    let (mut p, _, ends, _) = physical_common::fixture(2);
    let ids: Vec<_> = ends.iter().map(PhysicalTermination::terminal_id).collect();
    let pe = p
        .terminal(ids[0])
        .unwrap()
        .clone()
        .with_kind(TerminalKind::ProtectiveEarth);
    let strip = TerminalStrip::new(p.id(), part("X1")).with_terminals(ids.clone());
    p.edit_physical(PhysicalEdit::Batch(vec![
        PhysicalEdit::UpsertTerminal(pe),
        PhysicalEdit::UpsertStrip(strip),
    ]))
    .unwrap();
    let swap = ids
        .iter()
        .rev()
        .zip(["1", "2"])
        .map(|(id, n)| {
            PhysicalEdit::UpsertTerminal(p.terminal(*id).unwrap().clone().with_designation(part(n)))
        })
        .collect();
    p.edit_physical(PhysicalEdit::Batch(swap)).unwrap();
    assert_eq!(p.terminal(ids[0]).unwrap().designation().as_str(), "2");
    assert_eq!(
        p.terminal(ids[0]).unwrap().kind(),
        TerminalKind::ProtectiveEarth
    );
    // Deleting a specialized terminal that is still a strip member is refused.
    let snapshot = p.clone();
    assert!(
        p.edit_physical(PhysicalEdit::DeleteTerminal(ids[0]))
            .is_err()
    );
    assert_eq!(p, snapshot);
}
