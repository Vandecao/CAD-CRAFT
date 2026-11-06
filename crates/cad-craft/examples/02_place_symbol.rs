// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Place a coil and read its display list; start with main.
// Examples present their results on standard output.
#![allow(clippy::print_stdout)]

use cad_craft::{
    display::{DisplayItem, DisplayLayer},
    domain::*,
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
    let definition = SymbolLibrary::builtin()
        .get(&SymbolDefinitionId::new("cad-craft.coil")?)
        .expect("the built-in coil exists")
        .clone();
    let before = project.clone();
    let delta = project.place_symbol(SymbolPlacement {
        page,
        definition,
        transform: Transform2D::new(Point2D::new(100.0, 100.0)?, 0.0, false, false)?,
        device: PlacementDevice::New {
            designation: Designation::new("K9")?,
            classification: "Relay".into(),
        },
        function_kind: FunctionKind::Coil,
    })?;
    let list = project.page_display_list(page)?;
    assert!(!list.items.is_empty());
    assert!(list.items.iter().any(|item| matches!(
        item,
        DisplayItem::Path {
            layer: DisplayLayer::Symbols,
            ..
        }
    )));
    println!("{} display items", list.items.len());
    project.restore_placement(&delta, false)?;
    assert_eq!(project, before);
    Ok(())
}

#[test]
fn runs_without_external_data() {
    main().expect("the synthetic example must succeed");
}
