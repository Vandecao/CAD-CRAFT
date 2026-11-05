// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
mod document_common;
mod physical_common;
use cad_craft::domain::*;
use cad_craft::report_export::{export_csv, export_xlsx};
use cad_craft::{
    dxf_export,
    pdf::{self, PdfOptions, PdfScope},
};

#[test]
fn six_report_types_derive_rows_from_project() {
    let (mut project, first, _, device, connection) = document_common::fixture();
    project
        .insert_representation(document_common::rep(
            &project,
            first,
            RepresentationTarget::Device(device),
        ))
        .unwrap();
    project
        .set_connection_number(connection, Some(Text::new("17").unwrap()), false)
        .unwrap();
    let filter = ReportFilter::default();
    let index = project
        .report_table(ReportKind::PageIndex, &filter)
        .unwrap();
    assert_eq!(index.rows.len(), 2);
    assert!(index.rows[0].cells.iter().any(|c| c.text() == "Potência"));
    let devices = project.report_table(ReportKind::Devices, &filter).unwrap();
    assert_eq!(devices.rows.len(), 1);
    assert!(
        devices.rows[0]
            .cells
            .iter()
            .any(|c| c.text().contains("K1"))
    );
    let wires = project
        .report_table(ReportKind::Connections, &filter)
        .unwrap();
    assert_eq!(wires.rows.len(), 1);
    assert_eq!(wires.rows[0].cells[0].text(), "17");
    for kind in [
        ReportKind::Materials,
        ReportKind::Terminals,
        ReportKind::Cables,
    ] {
        assert!(project.report_table(kind, &filter).unwrap().rows.is_empty());
    }
}

#[test]
fn report_generation_is_one_reversible_delta() {
    let (mut project, first, _, _, _) = document_common::fixture();
    let document = project.page(first).unwrap().document_id();
    let before = project.clone();
    let request = ReportRequest::new(
        ReportKind::PageIndex,
        document,
        Size2D::new(420.0, 297.0).unwrap(),
    );
    let preview = project.preview_report(&request).unwrap();
    assert_eq!(preview.pages, 1);
    let delta = project.generate_report(&request).unwrap();
    let after = project.clone();
    assert_eq!(project.pages().filter(|p| p.report().is_some()).count(), 1);
    project.restore_report(&delta, false).unwrap();
    assert_eq!(project, before);
    project.restore_report(&delta, true).unwrap();
    assert_eq!(project, after);
}

#[test]
fn report_display_repeats_header_and_footer() {
    let (mut project, first, _, _, _) = document_common::fixture();
    let document = project.page(first).unwrap().document_id();
    let request = ReportRequest::new(
        ReportKind::PageIndex,
        document,
        Size2D::new(420.0, 297.0).unwrap(),
    );
    project.generate_report(&request).unwrap();
    let page = project.pages().find(|p| p.report().is_some()).unwrap();
    let texts: Vec<_> = project
        .page_display_list(page.id())
        .unwrap()
        .texts()
        .map(|t| t.text.clone())
        .collect();
    assert!(texts.contains(&"Documento".to_owned()));
    assert!(texts.contains(&"Potência".to_owned()));
    assert!(texts.contains(&"página 1 de 1 do relatório".to_owned()));
}

#[test]
fn csv_has_bom_semicolon_and_unicode() {
    let (project, _, _, _, _) = document_common::fixture();
    let table = project
        .report_table(ReportKind::PageIndex, &ReportFilter::default())
        .unwrap();
    let csv = export_csv(&table).unwrap();
    assert!(csv.starts_with(&[0xEF, 0xBB, 0xBF]));
    let text = std::str::from_utf8(&csv[3..]).unwrap();
    assert!(text.contains("Documento;Número;Nome"));
    assert!(text.contains("Potência"));
}

#[test]
fn xlsx_has_six_sheets_and_numeric_cells() {
    let (project, _, _, _, _) = document_common::fixture();
    let tables = ReportKind::ALL
        .iter()
        .map(|k| project.report_table(*k, &ReportFilter::default()).unwrap())
        .collect::<Vec<_>>();
    let bytes = export_xlsx(&tables).unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut workbook = String::new();
    use std::io::Read;
    zip.by_name("xl/workbook.xml")
        .unwrap()
        .read_to_string(&mut workbook)
        .unwrap();
    assert_eq!(workbook.matches("<sheet ").count(), 6);
    assert!(workbook.contains("Lista de materiais"));
    let mut numeric = project
        .report_table(ReportKind::Materials, &ReportFilter::default())
        .unwrap();
    numeric.rows.push(ReportRow {
        cells: vec![
            ReportCell::plain("P-01"),
            ReportCell::plain("Fábrica"),
            ReportCell::plain("Peça"),
            ReportCell::number(12.5),
            ReportCell::plain("K1"),
        ],
    });
    let bytes = export_xlsx(&[numeric]).unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut sheet = String::new();
    zip.by_name("xl/worksheets/sheet1.xml")
        .unwrap()
        .read_to_string(&mut sheet)
        .unwrap();
    assert!(sheet.contains("<v>12.5</v>"), "{sheet}");
    let mut strings = String::new();
    zip.by_name("xl/sharedStrings.xml")
        .unwrap()
        .read_to_string(&mut strings)
        .unwrap();
    assert!(strings.contains("Fábrica"));
}

