// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
mod common;
use cad_craft::{domain::*, error::DomainError};
use common::{connect, fixture};
use std::collections::BTreeSet;

fn sorted<T: Ord>(mut values: Vec<T>) -> Vec<T> {
    values.sort();
    values
}

#[test]
fn empty_project_and_isolated_points_have_distinct_networks() {
    assert!(Project::new("Empty").unwrap().networks().is_empty());
    let (p, ids) = fixture(3);
    let nets = p.networks();
    assert_eq!(nets.len(), 3);
    assert_eq!(
        nets.iter()
            .flat_map(|n| n.points())
            .copied()
            .collect::<Vec<_>>(),
        sorted(ids.clone())
    );
    for id in ids {
        let n = p.network_for_point(id).unwrap();
        assert_eq!(n.points(), &[id]);
        assert!(n.connections().is_empty());
        assert!(!n.has_cycle());
        assert!(p.are_connected(id, id).unwrap());
        assert!(p.directly_connected_points(id).unwrap().is_empty());
        assert!(p.incident_connections(id).unwrap().is_empty());
    }
}

#[test]
fn simple_pair_forms_one_network_with_its_explicit_edge() {
    let (mut p, ids) = fixture(2);
    let edge = connect(&mut p, ids[0], ids[1]);
    let networks = p.networks();
    assert_eq!(networks.len(), 1);
    assert_eq!(networks[0].points(), sorted(ids.clone()));
    assert_eq!(networks[0].connections(), &[edge]);
    assert!(!networks[0].has_cycle());
    assert!(p.are_connected(ids[1], ids[0]).unwrap());
    assert_eq!(p.network_for_point(ids[1]).unwrap(), networks[0]);
}

#[test]
fn chain_has_transitive_connectivity_but_only_direct_neighbors_and_incidence() {
    let (mut p, ids) = fixture(4);
    let edges: Vec<_> = ids
        .windows(2)
        .map(|ab| connect(&mut p, ab[0], ab[1]))
        .collect();
    assert!(p.are_connected(ids[0], ids[3]).unwrap());
    assert_eq!(p.networks().len(), 1);
    assert_eq!(p.networks()[0].points(), sorted(ids.clone()));
    assert_eq!(
        p.directly_connected_points(ids[1]).unwrap(),
        sorted(vec![ids[0], ids[2]])
    );
    assert_eq!(
        p.incident_connections(ids[1]).unwrap(),
        sorted(vec![edges[0], edges[1]])
    );
    assert!(!p.networks()[0].has_cycle());
}

#[test]
fn branching_and_independent_networks_keep_isolated_point_separate() {
    let (mut p, ids) = fixture(7);
    for (a, b) in [(0, 1), (1, 2), (1, 3), (4, 5)] {
        connect(&mut p, ids[a], ids[b]);
    }
    assert_eq!(p.networks().len(), 3);
    assert_eq!(
        p.network_for_point(ids[0]).unwrap().points(),
        sorted(ids[..4].to_vec())
    );
    assert_eq!(
        p.network_for_point(ids[4]).unwrap().points(),
        sorted(ids[4..6].to_vec())
    );
    assert_eq!(p.network_for_point(ids[6]).unwrap().points(), &[ids[6]]);
    assert!(!p.are_connected(ids[0], ids[4]).unwrap());
    assert_eq!(
        p.directly_connected_points(ids[1]).unwrap(),
        sorted(vec![ids[0], ids[2], ids[3]])
    );
}

#[test]
fn independent_pairs_merge_and_split_as_edges_change() {
    let (mut p, ids) = fixture(4);
    connect(&mut p, ids[0], ids[1]);
    connect(&mut p, ids[2], ids[3]);
    let separate = p.networks();
    assert_eq!(separate.len(), 2);
    let bridge = connect(&mut p, ids[1], ids[2]);
    assert_eq!(p.networks().len(), 1);
    p.remove_connection(bridge).unwrap();
    assert_eq!(p.networks(), separate);
    p.validate().unwrap();
}

#[test]
fn removing_bridge_preserves_pair_and_isolates_last_point() {
    let (mut p, ids) = fixture(3);
    let ab = connect(&mut p, ids[0], ids[1]);
    let bc = connect(&mut p, ids[1], ids[2]);
    let previous = p.network_for_point(ids[0]).unwrap();
    p.remove_connection(bc).unwrap();
    assert_eq!(previous.points().len(), 3); // Resultado possuído é uma fotografia.
    assert_eq!(p.networks().len(), 2);
    assert_eq!(p.network_for_point(ids[0]).unwrap().connections(), &[ab]);
    assert_eq!(p.network_for_point(ids[2]).unwrap().points(), &[ids[2]]);
    assert!(!p.are_connected(ids[0], ids[2]).unwrap());
    assert_eq!(p.directly_connected_points(ids[1]).unwrap(), vec![ids[0]]);
    assert!(p.incident_connections(ids[2]).unwrap().is_empty());
    p.validate().unwrap();
}

