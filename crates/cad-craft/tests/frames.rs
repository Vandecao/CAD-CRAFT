// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Frames, zones, special text fields and zone-based references.
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

use cad_craft::{domain::*, persistence};

fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}
fn key(k: &str) -> Text {
    Text::new(k).unwrap()
}
fn project() -> (Project, DocumentId, PageId, PageId) {
    let mut project = Project::new("Molduras").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "Esquema").unwrap())
        .unwrap();
    let mut pages = vec![];
    for (n, size) in [
        ("1", PaperFormat::A3.size(true)),
        ("2", PaperFormat::A4.size(false)),
    ] {
        pages.push(
            project
                .insert_page(
                    Page::new(
                        project.id(),
                        doc,
                        format!("Página {n}"),
                        n,
                        PageKind::MultiLine,
                        size,
                    )
                    .unwrap(),
                )
                .unwrap(),
        );
    }
    project.reorder_pages(doc, &pages).unwrap();
    (project, doc, pages[0], pages[1])
}
/// Test frame: usable area 0..300 by 0..200, columns 50/100/150 and three rows.
fn custom() -> FrameTemplate {
    FrameTemplate::new(
        "Teste",
        Size2D::new(320.0, 220.0).unwrap(),
        Rect2D::new(p(10.0, 10.0), Size2D::new(300.0, 200.0).unwrap()).unwrap(),
        ZoneLayout {
            columns: ZoneAxis {
                sizes: vec![50.0, 100.0, 150.0],
                style: ZoneLabelStyle::Numbers { start: 1 },
                labels_start: true,
                labels_end: true,
            },
            rows: ZoneAxis {
                sizes: vec![60.0, 60.0, 80.0],
                style: ZoneLabelStyle::Letters { start: 0 },
                labels_start: true,
                labels_end: false,
            },
        },
        vec![FrameElement::Special {
            origin: p(20.0, 5.0),
            field: SpecialField::ProjectName,
            index: None,
            style: FrameTextStyle::new(3.0, TextAlign::BottomLeft),
        }],
    )
    .unwrap()
}

#[test]
fn factory_frames_are_valid_and_divide_the_usable_area() {
    for (frame, columns, width) in [
        (FrameTemplate::factory_a3(), 10, 41.0),
        (FrameTemplate::factory_a4(), 5, 40.0),
    ] {
        frame.validate().unwrap();
        let zones = &frame.zones().columns;
        assert_eq!(zones.sizes.len(), columns);
        assert!(zones.sizes.iter().all(|s| (s - width).abs() < 1e-9));
        assert!((zones.total() - frame.usable().size().width()).abs() < 1e-9);
        assert!(frame.zones().rows.sizes.is_empty());
        assert!(frame.elements().iter().any(|e| matches!(
            e,
            FrameElement::Special {
                field: SpecialField::ProjectName,
                ..
            }
        )));
    }
    assert_eq!(
        FrameTemplate::factory_a3().sheet(),
        PaperFormat::A3.size(true)
    );
    assert_eq!(
        FrameTemplate::factory_a4().sheet(),
        PaperFormat::A4.size(false)
    );
}

#[test]
fn zones_with_different_widths_and_edges_are_exact() {
    let f = custom();
    let col = |x| f.zone_at(p(x, 100.0)).and_then(|z| z.column);
    assert_eq!(col(10.0).as_deref(), Some("1")); // borda esquerda
    assert_eq!(col(59.999).as_deref(), Some("1"));
    assert_eq!(col(60.0).as_deref(), Some("2")); // limite pertence à seguinte
    assert_eq!(col(159.9).as_deref(), Some("2"));
    assert_eq!(col(160.0).as_deref(), Some("3"));
    assert_eq!(col(310.0).as_deref(), Some("3")); // The right boundary is included.
    assert_eq!(col(310.1), None);
    assert_eq!(col(9.9), None);
    // Rows run top to bottom; the usable area ends at Y=210.
    let row = |y| f.zone_at(p(100.0, y)).and_then(|z| z.row);
    assert_eq!(row(210.0).as_deref(), Some("A"));
    assert_eq!(row(150.1).as_deref(), Some("A"));
    assert_eq!(row(150.0).as_deref(), Some("B"));
    assert_eq!(row(90.0).as_deref(), Some("C"));
    assert_eq!(row(10.0).as_deref(), Some("C"));
    assert_eq!(f.zone_at(p(0.0, 0.0)), None);
}

