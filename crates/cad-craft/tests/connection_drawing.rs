// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
mod physical_common;
use cad_craft::domain::*;
use serde_json::json;
fn pt(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}
fn tr(x: f64, y: f64) -> Transform2D {
    Transform2D::new(pt(x, y), 0., false, false).unwrap()
}
fn setup() -> (Project, PageId, Vec<PortRef>, Vec<ConnectionPointId>) {
    let (mut p, device, _, points) = physical_common::fixture(6);
    let d = p
        .insert_document(Document::new(p.id(), "D").unwrap())
        .unwrap();
    let page = p
        .insert_page(
            Page::new(
                p.id(),
                d,
                "P",
                "1",
                PageKind::MultiLine,
                Size2D::new(200., 200.).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let def = SymbolDefinition::new(
        SymbolDefinitionId::new("test.port").unwrap(),
        1,
        "Port",
        "Test",
        vec![SymbolPrimitive::Line {
            start: pt(-1., 0.),
            end: pt(1., 0.),
        }],
        vec![SymbolPort::new("p", pt(0., 0.), 0., "P", None).unwrap()],
    )
    .unwrap();
    let mut ports = vec![];
    for (i, pos) in [
        (0., 0.),
        (40., 0.),
        (20., -20.),
        (20., 20.),
        (0., 20.),
        (40., 20.),
    ]
    .into_iter()
    .enumerate()
    {
        let r = Representation::new_symbol(
            p.id(),
            page,
            &def,
            Some(RepresentationTarget::Device(device)),
            tr(pos.0, pos.1),
        )
        .unwrap();
        let port = PortRef::new(r.id(), "p").unwrap();
        p.insert_symbol_representation(r, def.clone()).unwrap();
        p.edit_connection_drawing(ConnectionDrawingEdit::BindPort {
            port: port.clone(),
            point: Some(points[i]),
        })
        .unwrap();
        ports.push(port);
    }
    (p, page, ports, points)
}
fn draw(p: &mut Project, a: &PortRef, b: &PortRef, points: Vec<Point2D>) -> ConnectionDrawingDelta {
    p.edit_connection_drawing(ConnectionDrawingEdit::CreateRoute {
        source: a.clone(),
        target: b.clone(),
        points,
    })
    .unwrap()
}
#[test]
fn binding_is_explicit_compatible_and_does_not_create_connections() {
    let (mut p, _, ports, points) = setup();
    assert_eq!(p.port_binding(&ports[0]), Some(points[0]));
    assert_eq!(p.connections().count(), 0);
    let before = p.clone();
    let bad = PortRef::new(ports[0].representation_id(), "absent").unwrap();
    assert!(
        p.edit_connection_drawing(ConnectionDrawingEdit::BindPort {
            port: bad,
            point: Some(points[0])
        })
        .is_err()
    );
    assert_eq!(p, before);
    let device = p
        .insert_device(Device::new(p.id(), Designation::new("OTHER").unwrap(), "other").unwrap())
        .unwrap();
    let function = p
        .insert_function(Function::new(
            p.id(),
            device,
            Designation::new("F").unwrap(),
            FunctionKind::Coil,
        ))
        .unwrap();
    let other = p
        .insert_connection_point(ConnectionPoint::new(
            p.id(),
            function,
            DesignationPart::new("X").unwrap(),
        ))
        .unwrap();
    let before = p.clone();
    assert!(
        p.edit_connection_drawing(ConnectionDrawingEdit::BindPort {
            port: ports[0].clone(),
            point: Some(other)
        })
        .is_err()
    );
    assert_eq!(p, before);
}
#[test]
fn create_route_has_shared_representation_identity_and_reversible_compact_delta() {
    let (mut p, _, ports, points) = setup();
    let before = p.clone();
    let delta = draw(&mut p, &ports[0], &ports[1], vec![pt(0., 0.), pt(40., 0.)]);
    assert!(!delta.is_empty());
    assert_eq!(delta.len(), 3);
    let r = p.connection_routes().next().unwrap();
    let id = r.id();
    let edge = r.connection_id();
    assert_eq!(
        p.representation(id).unwrap().target(),
        RepresentationTarget::Connection(edge)
    );
    assert!(p.are_connected(points[0], points[1]).unwrap());
    assert_eq!(p.conductors().count(), 0);
    p.validate().unwrap();
    let after = p.clone();
    p.restore_connection_drawing(&delta, false).unwrap();
    assert_eq!(p, before);
    p.restore_connection_drawing(&delta, true).unwrap();
    assert_eq!(p, after);
    assert!(p.restore_connection_drawing(&delta, true).is_err());
    assert_eq!(p, after);
}

#[test]
fn wire_number_label_position_is_reversible_without_changing_connections() {
    let (mut project, _, ports, _) = setup();
    draw(
        &mut project,
        &ports[0],
        &ports[1],
        vec![pt(0., 0.), pt(40., 0.)],
    );
    let route = project.connection_routes().next().unwrap().id();
    let network = project.networks();
    let before = project.clone();
    let delta = project
        .edit_connection_drawing(ConnectionDrawingEdit::SetNumberLabelPosition {
            id: route,
            position: Some(pt(20., 5.)),
        })
        .unwrap();
    assert_eq!(
        project
            .connection_route(route)
            .unwrap()
            .number_label_position(),
        Some(pt(20., 5.))
    );
    assert_eq!(project.networks(), network);
    project.restore_connection_drawing(&delta, false).unwrap();
    assert_eq!(project, before);
}
#[test]
fn invalid_and_duplicate_routes_are_atomic() {
    let (mut p, _, ports, _) = setup();
    let before = p.clone();
    for path in [
        vec![],
        vec![pt(0., 0.), pt(1., 1.), pt(40., 0.)],
        vec![pt(0., 0.), pt(0., 0.), pt(40., 0.)],
        vec![pt(1., 0.), pt(40., 0.)],
    ] {
        assert!(
            p.edit_connection_drawing(ConnectionDrawingEdit::CreateRoute {
                source: ports[0].clone(),
                target: ports[1].clone(),
                points: path
            })
            .is_err()
        );
        assert_eq!(p, before);
    }
    draw(&mut p, &ports[0], &ports[1], vec![pt(0., 0.), pt(40., 0.)]);
    let before = p.clone();
    assert!(
        p.edit_connection_drawing(ConnectionDrawingEdit::CreateRoute {
            source: ports[1].clone(),
            target: ports[0].clone(),
            points: vec![pt(40., 0.), pt(0., 0.)]
        })
        .is_err()
    );
    assert_eq!(p, before);
}
#[test]
fn unbound_port_cannot_create_route_and_rebinding_used_endpoint_is_rejected() {
    let (mut p, _, ports, _) = setup();
    p.edit_connection_drawing(ConnectionDrawingEdit::BindPort {
        port: ports[0].clone(),
        point: None,
    })
    .unwrap();
    let before = p.clone();
    assert!(
        p.edit_connection_drawing(ConnectionDrawingEdit::CreateRoute {
            source: ports[0].clone(),
            target: ports[1].clone(),
            points: vec![pt(0., 0.), pt(40., 0.)]
        })
        .is_err()
    );
    assert_eq!(p, before);
    let (mut p, _, ports, _) = setup();
    draw(&mut p, &ports[0], &ports[1], vec![pt(0., 0.), pt(40., 0.)]);
    let before = p.clone();
    assert!(
        p.edit_connection_drawing(ConnectionDrawingEdit::BindPort {
            port: ports[0].clone(),
            point: None
        })
        .is_err()
    );
    assert_eq!(p, before);
}
#[test]
fn crossing_does_not_connect_until_explicit_junction_and_undo_restores_topology() {
    let (mut p, page, ports, points) = setup();
    draw(&mut p, &ports[0], &ports[1], vec![pt(0., 0.), pt(40., 0.)]);
    draw(
        &mut p,
        &ports[2],
        &ports[3],
        vec![pt(20., -20.), pt(20., 20.)],
    );
    assert!(!p.are_connected(points[0], points[2]).unwrap());
    let routes = p.connection_routes().map(ConnectionRoute::id).collect();
    let before = p.clone();
    let delta = p
        .edit_connection_drawing(ConnectionDrawingEdit::CreateJunction {
            page,
            position: pt(20., 0.),
            routes,
        })
        .unwrap();
    assert!(p.are_connected(points[0], points[2]).unwrap());
    assert_eq!(p.devices().count(), before.devices().count());
    assert_eq!(p.connection_junctions().count(), 1);
    p.validate().unwrap();
    let after = p.clone();
    p.restore_connection_drawing(&delta, false).unwrap();
    assert_eq!(p, before);
    p.restore_connection_drawing(&delta, true).unwrap();
    assert_eq!(p, after);
    let id = p.connection_junctions().next().unwrap().id();
    let deletion = p
        .edit_connection_drawing(ConnectionDrawingEdit::DeleteJunction { id })
        .unwrap();
    assert!(!p.are_connected(points[0], points[2]).unwrap());
    p.restore_connection_drawing(&deletion, false).unwrap();
    assert_eq!(p, after);
}
#[test]
fn deleting_used_route_or_legacy_mutation_is_rejected_without_dangling_references() {
    let (mut p, page, ports, _) = setup();
    draw(&mut p, &ports[0], &ports[1], vec![pt(0., 0.), pt(40., 0.)]);
    let route = p.connection_routes().next().unwrap().clone();
    let before = p.clone();
    assert!(p.remove_representation(route.id()).is_err());
    assert_eq!(p, before);
    assert!(
        p.remove_representation(ports[0].representation_id())
            .is_err()
    );
    assert!(p.remove_connection(route.connection_id()).is_err());
    assert!(
        p.move_representation(ports[0].representation_id(), page, tr(1., 1.))
            .is_err()
    );
    assert_eq!(p, before);
    let delta = p
        .edit_connection_drawing(ConnectionDrawingEdit::DeleteRoute { id: route.id() })
        .unwrap();
    assert_eq!(p.connection_routes().count(), 0);
    assert_eq!(p.connections().count(), 0);
    p.restore_connection_drawing(&delta, false).unwrap();
    assert_eq!(p, before);
}
#[test]
fn transforms_keep_endpoints_and_common_translation_preserves_route_exactly() {
    let (mut p, _, ports, _) = setup();
    draw(
        &mut p,
        &ports[0],
        &ports[1],
        vec![pt(0., 0.), pt(0., 10.), pt(40., 10.), pt(40., 0.)],
    );
    let before = p.clone();
    let delta = p
        .transform_representations_with_drawing(&[
            (ports[0].representation_id(), tr(-10., -10.)),
            (ports[1].representation_id(), tr(30., -10.)),
        ])
        .unwrap();
    assert_eq!(
        p.connection_routes().next().unwrap().points(),
        &[pt(-10., -10.), pt(-10., 0.), pt(30., 0.), pt(30., -10.)]
    );
    p.validate().unwrap();
    p.restore_connection_drawing(&delta, false).unwrap();
    assert_eq!(p, before);
    p.transform_representations(&[(ports[0].representation_id(), tr(-5., 3.))])
        .unwrap();
    let route = p.connection_routes().next().unwrap();
    assert_eq!(route.points()[0], pt(-5., 3.));
    assert_eq!(*route.points().last().unwrap(), pt(40., 0.));
    p.validate().unwrap();
}
#[test]
fn snapshots_roundtrip_defaults_and_reject_dangling_duplicate_or_bad_geometry() {
    let (mut p, _, ports, _) = setup();
    draw(&mut p, &ports[0], &ports[1], vec![pt(0., 0.), pt(40., 0.)]);
    let bytes = serde_json::to_string(&p).unwrap();
    let copy: Project = serde_json::from_str(&bytes).unwrap();
    assert_eq!(copy, p);
    assert_eq!(serde_json::to_string(&copy).unwrap(), bytes);
    let original = json!(p);
    for key in ["connection_routes", "port_bindings"] {
        let mut bad = original.clone();
        let item = bad[key][0].clone();
        bad[key].as_array_mut().unwrap().push(item);
        assert!(serde_json::from_value::<Project>(bad).is_err());
    }
    let mut bad = original;
    bad["port_bindings"] = json!([]);
    assert!(serde_json::from_value::<Project>(bad).is_err());
    let p = Project::new("old").unwrap();
    let mut old = json!(p);
    for key in ["port_bindings", "connection_routes", "connection_junctions"] {
        old.as_object_mut().unwrap().remove(key);
    }
    assert_eq!(serde_json::from_value::<Project>(old).unwrap(), p);
}
#[test]
fn route_edit_noop_and_invalid_geometry_preserve_state_and_undo_is_exact() {
    let (mut p, _, ports, _) = setup();
    draw(&mut p, &ports[0], &ports[1], vec![pt(0., 0.), pt(40., 0.)]);
    let r = p.connection_routes().next().unwrap().clone();
    let noop = p
        .edit_connection_drawing(ConnectionDrawingEdit::SetRoute {
            id: r.id(),
            points: r.points().to_vec(),
        })
        .unwrap();
    assert!(noop.is_empty());
    let before = p.clone();
    assert!(
        p.edit_connection_drawing(ConnectionDrawingEdit::SetRoute {
            id: r.id(),
            points: vec![pt(0., 0.), pt(1., 1.), pt(40., 0.)]
        })
        .is_err()
    );
    assert_eq!(p, before);
    let delta = p
        .edit_connection_drawing(ConnectionDrawingEdit::SetRoute {
            id: r.id(),
            points: vec![pt(0., 0.), pt(0., -5.), pt(40., -5.), pt(40., 0.)],
        })
        .unwrap();
    p.restore_connection_drawing(&delta, false).unwrap();
    assert_eq!(p, before);
}
#[test]
fn duplicate_junction_position_is_rejected_in_snapshots() {
    let (mut p, page, ports, _) = setup();
    draw(&mut p, &ports[0], &ports[1], vec![pt(0., 0.), pt(40., 0.)]);
    draw(
        &mut p,
        &ports[0],
        &ports[3],
        vec![pt(0., 0.), pt(20., 0.), pt(20., 20.)],
    );
    let routes = p.connection_routes().map(ConnectionRoute::id).collect();
    p.edit_connection_drawing(ConnectionDrawingEdit::CreateJunction {
        page,
        position: pt(0., 0.),
        routes,
    })
    .unwrap();
    let mut bad = json!(p);
    let mut duplicate = bad["connection_junctions"][0].clone();
    duplicate["id"] = json!(ConnectionJunctionId::new());
    duplicate["connections"] = json!([]);
    bad["connection_junctions"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert!(serde_json::from_value::<Project>(bad).is_err());
}
#[test]
fn empty_port_identity_is_rejected_during_deserialization() {
    assert!(PortRef::new(RepresentationId::new(), " ").is_err());
    assert!(
        serde_json::from_value::<PortRef>(
            json!({"representation":RepresentationId::new(),"port":" "})
        )
        .is_err()
    );
}
#[test]
fn junction_group_translation_is_reversible_and_partial_disconnect_is_rejected() {
    let (mut p, page, ports, _) = setup();
    draw(&mut p, &ports[0], &ports[1], vec![pt(0., 0.), pt(40., 0.)]);
    draw(
        &mut p,
        &ports[2],
        &ports[3],
        vec![pt(20., -20.), pt(20., 20.)],
    );
    let routes = p.connection_routes().map(ConnectionRoute::id).collect();
    p.edit_connection_drawing(ConnectionDrawingEdit::CreateJunction {
        page,
        position: pt(20., 0.),
        routes,
    })
    .unwrap();
    let before = p.clone();
    assert!(
        p.transform_representations_with_drawing(&[
            (ports[0].representation_id(), tr(0., 10.)),
            (ports[1].representation_id(), tr(40., 10.))
        ])
        .is_err()
    );
    assert_eq!(p, before);
    let changes = ports
        .iter()
        .take(4)
        .map(|port| {
            let old = p.port_position(port).unwrap();
            (port.representation_id(), tr(old.x() + 3., old.y() - 7.))
        })
        .collect::<Vec<_>>();
    let delta = p.transform_representations_with_drawing(&changes).unwrap();
    assert_eq!(
        p.connection_junctions().next().unwrap().position(),
        pt(23., -7.)
    );
    p.validate().unwrap();
    p.restore_connection_drawing(&delta, false).unwrap();
    assert_eq!(p, before);
}
#[test]
fn tee_and_multiple_junctions_join_only_explicit_participants() {
    let (mut p, page, ports, points) = setup();
    draw(&mut p, &ports[0], &ports[1], vec![pt(0., 0.), pt(40., 0.)]);
    draw(
        &mut p,
        &ports[2],
        &ports[3],
        vec![pt(20., -20.), pt(20., 20.)],
    );
    draw(
        &mut p,
        &ports[4],
        &ports[5],
        vec![pt(0., 20.), pt(0., 0.), pt(40., 0.), pt(40., 20.)],
    );
    let routes = p.connection_routes().map(ConnectionRoute::id).collect();
    let before = p.clone();
    let delta = p
        .edit_connection_drawing(ConnectionDrawingEdit::CreateJunction {
            page,
            position: pt(20., 0.),
            routes,
        })
        .unwrap();
    for i in 1..6 {
        assert!(p.are_connected(points[0], points[i]).unwrap());
    }
    assert_eq!(
        p.connection_junctions().next().unwrap().connections().len(),
        2
    );
    p.validate().unwrap();
    p.restore_connection_drawing(&delta, false).unwrap();
    assert_eq!(p, before);
}
#[test]
fn deleting_route_with_external_reference_or_physical_conductor_is_atomic() {
    let (mut p, page, ports, points) = setup();
    draw(&mut p, &ports[0], &ports[1], vec![pt(0., 0.), pt(40., 0.)]);
    let route = p.connection_routes().next().unwrap().clone();
    let external = p
        .insert_representation(Representation::new(
            p.id(),
            page,
            RepresentationTarget::Connection(route.connection_id()),
            tr(0., 0.),
        ))
        .unwrap();
    let before = p.clone();
    assert!(
        p.edit_connection_drawing(ConnectionDrawingEdit::DeleteRoute { id: route.id() })
            .is_err()
    );
    assert_eq!(p, before);
    p.remove_representation(external).unwrap();
    let a = PhysicalTermination::new(
        p.connection_point(points[0])
            .unwrap()
            .terminal_id()
            .unwrap(),
        DesignationPart::new("A").unwrap(),
    );
    let b = PhysicalTermination::new(
        p.connection_point(points[1])
            .unwrap()
            .terminal_id()
            .unwrap(),
        DesignationPart::new("A").unwrap(),
    );
    let wire = physical_common::wire(&p, "W", &a, &b);
    let wire = p.insert_conductor(wire).unwrap();
    p.assign_conductor_connection(
        wire,
        ConductorConnection::new(route.connection_id(), points[0]),
    )
    .unwrap();
    let before = p.clone();
    assert!(
        p.edit_connection_drawing(ConnectionDrawingEdit::DeleteRoute { id: route.id() })
            .is_err()
    );
    assert_eq!(p, before);
    p.validate().unwrap();
}
#[test]
fn replay_after_out_of_band_endpoint_transform_is_rejected_atomically() {
    let (mut p, _, ports, _) = setup();
    let delta = draw(&mut p, &ports[0], &ports[1], vec![pt(0., 0.), pt(40., 0.)]);
    p.transform_representations(&[(ports[0].representation_id(), tr(0., 5.))])
        .unwrap();
    let before = p.clone();
    assert!(p.restore_connection_drawing(&delta, false).is_err());
    assert_eq!(p, before);
}
#[test]
fn junction_route_edit_removing_incidence_is_rejected_atomically() {
    let (mut p, page, ports, _) = setup();
    draw(&mut p, &ports[0], &ports[1], vec![pt(0., 0.), pt(40., 0.)]);
    let id = p.connection_routes().next().unwrap().id();
    draw(
        &mut p,
        &ports[2],
        &ports[3],
        vec![pt(20., -20.), pt(20., 20.)],
    );
    let routes = p.connection_routes().map(ConnectionRoute::id).collect();
    p.edit_connection_drawing(ConnectionDrawingEdit::CreateJunction {
        page,
        position: pt(20., 0.),
        routes,
    })
    .unwrap();
    let before = p.clone();
    assert!(
        p.edit_connection_drawing(ConnectionDrawingEdit::SetRoute {
            id,
            points: vec![pt(0., 0.), pt(0., 10.), pt(40., 10.), pt(40., 0.)]
        })
        .is_err()
    );
    assert_eq!(p, before);
}
#[test]
fn unassociated_occurrence_and_aliased_electrical_endpoint_cannot_bypass_validation() {
    let (mut p, page, ports, points) = setup();
    let def = p.symbol_definitions().next().unwrap().clone();
    let r = Representation::new_symbol(p.id(), page, &def, None, tr(100., 100.)).unwrap();
    let unassociated = PortRef::new(r.id(), "p").unwrap();
    p.insert_symbol_representation(r, def).unwrap();
    let before = p.clone();
    assert!(
        p.edit_connection_drawing(ConnectionDrawingEdit::BindPort {
            port: unassociated,
            point: Some(points[0])
        })
        .is_err()
    );
    assert_eq!(p, before);
    p.edit_connection_drawing(ConnectionDrawingEdit::BindPort {
        port: ports[1].clone(),
        point: Some(points[0]),
    })
    .unwrap();
    let before = p.clone();
    assert!(
        p.edit_connection_drawing(ConnectionDrawingEdit::CreateRoute {
            source: ports[0].clone(),
            target: ports[1].clone(),
            points: vec![pt(0., 0.), pt(40., 0.)]
        })
        .is_err()
    );
    assert_eq!(p, before);
}
#[test]
fn function_terminal_component_and_point_targets_bind_only_compatible_points() {
    let (mut p, page, _, points) = setup();
    let def = p.symbol_definitions().next().unwrap().clone();
    let point = p.connection_point(points[0]).unwrap().clone();
    let device = p.function(point.function_id()).unwrap().device_id();
    let component = p
        .insert_component(
            Component::new(p.id(), Designation::new("C").unwrap(), "test")
                .unwrap()
                .with_assignment(Some(device), Some(point.function_id())),
        )
        .unwrap();
    for target in [
        RepresentationTarget::Function(point.function_id()),
        RepresentationTarget::Terminal(point.terminal_id().unwrap()),
        RepresentationTarget::ConnectionPoint(point.id()),
        RepresentationTarget::Component(component),
    ] {
        let r = Representation::new_symbol(p.id(), page, &def, Some(target), tr(0., 0.)).unwrap();
        let port = PortRef::new(r.id(), "p").unwrap();
        p.insert_symbol_representation(r, def.clone()).unwrap();
        p.edit_connection_drawing(ConnectionDrawingEdit::BindPort {
            port: port.clone(),
            point: Some(points[0]),
        })
        .unwrap();
        if matches!(
            target,
            RepresentationTarget::Terminal(_) | RepresentationTarget::ConnectionPoint(_)
        ) {
            let before = p.clone();
            assert!(
                p.edit_connection_drawing(ConnectionDrawingEdit::BindPort {
                    port,
                    point: Some(points[1])
                })
                .is_err()
            );
            assert_eq!(p, before);
        }
    }
    p.validate().unwrap();
}
#[test]
fn tee_junction_joins_a_graphical_endpoint_to_an_independent_segment_explicitly() {
    let (mut p, page, ports, points) = setup();
    p.transform_representations(&[(ports[3].representation_id(), tr(20., 0.))])
        .unwrap();
    draw(&mut p, &ports[0], &ports[1], vec![pt(0., 0.), pt(40., 0.)]);
    draw(
        &mut p,
        &ports[2],
        &ports[3],
        vec![pt(20., -20.), pt(20., 0.)],
    );
    assert!(!p.are_connected(points[0], points[3]).unwrap());
    let routes = p.connection_routes().map(ConnectionRoute::id).collect();
    p.edit_connection_drawing(ConnectionDrawingEdit::CreateJunction {
        page,
        position: pt(20., 0.),
        routes,
    })
    .unwrap();
    assert!(p.are_connected(points[0], points[3]).unwrap());
    p.validate().unwrap();
}
#[test]
fn junction_snapshot_rejects_invalid_ids_positions_routes_and_auxiliary_ownership() {
    let (mut p, page, ports, _) = setup();
    draw(&mut p, &ports[0], &ports[1], vec![pt(0., 0.), pt(40., 0.)]);
    draw(
        &mut p,
        &ports[2],
        &ports[3],
        vec![pt(20., -20.), pt(20., 20.)],
    );
    let routes = p.connection_routes().map(ConnectionRoute::id).collect();
    p.edit_connection_drawing(ConnectionDrawingEdit::CreateJunction {
        page,
        position: pt(20., 0.),
        routes,
    })
    .unwrap();
    let original = json!(p);
    let copy: Project = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(copy, p);
    for (field, value) in [
        ("id", json!("bad")),
        ("page_id", json!(PageId::new())),
        ("position", json!(pt(21., 0.))),
        (
            "routes",
            json!([RepresentationId::new(), RepresentationId::new()]),
        ),
        ("connections", json!([])),
    ] {
        let mut bad = original.clone();
        bad["connection_junctions"][0][field] = value;
        assert!(
            serde_json::from_value::<Project>(bad).is_err(),
            "field {field}"
        );
    }
    let mut bad = original.clone();
    let first = bad["connection_junctions"][0].clone();
    bad["connection_junctions"]
        .as_array_mut()
        .unwrap()
        .push(first);
    assert!(serde_json::from_value::<Project>(bad).is_err());
    let mut bad = original;
    let owned = bad["connection_junctions"][0]["connections"][0].clone();
    bad["connection_junctions"][0]["connections"]
        .as_array_mut()
        .unwrap()
        .push(owned);
    assert!(serde_json::from_value::<Project>(bad).is_err());
}
#[test]
fn rotated_connected_symbol_updates_port_position_and_history_restores_exact_route() {
    let (mut p, _, ports, _) = setup();
    let def = SymbolDefinition::new(
        SymbolDefinitionId::new("offset").unwrap(),
        1,
        "Offset",
        "Test",
        vec![SymbolPrimitive::Line {
            start: pt(0., 0.),
            end: pt(4., 0.),
        }],
        vec![SymbolPort::new("p", pt(4., 0.), 0., "P", None).unwrap()],
    )
    .unwrap();
    let page = p
        .representation(ports[0].representation_id())
        .unwrap()
        .page_id();
    let target = p
        .representation(ports[0].representation_id())
        .unwrap()
        .target();
    let r = Representation::new_symbol(p.id(), page, &def, Some(target), tr(0., 0.)).unwrap();
    let source = PortRef::new(r.id(), "p").unwrap();
    p.insert_symbol_representation(r, def).unwrap();
    let binding = p.port_binding(&ports[0]);
    p.edit_connection_drawing(ConnectionDrawingEdit::BindPort {
        port: source.clone(),
        point: binding,
    })
    .unwrap();
    draw(&mut p, &source, &ports[1], vec![pt(4., 0.), pt(40., 0.)]);
    let before = p.clone();
    let delta = p
        .edit_connection_drawing(ConnectionDrawingEdit::Transform(vec![(
            source.representation_id(),
            Transform2D::new(pt(0., 0.), 90., false, false).unwrap(),
        )]))
        .unwrap();
    assert_eq!(p.port_position(&source).unwrap(), pt(0., 4.));
    assert_eq!(
        p.connection_routes().next().unwrap().points().first(),
        Some(&pt(0., 4.))
    );
    p.validate().unwrap();
    p.restore_connection_drawing(&delta, false).unwrap();
    assert_eq!(p, before);
}
#[test]
fn binding_candidates_are_sorted_compatible_and_missing_port_is_an_error() {
    let (mut p, page, ports, points) = setup();
    assert_eq!(p.port_binding_candidates(&ports[0]).unwrap(), points);
    let def = p.symbol_definitions().next().unwrap().clone();
    for (target, expected) in [
        (
            Some(RepresentationTarget::ConnectionPoint(points[2])),
            vec![points[2]],
        ),
        (None, vec![]),
    ] {
        let r = Representation::new_symbol(p.id(), page, &def, target, tr(1., 1.)).unwrap();
        let port = PortRef::new(r.id(), "p").unwrap();
        p.insert_symbol_representation(r, def.clone()).unwrap();
        assert_eq!(p.port_binding_candidates(&port).unwrap(), expected);
    }
    assert!(
        p.port_binding_candidates(&PortRef::new(RepresentationId::new(), "p").unwrap())
            .is_err()
    );
}
#[test]
fn route_preview_uses_exact_terminal_repair_and_common_translation_without_mutation() {
    let (mut p, _, ports, _) = setup();
    draw(
        &mut p,
        &ports[0],
        &ports[1],
        vec![pt(0., 0.), pt(0., 10.), pt(40., 10.), pt(40., 0.)],
    );
    let route = p.connection_routes().next().unwrap().clone();
    let before = p.clone();
    let preview = route
        .points_with_endpoints(pt(5., 0.), pt(40., 0.))
        .unwrap();
    assert_eq!(
        preview,
        vec![pt(5., 0.), pt(5., 10.), pt(40., 10.), pt(40., 0.)]
    );
    assert_eq!(p, before);
    let delta = p
        .transform_representations_with_drawing(&[(ports[0].representation_id(), tr(5., 0.))])
        .unwrap();
    assert_eq!(p.connection_route(route.id()).unwrap().points(), preview);
    p.restore_connection_drawing(&delta, false).unwrap();
    assert_eq!(p, before);
    let preview = route
        .points_with_endpoints(pt(-3., 7.), pt(37., 7.))
        .unwrap();
    assert_eq!(
        preview,
        vec![pt(-3., 7.), pt(-3., 17.), pt(37., 17.), pt(37., 7.)]
    );
    assert_eq!(p, before);
}

#[test]
fn horizontal_endpoint_repair_does_not_draw_a_residual_backtrack() {
    let (mut p, _, ports, _) = setup();
    let original = vec![pt(0., 0.), pt(0., 10.), pt(40., 10.), pt(40., 0.)];
    draw(&mut p, &ports[0], &ports[1], original.clone());
    let initial = p.clone();
    let id = p.connection_routes().next().unwrap().id();
    for x in [5., 20., 35., 5., 0.] {
        let delta = p
            .transform_representations_with_drawing(&[(ports[0].representation_id(), tr(x, 0.))])
            .unwrap();
        let route = p.connection_route(id).unwrap();
        if x != 0. {
            assert_eq!(
                route.points(),
                &[pt(x, 0.), pt(x, 10.), pt(40., 10.), pt(40., 0.)]
            );
        }
        let after = p.clone();
        p.restore_connection_drawing(&delta, false).unwrap();
        p.restore_connection_drawing(&delta, true).unwrap();
        assert_eq!(p, after);
    }
    assert_eq!(p, initial);
    assert_eq!(p.connection_route(id).unwrap().points(), original);
}

#[test]
fn rotated_rectangular_symbol_route_exits_its_port_without_crossing_body() {
    let (mut p, page, ports, points) = setup();
    let body = Rect2D::new(pt(-10., -5.), Size2D::new(20., 10.).unwrap()).unwrap();
    let def = SymbolDefinition::new(
        SymbolDefinitionId::new("route.body").unwrap(),
        1,
        "Body",
        "Test",
        vec![SymbolPrimitive::Rectangle { rect: body }],
        vec![SymbolPort::new("right", pt(10., 0.), 0., "R", None).unwrap()],
    )
    .unwrap();
    let r = Representation::new_symbol(
        p.id(),
        page,
        &def,
        Some(RepresentationTarget::ConnectionPoint(points[0])),
        tr(0., 0.),
    )
    .unwrap();
    let source = PortRef::new(r.id(), "right").unwrap();
    p.insert_symbol_representation(r, def).unwrap();
    p.edit_connection_drawing(ConnectionDrawingEdit::BindPort {
        port: source.clone(),
        point: Some(points[0]),
    })
    .unwrap();
    draw(&mut p, &source, &ports[1], vec![pt(10., 0.), pt(40., 0.)]);
    let before = p.clone();
    let delta = p
        .transform_representations_with_drawing(&[(
            source.representation_id(),
            Transform2D::new(pt(0., 0.), 180., false, false).unwrap(),
        )])
        .unwrap();
    let route = p.connection_routes().next().unwrap();
    assert_eq!(route.points()[0], pt(-10., 0.));
    assert_eq!(*route.points().last().unwrap(), pt(40., 0.));
    assert!(route.points().len() > 2);
    // The body interior is (-10,10) × (-5,5). Each axis-aligned
    // segment must stay outside this open rectangle.
    for pair in route.points().windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let crosses = if a.y() == b.y() {
            a.y() > -5. && a.y() < 5. && a.x().min(b.x()) < 10. && a.x().max(b.x()) > -10.
        } else {
            a.x() > -10. && a.x() < 10. && a.y().min(b.y()) < 5. && a.y().max(b.y()) > -5.
        };
        assert!(!crosses, "route crosses symbol body: {a:?} -> {b:?}");
    }
    let after = p.clone();
    p.restore_connection_drawing(&delta, false).unwrap();
    assert_eq!(p, before);
    p.restore_connection_drawing(&delta, true).unwrap();
    assert_eq!(p, after);
    assert_eq!(p.networks(), before.networks());
}

#[test]
fn orthogonal_rotations_and_mirrors_keep_port_exit_outside_rectangular_body() {
    for (angle, mx, my) in [
        (0., false, false),
        (90., false, false),
        (180., false, false),
        (270., false, false),
        (0., true, false),
        (0., false, true),
        (90., true, false),
        (90., false, true),
    ] {
        let (mut p, page, ports, points) = setup();
        let body = Rect2D::new(pt(-10., -5.), Size2D::new(20., 10.).unwrap()).unwrap();
        let port = SymbolPort::new("exit", pt(10., 0.), 0., "E", None).unwrap();
        let def = SymbolDefinition::new(
            SymbolDefinitionId::new("route.orientations").unwrap(),
            1,
            "Body",
            "Test",
            vec![SymbolPrimitive::Rectangle { rect: body }],
            vec![port.clone()],
        )
        .unwrap();
        let t = Transform2D::new(pt(0., 0.), angle, mx, my).unwrap();
        let transformed = port.transformed(t).unwrap();
        let start = transformed.position();
        let (dx, dy) = match transformed.orientation_degrees() {
            0. => (1., 0.),
            90. => (0., 1.),
            180. => (-1., 0.),
            270. => (0., -1.),
            value => panic!("unexpected direction {value}"),
        };
        let end = pt(start.x() - 50. * dx, start.y() - 50. * dy);
        p.transform_representations(&[(ports[1].representation_id(), tr(end.x(), end.y()))])
            .unwrap();
        let r = Representation::new_symbol(
            p.id(),
            page,
            &def,
            Some(RepresentationTarget::ConnectionPoint(points[0])),
            t,
        )
        .unwrap();
        let source = PortRef::new(r.id(), "exit").unwrap();
        p.insert_symbol_representation(r, def).unwrap();
        p.edit_connection_drawing(ConnectionDrawingEdit::BindPort {
            port: source.clone(),
            point: Some(points[0]),
        })
        .unwrap();
        draw(&mut p, &source, &ports[1], vec![start, end]);
        let route = p.connection_routes().next().unwrap();
        let outgoing = route.points()[1];
        assert!(
            (outgoing.x() - start.x()) * dx + (outgoing.y() - start.y()) * dy > 0.,
            "route does not leave transformed port: angle={angle} mx={mx} my={my}"
        );
        let bounds = t.transform_bounds(body).unwrap();
        let min = bounds.origin();
        let max = bounds.max();
        for pair in route.points().windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let crosses = if a.y() == b.y() {
                a.y() > min.y()
                    && a.y() < max.y()
                    && a.x().min(b.x()) < max.x()
                    && a.x().max(b.x()) > min.x()
            } else {
                a.x() > min.x()
                    && a.x() < max.x()
                    && a.y().min(b.y()) < max.y()
                    && a.y().max(b.y()) > min.y()
            };
            assert!(!crosses, "body crossing: angle={angle} mx={mx} my={my}");
        }
        p.validate().unwrap();
        let loaded: Project = serde_json::from_value(json!(p)).unwrap();
        assert_eq!(loaded, p);
    }
}

#[test]
fn route_detours_around_nearby_closed_symbol_without_electrical_change() {
    let (mut p, page, ports, _) = setup();
    let body = Rect2D::new(pt(-4., -4.), Size2D::new(8., 8.).unwrap()).unwrap();
    let def = SymbolDefinition::new(
        SymbolDefinitionId::new("route.nearby").unwrap(),
        1,
        "Nearby",
        "Test",
        vec![SymbolPrimitive::Rectangle { rect: body }],
        vec![],
    )
    .unwrap();
    let symbol = Representation::new_symbol(p.id(), page, &def, None, tr(20., 0.)).unwrap();
    p.insert_symbol_representation(symbol, def).unwrap();
    draw(&mut p, &ports[0], &ports[1], vec![pt(0., 0.), pt(40., 0.)]);
    let route = p.connection_routes().next().unwrap();
    assert!(route.points().len() > 2);
    for pair in route.points().windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let crosses = if a.y() == b.y() {
            a.y() > -4. && a.y() < 4. && a.x().min(b.x()) < 24. && a.x().max(b.x()) > 16.
        } else {
            a.x() > 16. && a.x() < 24. && a.y().min(b.y()) < 4. && a.y().max(b.y()) > -4.
        };
        assert!(!crosses);
    }
    assert_eq!(p.connections().count(), 1);
    assert_eq!(p.conductors().count(), 0);
    p.validate().unwrap();
}

