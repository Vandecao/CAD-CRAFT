// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

mod physical_common;
use cad_craft::domain::*;

fn pt(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}
fn fixture() -> (Project, Vec<PageId>, Vec<ConnectionPointId>) {
    let (mut p, _, ends, points) = physical_common::fixture(4);
    let wire = physical_common::wire(&p, "W1", &ends[0], &ends[1]);
    p.insert_conductor(wire).unwrap();
    let d = p
        .insert_document(Document::new(p.id(), "D").unwrap())
        .unwrap();
    let pages = (1..=3)
        .map(|i| {
            p.insert_page(
                Page::new(
                    p.id(),
                    d,
                    "Page",
                    i.to_string(),
                    PageKind::MultiLine,
                    Size2D::new(420., 297.).unwrap(),
                )
                .unwrap(),
            )
            .unwrap()
        })
        .collect();
    (p, pages, points)
}
fn definition() -> SymbolDefinition {
    SymbolDefinition::new(
        SymbolDefinitionId::new("test.marker").unwrap(),
        1,
        "Marker",
        "References",
        vec![SymbolPrimitive::Line {
            start: pt(0., 0.),
            end: pt(5., 0.),
        }],
        vec![SymbolPort::new("p", pt(0., 0.), 0., "P", None).unwrap()],
    )
    .unwrap()
}
fn insert(
    p: &mut Project,
    page: PageId,
    name: &str,
    kind: MarkerKind,
    point: Option<ConnectionPointId>,
) -> (RepresentationId, MarkerDelta) {
    let def = definition();
    let r = Representation::new_symbol(
        p.id(),
        page,
        &def,
        point.map(RepresentationTarget::ConnectionPoint),
        Transform2D::new(pt(10., 20.), 0., false, false).unwrap(),
    )
    .unwrap();
    let id = r.id();
    let d = p
        .edit_marker(MarkerEdit::Insert {
            marker: ElectricalMarker::new(id, Text::new(name).unwrap(), kind, point),
            representation: Box::new(r),
            definition: Box::new(def),
        })
        .unwrap();
    (id, d)
}
fn potential(kind: &str) -> MarkerKind {
    MarkerKind::Potential {
        kind: Some(Text::new(kind).unwrap()),
        description: "Explicit declaration".into(),
    }
}
fn group(
    p: &mut Project,
    members: Vec<RepresentationId>,
    mode: ReferenceMode,
    continuity: bool,
) -> (ReferenceGroupId, MarkerDelta) {
    let id = ReferenceGroupId::new();
    let d = p
        .edit_marker(MarkerEdit::SetGroup {
            id,
            mode,
            members,
            continuity,
        })
        .unwrap();
    (id, d)
}

