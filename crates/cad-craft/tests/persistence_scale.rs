// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Native projects larger than the old 8 MiB technical-snapshot limit.
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

mod common;

use cad_craft::persistence;
use std::time::Instant;

#[test]
fn project_larger_than_eight_mebibytes_round_trips_with_measured_time() {
    let (mut project, points) = common::fixture(40_000);
    for pair in points.windows(2) {
        common::connect(&mut project, pair[0], pair[1]);
    }
    let json = serde_json::to_vec(&project).unwrap().len();
    assert!(json > 8 * 1024 * 1024, "JSON de {json} bytes");
    let start = Instant::now();
    let bytes = persistence::write_project_to_vec(&project).unwrap();
    let write = start.elapsed();
    let start = Instant::now();
    let loaded = persistence::read_project_from_slice(&bytes).unwrap();
    let read = start.elapsed();
    assert_eq!(loaded.project, project);
    eprintln!(
        "persistência: JSON {:.1} MiB, .cadcraft {:.1} MiB, gravação {:?}, leitura {:?}",
        json as f64 / 1048576.0,
        bytes.len() as f64 / 1048576.0,
        write,
        read
    );
}
