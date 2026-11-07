// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Zone divisions generated from the zones (format 10): same drawing as the
//! former fixed lines, migration of old frames and clear axis errors.
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

use cad_craft::{domain::*, persistence};

fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}

/// Factory frame as written before format 10: fixed division lines and no
/// generated marks, same identity.
fn with_fixed_lines(frame: &FrameTemplate) -> FrameTemplate {
    let mut zones = frame.zones().clone();
    zones.generated_marks = true;
    let generated = FrameTemplate::new(
        frame.name().as_str(),
        frame.sheet(),
        frame.usable(),
        zones,
        frame.elements().to_vec(),
    )
    .unwrap();
    let mut elements = frame.elements().to_vec();
    for (start, end) in generated.zone_marks() {
        elements.push(FrameElement::Line {
            start,
            end,
            width: ZONE_MARK_WIDTH,
        });
    }
    let mut zones = frame.zones().clone();
    zones.generated_marks = false;
    let json = serde_json::to_value(frame).unwrap();
    let mut json = json;
    json["zones"] = serde_json::to_value(&zones).unwrap();
    json["elements"] = serde_json::to_value(&elements).unwrap();
    serde_json::from_value(json).unwrap()
}

/// Display items of the page drawn with `frame`, sorted (order ignored).
fn drawing(frame: FrameTemplate, size: Size2D) -> Vec<String> {
    let mut project = Project::new("Marcas").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "D").unwrap())
        .unwrap();
    let page = project
        .insert_page(Page::new(project.id(), doc, "P", "1", PageKind::MultiLine, size).unwrap())
        .unwrap();
    let id = frame.id();
    project.edit_frame(FrameEdit::UpsertFrame(frame)).unwrap();
    project
        .edit_frame(FrameEdit::SetDefaultFrame(Some(id)))
        .unwrap();
    let list = project.page_display_list(page).unwrap();
    let mut items: Vec<String> = list.items.iter().map(|i| format!("{i:?}")).collect();
    items.sort();
    items
}

#[test]
fn generated_divisions_draw_exactly_like_the_former_fixed_lines() {
    for (factory, size) in [
        (FrameTemplate::factory_a3(), PaperFormat::A3.size(true)),
        (FrameTemplate::factory_a4(), PaperFormat::A4.size(false)),
    ] {
        assert!(factory.zones().generated_marks);
        let old = with_fixed_lines(&factory);
        assert!(!old.zones().generated_marks);
        assert!(old.zone_marks().is_empty());
        assert_eq!(drawing(old, size), drawing(factory, size));
    }
}

#[test]
fn reading_converts_fixed_division_lines_and_keeps_other_lines() {
    let factory = FrameTemplate::factory_a3();
    let mut old = with_fixed_lines(&factory);
    let fixed = old.elements().len();
    assert!(old.adopt_generated_zone_marks());
    assert!(old.zones().generated_marks);
    assert_eq!(
        old.elements().len(),
        fixed - 9,
        "nove divisões fixas viram geradas"
    );
    assert_eq!(old.elements(), factory.elements());
    assert!(
        !old.adopt_generated_zone_marks(),
        "segunda vez não muda nada"
    );
    // A frame whose lines do not coincide with the zones keeps its drawing.
    let usable = Rect2D::new(p(10.0, 10.0), Size2D::new(300.0, 200.0).unwrap()).unwrap();
    let lone = FrameElement::Line {
        start: p(77.0, 210.0),
        end: p(77.0, 215.0),
        width: 0.25,
    };
    let mut custom = FrameTemplate::new(
        "Própria",
        Size2D::new(320.0, 220.0).unwrap(),
        usable,
        ZoneLayout {
            columns: ZoneAxis::uniform(3, 100.0, ZoneLabelStyle::Numbers { start: 1 }),
            rows: ZoneAxis::none(),
            generated_marks: false,
        },
        vec![lone.clone()],
    )
    .unwrap();
    assert!(!custom.adopt_generated_zone_marks());
    assert_eq!(custom.elements(), &[lone]);
    assert!(!custom.zones().generated_marks);
}

#[test]
fn old_project_files_open_with_generated_divisions() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/etapa24-v9.cadcraft"
    );
    let loaded = persistence::read_project_from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(loaded.source_format, 9);
    assert!(loaded.migrated());
    for frame in loaded.project.frames() {
        let marks = frame.zone_marks();
        let still_fixed = frame.elements().iter().any(|e| {
            matches!(e, FrameElement::Line { start, end, .. }
                if marks.iter().any(|(a, b)| a == start && b == end))
        });
        assert!(!still_fixed, "{}", frame.name().as_str());
    }
    // Saved again, the file is format 10 and keeps the generated divisions.
    let bytes = persistence::write_project_to_vec(&loaded.project).unwrap();
    let again = persistence::read_project_from_slice(&bytes).unwrap();
    assert_eq!(again.source_format, persistence::FORMAT);
    assert_eq!(
        again.project.frames().count(),
        loaded.project.frames().count()
    );
    for (a, b) in again.project.frames().zip(loaded.project.frames()) {
        assert_eq!(a, b);
    }
}