#[test]
fn moving_unconnected_symbol_into_route_recomputes_clearance_and_undo() {
    let (mut p, page, ports, _) = setup();
    let body = Rect2D::new(pt(-4., -4.), Size2D::new(8., 8.).unwrap()).unwrap();
    let def = SymbolDefinition::new(
        SymbolDefinitionId::new("route.moving-obstacle").unwrap(),
        1,
        "Moving obstacle",
        "Test",
        vec![SymbolPrimitive::Rectangle { rect: body }],
        vec![],
    )
    .unwrap();
    let symbol = Representation::new_symbol(p.id(), page, &def, None, tr(20., 20.)).unwrap();
    let symbol_id = symbol.id();
    p.insert_symbol_representation(symbol, def).unwrap();
    draw(&mut p, &ports[0], &ports[1], vec![pt(0., 0.), pt(40., 0.)]);
    let before = p.clone();
    let delta = p
        .transform_representations_with_drawing(&[(symbol_id, tr(20., 0.))])
        .unwrap();
    assert!(p.connection_routes().next().unwrap().points().len() > 2);
    assert_eq!(p.networks(), before.networks());
    p.restore_connection_drawing(&delta, false).unwrap();
    assert_eq!(p, before);
    p.restore_connection_drawing(&delta, true).unwrap();
    p.validate().unwrap();
}

