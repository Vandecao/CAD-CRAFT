// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Import the synthetic test fixture and print its loss report; start with main.
use cad_craft::interchange::{
    NoteKind,
    qet::{self, QetOptions},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = include_bytes!("../tests/fixtures/synthetic/entry.qet");
    let imported = qet::import(bytes, &QetOptions::default())?;
    imported.project.validate()?;
    assert_eq!(imported.stats.pages, 1);
    assert_eq!(imported.stats.texts, 1);
    assert!(imported.report.count(NoteKind::Ignored) > 0);
    println!(
        "{} page(s), {} text(s)",
        imported.stats.pages, imported.stats.texts
    );
    for line in imported.report.lines() {
        println!("{line}");
    }
    Ok(())
}

#[test]
fn runs_without_external_data() {
    main().expect("the synthetic example must succeed");
}