#[test]
fn divisions_follow_the_zones_on_every_labelled_side() {
    let usable = Rect2D::new(p(10.0, 10.0), Size2D::new(300.0, 200.0).unwrap()).unwrap();
    let frame = FrameTemplate::new(
        "Zonas",
        Size2D::new(320.0, 220.0).unwrap(),
        usable,
        ZoneLayout {
            columns: ZoneAxis {
                sizes: vec![75.0; 4],
                style: ZoneLabelStyle::Numbers { start: 1 },
                labels_start: true,
                labels_end: true,
            },
            rows: ZoneAxis {
                sizes: vec![25.0; 8],
                style: ZoneLabelStyle::Letters { start: 0 },
                labels_start: true,
                labels_end: false,
            },
            generated_marks: true,
        },
        vec![],
    )
    .unwrap();
    let marks = frame.zone_marks();
    // 4 columns: 3 inner divisions, above and below; 8 rows: 7, on the left.
    assert_eq!(marks.len(), 3 * 2 + 7);
    assert!(marks.contains(&(p(85.0, 210.0), p(85.0, 215.0))));
    assert!(marks.contains(&(p(85.0, 5.0), p(85.0, 10.0))));
    assert!(marks.contains(&(p(5.0, 185.0), p(10.0, 185.0))));
    // Zones shorter than the usable area get a closing division.
    let mut short = frame.zones().clone();
    short.columns.sizes = vec![75.0; 3];
    short.rows = ZoneAxis::none();
    let short = FrameTemplate::new("Curta", frame.sheet(), usable, short, vec![]).unwrap();
    assert!(
        short
            .zone_marks()
            .contains(&(p(235.0, 210.0), p(235.0, 215.0)))
    );
}

#[test]
fn too_many_zones_name_the_axis() {
    let usable = Rect2D::new(p(5.0, 35.0), Size2D::new(200.0, 252.0).unwrap()).unwrap();
    let sheet = PaperFormat::A4.size(false);
    let axis =
        |n: usize, size: f64| ZoneAxis::uniform(n, size, ZoneLabelStyle::Numbers { start: 1 });
    let rows = FrameTemplate::new(
        "A4",
        sheet,
        usable,
        ZoneLayout {
            columns: axis(5, 40.0),
            rows: axis(8, 40.0),
            generated_marks: true,
        },
        vec![],
    )
    .unwrap_err();
    assert!(rows.to_string().contains("linhas"), "{rows}");
    let columns = FrameTemplate::new(
        "A4",
        sheet,
        usable,
        ZoneLayout {
            columns: axis(6, 40.0),
            rows: ZoneAxis::none(),
            generated_marks: true,
        },
        vec![],
    )
    .unwrap_err();
    assert!(columns.to_string().contains("colunas"), "{columns}");
    // 8 rows of 31.5 mm fill the 252 mm exactly.
    assert!(
        FrameTemplate::new(
            "A4",
            sheet,
            usable,
            ZoneLayout {
                columns: axis(5, 40.0),
                rows: axis(8, 31.5),
                generated_marks: true,
            },
            vec![],
        )
        .is_ok()
    );
}

/// Project read straight from `project.json`, without the v9 → v10
/// conversion: frames keep their fixed division lines.
fn without_conversion(bytes: &[u8]) -> Project {
    use std::io::Read;
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut json = String::new();
    archive
        .by_name("project.json")
        .unwrap()
        .read_to_string(&mut json)
        .unwrap();
    serde_json::from_str(&json).unwrap()
}

fn sorted_items(project: &Project, page: PageId) -> Vec<String> {
    let mut items: Vec<String> = project
        .page_display_list(page)
        .unwrap()
        .items
        .iter()
        .map(|i| format!("{i:?}"))
        .collect();
    items.sort();
    items
}

#[test]
fn converted_fixtures_draw_the_same_items_as_before() {
    // The recorded outputs of these fixtures changed only in the order of
    // drawn items: the divisions now come after the frame elements.
    for name in ["etapa13-v2", "etapa14-v3", "etapa16-v4", "etapa17-v5"] {
        let path = format!(
            "{}/tests/fixtures/{name}.cadcraft",
            env!("CARGO_MANIFEST_DIR")
        );
        let bytes = std::fs::read(path).unwrap();
        let old = without_conversion(&bytes);
        let new = persistence::read_project_from_slice(&bytes)
            .unwrap()
            .project;
        assert!(
            new.frames().any(|f| f.zones().generated_marks),
            "{name}: alguma moldura convertida"
        );
        for page in new.pages().map(Page::id) {
            assert_eq!(sorted_items(&old, page), sorted_items(&new, page), "{name}");
        }
    }
}

#[test]
fn format_10_fixture_keeps_its_generated_divisions() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/etapa30-v10.cadcraft"
    );
    let loaded = persistence::read_project_from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(loaded.source_format, 10);
    assert!(!loaded.migrated());
    let frame = loaded.project.frames().next().unwrap();
    assert!(frame.zones().generated_marks);
    assert_eq!(frame.zones().columns.sizes.len(), 4);
    assert_eq!(frame.zones().rows.sizes.len(), 8);
    // 3 column divisions above and below, 7 row divisions at each side
    // where the A4 sheet leaves a strip.
    assert!(frame.zone_marks().len() >= 3 * 2);
}