#[test]
fn manually_set_route_is_cleared_around_existing_symbol_body() {
    let (mut p, page, ports, _) = setup();
    let body = Rect2D::new(pt(-4., -4.), Size2D::new(8., 8.).unwrap()).unwrap();
    let def = SymbolDefinition::new(
        SymbolDefinitionId::new("route.manual-obstacle").unwrap(),
        1,
        "Body",
        "Test",
        vec![SymbolPrimitive::Rectangle { rect: body }],
        vec![],
    )
    .unwrap();
    p.insert_symbol_representation(
        Representation::new_symbol(p.id(), page, &def, None, tr(20., 0.)).unwrap(),
        def,
    )
    .unwrap();
    draw(
        &mut p,
        &ports[0],
        &ports[1],
        vec![pt(0., 0.), pt(0., 20.), pt(40., 20.), pt(40., 0.)],
    );
    let id = p.connection_routes().next().unwrap().id();
    p.edit_connection_drawing(ConnectionDrawingEdit::SetRoute {
        id,
        points: vec![pt(0., 0.), pt(40., 0.)],
    })
    .unwrap();
    assert!(p.connection_route(id).unwrap().points().len() > 2);
    p.validate().unwrap();
}
#[test]
fn route_endpoint_preview_refuses_finite_translation_that_collapses_geometry() {
    let (mut p, _, ports, _) = setup();
    draw(&mut p, &ports[0], &ports[1], vec![pt(0., 0.), pt(40., 0.)]);
    let route = p.connection_routes().next().unwrap();
    assert!(
        route
            .points_with_endpoints(pt(1e308, 0.), pt(1e308, 0.))
            .is_err()
    );
}

