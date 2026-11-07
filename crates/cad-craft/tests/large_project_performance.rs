// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Large synthetic acceptance workload; start with large_project_operations_stay_within_ci_budgets.
//!
//! Run with `cargo test -p cad-craft --release --test large_project_performance -- --nocapture`.
//! Debug runs exercise only the small fixture unless CADCRAFT_PERFORMANCE is set.
//! Set CADCRAFT_PERFORMANCE_PROJECT to an ignored local path to retain the generated native file.
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

mod output_common;

use cad_craft::display::PageDisplayList;
use cad_craft::domain::*;
use cad_craft::{dxf_export, pdf, persistence, report_export};
use std::time::{Duration, Instant};

struct NativeFile(std::path::PathBuf);

impl Drop for NativeFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Build a real schematic through validated operations, then replicate its complete snapshot.
/// Deserialization validates all references and rebuilds derived indices, just as native loading
/// does. No entity validation is bypassed to make the synthetic workload cheaper to construct.
fn fixture(pages: usize) -> Project {
    use output_common::*;
    let mut project = Project::new("Synthetic acceptance workload").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "Schematics").unwrap())
        .unwrap();
    let page = project
        .insert_page(
            Page::new(
                project.id(),
                doc,
                "Synthetic",
                "1",
                PageKind::MultiLine,
                PaperFormat::A3.size(true),
            )
            .unwrap(),
        )
        .unwrap();
    let frame = FrameTemplate::factory_a3();
    let frame_id = frame.id();
    project.edit_frame(FrameEdit::UpsertFrame(frame)).unwrap();
    project
        .edit_frame(FrameEdit::SetDefaultFrame(Some(frame_id)))
        .unwrap();
    let mut reps = Vec::new();
    for i in 0..25 {
        let x = 30.0 + (i % 5) as f64 * 65.0;
        let y = 50.0 + (i / 5) as f64 * 40.0;
        let delta = place(
            &mut project,
            page,
            coil(),
            new_device(&format!("K{}", i + 1)),
            FunctionKind::Coil,
            (x, y),
        );
        reps.push((delta.representation_id(), x, y));
    }
    for (i, &(source, x, y)) in reps.iter().enumerate() {
        for offset in 1..=4 {
            let (target, tx, ty) = reps[(i + offset) % reps.len()];
            project
                .edit_connection_drawing(ConnectionDrawingEdit::CreateRoute {
                    source: PortRef::new(source, "2").unwrap(),
                    target: PortRef::new(target, "1").unwrap(),
                    points: vec![
                        p(x, y + 10.0),
                        p(x + 5.0 + offset as f64, y + 10.0),
                        p(x + 5.0 + offset as f64, ty - 10.0),
                        p(tx, ty - 10.0),
                    ],
                })
                .unwrap();
        }
    }
    let project = replicate_page(&project, pages);
    project.validate().unwrap();
    project
}

fn measured<T>(label: &str, action: impl FnOnce() -> T) -> (T, Duration) {
    let start = Instant::now();
    let value = action();
    let elapsed = start.elapsed();
    eprintln!("PERF {label}={:.3} ms", elapsed.as_secs_f64() * 1000.0);
    (value, elapsed)
}

#[test]
fn small_workload_has_drawn_electrical_content_and_valid_round_trip() {
    let project = fixture(2);
    assert_eq!(project.pages().count(), 2);
    assert_eq!(project.devices().count(), 50);
    assert_eq!(project.connections().count(), 200);
    assert_eq!(project.connection_routes().count(), 200);
    assert_eq!(project.representations().count(), 250);
    assert_eq!(project.networks().len(), 2);
    for page in project.pages() {
        assert_eq!(
            project.representations_on_page(page.id()).unwrap().len(),
            125
        );
        assert!(
            project
                .page_display_list(page.id())
                .unwrap()
                .texts()
                .count()
                >= 25
        );
    }
    let bytes = persistence::write_project_to_vec(&project).unwrap();
    assert_eq!(
        persistence::read_project_from_slice(&bytes)
            .unwrap()
            .project,
        project
    );
}

