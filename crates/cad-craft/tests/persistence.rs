// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Versioned native containers, legacy technical JSON, migrations and unsafe-input rejection.
//! Nothing is extracted to disk.
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

mod physical_common;

use cad_craft::{
    domain::*,
    persistence::{self, PersistenceError},
};
use std::io::{Cursor, Write};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}

/// Small complete project with pages, registered devices, a route and a derived network.
fn sample() -> Project {
    let mut project = Project::new("Fixture Etapa 12").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "Esquema").unwrap())
        .unwrap();
    let page = project
        .insert_page(
            Page::new(
                project.id(),
                doc,
                "Comando",
                "1",
                PageKind::MultiLine,
                Size2D::new(420.0, 297.0).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let library = SymbolLibrary::builtin();
    let mut reps = Vec::new();
    for (key, y) in [("contact-no", 100.0), ("coil", 150.0)] {
        let definition = library
            .get(&SymbolDefinitionId::new(format!("cad-craft.{key}")).unwrap())
            .unwrap()
            .clone();
        let delta = project
            .place_symbol(SymbolPlacement {
                page,
                definition,
                transform: Transform2D::new(p(100.0, y), 0.0, false, false).unwrap(),
                device: PlacementDevice::New {
                    designation: project.next_device_designation("K").unwrap(),
                    classification: "Relé".into(),
                },
                function_kind: FunctionKind::Coil,
            })
            .unwrap();
        reps.push(delta.representation_id());
    }
    let source = PortRef::new(reps[0], "2").unwrap();
    let target = PortRef::new(reps[1], "1").unwrap();
    let points = vec![
        project.port_position(&source).unwrap(),
        project.port_position(&target).unwrap(),
    ];
    project
        .edit_connection_drawing(ConnectionDrawingEdit::CreateRoute {
            source,
            target,
            points,
        })
        .unwrap();
    project
}

fn container(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in entries {
        zip.start_file(
            *name,
            SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
        )
        .unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap().into_inner()
}
fn manifest(format: u32, min_reader: u32) -> Vec<u8> {
    format!(r#"{{"format":{format},"min_reader":{min_reader},"writer":"teste","future":true}}"#)
        .into_bytes()
}
fn project_json(project: &Project) -> Vec<u8> {
    serde_json::to_vec(project).unwrap()
}
fn replace_all(bytes: &mut [u8], from: &[u8], to: &[u8]) {
    assert_eq!(from.len(), to.len());
    let mut i = 0;
    while i + from.len() <= bytes.len() {
        if &bytes[i..i + from.len()] == from {
            bytes[i..i + from.len()].copy_from_slice(to);
            i += from.len();
        } else {
            i += 1;
        }
    }
}

#[test]
fn round_trip_preserves_the_complete_aggregate() {
    for project in [sample(), physical_common::populated()] {
        let bytes = persistence::write_project_to_vec(&project).unwrap();
        assert_eq!(&bytes[..4], b"PK\x03\x04");
        let loaded = persistence::read_project_from_slice(&bytes).unwrap();
        assert_eq!(loaded.project, project);
        assert_eq!(loaded.source_format, persistence::FORMAT);
        assert!(!loaded.read_only);
        assert!(!loaded.migrated());
        assert_eq!(loaded.manifest.unwrap(), persistence::Manifest::current());
    }
}

#[test]
fn mimetype_is_the_first_uncompressed_entry() {
    let bytes = persistence::write_project_to_vec(&sample()).unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let first = archive.by_index(0).unwrap();
    assert_eq!(first.name().unwrap(), "mimetype");
    assert_eq!(first.compression(), CompressionMethod::Stored);
}

#[test]
fn versioned_fixtures_keep_opening() {
    // Historical fixtures v0-v5 are immutable after format 6.
    let v0 = format!("{FIXTURES}/etapa12-v0-snapshot.json");
    let v1 = format!("{FIXTURES}/etapa12-v1.cadcraft");
    // Never regenerate historical v0/v1 or v2 fixtures.
    let v2 = format!("{FIXTURES}/etapa13-v2.cadcraft");
    let v3 = format!("{FIXTURES}/etapa14-v3.cadcraft");
    // Version 4 is historical; deliberately regenerate v5 from it only with
    // CAD_CRAFT_WRITE_FIXTURES=1 cargo test -p cad-craft --test persistence.
    let v4 = format!("{FIXTURES}/etapa16-v4.cadcraft");
    let v5 = format!("{FIXTURES}/etapa17-v5.cadcraft");
    let v6 = format!("{FIXTURES}/etapa18-v6.cadcraft");
    if std::env::var_os("CAD_CRAFT_WRITE_FIXTURES").is_some() {
        let mut project = persistence::read_project_from_slice(&std::fs::read(&v4).unwrap())
            .unwrap()
            .project;
        let page = project.pages().next().unwrap().id();
        let mut styles = std::collections::BTreeMap::new();
        styles.insert(
            0,
            PrimitiveStyle {
                dash: LineDash::Dashed,
                width_mm: Some(0.35),
                fill: false,
            },
        );
        styles.insert(
            1,
            PrimitiveStyle {
                fill: true,
                ..Default::default()
            },
        );
        let symbol = SymbolDefinition::new(
            SymbolDefinitionId::new("fixture.etapa17.lamp").unwrap(),
            1,
            "Sinaleiro (fixture)",
            "Sinalização",
            vec![
                SymbolPrimitive::Ellipse {
                    center: p(0.0, 0.0),
                    radius_x: 4.0,
                    radius_y: 3.0,
                    rotation_degrees: 0.0,
                },
                SymbolPrimitive::Polyline {
                    points: vec![p(-1.0, -1.0), p(1.0, -1.0), p(0.0, 1.0)],
                    closed: true,
                },
                SymbolPrimitive::DynamicText {
                    origin: p(5.0, 0.0),
                    field: SymbolTextField::Designation,
                    height: 2.5,
                    align: TextAlign::MiddleLeft,
                    rotation_degrees: 0.0,
                },
            ],
            vec![
                SymbolPort::new("1", p(0.0, 6.0), 90.0, "X1", None).unwrap(),
                SymbolPort::new("2", p(0.0, -6.0), 270.0, "X2", None).unwrap(),
            ],
        )
        .unwrap()
        .with_styles(styles)
        .unwrap();
        project
            .place_symbol(SymbolPlacement {
                page,
                definition: symbol,
                transform: Transform2D::new(p(300.0, 150.0), 0.0, false, false).unwrap(),
                device: PlacementDevice::New {
                    designation: Designation::new("H1").unwrap(),
                    classification: "Sinaleiro".into(),
                },
                function_kind: FunctionKind::Other(Text::new("Sinalização").unwrap()),
            })
            .unwrap();
        let ellipse =
            PageGraphic::from_drag(GraphicKind::Ellipse, p(0.0, 0.0), p(20.0, 10.0), 0.25)
                .unwrap()
                .with_style(PrimitiveStyle {
                    dash: LineDash::DashDot,
                    width_mm: None,
                    fill: true,
                })
                .unwrap();
        let t = Transform2D::new(p(60.0, 120.0), 0.0, false, false).unwrap();
        project
            .insert_representation(
                Representation::new_graphic(project.id(), page, ellipse, t).unwrap(),
            )
            .unwrap();
        std::fs::write(&v5, persistence::write_project_to_vec(&project).unwrap()).unwrap();
    }
    let v5_loaded = persistence::read_project_from_slice(&std::fs::read(&v5).unwrap()).unwrap();
    let v6_loaded = persistence::read_project_from_slice(&std::fs::read(&v6).unwrap()).unwrap();
    assert_eq!(v6_loaded.source_format, 6);
    assert!(v6_loaded.migrated());
    assert_eq!(v6_loaded.project.parts().count(), 1);
    assert_eq!(v5_loaded.source_format, 5);
    assert!(v5_loaded.migrated());
    let lamp = v5_loaded
        .project
        .symbol_definition(&SymbolDefinitionId::new("fixture.etapa17.lamp").unwrap())
        .unwrap();
    assert_eq!(lamp.style(0).dash, LineDash::Dashed);
    assert!(lamp.style(1).fill);
    assert!(v5_loaded.project.representations().any(|r| {
        r.graphic()
            .is_some_and(|g| g.style().dash == LineDash::DashDot)
    }));
    let v4_loaded = persistence::read_project_from_slice(&std::fs::read(&v4).unwrap()).unwrap();
    assert_eq!(v4_loaded.source_format, 4);
    assert!(v4_loaded.migrated());
    assert_eq!(
        v4_loaded
            .project
            .representations()
            .filter(|r| r.graphic().is_some())
            .count(),
        5
    );
    assert_eq!(
        persistence::read_project_from_slice(
            &persistence::write_project_to_vec(&v4_loaded.project).unwrap()
        )
        .unwrap()
        .project,
        v4_loaded.project
    );
    let old = persistence::read_project_from_slice(&std::fs::read(&v0).unwrap()).unwrap();
    assert_eq!(old.source_format, 0);
    assert!(old.is_legacy_snapshot() && !old.read_only && old.manifest.is_none());
    let new = persistence::read_project_from_slice(&std::fs::read(&v1).unwrap()).unwrap();
    assert_eq!(new.source_format, 1);
    assert!(new.migrated() && !new.is_legacy_snapshot() && !new.read_only);
    assert_eq!(
        old.project, new.project,
        "v0 e v1 descrevem o mesmo projeto"
    );
    let current = persistence::read_project_from_slice(&std::fs::read(&v2).unwrap()).unwrap();
    assert_eq!(current.source_format, 2);
    assert!(current.migrated() && !current.is_legacy_snapshot());
    let latest = persistence::read_project_from_slice(&std::fs::read(&v3).unwrap()).unwrap();
    assert_eq!(latest.source_format, 3);
    assert!(latest.migrated());
    let asset = latest.project.assets().next().unwrap();
    assert_eq!((asset.width_px(), asset.height_px()), (40, 20));
    assert!(asset.bytes().is_some());
    let latest_frame = latest.project.frames().next().unwrap();
    assert!(
        latest_frame
            .elements()
            .iter()
            .any(|e| matches!(e, FrameElement::Text { style, .. } if style.watermark))
    );
    let framed = &current.project;
    let page = framed.pages().next().unwrap().id();
    assert_eq!(
        framed.page_frame(page).unwrap().name().as_str(),
        "CAD-CRAFT A3 paisagem"
    );
    let project = new.project;
    assert_eq!(project.devices().count(), 2);
    assert_eq!(project.connections().count(), 1);
    assert_eq!(project.connection_routes().count(), 1);
    assert_eq!(
        project
            .networks()
            .iter()
            .filter(|n| n.points().len() == 2)
            .count(),
        1
    );
}

#[test]
fn format_seven_preserves_numbering_and_old_fixtures_open() {
    let fixture = format!("{FIXTURES}/etapa20-v7.cadcraft");
    let mut project = sample();
    let id = project.connections().next().unwrap().id();
    project
        .set_connection_number(id, Some(Text::new("L1").unwrap()), true)
        .unwrap();
    let mut scheme = project.numbering_scheme().clone();
    scheme.device_format = DeviceNumberFormat::PageColumn;
    scheme.use_on_insert = true;
    project.set_numbering_scheme(scheme.clone()).unwrap();
    let bytes = persistence::write_project_to_vec(&project).unwrap();
    if std::env::var_os("CAD_CRAFT_WRITE_ETAPA20_FIXTURE").is_some() {
        std::fs::write(&fixture, &bytes).unwrap();
    }
    let loaded = persistence::read_project_from_slice(&std::fs::read(fixture).unwrap()).unwrap();
    assert_eq!(loaded.source_format, 7);
    assert!(loaded.migrated());
    assert_eq!(loaded.project.numbering_scheme(), &scheme);
    assert!(
        loaded
            .project
            .connections()
            .any(|c| c.number().is_some_and(|n| n.as_str() == "L1") && c.number_locked())
    );
    for (name, version) in [
        ("etapa12-v1.cadcraft", 1),
        ("etapa13-v2.cadcraft", 2),
        ("etapa14-v3.cadcraft", 3),
        ("etapa16-v4.cadcraft", 4),
        ("etapa17-v5.cadcraft", 5),
        ("etapa18-v6.cadcraft", 6),
    ] {
        let historical = persistence::read_project_from_slice(
            &std::fs::read(format!("{FIXTURES}/{name}")).unwrap(),
        )
        .unwrap();
        assert_eq!(historical.source_format, version);
        assert!(historical.migrated());
    }
}

#[test]
fn format_eight_preserves_report_definition() {
    let fixture = format!("{FIXTURES}/etapa23-v8.cadcraft");
    let mut project = sample();
    let doc = project.documents().next().unwrap().id();
    let request = ReportRequest::new(
        ReportKind::PageIndex,
        doc,
        Size2D::new(420.0, 297.0).unwrap(),
    );
    project.generate_report(&request).unwrap();
    let bytes = persistence::write_project_to_vec(&project).unwrap();
    let round_trip = persistence::read_project_from_slice(&bytes).unwrap();
    assert_eq!(round_trip.project, project);
    if std::env::var_os("CAD_CRAFT_WRITE_ETAPA23_FIXTURE").is_some() {
        std::fs::write(&fixture, &bytes).unwrap();
    }
    let loaded = persistence::read_project_from_slice(&std::fs::read(fixture).unwrap()).unwrap();
    assert_eq!(loaded.source_format, 8);
    assert_eq!(loaded.manifest.unwrap().min_reader, 8);
    assert!(loaded.project.pages().any(|page| page.report().is_some()));
}

#[test]
fn legacy_snapshot_with_bom_and_whitespace_is_migrated() {
    let mut bytes = "\u{feff}\n  ".as_bytes().to_vec();
    bytes.extend(project_json(&sample()));
    let loaded = persistence::read_project_from_slice(&bytes).unwrap();
    assert_eq!(loaded.source_format, 0);
}

#[test]
fn newer_formats_open_read_only_or_are_refused() {
    let json = project_json(&sample());
    let readable = container(&[
        ("mimetype", persistence::MIMETYPE.as_bytes()),
        ("manifest.json", &manifest(10, 9)),
        ("project.json", &json),
        ("assets/futuro.bin", b"ignorado"),
    ]);
    let loaded = persistence::read_project_from_slice(&readable).unwrap();
    assert!(loaded.read_only);
    assert_eq!(loaded.source_format, 10);
    let refused = container(&[
        ("mimetype", persistence::MIMETYPE.as_bytes()),
        ("manifest.json", &manifest(10, 10)),
        ("project.json", &json),
    ]);
    match persistence::read_project_from_slice(&refused) {
        Err(PersistenceError::UnsupportedFormat {
            min_reader: 10,
            supported: 9,
        }) => {}
        other => panic!("{other:?}"),
    }
    let incoherent = container(&[
        ("mimetype", persistence::MIMETYPE.as_bytes()),
        ("manifest.json", &manifest(1, 2)),
        ("project.json", &json),
    ]);
    assert!(matches!(
        persistence::read_project_from_slice(&incoherent),
        Err(PersistenceError::Manifest(_))
    ));
}

#[test]
fn missing_or_wrong_entries_are_reported() {
    let json = project_json(&sample());
    let cases: Vec<(Vec<u8>, &str)> = vec![
        (
            container(&[("manifest.json", &manifest(1, 1)), ("project.json", &json)]),
            "mimetype",
        ),
        (
            container(&[
                ("mimetype", b"text/plain"),
                ("manifest.json", &manifest(1, 1)),
                ("project.json", &json),
            ]),
            "tipo",
        ),
        (
            container(&[
                ("mimetype", persistence::MIMETYPE.as_bytes()),
                ("manifest.json", &manifest(1, 1)),
            ]),
            "project.json",
        ),
        (
            container(&[
                ("mimetype", persistence::MIMETYPE.as_bytes()),
                ("manifest.json", &manifest(1, 1)),
                ("project.json", b"{\"campo\":1}"),
            ]),
            "conteúdo",
        ),
        (b"%PDF-1.7".to_vec(), "não é um projeto"),
        (Vec::new(), "não é um projeto"),
    ];
    for (bytes, expected) in cases {
        let error = persistence::read_project_from_slice(&bytes).unwrap_err();
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn unsafe_entry_names_are_refused() {
    let json = project_json(&sample());
    for name in [
        "../fora.txt",
        "/absoluto.txt",
        "c:/x.txt",
        "a\\b.txt",
        "assets/../x",
    ] {
        let bytes = container(&[
            ("mimetype", persistence::MIMETYPE.as_bytes()),
            ("manifest.json", &manifest(1, 1)),
            ("project.json", &json),
            (name, b"x"),
        ]);
        assert!(
            matches!(
                persistence::read_project_from_slice(&bytes),
                Err(PersistenceError::UnsafeEntry(_))
            ),
            "{name}"
        );
    }
}

#[test]
fn duplicate_names_exact_or_by_case_are_refused() {
    let json = project_json(&sample());
    let mut exact = container(&[
        ("mimetype", persistence::MIMETYPE.as_bytes()),
        ("manifest.json", &manifest(1, 1)),
        ("project.json", &json),
        ("dup1.txt", b"a"),
        ("dup2.txt", b"b"),
    ]);
    replace_all(&mut exact, b"dup2.txt", b"dup1.txt");
    let cased = container(&[
        ("mimetype", persistence::MIMETYPE.as_bytes()),
        ("manifest.json", &manifest(1, 1)),
        ("project.json", &json),
        ("Project.json", b"{}"),
    ]);
    for bytes in [exact, cased] {
        assert!(matches!(
            persistence::read_project_from_slice(&bytes),
            Err(PersistenceError::UnsafeEntry(_))
        ));
    }
}

#[test]
fn declared_sizes_and_entry_counts_are_bounded() {
    let json = project_json(&sample());
    // Reject a declared central-directory size of 2 GiB before reading data.
    let mut big = container(&[
        ("mimetype", persistence::MIMETYPE.as_bytes()),
        ("manifest.json", &manifest(1, 1)),
        ("project.json", &json),
        ("assets/bomba.bin", b"0123"),
    ]);
    let central = big
        .windows(4)
        .rposition(|w| w == b"PK\x01\x02")
        .expect("último cabeçalho central");
    big[central + 24..central + 28].copy_from_slice(&0x8000_0000u32.to_le_bytes());
    assert!(matches!(
        persistence::read_project_from_slice(&big),
        Err(PersistenceError::TooLarge(_))
    ));
    // Reject an actual manifest larger than 64 KiB.
    let mut huge = manifest(1, 1);
    huge.truncate(huge.len() - 1);
    huge.extend(format!(",\"pad\":\"{}\"}}", "x".repeat(70_000)).bytes());
    let bytes = container(&[
        ("mimetype", persistence::MIMETYPE.as_bytes()),
        ("manifest.json", &huge),
        ("project.json", &json),
    ]);
    assert!(matches!(
        persistence::read_project_from_slice(&bytes),
        Err(PersistenceError::TooLarge(_))
    ));
    // Reject more than 4,096 entries.
    let names: Vec<String> = (0..=persistence::MAX_ENTRIES)
        .map(|i| format!("assets/{i}"))
        .collect();
    let mut entries: Vec<(&str, &[u8])> = vec![("mimetype", persistence::MIMETYPE.as_bytes())];
    entries.extend(names.iter().map(|n| (n.as_str(), &b""[..])));
    assert!(matches!(
        persistence::read_project_from_slice(&container(&entries)),
        Err(PersistenceError::TooLarge(_))
    ));
}

#[test]
fn encrypted_entries_are_refused() {
    let json = project_json(&sample());
    let mut bytes = container(&[
        ("mimetype", persistence::MIMETYPE.as_bytes()),
        ("manifest.json", &manifest(1, 1)),
        ("project.json", &json),
    ]);
    // Central-directory flag bit 0 marks an encrypted entry.
    let central = bytes.windows(4).rposition(|w| w == b"PK\x01\x02").unwrap();
    bytes[central + 8] |= 1;
    assert!(persistence::read_project_from_slice(&bytes).is_err());
}

#[test]
fn invalid_projects_are_not_written() {
    // Validate aggregate integrity before writing bytes, and revalidate decoded contents when
    // reading.
    let project = sample();
    let mut value = serde_json::to_value(&project).unwrap();
    value["connection_points"] = serde_json::json!([]);
    let bytes = container(&[
        ("mimetype", persistence::MIMETYPE.as_bytes()),
        ("manifest.json", &manifest(1, 1)),
        ("project.json", value.to_string().as_bytes()),
    ]);
    assert!(persistence::read_project_from_slice(&bytes).is_err());
}

/// Standard JSON decoding can change the last bit of long floats and break exact bounds.
/// float_roundtrip preserves every serialized f64 bit when reopening native packages.
#[test]
fn json_floats_round_trip_bit_exactly() {
    for i in 0..50_000u32 {
        let v = 20.4593349988129 + f64::from(i) * 0.000_123_456_789_012_3;
        let w = v / 3.0;
        for x in [v, w, -w] {
            let text = serde_json::to_string(&x).unwrap();
            let back: f64 = serde_json::from_str(&text).unwrap();
            assert_eq!(x.to_bits(), back.to_bits(), "{text}");
        }
    }
}