/// Reusing the last rendered polyline would leave generated terminal bends in
/// the final route. The user's initial constraint points must instead survive.
#[test]
fn connected_endpoint_resize_cycles_restore_manual_geometry_on_both_axes() {
    for vertical in [false, true] {
        let (mut p, _, ports, _) = setup();
        let path = if vertical {
            p.transform_representations(&[(ports[1].representation_id(), tr(0., 40.))])
                .unwrap();
            vec![
                pt(0., 0.),
                pt(10., 0.),
                pt(10., 20.),
                pt(10., 40.),
                pt(0., 40.),
            ]
        } else {
            vec![
                pt(0., 0.),
                pt(0., 10.),
                pt(20., 10.),
                pt(40., 10.),
                pt(40., 0.),
            ]
        };
        draw(&mut p, &ports[0], &ports[1], path.clone());
        let initial = p.clone();
        for offset in [40., 20., 0., 40., 0.] {
            let t = if vertical {
                tr(0., 40. + offset)
            } else {
                tr(40. + offset, 0.)
            };
            let before = p.clone();
            let delta = p
                .transform_representations_with_drawing(&[(ports[1].representation_id(), t)])
                .unwrap();
            let after = p.clone();
            p.restore_connection_drawing(&delta, false).unwrap();
            assert_eq!(p, before);
            p.restore_connection_drawing(&delta, true).unwrap();
            assert_eq!(p, after);
            assert_eq!(
                p.connections().collect::<Vec<_>>(),
                initial.connections().collect::<Vec<_>>()
            );
            assert_eq!(p.networks(), initial.networks());
            assert_eq!(
                p.conductors().collect::<Vec<_>>(),
                initial.conductors().collect::<Vec<_>>()
            );
            p.validate().unwrap();
            if offset == 0. {
                assert_eq!(p.connection_routes().next().unwrap().points(), path);
                assert_eq!(p, initial);
            }
        }
    }
}

