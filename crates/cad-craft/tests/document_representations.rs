// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

mod document_common;
use cad_craft::domain::*;
use document_common::*;

#[test]
fn multiple_occurrences_navigate_without_duplicating_electrical_entities() {
    let (mut p, a, b, d, _) = fixture();
    let before = p.networks();
    let target = RepresentationTarget::Device(d);
    let first = p.insert_representation(rep(&p, a, target)).unwrap();
    let second = p.insert_representation(rep(&p, a, target)).unwrap();
    let third = p.insert_representation(rep(&p, b, target)).unwrap();
    assert_eq!(p.representations_for_entity(target).unwrap().len(), 3);
    assert_eq!(p.representations_on_page(a).unwrap().len(), 2);
    assert_eq!(
        p.related_representations(first)
            .unwrap()
            .iter()
            .map(|r| r.id())
            .collect::<Vec<_>>(),
        vec![second, third]
    );
    assert_eq!(
        p.representation_location(third).unwrap(),
        (b, Point2D::new(10.0, 20.0).unwrap())
    );
    assert_eq!(p.devices().count(), 1);
    assert_eq!(p.networks(), before);
    p.validate().unwrap();
}

#[test]
fn missing_foreign_and_duplicate_references_are_atomic() {
    let (mut p, a, _, d, _) = fixture();
    let target = RepresentationTarget::Device(d);
    let r = rep(&p, a, target);
    p.insert_representation(r.clone()).unwrap();
    let before = p.clone();
    for invalid in [
        r,
        rep(&p, PageId::new(), target),
        rep(&p, a, RepresentationTarget::Device(DeviceId::new())),
        Representation::new(ProjectId::new(), a, target, transform(0.0, 0.0)),
        rep(&p, a, RepresentationTarget::Connection(ConnectionId::new())),
    ] {
        assert!(p.insert_representation(invalid).is_err());
        assert_eq!(p, before);
    }
    assert!(p.representations_on_page(PageId::new()).is_err());
    assert!(
        p.representations_for_entity(RepresentationTarget::Device(DeviceId::new()))
            .is_err()
    );
    assert!(p.related_representations(RepresentationId::new()).is_err());
}

#[test]
fn move_update_style_and_remove_keep_indices_consistent() {
    let (mut p, a, b, d, _) = fixture();
    let target = RepresentationTarget::Device(d);
    let id = p.insert_representation(rep(&p, a, target)).unwrap();
    p.move_representation(id, b, transform(-200.0, 700.0))
        .unwrap();
    assert!(p.representations_on_page(a).unwrap().is_empty());
    assert_eq!(
        p.representation_location(id).unwrap(),
        (b, Point2D::new(-200.0, 700.0).unwrap())
    );
    let style = VisualProperties {
        visible: false,
        layer: 4,
        label: Some(Text::new("K1 auxiliar").unwrap()),
    };
    p.set_representation_visual(id, style.clone()).unwrap();
    assert_eq!(p.representation(id).unwrap().visual(), &style);
    let before = p.clone();
    assert!(
        p.move_representation(id, PageId::new(), transform(0.0, 0.0))
            .is_err()
    );
    assert_eq!(p, before);
    p.validate().unwrap();
    p.remove_representation(id).unwrap();
    assert!(p.representations_for_entity(target).unwrap().is_empty());
    assert!(p.remove_representation(id).is_err());
    p.validate().unwrap();
}

#[test]
fn removal_is_explicit_and_preserves_electrical_graph() {
    let (mut p, a, _, d, c) = fixture();
    let electrical = p.networks();
    let id = p
        .insert_representation(rep(&p, a, RepresentationTarget::Connection(c)))
        .unwrap();
    let before = p.clone();
    assert!(p.remove_page(a).is_err());
    assert_eq!(p, before);
    assert!(p.remove_connection(c).is_err());
    assert_eq!(p, before);
    p.remove_representation(id).unwrap();
    p.remove_page(a).unwrap();
    assert!(p.device(d).is_some());
    assert_eq!(p.networks(), electrical);
    p.remove_connection(c).unwrap();
    p.validate().unwrap();
}

#[test]
fn content_bounds_are_not_clipped_to_sheet_and_unknown_size_uses_anchor() {
    let (mut p, a, _, d, _) = fixture();
    assert!(p.document_bounds(a).unwrap().content.is_none());
    let id = p
        .insert_representation(rep(&p, a, RepresentationTarget::Device(d)))
        .unwrap();
    p.move_representation(
        id,
        a,
        Transform2D::new(Point2D::new(-10.0, 400.0).unwrap(), 90.0, true, false).unwrap(),
    )
    .unwrap();
    let local = Rect2D::new(
        Point2D::new(0.0, 0.0).unwrap(),
        Size2D::new(2.0, 4.0).unwrap(),
    )
    .unwrap();
    p.set_representation_bounds(id, Some(local)).unwrap();
    let bounds = p.document_bounds(a).unwrap();
    assert_eq!(bounds.sheet.max(), Point2D::new(297.0, 210.0).unwrap());
    assert_eq!(
        bounds.content.unwrap().origin(),
        Point2D::new(-14.0, 398.0).unwrap()
    );
    assert_eq!(
        bounds.content.unwrap().max(),
        Point2D::new(-10.0, 400.0).unwrap()
    );
    p.set_representation_bounds(id, None).unwrap();
    assert_eq!(
        p.document_bounds(a).unwrap().content.unwrap().size(),
        Size2D::new(0.0, 0.0).unwrap()
    );
}

#[test]
fn invalid_transformed_bounds_are_rejected_before_mutation() {
    let (mut p, a, _, d, _) = fixture();
    let id = p
        .insert_representation(rep(&p, a, RepresentationTarget::Device(d)))
        .unwrap();
    let local = Rect2D::new(
        Point2D::new(f64::MAX, 0.0).unwrap(),
        Size2D::new(0.0, 0.0).unwrap(),
    )
    .unwrap();
    p.set_representation_bounds(id, Some(local)).unwrap();
    let before = p.clone();
    assert!(
        p.move_representation(id, a, transform(f64::MAX, 0.0))
            .is_err()
    );
    assert_eq!(p, before);
}