#[test]
fn update_grows_and_shrinks_without_touching_schematic_pages() {
    let (mut project, first, _, device, _) = document_common::fixture();
    let doc = project.page(first).unwrap().document_id();
    let request = ReportRequest::new(
        ReportKind::Connections,
        doc,
        Size2D::new(420.0, 297.0).unwrap(),
    );
    project.generate_report(&request).unwrap();
    let group = project
        .pages()
        .find_map(|p| p.report().map(|r| r.group))
        .unwrap();
    assert!(!project.report_stale(group).unwrap());
    let f = project.functions_for_device(device).next().unwrap().id();
    let anchor = project.connection_points().next().unwrap().id();
    let mut additions = Vec::new();
    for i in 0..45 {
        let point = project
            .insert_connection_point(ConnectionPoint::new(
                project.id(),
                f,
                DesignationPart::new(format!("X{i}")).unwrap(),
            ))
            .unwrap();
        additions.push(
            project
                .insert_connection(Connection::new(project.id(), anchor, point).unwrap())
                .unwrap(),
        );
    }
    assert!(project.report_stale(group).unwrap());
    let delta = project.update_report(group).unwrap();
    assert_eq!(project.pages().filter(|p| p.report().is_some()).count(), 2);
    assert!(!project.report_stale(group).unwrap());
    project.restore_report(&delta, false).unwrap();
    assert_eq!(project.pages().filter(|p| p.report().is_some()).count(), 1);
    project.restore_report(&delta, true).unwrap();
    for id in additions {
        project.remove_connection(id).unwrap();
    }
    assert!(project.report_stale(group).unwrap());
    project.update_report(group).unwrap();
    assert_eq!(project.pages().filter(|p| p.report().is_none()).count(), 2);
    assert_eq!(project.pages().filter(|p| p.report().is_some()).count(), 1);
}

#[test]
fn updating_all_groups_is_a_single_reversible_delta() {
    let (mut project, first, _, device, _) = document_common::fixture();
    let doc = project.page(first).unwrap().document_id();
    for kind in [ReportKind::PageIndex, ReportKind::Connections] {
        project
            .generate_report(&ReportRequest::new(kind, doc, PaperFormat::A3.size(true)))
            .unwrap();
    }
    let groups: Vec<_> = project
        .pages()
        .filter_map(|p| p.report().map(|r| r.group))
        .collect();
    let f = project.functions_for_device(device).next().unwrap().id();
    let anchor = project.connection_points().next().unwrap().id();
    for i in 0..45 {
        let point = project
            .insert_connection_point(ConnectionPoint::new(
                project.id(),
                f,
                DesignationPart::new(format!("B{i}")).unwrap(),
            ))
            .unwrap();
        project
            .insert_connection(Connection::new(project.id(), anchor, point).unwrap())
            .unwrap();
    }
    let before = project.clone();
    let delta = project.update_reports(&groups).unwrap();
    let after = project.clone();
    assert!(!delta.is_empty());
    project.restore_report(&delta, false).unwrap();
    assert_eq!(project, before);
    project.restore_report(&delta, true).unwrap();
    assert_eq!(project, after);
}

#[test]
fn terminal_and_cable_tables_show_registered_physical_data() {
    let project = physical_common::populated();
    let filter = ReportFilter::default();
    let terminals = project
        .report_table(ReportKind::Terminals, &filter)
        .unwrap();
    assert_eq!(terminals.rows.len(), 4);
    assert_eq!(terminals.columns[3].title, "Lado A");
    assert_eq!(terminals.columns[4].title, "Lado B");
    let cables = project.report_table(ReportKind::Cables, &filter).unwrap();
    assert_eq!(cables.rows.len(), 1);
    assert_eq!(cables.rows[0].cells[2].text(), "3");
    assert_eq!(cables.rows[0].cells[3].text(), "1");
}

