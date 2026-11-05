// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_craft::domain::*;
use cad_gui::{
    editing::{Edit, GraphicEdit},
    state::{Selection, Session},
};

fn pt(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}
fn setup() -> (
    Session,
    DeviceId,
    FunctionId,
    RepresentationId,
    PageId,
    PageId,
    ConnectionPointId,
) {
    let mut p = cad_gui::demo::project().unwrap();
    let d = p.devices().next().unwrap().id();
    let f = p.functions().next().unwrap().id();
    let pages: Vec<_> = p.pages().map(Page::id).collect();
    let cp = p
        .insert_connection_point(ConnectionPoint::new(
            p.id(),
            f,
            DesignationPart::new("A1").unwrap(),
        ))
        .unwrap();
    let r = p
        .insert_representation(Representation::new(
            p.id(),
            pages[1],
            RepresentationTarget::Function(f),
            Transform2D::new(pt(25., 45.), 0., false, false).unwrap(),
        ))
        .unwrap();
    let mut s = Session::default();
    s.replace(p, false).unwrap();
    s.open_page(pages[0]).unwrap();
    (s, d, f, r, pages[0], pages[1], cp)
}

#[test]
fn device_edits_and_document_changes_share_history_and_refresh_references() {
    let (mut s, d, f, r, _, page, _) = setup();
    let before = s.snapshot().unwrap();
    s.dispatch_graphic(GraphicEdit::Device(DeviceEdit::Rename {
        device: d,
        designation: Designation::new("K9")
            .unwrap()
            .with_functional_path(vec![DesignationPart::new("BOMBA").unwrap()]),
    }))
    .unwrap();
    s.dispatch_graphic(GraphicEdit::Device(DeviceEdit::SetMainFunction {
        device: d,
        function: Some(f),
    }))
    .unwrap();
    s.apply(Edit::PageNumber(page, "7".into())).unwrap();
    assert_eq!(s.history.undo_len(), 3);
    let refs = s.project().unwrap().device_references(d).unwrap();
    let dest = refs.iter().find(|v| v.representation == r).unwrap();
    assert_eq!(dest.designation.to_string(), "=BOMBA-K9");
    assert_eq!(dest.role, FunctionRole::Main);
    assert_eq!(dest.page_number, "7");
    let after = s.snapshot().unwrap();
    for _ in 0..3 {
        s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    }
    assert_eq!(s.snapshot().unwrap(), before);
    for _ in 0..3 {
        s.dispatch_graphic(GraphicEdit::Redo).unwrap();
    }
    assert_eq!(s.snapshot().unwrap(), after);
}

#[test]
fn reference_navigation_centers_target_and_keeps_document_and_tabs() {
    let (mut s, _, _, r, first, second, _) = setup();
    let before = s.snapshot().unwrap();
    let rev = s.revision;
    s.navigate_reference(r).unwrap();
    assert_eq!(s.active, Some(second));
    assert_eq!(s.selection, Selection::Representation(r));
    assert!(s.tabs.iter().any(|t| t.page == first));
    assert_eq!(s.history.undo_len(), 0);
    let viewport = eframe::egui::Rect::from_min_max(
        eframe::egui::pos2(50., 50.),
        eframe::egui::pos2(800., 600.),
    );
    assert_eq!(
        s.active_tab_mut()
            .unwrap()
            .camera
            .world_to_screen(pt(25., 45.), viewport),
        Some(viewport.center())
    );
    assert_eq!(s.snapshot().unwrap(), before);
    assert_eq!(s.revision, rev);
    assert!(s.navigate_reference(RepresentationId::new()).is_err());
    assert_eq!(s.active, Some(second));
}

#[test]
fn marker_insertion_edit_replay_and_snapshot_keep_ids_and_explicit_binding() {
    let (mut s, _, _, _, _, _, cp) = setup();
    let before = s.snapshot().unwrap();
    let id = s
        .insert_electrical_marker(
            MarkerKind::Potential {
                kind: None,
                description: String::new(),
            },
            Text::new("+24VDC").unwrap(),
            Some(cp),
            pt(60., 80.),
        )
        .unwrap();
    assert_eq!(s.project().unwrap().marker(id).unwrap().point(), Some(cp));
    assert_eq!(
        s.project()
            .unwrap()
            .port_binding(&PortRef::new(id, "p").unwrap()),
        Some(cp)
    );
    assert_eq!(s.project().unwrap().connections().count(), 0);
    let inserted = s.snapshot().unwrap();
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(s.snapshot().unwrap(), before);
    s.dispatch_graphic(GraphicEdit::Redo).unwrap();
    assert_eq!(s.snapshot().unwrap(), inserted);
    s.dispatch_graphic(GraphicEdit::Marker(MarkerEdit::Update {
        id,
        name: Text::new("0VDC").unwrap(),
        kind: MarkerKind::Potential {
            kind: Some(Text::new("DC").unwrap()),
            description: "Retorno".into(),
        },
        point: Some(cp),
    }))
    .unwrap();
    let saved = s.snapshot().unwrap();
    let mut restored = Session::default();
    restored.import_snapshot(&saved).unwrap();
    assert_eq!(
        restored
            .project()
            .unwrap()
            .marker(id)
            .unwrap()
            .name()
            .as_str(),
        "0VDC"
    );
    assert_eq!(
        restored
            .project()
            .unwrap()
            .representation(id)
            .unwrap()
            .visual()
            .label
            .as_ref()
            .unwrap()
            .as_str(),
        "0VDC"
    );
    assert_eq!(
        restored.project().unwrap().conductors().count(),
        s.project().unwrap().conductors().count()
    );
}
