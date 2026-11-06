// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Verify page DXF R12 with an independent group-code reader and the frame importer.
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

mod output_common;

use cad_craft::display::{DisplayItem, DisplayList};
use cad_craft::domain::*;
use cad_craft::dxf_export::{dxf_file_names, export_dxf_list, export_dxf_pages};
use cad_craft::interchange::dxf::{DxfImportOptions, import};
use cad_craft::pdf::{PdfError, PdfScope};
use output_common::*;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

/// Decode ANSI 1252 with the inverse of the Core WinAnsi mapping.
fn decode(bytes: &[u8]) -> String {
    let mut table: BTreeMap<u8, char> = BTreeMap::new();
    for c in (0u32..0x2200).filter_map(char::from_u32) {
        if let Some(b) = cad_craft::font::win_ansi(c) {
            table.entry(b).or_insert(c);
        }
    }
    bytes
        .iter()
        .map(|b| table.get(b).copied().unwrap_or('\u{FFFD}'))
        .collect()
}

/// Read the group-code/value pairs.
fn pairs(bytes: &[u8]) -> Vec<(i32, String)> {
    let lines: Vec<&[u8]> = bytes
        .split(|b| *b == b'\n')
        .map(|l| l.strip_suffix(b"\r").unwrap_or(l))
        .collect();
    lines
        .chunks(2)
        .filter(|c| c.len() == 2)
        .map(|c| {
            let code = String::from_utf8_lossy(c[0])
                .trim()
                .parse()
                .expect("código");
            (code, decode(c[1]))
        })
        .collect()
}

/// Group entity pairs by type; append VERTEX records to the preceding POLYLINE.
#[derive(Debug, Default)]
struct Entity {
    kind: String,
    values: Vec<(i32, String)>,
    vertices: Vec<(f64, f64)>,
}
impl Entity {
    fn get(&self, code: i32) -> Option<&str> {
        self.values
            .iter()
            .find(|(c, _)| *c == code)
            .map(|(_, v)| v.as_str())
    }
    fn f(&self, code: i32) -> f64 {
        self.get(code).unwrap().parse().unwrap()
    }
}

struct Dxf {
    header: BTreeMap<String, Vec<(i32, String)>>,
    layers: BTreeSet<String>,
    entities: Vec<Entity>,
}