#[test]
fn straight_route_combined_endpoint_moves_do_not_accumulate_generated_bends() {
    let (mut p, _, ports, _) = setup();
    draw(&mut p, &ports[0], &ports[1], vec![pt(0., 0.), pt(40., 0.)]);
    let initial = p.clone();
    for target in [tr(80., 40.), tr(60., 20.), tr(40., 0.)] {
        p.transform_representations_with_drawing(&[(ports[1].representation_id(), target)])
            .unwrap();
        p.validate().unwrap();
    }
    assert_eq!(p, initial);
}

#[test]
fn endpoint_cycle_after_snapshot_import_keeps_the_original_routing_constraints() {
    let (mut p, _, ports, _) = setup();
    draw(&mut p, &ports[0], &ports[1], vec![pt(0., 0.), pt(40., 0.)]);
    let initial = p.clone();
    p.transform_representations_with_drawing(&[(ports[1].representation_id(), tr(80., 40.))])
        .unwrap();
    let mut copy: Project = serde_json::from_value(json!(p)).unwrap();
    copy.transform_representations_with_drawing(&[(ports[1].representation_id(), tr(40., 0.))])
        .unwrap();
    assert_eq!(copy, initial);
}

#[test]
fn route_resize_cycle_preserves_manual_junction_incidence_and_electrical_graph() {
    let (mut p, page, ports, points) = setup();
    draw(
        &mut p,
        &ports[0],
        &ports[1],
        vec![
            pt(0., 0.),
            pt(0., 10.),
            pt(20., 10.),
            pt(40., 10.),
            pt(40., 0.),
        ],
    );
    draw(
        &mut p,
        &ports[2],
        &ports[3],
        vec![pt(20., -20.), pt(20., 20.)],
    );
    let routes = p.connection_routes().map(ConnectionRoute::id).collect();
    p.edit_connection_drawing(ConnectionDrawingEdit::CreateJunction {
        page,
        position: pt(20., 10.),
        routes,
    })
    .unwrap();
    let initial = p.clone();
    for x in [80., 60., 40.] {
        p.transform_representations_with_drawing(&[(ports[1].representation_id(), tr(x, 0.))])
            .unwrap();
        assert!(p.are_connected(points[0], points[3]).unwrap());
        assert_eq!(
            p.connection_junctions().collect::<Vec<_>>(),
            initial.connection_junctions().collect::<Vec<_>>()
        );
        p.validate().unwrap();
    }
    assert_eq!(p, initial);
}

