// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Fixture helpers use validated synthetic inputs and fail immediately on setup errors.
#![allow(clippy::unwrap_used)]
use cad_craft::domain::*;
use cad_gui::{
    editing::{Edit, GraphicEdit},
    state::{Selection, Session},
};

fn fixture() -> (Session, Vec<RepresentationId>) {
    let mut s = Session::default();
    s.replace(cad_gui::demo::project().unwrap(), true).unwrap();
    let page = s
        .project()
        .unwrap()
        .representations()
        .next()
        .unwrap()
        .page_id();
    let ids = s
        .project()
        .unwrap()
        .representations_on_page(page)
        .unwrap()
        .iter()
        .take(2)
        .map(|r| r.id())
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 2);
    s.open_page(page).unwrap();
    s.select_graphics(ids.clone());
    (s, ids)
}
fn transform(s: &Session, id: RepresentationId) -> Transform2D {
    s.project().unwrap().representation(id).unwrap().transform()
}
#[test]
fn group_translation_is_one_reversible_transaction_preserving_snapshot() {
    let (mut s, ids) = fixture();
    let before = s.snapshot().unwrap();
    let old = ids.iter().map(|id| transform(&s, *id)).collect::<Vec<_>>();
    s.dispatch_graphic(GraphicEdit::Translate(8.0, -4.0))
        .unwrap();
    for (id, t) in ids.iter().zip(old) {
        assert_eq!(
            transform(&s, *id).translation(),
            t.translation().translated(8.0, -4.0).unwrap()
        );
    }
    let after = s.snapshot().unwrap();
    assert_eq!(s.history.undo_len(), 1);
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(s.snapshot().unwrap(), before);
    s.dispatch_graphic(GraphicEdit::Redo).unwrap();
    assert_eq!(s.snapshot().unwrap(), after);
    s.project().unwrap().validate().unwrap();
}
#[test]
fn divergent_edit_invalidates_redo_but_noop_does_not() {
    let (mut s, _) = fixture();
    s.dispatch_graphic(GraphicEdit::Translate(1.0, 0.0))
        .unwrap();
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    s.dispatch_graphic(GraphicEdit::Translate(0.0, 0.0))
        .unwrap();
    assert!(s.history.can_redo());
    s.dispatch_graphic(GraphicEdit::Rotate(90.0)).unwrap();
    assert!(!s.history.can_redo());
}
#[test]
fn rotations_and_world_axis_mirrors_keep_anchors_and_are_reversible() {
    let (mut s, ids) = fixture();
    let before = s.snapshot().unwrap();
    let anchor = transform(&s, ids[0]).translation();
    for angle in [90.0, 180.0, 270.0] {
        s.dispatch_graphic(GraphicEdit::Rotate(angle)).unwrap();
        assert_eq!(transform(&s, ids[0]).translation(), anchor);
        s.dispatch_graphic(GraphicEdit::Undo).unwrap();
        assert_eq!(s.snapshot().unwrap(), before);
    }
    s.dispatch_graphic(GraphicEdit::Rotate(30.0)).unwrap();
    let t = transform(&s, ids[0]);
    let q = Point2D::new(3.0, 7.0).unwrap();
    let a = t.apply(q).unwrap();
    s.dispatch_graphic(GraphicEdit::MirrorHorizontal).unwrap();
    let b = transform(&s, ids[0]).apply(q).unwrap();
    assert!((a.x() + b.x() - 2.0 * anchor.x()).abs() < 1e-8);
    assert!((a.y() - b.y()).abs() < 1e-8);
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    s.dispatch_graphic(GraphicEdit::MirrorVertical).unwrap();
    let b = transform(&s, ids[0]).apply(q).unwrap();
    assert!((a.y() + b.y() - 2.0 * anchor.y()).abs() < 1e-8);
    assert!((a.x() - b.x()).abs() < 1e-8);
}
#[test]
fn invalid_numeric_edits_leave_revision_history_and_model_unchanged() {
    let (mut s, _) = fixture();
    let before = s.snapshot().unwrap();
    let revision = s.revision;
    for op in [
        GraphicEdit::Translate(f64::NAN, 0.0),
        GraphicEdit::Translate(0.0, f64::INFINITY),
        GraphicEdit::Rotate(f64::NAN),
    ] {
        assert!(s.dispatch_graphic(op).is_err());
    }
    assert_eq!(before, s.snapshot().unwrap());
    assert_eq!(revision, s.revision);
    assert_eq!(s.history.undo_len(), 0);
}
#[test]
fn existing_inspector_transform_path_is_recorded_and_noop_is_clean() {
    let (mut s, ids) = fixture();
    let t = transform(&s, ids[0]);
    let page = s.active.unwrap();
    s.apply(Edit::RepresentationTransform(ids[0], page, t))
        .unwrap();
    assert!(!s.dirty);
    let new = Transform2D::new(Point2D::new(-12.5, 7.25).unwrap(), 37.0, true, true).unwrap();
    s.apply(Edit::RepresentationTransform(ids[0], page, new))
        .unwrap();
    assert_eq!(transform(&s, ids[0]), new);
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(transform(&s, ids[0]), t);
}
#[test]
fn selection_and_camera_never_enter_history_or_change_electrical_data() {
    let (mut s, ids) = fixture();
    let before = s.snapshot().unwrap();
    s.select(Selection::Representation(ids[0]));
    s.tabs[0].camera.pan(eframe::egui::vec2(50.0, 30.0));
    assert_eq!(s.history.undo_len(), 0);
    assert_eq!(before, s.snapshot().unwrap());
    let camera = s.tabs[0].camera;
    s.dispatch_graphic(GraphicEdit::Translate(3.0, 4.0))
        .unwrap();
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(s.tabs[0].camera, camera);
    assert_eq!(s.selection, Selection::Representation(ids[0]));
}
#[test]
fn retained_history_is_bounded_and_import_resets_it() {
    let (mut s, _) = fixture();
    for _ in 0..120 {
        s.dispatch_graphic(GraphicEdit::Translate(1.0, 0.0))
            .unwrap();
    }
    assert_eq!(s.history.undo_len(), 100);
    let json = s.snapshot().unwrap();
    s.import_snapshot(&json).unwrap();
    assert_eq!(s.history.undo_len(), 0);
    assert_eq!(json, s.snapshot().unwrap());
    assert!(s.graphics.selection.ids().is_empty());
}
#[test]
fn page_identification_joins_history_while_unrelated_format_change_ends_it() {
    let (mut s, _) = fixture();
    let page = s.active.unwrap();
    s.dispatch_graphic(GraphicEdit::Translate(1.0, 0.0))
        .unwrap();
    let moved = s.snapshot().unwrap();
    s.apply(Edit::PageName(page, "Renomeada".into())).unwrap();
    assert_eq!(s.history.undo_len(), 2);
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(s.snapshot().unwrap(), moved);
    s.dispatch_graphic(GraphicEdit::Redo).unwrap();
    s.apply(Edit::PageFormat(
        page,
        PageKind::MultiLine,
        Size2D::new(300., 200.).unwrap(),
    ))
    .unwrap();
    assert!(!s.history.can_undo());
    assert!(!s.history.can_redo());
}

