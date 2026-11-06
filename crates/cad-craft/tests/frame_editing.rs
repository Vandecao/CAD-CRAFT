// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Frame editing, images, reversible batches, native frame packages and cross-project imports.
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

mod frames_common;

use cad_craft::{
    domain::*,
    persistence::{self, PersistenceError},
};
use frames_common::png;
use std::io::{Cursor, Write};

fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}
fn rect(x: f64, y: f64, w: f64, h: f64) -> Rect2D {
    Rect2D::new(p(x, y), Size2D::new(w, h).unwrap()).unwrap()
}
fn project_with_page() -> (Project, PageId) {
    let mut project = Project::new("Editor").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "Esquema").unwrap())
        .unwrap();
    let page = project
        .insert_page(
            Page::new(
                project.id(),
                doc,
                "Força",
                "1",
                PageKind::MultiLine,
                PaperFormat::A3.size(true),
            )
            .unwrap(),
        )
        .unwrap();
    (project, page)
}
/// Add a frame and logo in one transaction and return their identifiers.
fn framed() -> (Project, PageId, FrameTemplateId, AssetId) {
    let (mut project, page) = project_with_page();
    let frame = FrameTemplate::blank("Minha", PaperFormat::A3.size(true)).unwrap();
    let asset = ImageAsset::from_bytes(png(200, 100)).unwrap();
    let mut draft = frame.draft();
    draft.elements.push(FrameElement::Image {
        rect: rect(10.0, 10.0, 40.0, 20.0),
        asset: asset.id(),
    });
    let (fid, aid) = (frame.id(), asset.id());
    project
        .edit_frame(FrameEdit::Batch(vec![
            FrameEdit::AddAsset(asset),
            FrameEdit::UpsertFrame(frame.edited(draft).unwrap()),
            FrameEdit::SetDefaultFrame(Some(fid)),
        ]))
        .unwrap();
    (project, page, fid, aid)
}

#[test]
fn blank_frame_drafts_edits_and_duplicates() {
    let blank = FrameTemplate::blank("Nova", PaperFormat::A4.size(false)).unwrap();
    assert_eq!(blank.version(), 1);
    assert_eq!(blank.elements().len(), 1);
    let mut draft = blank.draft();
    draft.name = "Nova revisada".into();
    draft.elements.push(FrameElement::Line {
        start: p(5.0, 40.0),
        end: p(205.0, 40.0),
        width: 0.25,
    });
    draft.zones.columns = ZoneAxis::uniform(4, 50.0, ZoneLabelStyle::Letters { start: 0 });
    let edited = blank.edited(draft.clone()).unwrap();
    assert_eq!((edited.id(), edited.version()), (blank.id(), 2));
    assert_eq!(edited.elements().len(), 2);
    let copy = edited.duplicate("Cópia").unwrap();
    assert_ne!(copy.id(), edited.id());
    assert_eq!(copy.version(), 1);
    assert_eq!(copy.elements(), edited.elements());
    // Zones larger than the usable area are refused.
    draft.zones.columns = ZoneAxis::uniform(10, 50.0, ZoneLabelStyle::Numbers { start: 1 });
    assert!(blank.edited(draft).is_err());
}

#[test]
fn elements_translate_keeping_shape() {
    let image = FrameElement::Image {
        rect: rect(1.0, 2.0, 3.0, 4.0),
        asset: AssetId::new(),
    };
    let moved = image.translated(10.0, -1.0).unwrap();
    assert_eq!(moved.anchor(), p(11.0, 1.0));
    let line = FrameElement::Line {
        start: p(0.0, 0.0),
        end: p(5.0, 5.0),
        width: 0.25,
    };
    match line.translated(1.0, 1.0).unwrap() {
        FrameElement::Line { start, end, .. } => {
            assert_eq!((start, end), (p(1.0, 1.0), p(6.0, 6.0)));
        }
        _ => unreachable!(),
    }
}

#[test]
fn images_must_exist_cannot_be_removed_in_use_and_batches_roll_back() {
    let (mut project, _, fid, aid) = framed();
    assert_eq!(project.asset(aid).unwrap().width_px(), 200);
    assert!(project.edit_frame(FrameEdit::RemoveAsset(aid)).is_err());
    // A frame referencing a missing image is refused.
    let frame = project.frame(fid).unwrap().clone();
    let mut draft = frame.draft();
    draft.elements.push(FrameElement::Image {
        rect: rect(60.0, 10.0, 20.0, 20.0),
        asset: AssetId::new(),
    });
    let bad = frame.edited(draft).unwrap();
    assert!(
        project
            .edit_frame(FrameEdit::UpsertFrame(bad.clone()))
            .is_err()
    );
    // A failure at the end of a batch rolls back earlier edits.
    let before = project.clone();
    let extra = ImageAsset::from_bytes(png(10, 10)).unwrap();
    assert!(
        project
            .edit_frame(FrameEdit::Batch(vec![
                FrameEdit::AddAsset(extra),
                FrameEdit::SetReferenceFormat(ReferenceFormat::PageColumnRow),
                FrameEdit::UpsertFrame(bad),
            ]))
            .is_err()
    );
    assert_eq!(project, before);
}