fn read(bytes: &[u8]) -> Dxf {
    let pairs = pairs(bytes);
    let mut header = BTreeMap::new();
    let mut layers = BTreeSet::new();
    let mut entities: Vec<Entity> = vec![];
    let mut section = String::new();
    let mut i = 0;
    while i < pairs.len() {
        let (code, value) = &pairs[i];
        if *code == 0 && value == "ENDSEC" {
            section.clear();
            i += 1;
            continue;
        }
        if *code == 0 && value == "SECTION" {
            section = pairs[i + 1].1.clone();
            i += 2;
            continue;
        }
        match section.as_str() {
            "HEADER" if *code == 9 => {
                let mut values = vec![];
                let mut j = i + 1;
                while j < pairs.len() && pairs[j].0 != 9 && pairs[j].0 != 0 {
                    values.push(pairs[j].clone());
                    j += 1;
                }
                header.insert(value.clone(), values);
                i = j;
                continue;
            }
            "TABLES" if *code == 0 && value == "LAYER" => {
                layers.insert(pairs[i + 1].1.clone());
            }
            "ENTITIES" if *code == 0 && value != "ENDSEC" && value != "SEQEND" => {
                let mut entity = Entity {
                    kind: value.clone(),
                    ..Entity::default()
                };
                let mut j = i + 1;
                while j < pairs.len() && pairs[j].0 != 0 {
                    entity.values.push(pairs[j].clone());
                    j += 1;
                }
                if entity.kind == "VERTEX" {
                    let (x, y) = (entity.f(10), entity.f(20));
                    entities.last_mut().expect("POLYLINE").vertices.push((x, y));
                } else {
                    entities.push(entity);
                }
                i = j;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    Dxf {
        header,
        layers,
        entities,
    }
}

fn key(x: f64, y: f64) -> (i64, i64) {
    ((x * 1000.0).round() as i64, (y * 1000.0).round() as i64)
}

fn texts(dxf: &Dxf) -> Vec<String> {
    dxf.entities
        .iter()
        .filter(|e| e.kind == "TEXT")
        .map(|e| e.get(1).unwrap().to_owned())
        .collect()
}

fn page_dxf(f: &Fixture, index: usize) -> (DisplayList, Vec<u8>) {
    let list = f.project.page_display_list(f.pages[index]).unwrap();
    let (bytes, _) = export_dxf_list(&list);
    (list, bytes)
}

/// Verify millimetre units, sheet extents, category layers, text and frame-import compatibility.
#[test]
fn page_dxf_has_units_extent_layers_and_expected_texts() {
    let f = fixture("Painel CCM");
    let (_, bytes) = page_dxf(&f, 0);
    let dxf = read(&bytes);
    assert_eq!(dxf.header["$ACADVER"][0].1, "AC1009");
    assert_eq!(dxf.header["$INSUNITS"][0].1, "4");
    assert_eq!(dxf.header["$DWGCODEPAGE"][0].1, "ANSI_1252");
    let ext = &dxf.header["$EXTMAX"];
    assert_eq!((ext[0].1.as_str(), ext[1].1.as_str()), ("420", "297"));
    for layer in [
        "CAD-CRAFT Moldura",
        "CAD-CRAFT Simbolos 0.25mm",
        "CAD-CRAFT Fios 0.35mm",
        "CAD-CRAFT Textos",
        "CAD-CRAFT Referencias",
        "CAD-CRAFT Guias 0.18mm",
    ] {
        assert!(dxf.layers.contains(layer), "{layer}: {:?}", dxf.layers);
    }
    let shown = texts(&dxf);
    let coil_ref = f.project.short_reference(f.pages[1], p(60.0, 160.0));
    for expected in ["Painel CCM", "-K1", "-K2", "1"] {
        assert!(shown.iter().any(|t| t == expected), "{expected}: {shown:?}");
    }
    assert!(shown.iter().any(|t| t.contains(&coil_ref)), "{shown:?}");
    // Entities use the declared layers.
    for e in &dxf.entities {
        let layer = e.get(8).unwrap();
        assert!(dxf.layers.contains(layer), "{layer}");
    }
    let second = read(&page_dxf(&f, 1).1);
    let ref_k1 = f.project.short_reference(f.pages[0], p(130.0, 150.0));
    assert!(texts(&second).iter().any(|t| t == &ref_k1));
    // The frame importer accepts the exported file.
    let imported = import(&bytes, &DxfImportOptions::new("página")).unwrap();
    assert!(!imported.package.frame.elements().is_empty());
}

/// Path points and text origins match the display list within 0.001 mm; images become reported
/// boxes.
#[test]
fn geometry_matches_the_display_list() {
    let f = fixture("Geometria");
    let (list, bytes) = page_dxf(&f, 0);
    let (_, report) = export_dxf_list(&list);
    let dxf = read(&bytes);
    let mut points = BTreeSet::new();
    let mut origins = BTreeSet::new();
    for e in &dxf.entities {
        match e.kind.as_str() {
            "LINE" => {
                points.insert(key(e.f(10), e.f(20)));
                points.insert(key(e.f(11), e.f(21)));
            }
            "POLYLINE" => points.extend(e.vertices.iter().map(|(x, y)| key(*x, *y))),
            "SOLID" => {
                for c in [10, 11, 12] {
                    points.insert(key(e.f(c), e.f(c + 10)));
                }
            }
            "TEXT" => {
                origins.insert(key(e.f(10), e.f(20)));
            }
            _ => {}
        }
    }
    let mut paths = 0;
    for item in &list.items {
        match item {
            DisplayItem::Path { points: pts, .. } => {
                paths += 1;
                for q in pts {
                    assert!(points.contains(&key(q.x(), q.y())), "{q:?}");
                }
            }
            DisplayItem::Text(t) => {
                let (x, y) = t.map(0.0, t.baseline);
                assert!(origins.contains(&key(x, y)), "{}", t.text);
            }
            _ => {}
        }
    }
    assert!(paths > 10);
    assert_eq!(report.images, 1);
    assert!(report.summary().contains("caixa"));
}

/// Accents use ANSI 1252; other characters use reported DXF Unicode escapes.
#[test]
fn accents_are_ansi_and_other_characters_are_escaped() {
    let f = fixture("Ação elétrica Ω");
    let list = f.project.page_display_list(f.pages[0]).unwrap();
    let (bytes, report) = export_dxf_list(&list);
    assert!(bytes.windows(2).any(|w| w == [0xE7, 0xE3]), "ç ã em 1252");
    let shown = texts(&read(&bytes));
    assert!(
        shown.iter().any(|t| t == "Ação elétrica \\U+03A9"),
        "{shown:?}"
    );
    assert_eq!(report.escaped.iter().collect::<String>(), "Ω");
}

/// Verify safe unique names, scope selection, cancellation and empty output.
#[test]
fn file_names_are_safe_unique_and_scope_is_respected() {
    let mut project = Project::new("Painel: CCM/1").unwrap();
    let mut pages = vec![];
    for _ in 0..2 {
        let doc = project
            .insert_document(Document::new(project.id(), "Esquema").unwrap())
            .unwrap();
        pages.push(
            project
                .insert_page(
                    Page::new(
                        project.id(),
                        doc,
                        "Força",
                        "1",
                        PageKind::MultiLine,
                        PaperFormat::A4.size(true),
                    )
                    .unwrap(),
                )
                .unwrap(),
        );
    }
    let names: Vec<_> = dxf_file_names(&project, &pages)
        .into_iter()
        .map(|(_, n)| n)
        .collect();
    assert_eq!(
        names,
        [
            "Painel_ CCM_1_Esquema_1.dxf",
            "Painel_ CCM_1_Esquema_1_2.dxf"
        ]
    );
    let (files, report) =
        export_dxf_pages(&project, &PdfScope::Pages(vec![pages[1]]), &mut |_, _| true).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].page, pages[1]);
    assert_eq!(report.pages, 1);
    assert!(matches!(
        export_dxf_pages(&project, &PdfScope::Pages(vec![]), &mut |_, _| true),
        Err(PdfError::Empty)
    ));
    assert!(matches!(
        export_dxf_pages(&project, &PdfScope::Project, &mut |done, _| done < 1),
        Err(PdfError::Cancelled)
    ));
}

/// Export 200 typical pages within ten seconds in Release.
#[test]
fn two_hundred_pages_export_dxf_within_the_time_limit() {
    let project = scale_project(200);
    let start = Instant::now();
    let (files, report) = export_dxf_pages(&project, &PdfScope::Project, &mut |_, _| true).unwrap();
    let elapsed = start.elapsed();
    assert_eq!(files.len(), 200);
    assert_eq!(report.pages, 200);
    let size: usize = files.iter().map(|f| f.bytes.len()).sum();
    eprintln!(
        "DXF: 200 páginas, {:.1} MiB, {:?}",
        size as f64 / 1_048_576.0,
        elapsed
    );
    let limit = if cfg!(debug_assertions) { 120 } else { 10 };
    assert!(elapsed.as_secs() < limit, "{elapsed:?}");
}

/// Verify SOLID triangle fans, dashed line types and transformed rotated/mirrored text.
#[test]
fn fill_dash_and_text_orientation_are_written() {
    use cad_craft::display::{DisplayColor, DisplayLayer, DisplayStroke, DisplayText};
    let square = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)].map(|(x, y)| p(x, y));
    let text = |matrix| {
        DisplayItem::Text(DisplayText {
            text: "A".into(),
            matrix,
            size: 2.5,
            baseline: 0.0,
            width: None,
            color: DisplayColor::Ink,
            layer: DisplayLayer::Texts,
        })
    };
    let list = DisplayList {
        page: PageId::new(),
        width: 100.0,
        height: 50.0,
        items: vec![
            DisplayItem::Path {
                points: square.to_vec(),
                closed: true,
                stroke: None,
                fill: Some(DisplayColor::Ink),
                layer: DisplayLayer::Junctions,
            },
            DisplayItem::Path {
                points: vec![p(0.0, 20.0), p(50.0, 20.0)],
                closed: false,
                stroke: Some(DisplayStroke {
                    width: 0.25,
                    dash: vec![3.0, 1.5],
                    color: DisplayColor::Ink,
                }),
                fill: None,
                layer: DisplayLayer::Symbols,
            },
            text([0.0, 1.0, -1.0, 0.0, 5.0, 5.0]),
            text([-1.0, 0.0, 0.0, 1.0, 5.0, 5.0]),
        ],
    };
    let (bytes, report) = export_dxf_list(&list);
    let dxf = read(&bytes);
    let solids: Vec<_> = dxf.entities.iter().filter(|e| e.kind == "SOLID").collect();
    assert_eq!(solids.len(), 2, "quadrado em dois triângulos");
    let line = dxf.entities.iter().find(|e| e.kind == "LINE").unwrap();
    assert_eq!(line.get(6), Some("CAD-CRAFT-TRACO-1"));
    assert!(contains_ltype(&bytes, "CAD-CRAFT-TRACO-1"));
    let texts: Vec<_> = dxf.entities.iter().filter(|e| e.kind == "TEXT").collect();
    assert_eq!(texts[0].f(50), 90.0);
    assert_eq!(texts[1].get(71), Some("2"), "espelhado em X");
    assert_eq!(report.entities, 5);
}

