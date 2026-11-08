// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Terminal symbols placed as strip terminals (X1:1, X1:2…) and single-line
//! links: drawn on the page, in PDF/DXF lists, persisted in format 11, and
//! without effect on connections, networks or wire numbers.
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

use cad_craft::domain::*;
use cad_craft::dxf_export::export_dxf_list;
use cad_craft::persistence;

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/etapa35-v11.cadcraft"
);

fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}
fn at(x: f64, y: f64) -> Transform2D {
    Transform2D::new(p(x, y), 0.0, false, false).unwrap()
}
fn symbol(key: &str) -> SymbolDefinition {
    SymbolLibrary::builtin()
        .get(&SymbolDefinitionId::new(format!("cad-craft.{key}")).unwrap())
        .unwrap()
        .clone()
}
fn base(kind: PageKind) -> (Project, PageId) {
    let mut project = Project::new("Bornes e unifilar").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "Esquema").unwrap())
        .unwrap();
    let page = project
        .insert_page(
            Page::new(
                project.id(),
                doc,
                "Página",
                "1",
                kind,
                Size2D::new(420.0, 297.0).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    (project, page)
}
fn terminal(project: &mut Project, page: PageId, x: f64) -> StripTerminalDelta {
    project
        .place_strip_terminal(StripTerminalPlacement {
            page,
            definition: symbol("terminal"),
            transform: at(x, 100.0),
            strip: DesignationPart::new("X1").unwrap(),
        })
        .unwrap()
}
/// Single-line page with Q1 and K1 linked by a three-phase line.
fn single_line_project() -> (Project, PageId, SingleLineLink) {
    let (mut project, page) = base(PageKind::SingleLine);
    let mut place = |key: &str, prefix: &str, x: f64| {
        let definition = symbol(key);
        let kind = definition.function_kind_hint().unwrap();
        project
            .place_symbol(SymbolPlacement {
                page,
                definition,
                transform: at(x, 150.0),
                device: PlacementDevice::New {
                    designation: project.next_device_designation(prefix).unwrap(),
                    classification: "Teste".into(),
                },
                function_kind: kind,
            })
            .unwrap()
            .representation_id()
    };
    let q1 = place("contact-no", "Q", 60.0);
    let k1 = place("contact-no", "K", 160.0);
    let link = SingleLineLink::new(
        project.id(),
        page,
        q1,
        k1,
        vec![
            p(60.0, 140.0),
            p(60.0, 120.0),
            p(160.0, 120.0),
            p(160.0, 140.0),
        ],
        3,
    )
    .unwrap();
    (project, page, link)
}

#[test]
fn three_terminal_symbols_form_x1_1_to_3_in_the_terminal_list() {
    let (mut project, page) = base(PageKind::MultiLine);
    terminal(&mut project, page, 50.0);
    terminal(&mut project, page, 70.0);
    let before = project.clone();
    let third = terminal(&mut project, page, 90.0);
    project.validate().unwrap();
    let strip = project.terminal_strip(third.strip_id()).unwrap();
    assert_eq!(strip.designation().as_str(), "X1");
    let names: Vec<_> = strip
        .terminals()
        .iter()
        .map(|t| {
            project
                .terminal(*t)
                .unwrap()
                .designation()
                .as_str()
                .to_owned()
        })
        .collect();
    assert_eq!(names, ["1", "2", "3"]);
    let rows = project.physical_lists().terminals;
    let mut listed: Vec<_> = rows
        .iter()
        .filter(|r| r.strip_designation.as_deref() == Some("X1"))
        .map(|r| (r.order.unwrap(), r.designation.clone(), r.owner.clone()))
        .collect();
    listed.sort();
    assert_eq!(
        listed,
        [
            (1, "1".to_owned(), "-X1".to_owned()),
            (2, "2".to_owned(), "-X1".to_owned()),
            (3, "3".to_owned(), "-X1".to_owned())
        ]
    );
    // One device -X1 for the strip; the symbol shows X1:3.
    assert_eq!(project.devices().count(), 1);
    assert_eq!(
        project
            .symbol_text(third.representation_id(), &SymbolTextField::Designation)
            .as_deref(),
        Some("-X1:3")
    );
    // Both sides of the terminal are the same point.
    let points: std::collections::BTreeSet<_> = symbol("terminal")
        .ports()
        .iter()
        .map(|port| {
            project
                .port_binding(&PortRef::new(third.representation_id(), port.id()).unwrap())
                .unwrap()
        })
        .collect();
    assert_eq!(points.len(), 1);
    project.restore_strip_terminal(&third, false).unwrap();
    assert_eq!(project, before);
    project.restore_strip_terminal(&third, true).unwrap();
    assert_eq!(project.terminals().count(), 3);
}

#[test]
fn a_single_line_link_is_drawn_but_never_connects() {
    let (mut project, page, link) = single_line_project();
    let networks = project.networks().len();
    let connections = project.connections().count();
    let list_before = project.page_display_list(page).unwrap();
    let (dxf_before, _) = export_dxf_list(&list_before);
    let delta = project
        .edit_single_line(SingleLineEdit::Create(link.clone()))
        .unwrap();
    assert_eq!(project.networks().len(), networks);
    assert_eq!(project.connections().count(), connections);
    let list = project.page_display_list(page).unwrap();
    assert!(list.items.iter().any(|i| matches!(
        i,
        cad_craft::display::DisplayItem::Path { points, .. } if points == link.points()
    )));
    // Three phase strokes on the longest segment.
    assert_eq!(link.phase_marks().0.len(), 3);
    let (dxf, _) = export_dxf_list(&list);
    assert!(
        dxf.len() > dxf_before.len(),
        "a linha unifilar entra no DXF"
    );
    // The linked symbols cannot be removed while the link exists.
    let source = link.source();
    assert!(project.remove_representation(source).is_err());
    project.restore_single_line(&delta, false).unwrap();
    assert_eq!(project.single_lines().count(), 0);
}

#[test]
fn single_line_links_round_trip_in_format_11() {
    let (mut project, _, link) = single_line_project();
    project
        .edit_single_line(SingleLineEdit::Create(link.clone()))
        .unwrap();
    let bytes = persistence::write_project_to_vec(&project).unwrap();
    let loaded = persistence::read_project_from_slice(&bytes).unwrap();
    assert_eq!(loaded.source_format, 11);
    assert_eq!(loaded.project.single_line(link.id()), Some(&link));
    if std::env::var_os("CAD_CRAFT_WRITE_FIXTURES").is_some() {
        std::fs::write(FIXTURE, &bytes).unwrap();
    }
}

#[test]
fn the_format_11_fixture_keeps_its_single_line_link() {
    let loaded = persistence::read_project_from_slice(&std::fs::read(FIXTURE).unwrap()).unwrap();
    assert_eq!(loaded.source_format, 11);
    assert!(!loaded.migrated());
    let link = loaded
        .project
        .single_lines()
        .next()
        .expect("ligação unifilar");
    assert_eq!(link.phases(), 3);
    assert_eq!(link.points().len(), 4);
}

#[test]
fn strip_terminals_and_single_line_symbols_raise_no_false_warnings() {
    // App test: X1 got VER-004 (terminals counted as auxiliaries without a
    // main function) and single-line symbols got VER-007 for every port.
    let (mut project, page) = base(PageKind::MultiLine);
    terminal(&mut project, page, 50.0);
    terminal(&mut project, page, 70.0);
    let rules = |p: &Project| -> Vec<&'static str> {
        p.check(CheckScope::Project)
            .messages
            .iter()
            .map(|m| m.rule)
            .collect()
    };
    assert!(
        !rules(&project).contains(&"VER-004"),
        "{:?}",
        rules(&project)
    );
    let (mut project, _, link) = single_line_project();
    assert!(rules(&project).contains(&"VER-007"));
    project
        .edit_single_line(SingleLineEdit::Create(link))
        .unwrap();
    assert!(
        !rules(&project).contains(&"VER-007"),
        "{:?}",
        rules(&project)
    );
}

#[test]
fn terminals_saved_as_auxiliary_functions_raise_no_ver_004() {
    // Projects saved by the first Etapa 35 build marked terminal functions
    // as auxiliary; the check ignores functions whose points are terminals.
    let (mut project, page) = base(PageKind::MultiLine);
    terminal(&mut project, page, 50.0);
    let mut value = serde_json::to_value(&project).unwrap();
    for f in value["functions"].as_array_mut().unwrap() {
        f["role"] = serde_json::json!("Auxiliary");
    }
    let old: Project = serde_json::from_value(value).unwrap();
    assert!(old.functions().all(|f| f.role() == FunctionRole::Auxiliary));
    let rules: Vec<_> = old
        .check(CheckScope::Project)
        .messages
        .iter()
        .map(|m| m.rule)
        .collect();
    assert!(!rules.contains(&"VER-004"), "{rules:?}");
}