#[test]
fn quarter_turns_then_vertical_resize_return_to_the_same_route_and_physical_wiring() {
    let (mut p, page, ports, points) = setup();
    let def = SymbolDefinition::new(
        SymbolDefinitionId::new("offset-cycle").unwrap(),
        1,
        "Offset",
        "Test",
        vec![SymbolPrimitive::Line {
            start: pt(0., 0.),
            end: pt(4., 0.),
        }],
        vec![SymbolPort::new("p", pt(4., 0.), 0., "P", None).unwrap()],
    )
    .unwrap();
    let target = p
        .representation(ports[0].representation_id())
        .unwrap()
        .target();
    let r = Representation::new_symbol(p.id(), page, &def, Some(target), tr(0., 0.)).unwrap();
    let source = PortRef::new(r.id(), "p").unwrap();
    p.insert_symbol_representation(r, def).unwrap();
    p.edit_connection_drawing(ConnectionDrawingEdit::BindPort {
        port: source.clone(),
        point: Some(points[0]),
    })
    .unwrap();
    draw(&mut p, &source, &ports[1], vec![pt(4., 0.), pt(40., 0.)]);
    let edge = p.connection_routes().next().unwrap().connection_id();
    let ends: Vec<_> = points
        .iter()
        .take(2)
        .map(|id| {
            PhysicalTermination::new(
                p.connection_point(*id).unwrap().terminal_id().unwrap(),
                DesignationPart::new("A").unwrap(),
            )
        })
        .collect();
    let wire = p
        .insert_conductor(physical_common::wire(&p, "W", &ends[0], &ends[1]))
        .unwrap();
    p.assign_conductor_connection(wire, ConductorConnection::new(edge, points[0]))
        .unwrap();
    let initial = p.clone();
    for angle in [90., 180., 270.] {
        let t = Transform2D::new(pt(0., 0.), angle, false, false).unwrap();
        p.transform_representations_with_drawing(&[(source.representation_id(), t)])
            .unwrap();
        let rotated = p.clone();
        for y in [40., 20., 0.] {
            let t = Transform2D::new(pt(0., y), angle, false, false).unwrap();
            let before = p.clone();
            let delta = p
                .transform_representations_with_drawing(&[(source.representation_id(), t)])
                .unwrap();
            let after = p.clone();
            assert_eq!(p.conductor(wire), initial.conductor(wire));
            assert_eq!(p.networks(), initial.networks());
            p.restore_connection_drawing(&delta, false).unwrap();
            assert_eq!(p, before);
            p.restore_connection_drawing(&delta, true).unwrap();
            assert_eq!(p, after);
        }
        assert_eq!(p, rotated);
        p.transform_representations_with_drawing(&[(source.representation_id(), tr(0., 0.))])
            .unwrap();
        assert_eq!(p, initial);
    }
}