#[test]
fn batch_undo_and_redo_restore_exactly() {
    let (start, ..) = project_with_page();
    let mut project = start.clone();
    let frame = FrameTemplate::blank("Minha", PaperFormat::A3.size(true)).unwrap();
    let asset = ImageAsset::from_bytes(png(64, 64)).unwrap();
    let mut draft = frame.draft();
    draft.elements.push(FrameElement::Image {
        rect: rect(10.0, 10.0, 20.0, 20.0),
        asset: asset.id(),
    });
    let delta = project
        .edit_frame(FrameEdit::Batch(vec![
            FrameEdit::AddAsset(asset),
            FrameEdit::UpsertFrame(frame.edited(draft).unwrap()),
        ]))
        .unwrap();
    let end = project.clone();
    project.restore_frame_edit(&delta, false).unwrap();
    assert_eq!(project, start);
    project.restore_frame_edit(&delta, true).unwrap();
    assert_eq!(project, end);
    // An unused image can be removed and restored.
    let (mut project, _, fid, aid) = framed();
    let frame = project.frame(fid).unwrap().clone();
    let mut draft = frame.draft();
    draft.elements.retain(|e| e.asset().is_none());
    let delta = project
        .edit_frame(FrameEdit::Batch(vec![
            FrameEdit::UpsertFrame(frame.edited(draft).unwrap()),
            FrameEdit::RemoveAsset(aid),
        ]))
        .unwrap();
    assert!(project.asset(aid).is_none());
    project.restore_frame_edit(&delta, false).unwrap();
    assert!(project.asset(aid).is_some());
}

#[test]
fn watermark_and_images_survive_cadcraft_round_trip() {
    let (mut project, ..) = framed();
    let frame = project.frames().next().unwrap().clone();
    let mut draft = frame.draft();
    let mut style = FrameTextStyle::new(20.0, TextAlign::Center);
    style.watermark = true;
    draft.elements.push(FrameElement::Text {
        origin: p(210.0, 150.0),
        text: LocalizedText::new("CÓPIA"),
        style,
    });
    project
        .edit_frame(FrameEdit::UpsertFrame(frame.edited(draft).unwrap()))
        .unwrap();
    let bytes = persistence::write_project_to_vec(&project).unwrap();
    let loaded = persistence::read_project_from_slice(&bytes).unwrap();
    assert_eq!(loaded.project, project);
    assert_eq!(
        loaded.project.assets().next().unwrap().bytes(),
        Some(png(200, 100).as_slice())
    );
    // Project JSON excludes image bytes, so technical snapshots lose their content.
    let json = serde_json::to_string(&project).unwrap();
    assert!(!json.contains("bytes"));
    let from_snapshot: Project = serde_json::from_str(&json).unwrap();
    assert!(from_snapshot.assets().next().unwrap().bytes().is_none());
    assert!(persistence::write_project_to_vec(&from_snapshot).is_err());
}

#[test]
fn missing_or_mismatched_image_entries_are_refused() {
    let (project, ..) = framed();
    let good = persistence::write_project_to_vec(&project).unwrap();
    let asset = project.assets().next().unwrap();
    let name = format!("assets/{}.png", asset.id());
    // Rewrite the container with an image of a different size.
    let mut archive = zip::ZipArchive::new(Cursor::new(good)).unwrap();
    for replacement in [None, Some(png(201, 100))] {
        let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).unwrap();
            let entry_name = entry.name().unwrap().to_string();
            let mut data = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut data).unwrap();
            if entry_name == name {
                match &replacement {
                    None => continue,
                    Some(bytes) => data = bytes.clone(),
                }
            }
            out.start_file(
                entry_name,
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored),
            )
            .unwrap();
            out.write_all(&data).unwrap();
        }
        let bytes = out.finish().unwrap().into_inner();
        assert!(persistence::read_project_from_slice(&bytes).is_err());
    }
}

#[test]
fn cadframe_round_trip_and_import_reuses_images_and_ids() {
    let (source, _, fid, _) = framed();
    let package = source.frame_package(fid).unwrap();
    let bytes = persistence::write_frame_package(&package).unwrap();
    let read = persistence::read_frame_package(&bytes).unwrap();
    assert_eq!(read, package);
    assert_eq!(read.assets[0].bytes(), Some(png(200, 100).as_slice()));

    // Importing into another project preserves identifiers.
    let (mut other, _) = project_with_page();
    let (edits, id) = other.frame_import_edits(&read).unwrap();
    assert_eq!(id, fid);
    other.edit_frame(FrameEdit::Batch(edits)).unwrap();
    assert_eq!(other.frame(fid).unwrap(), source.frame(fid).unwrap());
    assert_eq!(other.assets().count(), 1);

    // Importing into the same project creates a new frame identifier and reuses identical image
    // content.
    let mut same = source.clone();
    let (edits, id) = same.frame_import_edits(&read).unwrap();
    assert_ne!(id, fid);
    same.edit_frame(FrameEdit::Batch(edits)).unwrap();
    assert_eq!(same.frames().count(), 2);
    assert_eq!(same.assets().count(), 1);
    assert_eq!(
        same.frame(id).unwrap().elements(),
        source.frame(fid).unwrap().elements()
    );
}

#[test]
fn invalid_cadframe_files_are_refused() {
    let (project, _, fid, _) = framed();
    let package = project.frame_package(fid).unwrap();
    let good = persistence::write_frame_package(&package).unwrap();
    // A project container is not a frame package.
    let cadcraft = persistence::write_project_to_vec(&project).unwrap();
    assert!(matches!(
        persistence::read_frame_package(&cadcraft),
        Err(PersistenceError::WrongMimetype)
    ));
    assert!(persistence::read_frame_package(b"nao e zip").is_err());
    // Reject unsafe input.
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default();
    for (name, data) in [
        ("mimetype", persistence::FRAME_MIMETYPE.as_bytes()),
        ("../fora.txt", b"x".as_slice()),
    ] {
        zip.start_file(name, opts).unwrap();
        zip.write_all(data).unwrap();
    }
    let evil = zip.finish().unwrap().into_inner();
    assert!(matches!(
        persistence::read_frame_package(&evil),
        Err(PersistenceError::UnsafeEntry(_))
    ));
    // Reject a truncated package.
    assert!(persistence::read_frame_package(&good[..good.len() / 2]).is_err());
}
