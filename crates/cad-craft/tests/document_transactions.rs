// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
mod document_common;
mod physical_common;
use cad_craft::domain::*;
use document_common::*;

#[test]
fn batch_changes_only_transforms_and_preserves_indices_and_electrical_data() {
    let (mut p, page, _, device, connection) = fixture();
    let a = p
        .insert_representation(rep(&p, page, RepresentationTarget::Device(device)))
        .unwrap();
    let b = p
        .insert_representation(rep(&p, page, RepresentationTarget::Connection(connection)))
        .unwrap();
    let before = serde_json::to_value(&p).unwrap();
    p.transform_representations(&[(a, transform(-4.0, 32.0)), (b, transform(8.0, 44.0))])
        .unwrap();
    assert_eq!(
        p.representation(a).unwrap().transform().translation().x(),
        -4.0
    );
    assert_eq!(
        p.representation(b).unwrap().transform().translation().y(),
        44.0
    );
    let mut after = serde_json::to_value(&p).unwrap();
    after["representations"] = before["representations"].clone();
    assert_eq!(before, after);
    assert_eq!(p.representations_on_page(page).unwrap().len(), 2);
    p.validate().unwrap();
}

#[test]
fn invalid_last_id_or_duplicate_leaves_entire_batch_unchanged() {
    let (mut p, page, _, device, _) = fixture();
    let a = p
        .insert_representation(rep(&p, page, RepresentationTarget::Device(device)))
        .unwrap();
    let before = serde_json::to_string(&p).unwrap();
    assert!(
        p.transform_representations(&[
            (a, transform(1.0, 2.0)),
            (RepresentationId::new(), transform(3.0, 4.0))
        ])
        .is_err()
    );
    assert_eq!(before, serde_json::to_string(&p).unwrap());
    assert!(
        p.transform_representations(&[(a, transform(1.0, 2.0)), (a, transform(3.0, 4.0))])
            .is_err()
    );
    assert_eq!(before, serde_json::to_string(&p).unwrap());
}

#[test]
fn overflow_in_last_transformed_bounds_rolls_back_everything() {
    let (mut p, page, _, device, _) = fixture();
    let a = p
        .insert_representation(rep(&p, page, RepresentationTarget::Device(device)))
        .unwrap();
    let b = p
        .insert_representation(
            rep(&p, page, RepresentationTarget::Device(device))
                .with_local_bounds(
                    Rect2D::new(
                        Point2D::new(0.0, 0.0).unwrap(),
                        Size2D::new(f64::MAX, 1.0).unwrap(),
                    )
                    .unwrap(),
                )
                .unwrap(),
        )
        .unwrap();
    let before = serde_json::to_string(&p).unwrap();
    assert!(
        p.transform_representations(&[(a, transform(1.0, 2.0)), (b, transform(f64::MAX, 0.0))])
            .is_err()
    );
    assert_eq!(before, serde_json::to_string(&p).unwrap());
    p.validate().unwrap();
}

#[test]
fn graphical_batch_preserves_real_wiring_cables_terminals_and_derived_networks() {
    let mut p = physical_common::populated();
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
    let target = RepresentationTarget::Conductor(p.conductors().next().unwrap().id());
    let id = p.insert_representation(rep(&p, page, target)).unwrap();
    let before = serde_json::to_value(&p).unwrap();
    let networks = p.networks();
    p.transform_representations(&[(
        id,
        Transform2D::new(Point2D::new(-20.0, 340.0).unwrap(), 90.0, true, false).unwrap(),
    )])
    .unwrap();
    let mut after = serde_json::to_value(&p).unwrap();
    after["representations"] = before["representations"].clone();
    assert_eq!(before, after);
    assert_eq!(p.networks(), networks);
    p.validate().unwrap();
}