#[test]
fn marker_insertion_binds_real_point_and_replay_restores_exact_ids_and_physical_data() {
    let (mut p, pages, points) = fixture();
    let before = p.clone();
    let (id, delta) = insert(
        &mut p,
        pages[0],
        "L1",
        MarkerKind::Interruption,
        Some(points[0]),
    );
    assert_eq!(p.marker(id).unwrap().point(), Some(points[0]));
    assert_eq!(
        p.port_binding(&PortRef::new(id, "p").unwrap()),
        Some(points[0])
    );
    assert_eq!(
        p.representation(id)
            .unwrap()
            .visual()
            .label
            .as_ref()
            .unwrap()
            .as_str(),
        "L1"
    );
    assert_eq!(p.connections().count(), 0);
    assert_eq!(p.conductors().count(), 1);
    let after = p.clone();
    p.validate().unwrap();
    p.restore_marker(&delta, false).unwrap();
    assert_eq!(p, before);
    p.restore_marker(&delta, true).unwrap();
    assert_eq!(p, after);
    assert!(p.restore_marker(&delta, true).is_err());
    assert_eq!(p, after);
}
#[test]
fn equal_names_do_not_connect_and_star_references_use_explicit_members() {
    let (mut p, pages, points) = fixture();
    let a = insert(
        &mut p,
        pages[0],
        "L1",
        MarkerKind::Interruption,
        Some(points[0]),
    )
    .0;
    let b = insert(
        &mut p,
        pages[1],
        "L1",
        MarkerKind::Interruption,
        Some(points[1]),
    )
    .0;
    let c = insert(
        &mut p,
        pages[2],
        "Other",
        MarkerKind::Interruption,
        Some(points[2]),
    )
    .0;
    assert!(p.marker_references(a).unwrap().is_empty());
    let (_, d) = group(&mut p, vec![b, c, a], ReferenceMode::Star, false);
    assert_eq!(p.marker_references(b).unwrap(), vec![c, a]);
    assert_eq!(p.marker_references(a).unwrap(), vec![b]);
    assert!(!p.are_connected(points[0], points[1]).unwrap());
    p.restore_marker(&d, false).unwrap();
    assert!(p.marker_references(a).unwrap().is_empty());
}
#[test]
fn chain_order_is_explicit_and_continuity_is_reversible_in_existing_graph() {
    let (mut p, pages, points) = fixture();
    let ids: Vec<_> = (0..3)
        .map(|i| {
            insert(
                &mut p,
                pages[i],
                "L1",
                MarkerKind::Interruption,
                Some(points[i]),
            )
            .0
        })
        .collect();
    let physical = p.conductors().cloned().collect::<Vec<_>>();
    let before = p.clone();
    let (g, d) = group(
        &mut p,
        vec![ids[2], ids[0], ids[1]],
        ReferenceMode::Chain,
        true,
    );
    assert_eq!(p.marker_references(ids[0]).unwrap(), vec![ids[2], ids[1]]);
    assert_eq!(p.marker_references(ids[2]).unwrap(), vec![ids[0]]);
    assert_eq!(p.marker_group(g).unwrap().connections().len(), 2);
    assert!(p.are_connected(points[1], points[2]).unwrap());
    assert_eq!(p.conductors().cloned().collect::<Vec<_>>(), physical);
    let after = p.clone();
    p.restore_marker(&d, false).unwrap();
    assert_eq!(p, before);
    p.restore_marker(&d, true).unwrap();
    assert_eq!(p, after);
}
#[test]
fn continuity_never_appropriates_existing_edges_or_allows_legacy_edge_removal() {
    let (mut p, pages, points) = fixture();
    let a = insert(
        &mut p,
        pages[0],
        "A",
        MarkerKind::Interruption,
        Some(points[0]),
    )
    .0;
    let b = insert(
        &mut p,
        pages[1],
        "B",
        MarkerKind::Interruption,
        Some(points[1]),
    )
    .0;
    let edge = p
        .insert_connection(Connection::new(p.id(), points[0], points[1]).unwrap())
        .unwrap();
    let before = p.clone();
    assert!(
        p.edit_marker(MarkerEdit::SetGroup {
            id: ReferenceGroupId::new(),
            mode: ReferenceMode::Star,
            members: vec![a, b],
            continuity: true
        })
        .is_err()
    );
    assert_eq!(p, before);
    p.remove_connection(edge).unwrap();
    let (g, _) = group(&mut p, vec![a, b], ReferenceMode::Star, true);
    let owned = p.marker_group(g).unwrap().connections()[0];
    let before = p.clone();
    assert!(p.remove_connection(owned).is_err());
    assert_eq!(p, before);
    let delta = p.edit_marker(MarkerEdit::DeleteGroup { id: g }).unwrap();
    assert!(!p.are_connected(points[0], points[1]).unwrap());
    p.restore_marker(&delta, false).unwrap();
    assert_eq!(p, before);
}
#[test]
fn marker_group_rejects_missing_duplicate_potential_or_shared_members_atomically() {
    let (mut p, pages, points) = fixture();
    let a = insert(
        &mut p,
        pages[0],
        "A",
        MarkerKind::Interruption,
        Some(points[0]),
    )
    .0;
    let b = insert(&mut p, pages[1], "B", MarkerKind::Interruption, None).0;
    let c = insert(&mut p, pages[2], "L1", potential("AC"), Some(points[2])).0;
    for (members, continuity) in [
        (vec![a, a], false),
        (vec![a, RepresentationId::new()], false),
        (vec![a, c], false),
        (vec![a, b], true),
    ] {
        let before = p.clone();
        assert!(
            p.edit_marker(MarkerEdit::SetGroup {
                id: ReferenceGroupId::new(),
                mode: ReferenceMode::Chain,
                members,
                continuity
            })
            .is_err()
        );
        assert_eq!(p, before);
    }
    group(&mut p, vec![a, b], ReferenceMode::Star, false);
    let before = p.clone();
    assert!(
        p.edit_marker(MarkerEdit::SetGroup {
            id: ReferenceGroupId::new(),
            mode: ReferenceMode::Chain,
            members: vec![a, b],
            continuity: false
        })
        .is_err()
    );
    assert_eq!(p, before);
}
#[test]
fn marker_update_and_move_keep_identity_and_reference_destinations_current() {
    let (mut p, pages, points) = fixture();
    let a = insert(
        &mut p,
        pages[0],
        "L1",
        MarkerKind::Interruption,
        Some(points[0]),
    )
    .0;
    let b = insert(&mut p, pages[1], "L1", MarkerKind::Interruption, None).0;
    group(&mut p, vec![a, b], ReferenceMode::Star, false);
    let before = p.clone();
    let d = p
        .edit_marker(MarkerEdit::Update {
            id: a,
            name: Text::new("NEW").unwrap(),
            kind: MarkerKind::Interruption,
            point: Some(points[2]),
        })
        .unwrap();
    assert_eq!(p.marker(a).unwrap().name().as_str(), "NEW");
    assert_eq!(
        p.port_binding(&PortRef::new(a, "p").unwrap()),
        Some(points[2])
    );
    assert_eq!(p.marker_references(b).unwrap(), vec![a]);
    let movement = p
        .edit_marker(MarkerEdit::Move {
            id: a,
            page: pages[2],
            transform: Transform2D::new(pt(60., 70.), 90., false, false).unwrap(),
        })
        .unwrap();
    assert_eq!(
        p.representation_location(a).unwrap(),
        (pages[2], pt(60., 70.))
    );
    p.restore_marker(&movement, false).unwrap();
    p.restore_marker(&d, false).unwrap();
    assert_eq!(p, before);
}
#[test]
fn bound_marker_and_group_dependencies_cannot_be_modified_through_legacy_apis() {
    let (mut p, pages, points) = fixture();
    let a = insert(
        &mut p,
        pages[0],
        "A",
        MarkerKind::Interruption,
        Some(points[0]),
    )
    .0;
    let b = insert(
        &mut p,
        pages[1],
        "B",
        MarkerKind::Interruption,
        Some(points[1]),
    )
    .0;
    group(&mut p, vec![a, b], ReferenceMode::Star, true);
    let before = p.clone();
    assert!(p.remove_representation(a).is_err());
    assert!(p.edit_marker(MarkerEdit::Delete { id: a }).is_err());
    assert!(
        p.edit_marker(MarkerEdit::Update {
            id: a,
            name: Text::new("A").unwrap(),
            kind: MarkerKind::Interruption,
            point: Some(points[2])
        })
        .is_err()
    );
    assert!(
        p.edit_connection_drawing(ConnectionDrawingEdit::BindPort {
            port: PortRef::new(a, "p").unwrap(),
            point: None
        })
        .is_err()
    );
    let mut visual = p.representation(a).unwrap().visual().clone();
    visual.label = Some(Text::new("LIE").unwrap());
    assert!(p.set_representation_visual(a, visual).is_err());
    assert_eq!(p, before);
}
#[test]
fn potential_diagnostics_follow_real_network_and_same_text_never_connects() {
    let (mut p, pages, points) = fixture();
    let a = insert(&mut p, pages[0], "L1", potential("AC"), Some(points[0])).0;
    let b = insert(&mut p, pages[1], "L1", potential("AC"), Some(points[1])).0;
    let c = insert(&mut p, pages[2], "24VDC", potential("DC"), Some(points[2])).0;
    assert!(p.marker_references(a).unwrap().is_empty());
    assert!(p.potential_conflicts().is_empty());
    p.insert_connection(Connection::new(p.id(), points[0], points[2]).unwrap())
        .unwrap();
    assert_eq!(p.marker_references(a).unwrap(), vec![c]);
    let conflicts = p.potential_conflicts();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(
        (conflicts[0].first, conflicts[0].second),
        (a.min(c), a.max(c))
    );
    assert!(!p.are_connected(points[0], points[1]).unwrap());
    assert!(p.marker_references(b).unwrap().is_empty());
}
#[test]
fn snapshots_preserve_markers_groups_and_reject_invalid_references() {
    let (mut p, pages, points) = fixture();
    let a = insert(
        &mut p,
        pages[0],
        "A",
        MarkerKind::Interruption,
        Some(points[0]),
    )
    .0;
    let b = insert(
        &mut p,
        pages[1],
        "B",
        MarkerKind::Interruption,
        Some(points[1]),
    )
    .0;
    group(&mut p, vec![a, b], ReferenceMode::Chain, true);
    let json = serde_json::to_value(&p).unwrap();
    assert_eq!(serde_json::from_value::<Project>(json.clone()).unwrap(), p);
    let mut bad = json.clone();
    bad["markers"][0]["point"] = serde_json::to_value(ConnectionPointId::new()).unwrap();
    assert!(serde_json::from_value::<Project>(bad).is_err());
    let mut bad = json.clone();
    bad["marker_groups"][0]["members"][0] = serde_json::to_value(RepresentationId::new()).unwrap();
    assert!(serde_json::from_value::<Project>(bad).is_err());
    let mut bad = json;
    let duplicate = bad["markers"][0].clone();
    bad["markers"].as_array_mut().unwrap().push(duplicate);
    assert!(serde_json::from_value::<Project>(bad).is_err());
    let (old, _, _) = fixture();
    let mut legacy = serde_json::to_value(&old).unwrap();
    legacy.as_object_mut().unwrap().remove("markers");
    legacy.as_object_mut().unwrap().remove("marker_groups");
    assert_eq!(serde_json::from_value::<Project>(legacy).unwrap(), old);
}
#[test]
fn deletion_and_noop_update_are_compact_and_reversible() {
    let (mut p, pages, _) = fixture();
    let (a, _) = insert(&mut p, pages[0], "A", MarkerKind::Interruption, None);
    let before = p.clone();
    let d = p
        .edit_marker(MarkerEdit::Update {
            id: a,
            name: Text::new("A").unwrap(),
            kind: MarkerKind::Interruption,
            point: None,
        })
        .unwrap();
    assert!(d.is_empty());
    assert_eq!(p, before);
    let d = p.edit_marker(MarkerEdit::Delete { id: a }).unwrap();
    assert!(p.marker(a).is_none());
    assert!(p.representation(a).is_none());
    assert!(p.symbol_definition(&definition().id().clone()).is_none());
    p.restore_marker(&d, false).unwrap();
    assert_eq!(p, before);
}