#[test]
fn large_project_operations_stay_within_ci_budgets() {
    if cfg!(debug_assertions) && std::env::var_os("CADCRAFT_PERFORMANCE").is_none() {
        return;
    }
    let (mut project, _) = measured("fixture", || fixture(200));
    assert_eq!(project.pages().count(), 200);
    assert_eq!(project.devices().count(), 5_000);
    assert_eq!(project.connections().count(), 20_000);
    assert_eq!(project.connection_routes().count(), 20_000);
    let (bytes, _) = measured("serialize", || {
        persistence::write_project_to_vec(&project).unwrap()
    });
    if let Some(path) = std::env::var_os("CADCRAFT_PERFORMANCE_PROJECT") {
        std::fs::write(path, &bytes).unwrap();
    }
    eprintln!("PERF native_bytes={}", bytes.len());
    let path = NativeFile(std::env::temp_dir().join(format!(
        "cadcraft-performance-{}.cadcraft",
        uuid::Uuid::now_v7()
    )));
    let (_, save) = measured("save_disk", || {
        let file = std::fs::File::create(&path.0).unwrap();
        let file = persistence::write_project(&project, file).unwrap();
        file.sync_all().unwrap();
    });
    let (loaded, open) = measured("open_disk", || {
        persistence::read_project(std::fs::File::open(&path.0).unwrap()).unwrap()
    });
    assert_eq!(loaded.project, project);
    drop(loaded);
    let page = project.pages().next().unwrap().id();
    let mut displays = Vec::new();
    for _ in 0..11 {
        let (list, duration) =
            measured("display_page", || project.page_display_list(page).unwrap());
        assert!(list.texts().count() >= 25);
        displays.push(duration);
    }
    displays.sort();
    let display = displays[displays.len() / 2];
    let (_, numbering) = measured("number", || {
        let devices = project
            .preview_device_numbering(NumberingFilter::Project)
            .unwrap();
        project.apply_device_numbering(&devices).unwrap();
        let preview = project.preview_connection_numbering().unwrap();
        project.apply_connection_numbering(&preview).unwrap();
    });
    assert!(project.connections().all(|c| c.number().is_some()));
    let (checks, checks_time) = measured("checks", || project.check(CheckScope::Project));
    eprintln!("PERF check_messages={}", checks.messages.len());
    let (pdf, pdf_time) = measured("pdf_200", || {
        pdf::export_pdf(&project, &pdf::PdfOptions::default(), &mut |_, _| true).unwrap()
    });
    assert_eq!(pdf.report.pages, 200);
    assert_eq!(
        lopdf::Document::load_mem(&pdf.bytes)
            .unwrap()
            .get_pages()
            .len(),
        200
    );
    let ((dxf, _), dxf_time) = measured("dxf_200", || {
        dxf_export::export_dxf_pages(&project, &pdf::PdfScope::Project, &mut |_, _| true).unwrap()
    });
    assert_eq!(dxf.len(), 200);
    assert!(dxf.iter().all(|file| !file.bytes.is_empty()));
    let (tables, reports_time) = measured("reports", || {
        ReportKind::ALL
            .iter()
            .map(|kind| {
                project
                    .report_table(*kind, &ReportFilter::default())
                    .unwrap()
            })
            .collect::<Vec<_>>()
    });
    for (kind, expected) in [
        (ReportKind::PageIndex, 200),
        (ReportKind::Devices, 5_000),
        (ReportKind::Connections, 20_000),
    ] {
        let index = ReportKind::ALL.iter().position(|k| *k == kind).unwrap();
        assert_eq!(tables[index].rows.len(), expected);
    }
    let (xlsx, _) = measured("reports_xlsx", || {
        report_export::export_xlsx(&tables).unwrap()
    });
    assert!(xlsx.starts_with(b"PK"));
    // Acceptance targets remain visible separately from deliberately loose CI regression limits.
    for (label, actual, target, ci_limit) in [
        ("save", save, Some(3.0), 15.0),
        ("open", open, Some(3.0), 15.0),
        ("display_median", display, Some(0.016), 0.160),
        ("number", numbering, Some(2.0), 10.0),
        ("pdf_200", pdf_time, Some(30.0), 90.0),
        ("dxf_200", dxf_time, None, 90.0),
        ("checks", checks_time, None, 90.0),
        ("reports", reports_time, None, 90.0),
    ] {
        if let Some(target) = target {
            eprintln!(
                "PERF target {label}: {:.3} ms / {:.3} ms ({})",
                actual.as_secs_f64() * 1000.0,
                target * 1000.0,
                if actual.as_secs_f64() <= target {
                    "met"
                } else {
                    "exceeded"
                }
            );
        }
        assert!(
            actual.as_secs_f64() < ci_limit,
            "{label}: {actual:?} exceeds CI budget {ci_limit}s"
        );
    }
}
