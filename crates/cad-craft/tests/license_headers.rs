// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Every Rust source file in the workspace starts with the SPDX license
//! header, and the license text is present at the repository root.
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

const HEADER: &str = "// SPDX-License-Identifier: GPL-3.0-or-later";

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy();
        if path.is_dir() {
            if name != "target" && !name.starts_with('.') {
                rust_files(&path, out);
            }
        } else if name.ends_with(".rs") {
            out.push(path);
        }
    }
}

#[test]
fn every_rust_file_has_the_spdx_header() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();
    rust_files(&crates, &mut files);
    assert!(
        files.len() > 100,
        "poucos arquivos encontrados: {}",
        files.len()
    );
    let missing: Vec<_> = files
        .iter()
        .filter(|f| {
            std::fs::read_to_string(f)
                .unwrap()
                .lines()
                .next()
                .is_none_or(|first| first.trim_end() != HEADER)
        })
        .collect();
    assert!(missing.is_empty(), "sem cabeçalho SPDX: {missing:#?}");
}

#[test]
fn gpl_license_text_is_present() {
    let license = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../LICENSE");
    let text = std::fs::read_to_string(license).unwrap();
    assert!(text.contains("GNU GENERAL PUBLIC LICENSE"));
    assert!(text.contains("Version 3, 29 June 2007"));
}