#[test]
fn marker_ports_support_real_routes_and_route_dependencies_prevent_rebinding_or_deletion() {
    let (mut p, pages, points) = fixture();
    let a = insert(
        &mut p,
        pages[0],
        "A",
        MarkerKind::Interruption,
        Some(points[0]),
    )
    .0;
    let b = insert(
        &mut p,
        pages[0],
        "B",
        MarkerKind::Interruption,
        Some(points[1]),
    )
    .0;
    p.edit_marker(MarkerEdit::Move {
        id: b,
        page: pages[0],
        transform: Transform2D::new(pt(50., 20.), 0., false, false).unwrap(),
    })
    .unwrap();
    p.edit_connection_drawing(ConnectionDrawingEdit::CreateRoute {
        source: PortRef::new(a, "p").unwrap(),
        target: PortRef::new(b, "p").unwrap(),
        points: vec![pt(10., 20.), pt(50., 20.)],
    })
    .unwrap();
    let before = p.clone();
    assert!(
        p.edit_marker(MarkerEdit::Update {
            id: a,
            name: Text::new("A").unwrap(),
            kind: MarkerKind::Interruption,
            point: Some(points[2])
        })
        .is_err()
    );
    assert!(p.edit_marker(MarkerEdit::Delete { id: a }).is_err());
    assert!(
        p.edit_marker(MarkerEdit::Move {
            id: a,
            page: pages[1],
            transform: Transform2D::new(pt(10., 20.), 0., false, false).unwrap()
        })
        .is_err()
    );
    assert_eq!(p, before);
    let d = p
        .transform_representations_with_drawing(&[(
            a,
            Transform2D::new(pt(-30., 20.), 0., false, false).unwrap(),
        )])
        .unwrap();
    assert_eq!(
        p.connection_routes().next().unwrap().points(),
        &[pt(-30., 20.), pt(50., 20.)]
    );
    assert!(p.are_connected(points[0], points[1]).unwrap());
    p.restore_connection_drawing(&d, false).unwrap();
    assert_eq!(p, before);
}