#[test]
fn hundred_twenty_connections_paginate_a3_with_repeated_header() {
    let (mut project, first, _, device, _) = document_common::fixture();
    let doc = project.page(first).unwrap().document_id();
    let f = project.functions_for_device(device).next().unwrap().id();
    let anchor = project.connection_points().next().unwrap().id();
    for i in 1..120 {
        let point = project
            .insert_connection_point(ConnectionPoint::new(
                project.id(),
                f,
                DesignationPart::new(format!("L{i}")).unwrap(),
            ))
            .unwrap();
        project
            .insert_connection(Connection::new(project.id(), anchor, point).unwrap())
            .unwrap();
    }
    let request = ReportRequest::new(ReportKind::Connections, doc, PaperFormat::A3.size(true));
    assert_eq!(project.preview_report(&request).unwrap().pages, 3);
    project.generate_report(&request).unwrap();
    let mut report_pages: Vec<_> = project.pages().filter(|p| p.report().is_some()).collect();
    report_pages.sort_by_key(|p| p.report().unwrap().index);
    for (i, page) in report_pages.iter().enumerate() {
        let texts: Vec<_> = project
            .page_display_list(page.id())
            .unwrap()
            .texts()
            .map(|t| t.text.clone())
            .collect();
        assert!(texts.contains(&"Fio/Rede".to_owned()));
        assert!(texts.contains(&format!("página {} de 3 do relatório", i + 1)));
    }
}

#[test]
fn report_page_flows_to_pdf_and_dxf_and_keeps_reference_link() {
    let (mut project, first, _, device, _) = document_common::fixture();
    project
        .insert_representation(document_common::rep(
            &project,
            first,
            RepresentationTarget::Device(device),
        ))
        .unwrap();
    let doc = project.page(first).unwrap().document_id();
    project
        .generate_report(&ReportRequest::new(
            ReportKind::Devices,
            doc,
            PaperFormat::A3.size(true),
        ))
        .unwrap();
    let report_page = project.pages().find(|p| p.report().is_some()).unwrap().id();
    let list = project.page_display_list(report_page).unwrap();
    assert!(list.links().next().is_some());
    let (dxf, _) = dxf_export::export_dxf_list(&list);
    assert!(dxf.windows(b"Designa".len()).any(|w| w == b"Designa"));
    let output = pdf::export_pdf(
        &project,
        &PdfOptions {
            scope: PdfScope::Project,
            ..Default::default()
        },
        &mut |_, _| true,
    )
    .unwrap();
    assert_eq!(output.report.links, 1);
}

#[test]
fn long_table_cell_is_truncated_with_ellipsis() {
    let (mut project, first, _, _, _) = document_common::fixture();
    project
        .rename_page(
            first,
            "Nome muito longo que não cabe na coluna do relatório e precisa ser cortado na saída",
        )
        .unwrap();
    let doc = project.page(first).unwrap().document_id();
    project
        .generate_report(&ReportRequest::new(
            ReportKind::PageIndex,
            doc,
            PaperFormat::A4.size(true),
        ))
        .unwrap();
    let report_page = project.pages().find(|p| p.report().is_some()).unwrap().id();
    let texts: Vec<_> = project
        .page_display_list(report_page)
        .unwrap()
        .texts()
        .map(|t| t.text.clone())
        .collect();
    assert!(texts.iter().any(|t| t.ends_with('…')), "{texts:?}");
}

#[test]
fn page_index_includes_its_own_page_without_starting_stale() {
    let (mut project, first, _, _, _) = document_common::fixture();
    let doc = project.page(first).unwrap().document_id();
    let request = ReportRequest::new(ReportKind::PageIndex, doc, PaperFormat::A3.size(true));
    project.generate_report(&request).unwrap();
    let page = project.pages().find(|p| p.report().is_some()).unwrap();
    assert_eq!(
        project
            .report_table(ReportKind::PageIndex, &ReportFilter::default())
            .unwrap()
            .rows
            .len(),
        3
    );
    assert!(!project.report_stale(page.report().unwrap().group).unwrap());
}

