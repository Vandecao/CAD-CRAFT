// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

mod common;
use cad_craft::domain::*;
use common::{connect, fixture};
use std::time::{Duration, Instant};

fn median_ms(mut samples: Vec<Duration>) -> f64 {
    samples.sort();
    samples[samples.len() / 2].as_secs_f64() * 1000.0
}

#[test]
fn growing_chains_traverse_iteratively_split_and_reconstruct() {
    for size in [100, 1_000, 10_000] {
        let mut builds = Vec::new();
        let mut derives = Vec::new();
        let mut removals = Vec::new();
        let mut splits = Vec::new();
        let mut imports = Vec::new();
        for _ in 0..3 {
            let start = Instant::now();
            let (mut p, ids) = fixture(size);
            let edges: Vec<_> = ids
                .windows(2)
                .map(|ab| connect(&mut p, ab[0], ab[1]))
                .collect();
            builds.push(start.elapsed());
            let start = Instant::now();
            let networks = p.networks();
            derives.push(start.elapsed());
            assert_eq!(networks.len(), 1);
            assert_eq!(networks[0].points().len(), size);
            assert_eq!(networks[0].connections().len(), size - 1);
            assert!(!networks[0].has_cycle());
            assert!(p.are_connected(ids[0], ids[size - 1]).unwrap());
            let start = Instant::now();
            p.remove_connection(edges[size / 2 - 1]).unwrap();
            removals.push(start.elapsed());
            let start = Instant::now();
            let networks = p.networks();
            splits.push(start.elapsed());
            assert_eq!(networks.len(), 2);
            assert!(networks.iter().all(|n| n.points().len() == size / 2));
            assert!(!p.are_connected(ids[0], ids[size - 1]).unwrap());
            let bytes = serde_json::to_vec(&p).unwrap();
            let start = Instant::now();
            let loaded: Project = serde_json::from_slice(&bytes).unwrap();
            imports.push(start.elapsed());
            assert_eq!(loaded.networks(), networks);
            loaded.validate().unwrap();
        }
        eprintln!(
            "chain n={size} samples=3 median_ms build={:.3} networks={:.3} remove={:.3} split={:.3} import={:.3}",
            median_ms(builds),
            median_ms(derives),
            median_ms(removals),
            median_ms(splits),
            median_ms(imports)
        );
    }
}

#[test]
fn growing_stars_keep_neighbor_and_incidence_queries_correct_after_removals() {
    for size in [100, 1_000, 10_000] {
        let mut builds = Vec::new();
        let mut derives = Vec::new();
        let mut queries = Vec::new();
        for _ in 0..3 {
            let start = Instant::now();
            let (mut p, ids) = fixture(size);
            let edges: Vec<_> = ids[1..]
                .iter()
                .map(|&id| connect(&mut p, ids[0], id))
                .collect();
            builds.push(start.elapsed());
            let start = Instant::now();
            let networks = p.networks();
            derives.push(start.elapsed());
            assert_eq!(networks.len(), 1);
            assert!(!networks[0].has_cycle());
            let start = Instant::now();
            let neighbors = p.directly_connected_points(ids[0]).unwrap();
            let incident = p.incident_connections(ids[0]).unwrap();
            queries.push(start.elapsed());
            assert_eq!(neighbors.len(), size - 1);
            assert_eq!(incident.len(), size - 1);
            assert!(neighbors.windows(2).all(|ab| ab[0] < ab[1]));
            assert!(incident.windows(2).all(|ab| ab[0] < ab[1]));
            for edge in edges.iter().step_by(2) {
                p.remove_connection(*edge).unwrap();
            }
            assert_eq!(p.networks().len(), 1 + size / 2);
            p.validate().unwrap();
        }
        eprintln!(
            "star n={size} samples=3 median_ms build={:.3} networks={:.3} neighbors_and_incidence={:.3}",
            median_ms(builds),
            median_ms(derives),
            median_ms(queries)
        );
    }
}

#[test]
fn growing_isolated_sets_include_explicit_components_without_implicit_edges() {
    for size in [100, 1_000, 10_000] {
        let mut builds = Vec::new();
        let mut derives = Vec::new();
        for _ in 0..3 {
            let start = Instant::now();
            let mut p = Project::new("Isolated").unwrap();
            let d = p
                .insert_device(
                    Device::new(p.id(), Designation::new("K1").unwrap(), "test").unwrap(),
                )
                .unwrap();
            for i in 0..size {
                let f = p
                    .insert_function(Function::new(
                        p.id(),
                        d,
                        Designation::new(format!("F{i}")).unwrap(),
                        FunctionKind::Coil,
                    ))
                    .unwrap();
                p.insert_component(
                    Component::new(p.id(), Designation::new(format!("C{i}")).unwrap(), "test")
                        .unwrap()
                        .with_assignment(Some(d), Some(f)),
                )
                .unwrap();
                p.insert_connection_point(ConnectionPoint::new(
                    p.id(),
                    f,
                    DesignationPart::new("A1").unwrap(),
                ))
                .unwrap();
            }
            builds.push(start.elapsed());
            let start = Instant::now();
            let networks = p.networks();
            derives.push(start.elapsed());
            assert_eq!(networks.len(), size);
            for n in networks {
                assert_eq!(n.points().len(), 1);
                assert!(n.connections().is_empty());
                assert_eq!(n.functions().len(), 1);
                assert_eq!(n.devices(), &[d]);
                assert_eq!(n.components().len(), 1);
                assert!(!n.has_cycle());
                let component = p.component(n.components()[0]).unwrap();
                assert_eq!(component.function_id(), Some(n.functions()[0]));
            }
            p.validate().unwrap();
        }
        eprintln!(
            "isolated n={size} samples=3 median_ms build={:.3} networks={:.3}",
            median_ms(builds),
            median_ms(derives)
        );
    }
}