#[test]
fn cycle_addition_is_valid_and_removing_cycle_edge_keeps_points_connected() {
    let (mut p, ids) = fixture(3);
    connect(&mut p, ids[0], ids[1]);
    connect(&mut p, ids[1], ids[2]);
    let closing = connect(&mut p, ids[2], ids[0]);
    let cycle = p.networks();
    assert_eq!(cycle.len(), 1);
    assert_eq!(cycle[0].points(), sorted(ids.clone()));
    assert_eq!(cycle[0].connections().len(), 3);
    assert!(cycle[0].has_cycle());
    p.remove_connection(closing).unwrap();
    assert!(p.are_connected(ids[0], ids[2]).unwrap());
    assert!(!p.networks()[0].has_cycle());
    p.validate().unwrap();
}

#[test]
fn nonexistent_point_queries_return_errors_including_reflexive_queries() {
    let (p, ids) = fixture(1);
    let missing = ConnectionPointId::new();
    let error = DomainError::MissingEntity(EntityRef::ConnectionPoint(missing));
    assert_eq!(p.are_connected(missing, missing), Err(error.clone()));
    assert_eq!(p.are_connected(ids[0], missing), Err(error.clone()));
    assert_eq!(p.are_connected(missing, ids[0]), Err(error.clone()));
    assert_eq!(p.network_for_point(missing), Err(error.clone()));
    assert_eq!(p.directly_connected_points(missing), Err(error.clone()));
    assert_eq!(p.incident_connections(missing), Err(error));
}

#[test]
fn ownership_and_shared_terminal_never_create_implicit_electrical_edges() {
    let (mut p, ids) = fixture(2);
    let device = p.devices().next().unwrap().id();
    let function = p.functions().next().unwrap().id();
    let aux = p
        .insert_function(Function::new(
            p.id(),
            device,
            Designation::new("K1").unwrap(),
            FunctionKind::AuxiliaryContact,
        ))
        .unwrap();
    let terminal = p
        .insert_terminal(Terminal::new(
            p.id(),
            TerminalOwner::Device(device),
            DesignationPart::new("13").unwrap(),
        ))
        .unwrap();
    let mut more = Vec::new();
    for f in [function, aux] {
        more.push(
            p.insert_connection_point(
                ConnectionPoint::new(p.id(), f, DesignationPart::new("13").unwrap())
                    .with_terminal(terminal),
            )
            .unwrap(),
        );
    }
    assert_eq!(p.networks().len(), 4);
    assert!(!p.are_connected(ids[0], ids[1]).unwrap());
    assert!(!p.are_connected(more[0], more[1]).unwrap());
    connect(&mut p, ids[0], more[0]);
    assert_eq!(p.networks().len(), 3);
    assert!(!p.are_connected(ids[0], more[1]).unwrap());
}

#[test]
fn networks_report_devices_functions_and_explicitly_associated_components() {
    let (mut p, ids) = fixture(2);
    let d1 = p.devices().next().unwrap().id();
    let f1 = p.functions().next().unwrap().id();
    let d2 = p
        .insert_device(Device::new(p.id(), Designation::new("K2").unwrap(), "relay").unwrap())
        .unwrap();
    let f2 = p
        .insert_function(Function::new(
            p.id(),
            d2,
            Designation::new("K2").unwrap(),
            FunctionKind::Coil,
        ))
        .unwrap();
    let auxiliary = p
        .insert_function(Function::new(
            p.id(),
            d1,
            Designation::new("K1").unwrap(),
            FunctionKind::AuxiliaryContact,
        ))
        .unwrap();
    let assigned = p
        .insert_component(
            Component::new(p.id(), Designation::new("K1").unwrap(), "coil")
                .unwrap()
                .with_assignment(Some(d1), Some(f1)),
        )
        .unwrap();
    let terminal_owner = p
        .insert_component(
            Component::new(p.id(), Designation::new("K2").unwrap(), "relay")
                .unwrap()
                .with_assignment(Some(d2), None),
        )
        .unwrap();
    let unrelated = p
        .insert_component(
            Component::new(p.id(), Designation::new("K1").unwrap(), "aux")
                .unwrap()
                .with_assignment(Some(d1), Some(auxiliary)),
        )
        .unwrap();
    let device_only = p
        .insert_component(
            Component::new(p.id(), Designation::new("K1").unwrap(), "generic")
                .unwrap()
                .with_assignment(Some(d1), None),
        )
        .unwrap();
    let terminal = p
        .insert_terminal(Terminal::new(
            p.id(),
            TerminalOwner::Component(terminal_owner),
            DesignationPart::new("A1").unwrap(),
        ))
        .unwrap();
    let other = p
        .insert_connection_point(
            ConnectionPoint::new(p.id(), f2, DesignationPart::new("A1").unwrap())
                .with_terminal(terminal),
        )
        .unwrap();
    assert!(!p.are_devices_connected(d1, d2).unwrap());
    let edge = connect(&mut p, ids[0], other);
    let n = p.network_for_point(ids[0]).unwrap();
    assert_eq!(n.devices(), sorted(vec![d1, d2]));
    assert_eq!(n.functions(), sorted(vec![f1, f2]));
    assert_eq!(n.components(), sorted(vec![assigned, terminal_owner]));
    assert!(!n.components().contains(&unrelated));
    assert!(!n.components().contains(&device_only));
    assert!(p.are_devices_connected(d2, d1).unwrap());
    assert!(!p.are_connected(ids[0], ids[1]).unwrap());
    p.remove_connection(edge).unwrap();
    assert!(!p.are_devices_connected(d1, d2).unwrap());
}