fn contains_ltype(bytes: &[u8], name: &str) -> bool {
    let pairs = pairs(bytes);
    pairs
        .windows(2)
        .any(|w| w[0] == (0, "LTYPE".into()) && w[1] == (2, name.into()))
}

/// Wire numbers sit beside vertical lines and above horizontal lines, never on top of the wire.
#[test]
fn wire_number_sits_beside_the_wire() {
    use cad_craft::display::{WIRE_NUMBER_GAP, wire_number_placement};
    let vertical = [p(130.0, 140.0), p(130.0, 110.0)];
    let (at, align) = wire_number_placement(&vertical, None).unwrap();
    assert_eq!((at.x(), at.y()), (130.0 + WIRE_NUMBER_GAP, 125.0));
    assert_eq!(align, TextAlign::MiddleLeft);
    let horizontal = [p(10.0, 50.0), p(15.0, 50.0), p(15.0, 20.0), p(60.0, 20.0)];
    let (at, align) = wire_number_placement(&horizontal, None).unwrap();
    assert_eq!((at.x(), at.y()), (37.5, 20.0 + WIRE_NUMBER_GAP));
    assert_eq!(align, TextAlign::BottomCenter);
    let explicit = p(134.0, 125.0);
    assert_eq!(
        wire_number_placement(&vertical, Some(explicit)),
        Some((explicit, TextAlign::MiddleLeft))
    );
    // In the shared sheet fixture, number 1 is to the right of the vertical wire.
    let f = fixture("Fio");
    let list = f.project.page_display_list(f.pages[0]).unwrap();
    let number = list
        .texts()
        .find(|t| t.text == "1" && t.layer == cad_craft::display::DisplayLayer::Texts)
        .unwrap();
    let (x, _) = number.map(0.0, 0.0);
    assert!(x > 130.0, "{x}");
}