#[test]
fn removing_group_refuses_physical_dependents_and_rolls_back_all_indices() {
    let (mut p, pages, points) = fixture();
    let a = insert(
        &mut p,
        pages[0],
        "A",
        MarkerKind::Interruption,
        Some(points[0]),
    )
    .0;
    let b = insert(
        &mut p,
        pages[1],
        "B",
        MarkerKind::Interruption,
        Some(points[1]),
    )
    .0;
    let (g, _) = group(&mut p, vec![a, b], ReferenceMode::Star, true);
    let edge = p.marker_group(g).unwrap().connections()[0];
    let conductor = p.conductors().next().unwrap().id();
    p.assign_conductor_connection(conductor, ConductorConnection::new(edge, points[0]))
        .unwrap();
    let before = p.clone();
    assert!(p.edit_marker(MarkerEdit::DeleteGroup { id: g }).is_err());
    assert!(
        p.edit_marker(MarkerEdit::SetGroup {
            id: g,
            mode: ReferenceMode::Star,
            members: vec![a, b],
            continuity: false
        })
        .is_err()
    );
    assert_eq!(p, before);
    p.validate().unwrap();
}

#[test]
fn shared_electrical_point_occurrences_need_no_fictitious_continuity_edge() {
    let (mut p, pages, points) = fixture();
    let a = insert(
        &mut p,
        pages[0],
        "A",
        MarkerKind::Interruption,
        Some(points[0]),
    )
    .0;
    let b = insert(
        &mut p,
        pages[1],
        "B",
        MarkerKind::Interruption,
        Some(points[0]),
    )
    .0;
    let (g, _) = group(&mut p, vec![a, b], ReferenceMode::Star, true);
    assert!(p.marker_group(g).unwrap().connections().is_empty());
    assert_eq!(p.marker_references(a).unwrap(), vec![b]);
    assert_eq!(p.connections().count(), 0);
    let d = p
        .edit_marker(MarkerEdit::SetGroup {
            id: g,
            mode: ReferenceMode::Star,
            members: vec![a, b],
            continuity: true,
        })
        .unwrap();
    assert!(d.is_empty());
    p.validate().unwrap();
}

