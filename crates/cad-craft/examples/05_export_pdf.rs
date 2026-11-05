// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Export a synthetic sheet to a temporary PDF; start with main.
use cad_craft::{
    domain::*,
    pdf::{self, PdfOptions},
};

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
    assert!(project.page(page).is_some());
    let output = pdf::export_pdf(&project, &PdfOptions::default(), &mut |_, _| true)?;
    assert_eq!(output.report.pages, 1);
    assert!(output.bytes.starts_with(b"%PDF-"));
    let directory = std::env::temp_dir().join(format!("cadcraft-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir(&directory)?;
    let path = directory.join("panel.pdf");
    std::fs::write(&path, &output.bytes)?;
    assert_eq!(std::fs::read(&path)?, output.bytes);
    println!("{}: {}", path.display(), output.report.summary());
    std::fs::remove_dir_all(directory)?;
    Ok(())
}

#[test]
fn runs_without_external_data() {
    main().expect("the synthetic example must succeed");
}
