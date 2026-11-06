// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Create and save a document and page; start with main.
// Examples present their results on standard output.
#![allow(clippy::print_stdout)]

use cad_craft::{domain::*, persistence};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut project = Project::new("Example panel")?;
    let document = project.insert_document(Document::new(project.id(), "Schematic")?)?;
    let page = project.insert_page(Page::new(
        project.id(),
        document,
        "Control",
        "1",
        PageKind::MultiLine,
        Size2D::new(420.0, 297.0)?,
    )?)?;
    let directory = std::env::temp_dir().join(format!("cadcraft-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir(&directory)?;
    let path = directory.join("panel.cadcraft");
    persistence::write_project(&project, std::fs::File::create(&path)?)?;
    let loaded = persistence::read_project(std::fs::File::open(&path)?)?;
    assert_eq!(loaded.project, project);
    assert!(loaded.project.page(page).is_some());
    println!("Saved and reloaded {}", path.display());
    std::fs::remove_dir_all(directory)?;
    Ok(())
}

#[test]
fn runs_without_external_data() {
    main().expect("the synthetic example must succeed");
}