#[test]
fn failed_import_preserves_existing_history_and_successful_roundtrip_keeps_ids() {
    let (mut s, ids) = fixture();
    s.dispatch_graphic(GraphicEdit::Rotate(37.5)).unwrap();
    s.dispatch_graphic(GraphicEdit::MirrorHorizontal).unwrap();
    let before = s.snapshot().unwrap();
    assert!(s.import_snapshot("{}").is_err());
    assert_eq!(s.history.undo_len(), 2);
    let mut loaded = Session::default();
    loaded.import_snapshot(&before).unwrap();
    assert_eq!(loaded.snapshot().unwrap(), before);
    for id in ids {
        assert_eq!(transform(&s, id), transform(&loaded, id));
    }
    assert!(!loaded.history.can_undo());
}
#[test]
fn legacy_snapshot_without_document_additions_still_loads() {
    let (s, _) = fixture();
    let mut legacy: serde_json::Value = serde_json::from_str(&s.snapshot().unwrap()).unwrap();
    legacy.as_object_mut().unwrap().remove("representations");
    legacy.as_object_mut().unwrap().remove("identification");
    for page in legacy["pages"].as_array_mut().unwrap() {
        page.as_object_mut().unwrap().remove("order");
        page.as_object_mut().unwrap().remove("structure");
    }
    let mut loaded = Session::default();
    loaded.import_snapshot(&legacy.to_string()).unwrap();
    assert_eq!(loaded.project().unwrap().id(), s.project().unwrap().id());
    assert_eq!(loaded.project().unwrap().representations().count(), 0);
    loaded.project().unwrap().validate().unwrap();
}

#[test]
fn changing_graphic_selection_discards_pending_preview() {
    let (mut s, ids) = fixture();
    let before = s.snapshot().unwrap();
    let page = s.active.unwrap();
    let anchor = transform(&s, ids[0]).translation();
    let mut i = std::mem::take(&mut s.graphics);
    i.begin_move(s.project().unwrap(), page, ids[0], anchor)
        .unwrap();
    i.update_move(anchor.translated(12.0, 8.0).unwrap(), 4.0, true)
        .unwrap();
    s.graphics = i;
    s.select_graphics([ids[1]]);
    assert!(!s.graphics.is_active());
    assert_eq!(before, s.snapshot().unwrap());
    assert_eq!(s.history.undo_len(), 0);
}
