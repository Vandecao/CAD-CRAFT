// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_craft::domain::*;

#[test]
fn oblique_rotation_bounds_and_inverse_are_cartesian() {
    let t = Transform2D::new(Point2D::new(0.0, 0.0).unwrap(), 45.0, true, false).unwrap();
    close(
        t.apply(Point2D::new(2.0, 0.0).unwrap()).unwrap(),
        -std::f64::consts::SQRT_2,
        -std::f64::consts::SQRT_2,
    );
    let bounds = t
        .transform_bounds(
            Rect2D::new(
                Point2D::new(0.0, 0.0).unwrap(),
                Size2D::new(2.0, 2.0).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    close(
        bounds.origin(),
        -2.0 * std::f64::consts::SQRT_2,
        -std::f64::consts::SQRT_2,
    );
    close(bounds.max(), 0.0, std::f64::consts::SQRT_2);
    close(
        t.inverse_apply(
            Point2D::new(-std::f64::consts::SQRT_2, -std::f64::consts::SQRT_2).unwrap(),
        )
        .unwrap(),
        2.0,
        0.0,
    );
}

fn close(p: Point2D, x: f64, y: f64) {
    assert!((p.x() - x).abs() < 1e-10, "{p:?}");
    assert!((p.y() - y).abs() < 1e-10, "{p:?}");
}

#[test]
fn mirror_then_ccw_rotation_then_translation_and_inverse() {
    let p = Point2D::new(2.0, 3.0).unwrap();
    for (mx, my, x, y) in [
        (false, false, 7.0, 22.0),
        (true, false, 7.0, 18.0),
        (false, true, 13.0, 22.0),
        (true, true, 13.0, 18.0),
    ] {
        let t = Transform2D::new(Point2D::new(10.0, 20.0).unwrap(), 450.0, mx, my).unwrap();
        assert_eq!(t.rotation_degrees(), 90.0);
        close(t.apply(p).unwrap(), x, y);
        close(
            t.inverse_apply(Point2D::new(x, y).unwrap()).unwrap(),
            2.0,
            3.0,
        );
    }
}

#[test]
fn negative_rotation_and_bounds_use_all_corners() {
    let t = Transform2D::new(Point2D::new(-10.0, 5.0).unwrap(), -90.0, false, false).unwrap();
    let r = Rect2D::new(
        Point2D::new(1.0, 2.0).unwrap(),
        Size2D::new(3.0, 4.0).unwrap(),
    )
    .unwrap();
    let bounds = t.transform_bounds(r).unwrap();
    close(bounds.origin(), -8.0, 1.0);
    close(bounds.max(), -4.0, 4.0);
}

#[test]
fn non_finite_and_overflow_are_errors() {
    for angle in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(Transform2D::new(Point2D::new(0.0, 0.0).unwrap(), angle, false, false).is_err());
    }
    let t = Transform2D::new(Point2D::new(f64::MAX, 0.0).unwrap(), 0.0, false, false).unwrap();
    assert!(t.apply(Point2D::new(f64::MAX, 0.0).unwrap()).is_err());
    assert!(
        t.inverse_apply(Point2D::new(-f64::MAX, 0.0).unwrap())
            .is_err()
    );
    let bad = serde_json::json!({"translation":{"x":0,"y":0},"rotation_degrees":null,"mirror_x":false,"mirror_y":false});
    assert!(serde_json::from_value::<Transform2D>(bad).is_err());
}

#[test]
fn orientation_is_derived_without_changing_sheet_dimensions() {
    let p = Project::new("P").unwrap();
    for (w, h, want) in [
        (210.0, 297.0, PageOrientation::Portrait),
        (297.0, 210.0, PageOrientation::Landscape),
        (0.0, 0.0, PageOrientation::Square),
    ] {
        let page = Page::new(
            p.id(),
            DocumentId::new(),
            "P",
            "1",
            PageKind::MultiLine,
            Size2D::new(w, h).unwrap(),
        )
        .unwrap();
        assert_eq!(page.orientation(), want);
        assert_eq!(page.size().width(), w);
    }
}