#[test]
fn invalid_frames_are_refused() {
    let usable = Rect2D::new(p(10.0, 10.0), Size2D::new(300.0, 200.0).unwrap()).unwrap();
    let sheet = Size2D::new(320.0, 220.0).unwrap();
    let zones = |sizes: Vec<f64>| ZoneLayout {
        columns: ZoneAxis {
            sizes,
            style: ZoneLabelStyle::Numbers { start: 1 },
            labels_start: true,
            labels_end: false,
        },
        rows: ZoneAxis::none(),
    };
    assert!(FrameTemplate::new("x", sheet, usable, zones(vec![200.0, 150.0]), vec![]).is_err());
    assert!(FrameTemplate::new("x", sheet, usable, zones(vec![0.0]), vec![]).is_err());
    let outside = Rect2D::new(p(100.0, 10.0), Size2D::new(300.0, 200.0).unwrap()).unwrap();
    assert!(FrameTemplate::new("x", sheet, outside, zones(vec![]), vec![]).is_err());
    let empty_text = FrameElement::Text {
        origin: p(1.0, 1.0),
        text: LocalizedText::new(" "),
        style: FrameTextStyle::new(2.0, TextAlign::Center),
    };
    assert!(FrameTemplate::new("x", sheet, usable, zones(vec![]), vec![empty_text]).is_err());
}

#[test]
fn reference_labels_follow_frame_zones_and_format() {
    let (mut project, _, a, b) = project();
    let point = p(70.0, 100.0);
    assert!(
        project
            .reference_label(a, point)
            .starts_with("Página 1 · X 70.000")
    );
    let a3 = FrameTemplate::factory_a3();
    let a3_id = a3.id();
    project.edit_frame(FrameEdit::UpsertFrame(a3)).unwrap();
    project
        .edit_frame(FrameEdit::SetDefaultFrame(Some(a3_id)))
        .unwrap();
    // With 41 mm columns starting at X=5, X=70 is in column 2.
    assert_eq!(project.reference_label(a, point), "/1.2");
    assert_eq!(project.reference_label(a, p(414.0, 100.0)), "/1.10");
    // The title block is outside the zone area, so use coordinate references.
    assert!(
        project
            .reference_label(a, p(70.0, 10.0))
            .starts_with("Página 1")
    );
    let custom = custom();
    let custom_id = custom.id();
    project.edit_frame(FrameEdit::UpsertFrame(custom)).unwrap();
    project
        .edit_frame(FrameEdit::SetPageFrame {
            page: b,
            frame: Some(custom_id),
        })
        .unwrap();
    assert_eq!(project.reference_label(b, p(200.0, 100.0)), "/2.3");
    project
        .edit_frame(FrameEdit::SetReferenceFormat(
            ReferenceFormat::PageColumnRow,
        ))
        .unwrap();
    assert_eq!(project.reference_label(b, p(200.0, 100.0)), "/2.3B");
    // Changing the page frame recalculates the reference.
    project
        .edit_frame(FrameEdit::SetPageFrame {
            page: b,
            frame: None,
        })
        .unwrap();
    assert_eq!(project.reference_label(b, p(200.0, 100.0)), "/2.5");
}

#[test]
fn device_references_carry_the_zone_label() {
    let (mut project, _, a, _) = project();
    let a3 = FrameTemplate::factory_a3();
    let id = a3.id();
    project.edit_frame(FrameEdit::UpsertFrame(a3)).unwrap();
    project
        .edit_frame(FrameEdit::SetDefaultFrame(Some(id)))
        .unwrap();
    let coil = SymbolLibrary::builtin()
        .get(&SymbolDefinitionId::new("cad-craft.coil").unwrap())
        .unwrap()
        .clone();
    let delta = project
        .place_symbol(SymbolPlacement {
            page: a,
            definition: coil,
            transform: Transform2D::new(p(130.0, 150.0), 0.0, false, false).unwrap(),
            device: PlacementDevice::New {
                designation: Designation::new("K1").unwrap(),
                classification: "Relé".into(),
            },
            function_kind: FunctionKind::Coil,
        })
        .unwrap();
    let refs = project.device_references(delta.device_id()).unwrap();
    assert_eq!(refs[0].reference, "/1.4");
}

