// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Fixture helpers use validated synthetic inputs and fail immediately on setup errors.
#![allow(clippy::unwrap_used)]
use cad_craft::domain::*;
use cad_gui::{
    editing::{Edit, GraphicEdit},
    state::Session,
};

fn session() -> (Session, RepresentationId) {
    let mut s = Session::default();
    s.replace(cad_gui::demo::project().unwrap(), true).unwrap();
    let r = s.project().unwrap().representations().next().unwrap();
    let (id, page) = (r.id(), r.page_id());
    s.open_page(page).unwrap();
    s.select_graphics([id]);
    (s, id)
}

#[test]
fn visual_properties_are_reversible_and_interleave_with_transforms() {
    let (mut s, id) = session();
    let before = s.snapshot().unwrap();
    s.dispatch_graphic(GraphicEdit::Translate(8.0, -4.0))
        .unwrap();
    let moved = s.snapshot().unwrap();
    let visual = VisualProperties {
        visible: false,
        layer: 3,
        label: Some(Text::new("K1").unwrap()),
    };
    s.apply(Edit::RepresentationVisual(id, visual)).unwrap();
    let after = s.snapshot().unwrap();
    assert_eq!(s.history.undo_len(), 2);
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(s.snapshot().unwrap(), moved);
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(s.snapshot().unwrap(), before);
    s.dispatch_graphic(GraphicEdit::Redo).unwrap();
    s.dispatch_graphic(GraphicEdit::Redo).unwrap();
    assert_eq!(s.snapshot().unwrap(), after);
    s.project().unwrap().validate().unwrap();
}

#[test]
fn unchanged_visual_properties_preserve_redo_and_revision() {
    let (mut s, id) = session();
    s.dispatch_graphic(GraphicEdit::Translate(4.0, 0.0))
        .unwrap();
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    let revision = s.revision;
    let visual = s
        .project()
        .unwrap()
        .representation(id)
        .unwrap()
        .visual()
        .clone();
    s.apply(Edit::RepresentationVisual(id, visual)).unwrap();
    assert!(s.history.can_redo());
    assert_eq!(s.revision, revision);
}

fn definition() -> SymbolDefinitionId {
    SymbolLibrary::builtin()
        .definitions()
        .next()
        .unwrap()
        .id()
        .clone()
}

#[test]
fn insertion_preview_cancel_and_repetition_keep_ids_and_electrical_data() {
    let (mut s, _) = session();
    let before = s.snapshot().unwrap();
    let t = Transform2D::new(Point2D::new(0.0, 0.0).unwrap(), 90.0, true, false).unwrap();
    s.begin_symbol_insertion(definition(), None, t).unwrap();
    for x in 0..40 {
        s.graphics
            .update_insertion(Point2D::new(x as f64 + 0.4, -5.3).unwrap(), 4.0, true)
            .unwrap();
    }
    assert_eq!(before, s.snapshot().unwrap());
    assert_eq!(s.history.undo_len(), 0);
    s.graphics.cancel();
    assert!(s.confirm_symbol_insertion().is_err());
    assert_eq!(before, s.snapshot().unwrap());
    s.begin_symbol_insertion(definition(), None, t).unwrap();
    s.graphics
        .update_insertion(Point2D::new(-5.2, 6.1).unwrap(), 4.0, true)
        .unwrap();
    let a = s.confirm_symbol_insertion().unwrap();
    let b = s.confirm_symbol_insertion().unwrap();
    assert_ne!(a, b);
    assert!(s.graphics.insertion.is_some());
    let p = s.project().unwrap();
    let ra = p.representation(a).unwrap();
    assert_eq!(ra.target(), RepresentationTarget::Unassociated);
    assert_eq!(
        ra.transform().translation(),
        Point2D::new(-4.0, 8.0).unwrap()
    );
    assert_eq!(ra.transform().rotation_degrees(), 90.0);
    assert!(ra.transform().mirror_x());
    let after = s.snapshot().unwrap();
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(s.snapshot().unwrap(), before);
    assert!(s.graphics.selection.ids().is_empty());
    s.dispatch_graphic(GraphicEdit::Redo).unwrap();
    s.dispatch_graphic(GraphicEdit::Redo).unwrap();
    assert_eq!(s.snapshot().unwrap(), after);
    s.project().unwrap().validate().unwrap();
}

#[test]
fn associated_insertion_roundtrips_after_transform_and_style_history() {
    let (mut s, original) = session();
    let target = s
        .project()
        .unwrap()
        .representation(original)
        .unwrap()
        .target();
    let before = s.snapshot().unwrap();
    let t = Transform2D::new(Point2D::new(0.0, 0.0).unwrap(), 0.0, false, false).unwrap();
    s.begin_symbol_insertion(definition(), Some(target), t)
        .unwrap();
    s.graphics
        .update_insertion(Point2D::new(22.3, -13.2).unwrap(), 4.0, false)
        .unwrap();
    let id = s.confirm_symbol_insertion().unwrap();
    s.graphics.cancel();
    s.dispatch_graphic(GraphicEdit::Rotate(270.0)).unwrap();
    s.dispatch_graphic(GraphicEdit::MirrorVertical).unwrap();
    let mut v = s
        .project()
        .unwrap()
        .representation(id)
        .unwrap()
        .visual()
        .clone();
    v.label = Some(Text::new("Contato associado").unwrap());
    s.apply(Edit::RepresentationVisual(id, v)).unwrap();
    let after = s.snapshot().unwrap();
    for _ in 0..4 {
        s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    }
    assert_eq!(before, s.snapshot().unwrap());
    for _ in 0..4 {
        s.dispatch_graphic(GraphicEdit::Redo).unwrap();
    }
    assert_eq!(after, s.snapshot().unwrap());
    assert_eq!(
        s.project().unwrap().representation(id).unwrap().target(),
        target
    );
    let mut loaded = Session::default();
    loaded.import_snapshot(&after).unwrap();
    assert_eq!(after, loaded.snapshot().unwrap());
    assert!(!loaded.history.can_undo());
}

#[test]
fn page_switch_and_invalid_target_cannot_insert_or_dirty_session() {
    let (mut s, _) = session();
    let t = Transform2D::new(Point2D::new(0.0, 0.0).unwrap(), 0.0, false, false).unwrap();
    let before = s.snapshot().unwrap();
    s.begin_symbol_insertion(
        definition(),
        Some(RepresentationTarget::Device(DeviceId::new())),
        t,
    )
    .unwrap();
    s.graphics
        .update_insertion(Point2D::new(1.0, 2.0).unwrap(), 4.0, true)
        .unwrap();
    assert!(s.confirm_symbol_insertion().is_err());
    assert_eq!(before, s.snapshot().unwrap());
    assert!(!s.dirty);
    let page = s.active.unwrap();
    s.open_page(page).unwrap();
    assert!(s.graphics.insertion.is_none());
    assert!(s.confirm_symbol_insertion().is_err());
    assert_eq!(s.history.undo_len(), 0);
}
