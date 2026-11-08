// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Fixture helpers use validated synthetic inputs and fail immediately on setup errors.
#![allow(clippy::unwrap_used)]
use cad_craft::domain::*;
use cad_gui::{
    demo,
    editing::Edit,
    navigation::{PageFilter, PageIndex},
    state::{Selection, Session},
};

fn session() -> Session {
    let mut s = Session::default();
    s.replace(demo::project().unwrap(), true).unwrap();
    s
}
#[test]
fn initial_session_has_no_project_selection_or_tabs() {
    let s = Session::default();
    assert!(s.project().is_none());
    assert!(s.tabs.is_empty());
    assert_eq!(s.selection, Selection::None);
    assert!(!s.dirty);
}
#[test]
fn demonstration_is_real_valid_core_with_varied_pages() {
    let p = demo::project().unwrap();
    p.validate().unwrap();
    assert!(p.documents().count() >= 2);
    assert!(p.pages().count() >= 5);
    assert!(p.representations().count() > 0);
    assert!(p.name().as_str().contains("Demonstração"));
}
#[test]
fn selecting_does_not_open_page() {
    let mut s = session();
    let id = s.project().unwrap().pages().next().unwrap().id();
    s.select(Selection::Page(id));
    assert_eq!(s.selected_page(), Some(id));
    assert!(s.active.is_none());
}
#[test]
fn opening_and_closing_tab_never_deletes_page() {
    let mut s = session();
    let id = s.project().unwrap().pages().next().unwrap().id();
    s.open_page(id).unwrap();
    s.open_page(id).unwrap();
    assert_eq!(s.tabs.len(), 1);
    s.close_tab(id);
    assert!(s.active.is_none());
    assert!(s.project().unwrap().page(id).is_some());
}
#[test]
fn same_names_still_have_distinct_tabs_and_cameras() {
    let mut s = session();
    let ids: Vec<_> = s.project().unwrap().pages().take(2).map(Page::id).collect();
    for id in &ids {
        s.apply(Edit::PageName(*id, "Mesmo nome".into())).unwrap();
        s.open_page(*id).unwrap();
    }
    s.tabs[0].camera.pan(eframe::egui::vec2(99.0, 55.0));
    let before = s.tabs[0].camera;
    s.open_page(ids[0]).unwrap();
    assert_eq!(s.tabs.len(), 2);
    assert_eq!(s.tabs[0].camera, before);
    assert_eq!(s.selection, Selection::Page(ids[0]));
}
#[test]
fn failed_edit_keeps_model_and_revision_unchanged() {
    let mut s = session();
    let id = s.project().unwrap().pages().next().unwrap().id();
    let old = s.snapshot().unwrap();
    let revision = s.revision;
    s.apply(Edit::PageName(id, "  ".into())).unwrap_err();
    assert_eq!(old, s.snapshot().unwrap());
    assert_eq!(revision, s.revision);
}
#[test]
fn page_edit_updates_properties_without_replacing_identity() {
    let mut s = session();
    let id = s.project().unwrap().pages().next().unwrap().id();
    s.apply(Edit::PageNumber(id, "A.42".into())).unwrap();
    assert_eq!(
        s.project().unwrap().page(id).unwrap().number().as_str(),
        "A.42"
    );
    assert!(s.dirty);
}
#[test]
fn deleting_represented_page_is_blocked() {
    let mut s = session();
    let id = s
        .project()
        .unwrap()
        .representations()
        .next()
        .unwrap()
        .page_id();
    s.open_page(id).unwrap();
    let before = s.snapshot().unwrap();
    assert!(s.delete_page(id).is_err());
    assert_eq!(before, s.snapshot().unwrap());
    assert_eq!(s.active, Some(id));
}
#[test]
fn deleting_empty_page_cleans_its_tabs_and_selection() {
    let mut s = session();
    let id = s
        .project()
        .unwrap()
        .pages()
        .find(|p| {
            s.project()
                .unwrap()
                .representations_on_page(p.id())
                .unwrap()
                .is_empty()
        })
        .unwrap()
        .id();
    s.open_page(id).unwrap();
    s.delete_page(id).unwrap();
    assert!(s.project().unwrap().page(id).is_none());
    assert!(!s.tabs.iter().any(|t| t.page == id));
    assert_ne!(s.selection, Selection::Page(id));
}
#[test]
fn ordering_and_filtering_use_core_document_order() {
    let mut s = session();
    let doc = s.project().unwrap().documents().next().unwrap().id();
    let mut order: Vec<_> = s
        .project()
        .unwrap()
        .ordered_pages(doc)
        .unwrap()
        .iter()
        .map(|p| p.id())
        .collect();
    order.reverse();
    s.reorder(doc, &order).unwrap();
    let index = PageIndex::build(s.project().unwrap());
    let filter = PageFilter {
        document: Some(doc),
        ..Default::default()
    };
    assert_eq!(
        index
            .filtered(&filter)
            .iter()
            .map(|r| r.id)
            .collect::<Vec<_>>(),
        order
    );
}
#[test]
fn combined_filters_do_not_modify_project() {
    let s = session();
    let p = s.project().unwrap();
    let before = s.snapshot().unwrap();
    let index = PageIndex::build(p);
    let row = &index.rows[0];
    let f = PageFilter {
        text: row.name.clone(),
        kind: Some(row.kind.clone()),
        document: Some(row.document),
        structure: row.functional.clone(),
    };
    let filtered = index.filtered(&f);
    assert!(!filtered.is_empty());
    assert!(filtered.iter().all(|r| r.name == row.name));
    assert_eq!(before, s.snapshot().unwrap());
}
#[test]
fn hierarchy_is_derived_from_both_core_paths() {
    let s = session();
    let index = PageIndex::build(s.project().unwrap());
    let row = index
        .rows
        .iter()
        .find(|r| !r.functional.is_empty() && !r.location.is_empty())
        .unwrap();
    assert!(row.path.iter().any(|p| p.starts_with('=')));
    assert!(row.path.iter().any(|p| p.starts_with('+')));
}
#[test]
fn snapshot_roundtrip_preserves_ids_but_resets_transient_state() {
    let s = session();
    let json = s.snapshot().unwrap();
    let mut target = Session::default();
    target.import_snapshot(&json).unwrap();
    assert_eq!(s.project().unwrap().id(), target.project().unwrap().id());
    assert_eq!(json, target.snapshot().unwrap());
    assert!(target.tabs.is_empty());
    assert!(!target.dirty);
}
#[test]
fn invalid_snapshot_never_replaces_session() {
    let mut s = session();
    let before = s.snapshot().unwrap();
    assert!(s.import_snapshot("{}").is_err());
    assert_eq!(before, s.snapshot().unwrap());
    assert!(s.import_snapshot(&" ".repeat(8 * 1024 * 1024 + 1)).is_err());
    assert_eq!(before, s.snapshot().unwrap());
}
#[test]
fn presentation_changes_do_not_change_electrical_snapshot() {
    let mut s = session();
    let before = s.snapshot().unwrap();
    let id = s.project().unwrap().pages().next().unwrap().id();
    s.open_page(id).unwrap();
    s.tabs[0].camera.pan(eframe::egui::vec2(90.0, -52.0));
    s.close_tab(id);
    assert_eq!(before, s.snapshot().unwrap());
}
#[test]
fn representation_edit_preserves_electrical_target() {
    let mut s = session();
    let r = s.project().unwrap().representations().next().unwrap();
    let id = r.id();
    let target = r.target();
    let page = r.page_id();
    s.apply(Edit::RepresentationTransform(
        id,
        page,
        Transform2D::new(Point2D::new(-10.0, 400.0).unwrap(), 90.0, true, false).unwrap(),
    ))
    .unwrap();
    let r = s.project().unwrap().representation(id).unwrap();
    assert_eq!(r.target(), target);
    assert_eq!(r.transform().translation().y(), 400.0);
    s.project().unwrap().validate().unwrap();
}
#[test]
fn editing_a_page_that_is_not_the_open_tab_keeps_it_in_the_inspector() {
    // Star-delta test: renaming page 2 while page 4 was open moved the
    // inspector to page 4, and the next edit renumbered the wrong page.
    let mut s = session();
    let ids: Vec<_> = s.project().unwrap().pages().take(2).map(Page::id).collect();
    s.open_page(ids[0]).unwrap();
    s.select(Selection::Page(ids[1]));
    s.apply(Edit::PageName(ids[1], "Diagrama unifilar".into()))
        .unwrap();
    assert_eq!(s.selection, Selection::Page(ids[1]));
    s.apply(Edit::PageNumber(ids[1], "2".into())).unwrap();
    let p = s.project().unwrap();
    assert_eq!(p.page(ids[1]).unwrap().number().as_str(), "2");
    assert_ne!(p.page(ids[0]).unwrap().number().as_str(), "2");
    assert_eq!(s.active, Some(ids[0]));
}