#[test]
fn document_and_page_filters_select_only_matching_occurrences() {
    let (project, first, second, _, _) = document_common::fixture();
    let document = project.page(first).unwrap().document_id();
    let by_document = project
        .report_table(
            ReportKind::PageIndex,
            &ReportFilter {
                document: Some(document),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(by_document.rows.len(), 2);
    let by_page = project
        .report_table(
            ReportKind::PageIndex,
            &ReportFilter {
                page: Some(second),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(by_page.rows.len(), 1);
    assert_eq!(
        by_page.rows[0].cells[1].text(),
        project.page(second).unwrap().number().as_str()
    );
}

#[test]
fn update_all_recalculates_page_index_after_another_group_grows() {
    let (mut project, first, _, device, _) = document_common::fixture();
    let doc = project.page(first).unwrap().document_id();
    project
        .generate_report(&ReportRequest::new(
            ReportKind::PageIndex,
            doc,
            PaperFormat::A3.size(true),
        ))
        .unwrap();
    project
        .generate_report(&ReportRequest::new(
            ReportKind::Connections,
            doc,
            PaperFormat::A3.size(true),
        ))
        .unwrap();
    let groups: Vec<_> = project
        .pages()
        .filter_map(|p| p.report().map(|r| r.group))
        .collect();
    let function = project.functions_for_device(device).next().unwrap().id();
    let anchor = project.connection_points().next().unwrap().id();
    for i in 0..50 {
        let point = project
            .insert_connection_point(ConnectionPoint::new(
                project.id(),
                function,
                DesignationPart::new(format!("U{i}")).unwrap(),
            ))
            .unwrap();
        project
            .insert_connection(Connection::new(project.id(), anchor, point).unwrap())
            .unwrap();
    }
    let before = project.clone();
    let delta = project.update_reports(&groups).unwrap();
    assert!(groups.iter().all(|id| !project.report_stale(*id).unwrap()));
    project.restore_report(&delta, false).unwrap();
    assert_eq!(project, before);
}

#[test]
fn materials_sum_quantities_or_split_by_device() {
    let mut project = Project::new("BOM").unwrap();
    let part = Part::new("ACME.P1", "ACME", "P1", "Peça de teste", "Módulo").unwrap();
    for (name, quantity) in [("K1", 2), ("K2", 3)] {
        let device = project
            .insert_device(
                Device::new(project.id(), Designation::new(name).unwrap(), "Módulo").unwrap(),
            )
            .unwrap();
        project.assign_part(device, part.clone(), quantity).unwrap();
    }
    let grouped = project
        .report_table(ReportKind::Materials, &ReportFilter::default())
        .unwrap();
    assert_eq!(grouped.rows.len(), 1);
    assert_eq!(grouped.rows[0].cells[3].text(), "5");
    assert!(grouped.rows[0].cells[4].text().contains("K1"));
    assert!(grouped.rows[0].cells[4].text().contains("K2"));
    let split = project
        .report_table(
            ReportKind::Materials,
            &ReportFilter {
                materials_by_device: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(split.rows.len(), 2);
    assert_eq!(
        split
            .rows
            .iter()
            .map(|r| r.cells[3].text().parse::<u32>().unwrap())
            .sum::<u32>(),
        5
    );
}

#[test]
fn five_thousand_connection_rows_generate_pages_and_xlsx_within_budget() {
    let (mut project, first, _, device, _) = document_common::fixture();
    let doc = project.page(first).unwrap().document_id();
    let function = project.functions_for_device(device).next().unwrap().id();
    let anchor = project.connection_points().next().unwrap().id();
    for i in 1..5_000 {
        let point = project
            .insert_connection_point(ConnectionPoint::new(
                project.id(),
                function,
                DesignationPart::new(format!("P{i}")).unwrap(),
            ))
            .unwrap();
        project
            .insert_connection(Connection::new(project.id(), anchor, point).unwrap())
            .unwrap();
    }
    let start = std::time::Instant::now();
    let request = ReportRequest::new(ReportKind::Connections, doc, PaperFormat::A3.size(true));
    project.generate_report(&request).unwrap();
    let table = project
        .report_table(ReportKind::Connections, &ReportFilter::default())
        .unwrap();
    assert_eq!(table.rows.len(), 5_000);
    assert!(export_xlsx(&[table]).unwrap().starts_with(b"PK"));
    let limit = if cfg!(debug_assertions) { 60.0 } else { 5.0 };
    assert!(
        start.elapsed().as_secs_f64() < limit,
        "{} s",
        start.elapsed().as_secs_f64()
    );
}

#[test]
fn five_thousand_material_rows_generate_pages_and_xlsx_within_budget() {
    let mut project = Project::new("Grande BOM").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "D").unwrap())
        .unwrap();
    let device = project
        .insert_device(
            Device::new(project.id(), Designation::new("K1").unwrap(), "Módulo").unwrap(),
        )
        .unwrap();
    for i in 0..5_000 {
        let part = Part::new(
            format!("P{i}"),
            "Fabricante",
            format!("P{i}"),
            "Peça",
            "Módulo",
        )
        .unwrap();
        project.assign_part(device, part, 1).unwrap();
    }
    let start = std::time::Instant::now();
    let request = ReportRequest::new(ReportKind::Materials, doc, PaperFormat::A3.size(true));
    project.generate_report(&request).unwrap();
    let table = project
        .report_table(ReportKind::Materials, &ReportFilter::default())
        .unwrap();
    assert_eq!(table.rows.len(), 5_000);
    assert!(export_xlsx(&[table]).unwrap().starts_with(b"PK"));
    let limit = if cfg!(debug_assertions) { 60.0 } else { 5.0 };
    assert!(
        start.elapsed().as_secs_f64() < limit,
        "{} s",
        start.elapsed().as_secs_f64()
    );
}