#[test]
fn special_texts_resolve_project_page_and_indexed_properties() {
    let (mut project, _, a, b) = project();
    project.set_identification(Some(key("PRJ-7")));
    for (k, v) in [
        ("Autor", "Ana"),
        ("Revisão.1", "Emissão inicial"),
        ("Revisão.2", "Ajuste"),
    ] {
        project
            .edit_frame(FrameEdit::SetProjectMetadata {
                key: key(k),
                value: Some(v.into()),
            })
            .unwrap();
    }
    project
        .edit_frame(FrameEdit::SetPageMetadata {
            page: b,
            key: key("Data"),
            value: Some("03/11/2026".into()),
        })
        .unwrap();
    let r = |page, field: SpecialField, index| project.resolve_special(page, &field, index);
    assert_eq!(r(a, SpecialField::ProjectName, None), "Molduras");
    assert_eq!(r(a, SpecialField::ProjectIdentification, None), "PRJ-7");
    assert_eq!(
        r(a, SpecialField::ProjectProperty(key("Autor")), None),
        "Ana"
    );
    assert_eq!(
        r(a, SpecialField::ProjectProperty(key("Revisão")), Some(2)),
        "Ajuste"
    );
    assert_eq!(
        r(a, SpecialField::ProjectProperty(key("Revisão")), Some(3)),
        ""
    );
    assert_eq!(
        r(b, SpecialField::PageProperty(key("Data")), None),
        "03/11/2026"
    );
    assert_eq!(r(a, SpecialField::PageProperty(key("Data")), None), "");
    assert_eq!(r(b, SpecialField::PageNumber, None), "2");
    assert_eq!(r(b, SpecialField::PageName, None), "Página 2");
    assert_eq!(r(b, SpecialField::DocumentName, None), "Esquema");
    assert_eq!(r(b, SpecialField::PagePosition, None), "2");
    assert_eq!(r(b, SpecialField::PageCountInDocument, None), "2");
    assert_eq!(r(b, SpecialField::PageCountInProject, None), "2");
    assert_eq!(r(b, SpecialField::PreviousPageNumber, None), "1");
    assert_eq!(r(a, SpecialField::PreviousPageNumber, None), "");
    assert_eq!(r(a, SpecialField::NextPageNumber, None), "2");
    // Renaming the project and page number updates field values without modifying the frame.
    project.identify_page(b, "20").unwrap();
    assert_eq!(
        project.resolve_special(b, &SpecialField::PageNumber, None),
        "20"
    );
    assert_eq!(
        project.resolve_special(a, &SpecialField::NextPageNumber, None),
        "20"
    );
}

