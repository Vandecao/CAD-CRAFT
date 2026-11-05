// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Enforce source module introductions; start with the workspace scan test.
use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

fn has_introduction(source: &str) -> bool {
    let mut lines = source.lines();
    if lines.next() != Some("// SPDX-License-Identifier: GPL-3.0-or-later") {
        return false;
    }
    if !lines
        .next()
        .is_some_and(|line| line.starts_with("// Copyright"))
    {
        return false;
    }
    lines
        .find(|line| !line.trim().is_empty())
        .is_some_and(|line| line.starts_with("//!") && !line[3..].trim().is_empty())
}

#[test]
fn every_source_module_starts_with_an_introduction() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();
    let mut crate_count = 0;
    for entry in std::fs::read_dir(crates).unwrap() {
        let path = entry.unwrap().path();
        if path.join("Cargo.toml").is_file() && path.join("src").is_dir() {
            crate_count += 1;
            rust_files(&path.join("src"), &mut files);
        }
    }
    assert!(crate_count >= 8, "expected at least eight workspace crates");
    assert!(
        files.len() > 100,
        "source scan must not silently become empty"
    );
    files.sort();
    let missing: Vec<_> = files
        .iter()
        .filter(|path| !has_introduction(&std::fs::read_to_string(path).unwrap()))
        .collect();
    assert!(
        missing.is_empty(),
        "missing module introduction: {missing:#?}"
    );
}

#[test]
fn introduction_must_precede_code_and_describe_the_module() {
    let header = "// SPDX-License-Identifier: GPL-3.0-or-later\n// Copyright (C) example\n";
    assert!(has_introduction(&format!(
        "{header}\n//! Project edits; start with edit.\nuse std::fmt;"
    )));
    for body in [
        "use std::fmt;\n//! Too late.",
        "/// Outer docs.",
        "//!",
        "// Ordinary comment.\n//! Later.",
    ] {
        assert!(!has_introduction(&format!("{header}{body}")), "{body}");
    }
}
