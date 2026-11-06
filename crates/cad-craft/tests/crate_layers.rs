// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! The workspace crates depend on each other in one direction only:
//! model ← format, render, import; render ← export; all ← facade ← GUI.
//! A new dependency against this direction fails here.
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

use std::collections::{BTreeMap, BTreeSet};

fn project_dependencies() -> BTreeMap<String, BTreeSet<String>> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let output = std::process::Command::new(cargo)
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--offline",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo metadata");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let packages = metadata["packages"].as_array().unwrap();
    let names: BTreeSet<String> = packages
        .iter()
        .map(|p| p["name"].as_str().unwrap().to_owned())
        .collect();
    packages
        .iter()
        .map(|p| {
            let deps = p["dependencies"]
                .as_array()
                .unwrap()
                .iter()
                // Only normal dependencies between crates of this workspace.
                .filter(|d| d["kind"].is_null())
                .map(|d| d["name"].as_str().unwrap().to_owned())
                .filter(|d| names.contains(d))
                .collect();
            (p["name"].as_str().unwrap().to_owned(), deps)
        })
        .collect()
}

#[test]
fn crates_depend_in_one_direction_only() {
    let deps = project_dependencies();
    let set =
        |items: &[&str]| -> BTreeSet<String> { items.iter().map(|s| (*s).to_owned()).collect() };
    let expected: BTreeMap<String, BTreeSet<String>> = [
        ("cadcraft-model", set(&[])),
        ("cadcraft-format", set(&["cadcraft-model"])),
        ("cadcraft-render", set(&["cadcraft-model"])),
        ("cadcraft-import", set(&["cadcraft-model"])),
        ("cadcraft-assistant", set(&["cadcraft-model"])),
        (
            "cadcraft-export",
            set(&["cadcraft-model", "cadcraft-render"]),
        ),
        (
            "cad-craft",
            set(&[
                "cadcraft-model",
                "cadcraft-format",
                "cadcraft-render",
                "cadcraft-import",
                "cadcraft-export",
                "cadcraft-assistant",
            ]),
        ),
        ("cadcraft-gui", set(&["cad-craft"])),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v))
    .collect();
    assert_eq!(deps, expected);
}
