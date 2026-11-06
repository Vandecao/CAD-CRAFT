// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Fixture helpers use validated synthetic inputs and fail immediately on setup errors.
#![allow(clippy::unwrap_used)]
use cad_craft::domain::*;
use cad_gui::{
    camera::Camera,
    hit_test::{hit_candidates, rectangle_selection},
    interaction::Interaction,
};
use eframe::egui::{Pos2, Rect, vec2};
fn fixture() -> (Project, PageId, RepresentationId, RepresentationId) {
    let mut p = Project::new("Editor").unwrap();
    let doc = p
        .insert_document(Document::new(p.id(), "D").unwrap())
        .unwrap();
    let page = p
        .insert_page(
            Page::new(
                p.id(),
                doc,
                "P",
                "1",
                PageKind::Overview,
                Size2D::new(297.0, 210.0).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let device = p
        .insert_device(Device::new(p.id(), Designation::new("K1").unwrap(), "Contator").unwrap())
        .unwrap();
    let mut insert = |x, y, angle| {
        let r = Representation::new(
            p.id(),
            page,
            RepresentationTarget::Device(device),
            Transform2D::new(pt(x, y), angle, false, false).unwrap(),
        )
        .with_local_bounds(Rect2D::new(pt(0.0, 0.0), Size2D::new(10.0, 4.0).unwrap()).unwrap())
        .unwrap();
        p.insert_representation(r).unwrap()
    };
    let a = insert(1.0, 2.0, 0.0);
    let b = insert(22.5, 6.25, 45.0);
    (p, page, a, b)
}
fn pt(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}
#[test]
fn clicks_toggle_ids_deterministically_and_empty_clears() {
    let (_, _, a, b) = fixture();
    let mut i = Interaction::default();
    i.click(Some(a), false);
    assert_eq!(i.selection.ids().len(), 1);
    i.click(Some(b), true);
    assert_eq!(i.selection.ids().len(), 2);
    i.click(Some(a), true);
    assert_eq!(
        i.selection.ids().iter().copied().collect::<Vec<_>>(),
        vec![b]
    );
    i.click(None, false);
    assert!(i.selection.ids().is_empty());
}
#[test]
fn hover_does_not_select_or_edit() {
    let (p, _, a, _) = fixture();
    let before = serde_json::to_string(&p).unwrap();
    let i = Interaction {
        hover: Some(a),
        ..Default::default()
    };
    assert!(i.selection.ids().is_empty());
    assert_eq!(before, serde_json::to_string(&p).unwrap());
}
#[test]
fn hit_tolerance_is_six_logical_points_at_any_zoom_and_pan() {
    let (p, page, a, _) = fixture();
    let view = Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0));
    for scale in [0.5, 1.0, 10.0, 100.0] {
        let c = Camera::new([1.0, 2.0], scale).unwrap();
        let edge = c.world_to_screen(pt(1.0, 4.0), view).unwrap();
        assert_eq!(
            hit_candidates(&p, page, c, view, edge - vec2(5.0, 0.0)).first(),
            Some(&a)
        );
        assert!(!hit_candidates(&p, page, c, view, edge - vec2(7.0, 0.0)).contains(&a));
    }
}
#[test]
fn transformed_hit_testing_rejects_empty_aabb_corner_and_hidden_objects() {
    let (mut p, page, _, b) = fixture();
    let view = Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0));
    let c = Camera::new([25.0, 10.0], 100.0).unwrap();
    let screen = c.world_to_screen(pt(29.0, 6.5), view).unwrap();
    assert!(!hit_candidates(&p, page, c, view, screen).contains(&b));
    let t = Transform2D::new(pt(22.5, 6.25), 90.0, true, false).unwrap();
    p.transform_representations(&[(b, t)]).unwrap();
    let hit = c
        .world_to_screen(t.apply(pt(2.0, 2.0)).unwrap(), view)
        .unwrap();
    assert!(hit_candidates(&p, page, c, view, hit).contains(&b));
    p.set_representation_visual(
        b,
        VisualProperties {
            visible: false,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!hit_candidates(&p, page, c, view, hit).contains(&b));
}
#[test]
fn overlapping_objects_follow_layer_then_id_and_candidates_prepare_cycling() {
    let (mut p, page, a, b) = fixture();
    let t = p.representation(a).unwrap().transform();
    p.transform_representations(&[(b, t)]).unwrap();
    let view = Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0));
    let c = Camera::new([5.0, 4.0], 10.0).unwrap();
    assert_eq!(hit_candidates(&p, page, c, view, view.center()), vec![b, a]);
    p.set_representation_visual(
        a,
        VisualProperties {
            layer: 9,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(hit_candidates(&p, page, c, view, view.center()), vec![a, b]);
}
#[test]
fn rectangle_window_requires_containment_and_crossing_uses_actual_edges() {
    let (p, page, a, b) = fixture();
    assert_eq!(
        rectangle_selection(&p, page, pt(0.0, 0.0), pt(12.0, 7.0), false),
        vec![a]
    );
    assert!(rectangle_selection(&p, page, pt(0.0, 0.0), pt(3.0, 3.0), false).is_empty());
    assert_eq!(
        rectangle_selection(&p, page, pt(0.0, 0.0), pt(3.0, 3.0), true),
        vec![a]
    );
    assert!(!rectangle_selection(&p, page, pt(28.5, 6.25), pt(29.5, 6.75), true).contains(&b));
    assert!(rectangle_selection(&p, page, pt(23.0, 8.0), pt(24.0, 9.0), true).contains(&b));
}
#[test]
fn snapped_group_preview_uses_grabbed_anchor_and_preserves_relative_positions() {
    let (p, page, a, b) = fixture();
    let before = serde_json::to_string(&p).unwrap();
    let mut i = Interaction::default();
    i.click(Some(a), false);
    i.click(Some(b), true);
    i.begin_move(&p, page, a, pt(2.0, 3.0)).unwrap();
    i.update_move(pt(7.0, 6.0), 4.0, true).unwrap();
    let ta = i.preview(a).unwrap();
    let tb = i.preview(b).unwrap();
    assert_eq!(ta.translation(), pt(8.0, 4.0));
    assert_eq!(tb.translation(), pt(29.5, 8.25));
    assert_eq!(tb.rotation_degrees(), 45.0);
    assert_eq!(before, serde_json::to_string(&p).unwrap());
    assert_eq!(i.finish_move().unwrap().len(), 2);
    assert!(!i.is_active());
}
#[test]
fn zero_drag_and_escape_leave_no_preview_and_no_document_change() {
    let (p, page, a, _) = fixture();
    let mut i = Interaction::default();
    i.click(Some(a), false);
    i.begin_move(&p, page, a, pt(2.0, 3.0)).unwrap();
    i.update_move(pt(2.0, 3.0), 4.0, true).unwrap();
    assert_eq!(i.preview(a).unwrap().translation(), pt(1.0, 2.0));
    i.cancel();
    assert!(!i.is_active());
    assert!(i.finish_move().is_none());
    assert!(i.selection.ids().contains(&a));
}
#[test]
fn disabled_snap_and_negative_fractional_coordinates_are_documental() {
    let (p, page, a, _) = fixture();
    let mut i = Interaction::default();
    i.click(Some(a), false);
    i.begin_move(&p, page, a, pt(1.0, 2.0)).unwrap();
    i.update_move(pt(-3.3, -7.2), 4.0, false).unwrap();
    let q = i.preview(a).unwrap().translation();
    assert!((q.x() + 3.3).abs() < 1e-12 && (q.y() + 7.2).abs() < 1e-12);
    i.update_move(pt(-3.3, -7.2), 2.5, true).unwrap();
    assert_eq!(i.preview(a).unwrap().translation(), pt(-2.5, -7.5));
}
