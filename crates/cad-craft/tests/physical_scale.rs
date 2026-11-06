// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

mod physical_common;
use cad_craft::domain::*;
use physical_common::*;
use std::time::Instant;

fn median(mut samples: Vec<f64>) -> f64 {
    samples.sort_by(f64::total_cmp);
    samples[samples.len() / 2]
}

#[test]
fn growing_cabled_wiring_keeps_traceability_and_logical_topology() {
    for n in [100, 1_000, 10_000] {
        let mut builds = Vec::new();
        let mut queries = Vec::new();
        let mut validations = Vec::new();
        let mut imports = Vec::new();
        for _ in 0..3 {
            let start = Instant::now();
            let (mut p, device, ends, points) = fixture(2);
            let edge = connection(&mut p, points[0], points[1]);
            let networks = p.networks();
            for group in 0..n / 4 {
                let cable = p
                    .insert_cable(
                        Cable::new(p.id(), part(format!("C{group}")), CableKind::Control, 4)
                            .unwrap(),
                    )
                    .unwrap();
                for i in 0..4 {
                    let w = p
                        .insert_conductor(wire(
                            &p,
                            format!("W{}", group * 4 + i),
                            &ends[0],
                            &ends[1],
                        ))
                        .unwrap();
                    let core = core(&mut p, cable, format!("{i}"));
                    p.assign_core_conductor(core, w).unwrap();
                    p.assign_conductor_connection(w, ConductorConnection::new(edge, points[0]))
                        .unwrap();
                }
            }
            builds.push(start.elapsed().as_secs_f64() * 1000.0);
            let start = Instant::now();
            let trace = p.physical_for_connection(edge).unwrap();
            let by_device = p.physical_for_device(device).unwrap();
            queries.push(start.elapsed().as_secs_f64() * 1000.0);
            assert_eq!(trace.conductors().len(), n);
            assert_eq!(trace.cable_cores().len(), n);
            assert_eq!(trace.cables().len(), n / 4);
            assert_eq!(trace, by_device);
            assert_eq!(p.networks(), networks);
            let start = Instant::now();
            p.validate().unwrap();
            validations.push(start.elapsed().as_secs_f64() * 1000.0);
            let bytes = serde_json::to_vec(&p).unwrap();
            let start = Instant::now();
            let mut loaded: Project = serde_json::from_slice(&bytes).unwrap();
            imports.push(start.elapsed().as_secs_f64() * 1000.0);
            assert_eq!(loaded, p);
            for &w in trace.conductors() {
                loaded.unassign_conductor_connection(w).unwrap();
            }
            loaded.remove_connection(edge).unwrap();
            assert_eq!(loaded.networks().len(), 2);
            assert_eq!(loaded.conductors().count(), n);
            loaded.validate().unwrap();
        }
        eprintln!(
            "WIRES n={n} build_ms={:.3} query_ms={:.3} validate_ms={:.3} import_ms={:.3}",
            median(builds),
            median(queries),
            median(validations),
            median(imports)
        );
    }
}

#[test]
fn growing_terminal_strips_preserve_order_without_implicit_continuity() {
    for n in [100, 1_000, 10_000] {
        let mut builds = Vec::new();
        let mut queries = Vec::new();
        let mut validations = Vec::new();
        let mut imports = Vec::new();
        for _ in 0..3 {
            let start = Instant::now();
            let (mut p, device, ends, _) = fixture(n);
            let strip = p
                .insert_terminal_strip(TerminalStrip::new(p.id(), part("X1")))
                .unwrap();
            for end in ends.iter().rev() {
                p.attach_terminal(strip, end.terminal_id()).unwrap();
            }
            builds.push(start.elapsed().as_secs_f64() * 1000.0);
            let start = Instant::now();
            let trace = p.physical_for_device(device).unwrap();
            queries.push(start.elapsed().as_secs_f64() * 1000.0);
            assert_eq!(trace.terminals().len(), n);
            assert_eq!(trace.terminal_strips(), &[strip]);
            assert_eq!(
                p.terminal_strip(strip).unwrap().terminals(),
                ends.iter()
                    .rev()
                    .map(PhysicalTermination::terminal_id)
                    .collect::<Vec<_>>()
            );
            assert_eq!(p.networks().len(), n);
            let start = Instant::now();
            p.validate().unwrap();
            validations.push(start.elapsed().as_secs_f64() * 1000.0);
            let bytes = serde_json::to_vec(&p).unwrap();
            let start = Instant::now();
            let loaded: Project = serde_json::from_slice(&bytes).unwrap();
            imports.push(start.elapsed().as_secs_f64() * 1000.0);
            assert_eq!(loaded, p);
        }
        eprintln!(
            "STRIP n={n} build_ms={:.3} query_ms={:.3} validate_ms={:.3} import_ms={:.3}",
            median(builds),
            median(queries),
            median(validations),
            median(imports)
        );
    }
}