#[test]
fn invalid_marker_insert_and_stale_replay_never_leave_partial_entities() {
    let (mut p, pages, points) = fixture();
    let before = p.clone();
    let def = definition();
    let rep = Representation::new_symbol(
        ProjectId::new(),
        pages[0],
        &def,
        None,
        Transform2D::new(pt(10., 20.), 0., false, false).unwrap(),
    )
    .unwrap();
    assert!(
        p.edit_marker(MarkerEdit::Insert {
            marker: ElectricalMarker::new(
                rep.id(),
                Text::new("A").unwrap(),
                MarkerKind::Interruption,
                Some(points[0])
            ),
            representation: Box::new(rep),
            definition: Box::new(def)
        })
        .is_err()
    );
    assert_eq!(p, before);
    let (a, _) = insert(&mut p, pages[0], "A", MarkerKind::Interruption, None);
    let d = p
        .edit_marker(MarkerEdit::Update {
            id: a,
            name: Text::new("B").unwrap(),
            kind: MarkerKind::Interruption,
            point: None,
        })
        .unwrap();
    p.edit_marker(MarkerEdit::Update {
        id: a,
        name: Text::new("C").unwrap(),
        kind: MarkerKind::Interruption,
        point: None,
    })
    .unwrap();
    let before = p.clone();
    assert!(p.restore_marker(&d, false).is_err());
    assert_eq!(p, before);
}