#[test]
fn group_translation_after_endpoint_resize_keeps_authored_constraints_reversible() {
    let (mut p, _, ports, _) = setup();
    draw(
        &mut p,
        &ports[0],
        &ports[1],
        vec![pt(0., 0.), pt(0., 10.), pt(40., 10.), pt(40., 0.)],
    );
    let initial = p.clone();
    p.transform_representations_with_drawing(&[(ports[1].representation_id(), tr(80., 20.))])
        .unwrap();
    let resized = p.clone();
    for offset in [40., 20., 0.] {
        p.transform_representations_with_drawing(&[
            (ports[0].representation_id(), tr(offset, offset)),
            (ports[1].representation_id(), tr(80. + offset, 20. + offset)),
        ])
        .unwrap();
    }
    assert_eq!(p, resized);
    p.transform_representations_with_drawing(&[(ports[1].representation_id(), tr(40., 0.))])
        .unwrap();
    assert_eq!(p, initial);
}

#[test]
fn legacy_route_snapshot_defaults_and_malformed_routing_base_are_checked() {
    let (mut p, _, ports, _) = setup();
    draw(&mut p, &ports[0], &ports[1], vec![pt(0., 0.), pt(40., 0.)]);
    let mut legacy = json!(p);
    legacy["connection_routes"][0]
        .as_object_mut()
        .unwrap()
        .remove("routing_base");
    assert_eq!(
        serde_json::from_value::<Project>(legacy.clone()).unwrap(),
        p
    );
    for base in [
        json!([]),
        json!([pt(0., 0.), pt(2., 2.)]),
        json!([pt(0., 0.), pt(0., 10.), pt(40., 10.), pt(40., 0.)]),
    ] {
        let mut bad = legacy.clone();
        bad["connection_routes"][0]["routing_base"] = base;
        assert!(serde_json::from_value::<Project>(bad).is_err());
    }
}