#[test]
fn frame_edits_undo_and_redo_exactly() {
    let (mut project, _, a, b) = project();
    let start = project.clone();
    let a3 = FrameTemplate::factory_a3();
    let a4 = FrameTemplate::factory_a4();
    let (a3_id, a4_id) = (a3.id(), a4.id());
    let edits = vec![
        FrameEdit::UpsertFrame(a3.clone()),
        FrameEdit::UpsertFrame(a4),
        FrameEdit::SetDefaultFrame(Some(a3_id)),
        FrameEdit::SetPageFrame {
            page: b,
            frame: Some(a4_id),
        },
        FrameEdit::SetReferenceFormat(ReferenceFormat::PageColumnRow),
        FrameEdit::SetProjectMetadata {
            key: key("Cliente"),
            value: Some("ACME".into()),
        },
        FrameEdit::SetPageMetadata {
            page: a,
            key: key("Desenhista"),
            value: Some("Bia".into()),
        },
        FrameEdit::UpsertFrame(
            a3.revised(
                key("A3 revisada"),
                a3.zones().clone(),
                a3.elements().to_vec(),
            )
            .unwrap(),
        ),
    ];
    let deltas: Vec<_> = edits
        .into_iter()
        .map(|e| project.edit_frame(e).unwrap())
        .collect();
    let end = project.clone();
    assert_eq!(project.page_frame(b).unwrap().id(), a4_id);
    assert_eq!(project.page_frame(a).unwrap().version(), 2);
    for d in deltas.iter().rev() {
        project.restore_frame_edit(d, false).unwrap();
    }
    assert_eq!(project, start);
    for d in &deltas {
        project.restore_frame_edit(d, true).unwrap();
    }
    assert_eq!(project, end);
    assert!(project.restore_frame_edit(&deltas[0], true).is_err());
    assert_eq!(project, end);
    // Empty metadata removes the key.
    let removal = project
        .edit_frame(FrameEdit::SetProjectMetadata {
            key: key("Cliente"),
            value: Some("  ".into()),
        })
        .unwrap();
    assert!(project.metadata().get(&key("Cliente")).is_none());
    project.restore_frame_edit(&removal, false).unwrap();
    assert_eq!(project, end);
}

#[test]
fn frames_in_use_cannot_be_removed_and_references_must_exist() {
    let (mut project, _, a, _) = project();
    let a3 = FrameTemplate::factory_a3();
    let id = a3.id();
    project.edit_frame(FrameEdit::UpsertFrame(a3)).unwrap();
    project
        .edit_frame(FrameEdit::SetPageFrame {
            page: a,
            frame: Some(id),
        })
        .unwrap();
    assert!(project.edit_frame(FrameEdit::RemoveFrame(id)).is_err());
    assert!(
        project
            .edit_frame(FrameEdit::SetDefaultFrame(Some(FrameTemplateId::new())))
            .is_err()
    );
    project
        .edit_frame(FrameEdit::SetPageFrame {
            page: a,
            frame: None,
        })
        .unwrap();
    let removed = project.edit_frame(FrameEdit::RemoveFrame(id)).unwrap();
    assert_eq!(project.frames().count(), 0);
    project.restore_frame_edit(&removed, false).unwrap();
    assert_eq!(project.frames().count(), 1);
}

#[test]
fn frames_round_trip_and_dangling_frame_references_are_refused() {
    let (mut project, _, a, _) = project();
    let custom = custom();
    let id = custom.id();
    project.edit_frame(FrameEdit::UpsertFrame(custom)).unwrap();
    project
        .edit_frame(FrameEdit::SetPageFrame {
            page: a,
            frame: Some(id),
        })
        .unwrap();
    project
        .edit_frame(FrameEdit::SetReferenceFormat(
            ReferenceFormat::PageColumnRow,
        ))
        .unwrap();
    let bytes = persistence::write_project_to_vec(&project).unwrap();
    assert_eq!(
        persistence::read_project_from_slice(&bytes)
            .unwrap()
            .project,
        project
    );
    let json = serde_json::to_value(&project).unwrap();
    let back: Project = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(back, project);
    let mut broken = json;
    broken["frames"] = serde_json::json!([]);
    assert!(serde_json::from_value::<Project>(broken).is_err());
}

#[test]
fn projects_without_frames_serialize_as_before() {
    let (project, ..) = project();
    let json = serde_json::to_value(&project).unwrap();
    for field in ["frames", "default_frame", "reference_format"] {
        assert!(json.get(field).is_none(), "{field}");
    }
    assert!(json["pages"][0].get("frame").is_none());
}

#[test]
fn paper_formats_match_sizes_in_both_orientations() {
    assert_eq!(
        PaperFormat::A0.size(true),
        Size2D::new(1189.0, 841.0).unwrap()
    );
    assert_eq!(
        PaperFormat::A4.size(false),
        Size2D::new(210.0, 297.0).unwrap()
    );
    assert_eq!(
        PaperFormat::matching(Size2D::new(297.0, 420.0).unwrap()),
        Some((PaperFormat::A3, false))
    );
    assert_eq!(
        PaperFormat::matching(Size2D::new(300.0, 300.0).unwrap()),
        None
    );
}
