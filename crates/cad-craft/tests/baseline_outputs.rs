// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Exact output baselines for versioned fixtures with fixed identifiers.
//!
//! Each fixture pins native project bytes, JSON, page display lists, PDF (with the current
//! creation date masked), and per-page DXF. Any unexpected byte change fails.
//!
//! Intentional output changes require CAD_CRAFT_WRITE_BASELINE=1 and a documented explanation.
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

use cad_craft::display::PageDisplayList;
use cad_craft::dxf_export;
use cad_craft::pdf::{self, PdfOptions, PdfScope};
use cad_craft::persistence;
use std::fmt::Write as _;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
const HASHES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/baseline/hashes.txt"
);

/// FNV-1a detects byte changes; this hash is not used for cryptography.
fn fnv(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

/// Replace the PDF creation date with zeros of the same length.
fn mask_creation_date(pdf: &[u8]) -> Vec<u8> {
    let mut out = pdf.to_vec();
    let key = b"/CreationDate (";
    if let Some(start) = out.windows(key.len()).position(|w| w == key) {
        let mut i = start + key.len();
        while i < out.len() && out[i] != b')' {
            out[i] = b'0';
            i += 1;
        }
    }
    out
}

fn fixture_files() -> Vec<std::path::PathBuf> {
    let mut files: Vec<_> = std::fs::read_dir(FIXTURES)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .is_some_and(|e| e == "cadcraft" || e == "json")
        })
        .collect();
    files.sort();
    files
}

fn current_hashes() -> String {
    let mut out = String::new();
    for path in fixture_files() {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let project = persistence::read_project_from_slice(&std::fs::read(&path).unwrap())
            .unwrap()
            .project;
        let saved = persistence::write_project_to_vec(&project).unwrap();
        writeln!(out, "{name} cadcraft {}", fnv(&saved)).unwrap();
        writeln!(
            out,
            "{name} json {}",
            fnv(&serde_json::to_vec(&project).unwrap())
        )
        .unwrap();
        let mut pages: Vec<_> = project.pages().map(|p| p.id()).collect();
        pages.sort();
        for (i, page) in pages.iter().enumerate() {
            let list = project.page_display_list(*page).unwrap();
            writeln!(
                out,
                "{name} display.{i} {}",
                fnv(format!("{list:?}").as_bytes())
            )
            .unwrap();
        }
        if pages.is_empty() {
            continue;
        }
        let pdf = pdf::export_pdf(&project, &PdfOptions::default(), &mut |_, _| true).unwrap();
        writeln!(out, "{name} pdf {}", fnv(&mask_creation_date(&pdf.bytes))).unwrap();
        let (files, _) =
            dxf_export::export_dxf_pages(&project, &PdfScope::Project, &mut |_, _| true).unwrap();
        for (i, file) in files.iter().enumerate() {
            writeln!(out, "{name} dxf.{i} {}", fnv(&file.bytes)).unwrap();
        }
    }
    out
}

#[test]
// Recorded on Windows (the reference platform): trigonometric functions may
// differ in the last bit on other systems, changing PDF/DXF bytes.
#[cfg_attr(not(windows), ignore = "baseline recorded on Windows")]
fn outputs_match_the_recorded_baseline() {
    let current = current_hashes();
    // Identical input must produce identical output on repeated runs.
    assert_eq!(current, current_hashes(), "saída não determinística");
    if std::env::var_os("CAD_CRAFT_WRITE_BASELINE").is_some() {
        std::fs::create_dir_all(std::path::Path::new(HASHES).parent().unwrap()).unwrap();
        std::fs::write(HASHES, &current).unwrap();
        return;
    }
    let recorded = std::fs::read_to_string(HASHES)
        .expect("tests/fixtures/baseline/hashes.txt ausente: gerar com CAD_CRAFT_WRITE_BASELINE=1")
        .replace("\r\n", "\n");
    let diff: Vec<String> = current
        .lines()
        .zip(recorded.lines())
        .filter(|(a, b)| a != b)
        .map(|(a, b)| format!("agora {a} · referência {b}"))
        .collect();
    assert!(
        diff.is_empty() && current.lines().count() == recorded.lines().count(),
        "saídas diferentes da referência:\n{}",
        diff.join("\n")
    );
}