#[test]
fn clicked_waypoints_survive_while_automatic_elbows_follow_current_endpoints() {
    let (mut p, _, ports, _) = setup();
    let intent = RouteIntent::new(
        vec![pt(0., 0.), pt(20., 20.), pt(40., 0.)],
        vec![true, false],
    )
    .unwrap();
    p.edit_connection_drawing(ConnectionDrawingEdit::CreateRouteWithIntent {
        source: ports[0].clone(),
        target: ports[1].clone(),
        intent,
    })
    .unwrap();
    let initial = p.clone();
    for x in [80., 60., 40.] {
        p.transform_representations(&[(ports[1].representation_id(), tr(x, 0.))])
            .unwrap();
        let route = p.connection_routes().next().unwrap();
        assert_eq!(
            route.points(),
            &[
                pt(0., 0.),
                pt(20., 0.),
                pt(20., 20.),
                pt(20., 0.),
                pt(x, 0.)
            ]
        );
        p.validate().unwrap();
    }
    assert_eq!(p, initial);
    let mut imported: Project = serde_json::from_value(json!(p)).unwrap();
    imported
        .transform_representations(&[(ports[0].representation_id(), tr(-40., -40.))])
        .unwrap();
    imported
        .transform_representations(&[(ports[0].representation_id(), tr(0., 0.))])
        .unwrap();
    assert_eq!(imported, initial);
}

#[test]
fn malformed_intent_and_snapshot_disagreement_are_rejected_without_mutation() {
    assert!(RouteIntent::new(vec![pt(0., 0.), pt(40., 40.)], vec![]).is_err());
    assert!(RouteIntent::new(vec![pt(0., 0.), pt(0., 0.)], vec![true]).is_err());
    let (mut p, _, ports, _) = setup();
    p.edit_connection_drawing(ConnectionDrawingEdit::CreateRouteWithIntent {
        source: ports[0].clone(),
        target: ports[1].clone(),
        intent: RouteIntent::new(vec![pt(0., 0.), pt(40., 0.)], vec![true]).unwrap(),
    })
    .unwrap();
    let mut data = json!(p);
    data["connection_routes"][0]["routing_intent"]["anchors"] = json!([pt(0., 0.), pt(40., 40.)]);
    assert!(serde_json::from_value::<Project>(data).is_err());
    let before = p.clone();
    assert!(
        p.edit_connection_drawing(ConnectionDrawingEdit::CreateRouteWithIntent {
            source: ports[0].clone(),
            target: ports[2].clone(),
            intent: RouteIntent::new(vec![pt(0., 0.), pt(40., 40.)], vec![true]).unwrap(),
        })
        .is_err()
    );
    assert_eq!(p, before);
}