#[test]
fn device_queries_validate_ids_and_handle_devices_without_points() {
    let (mut p, _) = fixture(1);
    let with_points = p.devices().next().unwrap().id();
    let empty = p
        .insert_device(Device::new(p.id(), Designation::new("K2").unwrap(), "relay").unwrap())
        .unwrap();
    assert!(!p.are_devices_connected(empty, with_points).unwrap());
    assert!(!p.are_devices_connected(empty, empty).unwrap());
    assert!(p.are_devices_connected(with_points, with_points).unwrap());
    let missing = DeviceId::new();
    for (a, b) in [(missing, empty), (empty, missing), (missing, missing)] {
        assert_eq!(
            p.are_devices_connected(a, b),
            Err(DomainError::MissingEntity(EntityRef::Device(missing)))
        );
    }
}

#[test]
fn ordering_and_derived_networks_survive_reconstruction_and_removal() {
    let (mut p, ids) = fixture(6);
    let edges: Vec<_> = [(0, 2), (2, 4), (4, 0), (1, 3)]
        .into_iter()
        .map(|(a, b)| connect(&mut p, ids[a], ids[b]))
        .collect();
    let networks = p.networks();
    let mut data = serde_json::to_value(&p).unwrap();
    for key in ["connections", "connection_points"] {
        data[key].as_array_mut().unwrap().reverse();
    }
    let mut loaded: Project = serde_json::from_value(data).unwrap();
    assert_eq!(loaded.networks(), networks);
    assert!(
        networks
            .windows(2)
            .all(|ab| ab[0].points()[0] < ab[1].points()[0])
    );
    for n in &networks {
        assert!(n.points().windows(2).all(|ab| ab[0] < ab[1]));
        assert!(n.connections().windows(2).all(|ab| ab[0] < ab[1]));
        assert_eq!(
            n.points().iter().collect::<BTreeSet<_>>().len(),
            n.points().len()
        );
    }
    p.remove_connection(edges[0]).unwrap();
    loaded.remove_connection(edges[0]).unwrap();
    assert_eq!(loaded.networks(), p.networks());
    assert!(loaded.are_connected(ids[0], ids[2]).unwrap());
    loaded.validate().unwrap();
}

#[test]
fn all_small_graphs_match_independent_transitive_closure_before_and_after_removal() {
    let (base, ids) = fixture(4);
    let pairs = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
    for mask in 0..64 {
        let mut p = base.clone();
        let mut edges = Vec::new();
        let mut matrix = [[false; 4]; 4];
        for (bit, &(a, b)) in pairs.iter().enumerate() {
            if mask & (1 << bit) != 0 {
                edges.push((connect(&mut p, ids[a], ids[b]), a, b));
                matrix[a][b] = true;
                matrix[b][a] = true;
            }
        }
        loop {
            let mut closure = matrix;
            for (i, row) in closure.iter_mut().enumerate() {
                row[i] = true;
            }
            for k in 0..4 {
                for i in 0..4 {
                    for j in 0..4 {
                        closure[i][j] |= closure[i][k] && closure[k][j];
                    }
                }
            }
            let expected_count = (0..4).filter(|&i| !(0..i).any(|j| closure[i][j])).count();
            let networks = p.networks();
            assert_eq!(networks.len(), expected_count);
            for i in 0..4 {
                for j in 0..4 {
                    assert_eq!(
                        p.are_connected(ids[i], ids[j]).unwrap(),
                        closure[i][j],
                        "mask {mask}"
                    );
                }
            }
            for network in networks {
                let members: Vec<_> = (0..4)
                    .filter(|&i| network.points().contains(&ids[i]))
                    .collect();
                let edge_count = members
                    .iter()
                    .map(|&i| members.iter().filter(|&&j| matrix[i][j]).count())
                    .sum::<usize>()
                    / 2;
                assert_eq!(network.connections().len(), edge_count);
                assert_eq!(network.has_cycle(), edge_count >= members.len());
            }
            p.validate().unwrap();
            let Some((edge, a, b)) = edges.pop() else {
                break;
            };
            p.remove_connection(edge).unwrap();
            matrix[a][b] = false;
            matrix[b][a] = false;
        }
    }
}
