// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_craft::domain::*;
use serde_json::json;
fn pt(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}
fn definition(primitives: Vec<SymbolPrimitive>) -> SymbolDefinition {
    SymbolDefinition::new(
        SymbolDefinitionId::new("test.symbol").unwrap(),
        1,
        "Test",
        "Tests",
        primitives,
        vec![],
    )
    .unwrap()
}
#[test]
fn builtin_is_static_unique_valid_and_has_ten_industrial_examples() {
    let library = SymbolLibrary::builtin();
    assert!(std::ptr::eq(library, SymbolLibrary::builtin()));
    assert_eq!(library.definitions().count(), 10);
    let mut ids = std::collections::BTreeSet::new();
    for d in library.definitions() {
        d.validate().unwrap();
        assert!(ids.insert(d.id().clone()));
        assert_eq!(library.get(d.id()), Some(d));
        assert!(!d.ports().is_empty());
    }
}
#[test]
fn primitive_bounds_include_arc_extrema_and_text_metrics() {
    let d = definition(vec![SymbolPrimitive::Arc {
        center: pt(0., 0.),
        radius: 10.,
        start_degrees: 45.,
        sweep_degrees: 180.,
    }]);
    assert_eq!(d.bounds().origin().x(), -10.);
    assert_eq!(d.bounds().max().y(), 10.);
    let d = definition(vec![SymbolPrimitive::Text {
        origin: pt(-2., 3.),
        text: "AB".into(),
        height: 5.,
    }]);
    assert_eq!(d.bounds().origin(), pt(-2., 3.));
    assert_eq!(d.bounds().max(), pt(4., 8.));
    let arc = SymbolPrimitive::Arc {
        center: pt(0., 0.),
        radius: 2.,
        start_degrees: 0.,
        sweep_degrees: -180.,
    };
    assert_eq!(arc.bounds().unwrap().origin(), pt(-2., -2.));
}
#[test]
fn invalid_geometry_and_definition_contracts_are_rejected() {
    for primitive in [
        SymbolPrimitive::Circle {
            center: pt(0., 0.),
            radius: -1.,
        },
        SymbolPrimitive::Circle {
            center: pt(0., 0.),
            radius: f64::NAN,
        },
        SymbolPrimitive::Arc {
            center: pt(0., 0.),
            radius: 1.,
            start_degrees: 0.,
            sweep_degrees: 0.,
        },
        SymbolPrimitive::Polyline {
            points: vec![pt(0., 0.)],
            closed: false,
        },
        SymbolPrimitive::Text {
            origin: pt(0., 0.),
            text: "\n".into(),
            height: 1.,
        },
        SymbolPrimitive::Circle {
            center: pt(f64::MAX, 0.),
            radius: f64::MAX,
        },
    ] {
        assert!(primitive.bounds().is_err());
    }
    assert!(
        SymbolDefinition::new(
            SymbolDefinitionId::new("x").unwrap(),
            0,
            "X",
            "X",
            vec![],
            vec![]
        )
        .is_err()
    );
    assert!(SymbolDefinitionId::new(" ").is_err());
}
#[test]
fn ports_transform_position_direction_and_retain_identity() {
    let p = SymbolPort::new("a", pt(2., 3.), 0., "1", Some("input".into())).unwrap();
    let t = Transform2D::new(pt(10., 20.), 90., true, false).unwrap();
    let q = p.transformed(t).unwrap();
    assert_eq!(q.position(), pt(7., 18.));
    assert_eq!(q.orientation_degrees(), 270.);
    assert_eq!(q.id(), p.id());
    assert_eq!(q.identification(), "1");
}
#[test]
fn definition_snapshot_recomputes_bounds_and_rejects_duplicate_ports() {
    let mut v = json!(SymbolLibrary::builtin().definitions().next().unwrap());
    let d: SymbolDefinition = serde_json::from_value(v.clone()).unwrap();
    assert_eq!(json!(d), v);
    let duplicate = v["ports"][0].clone();
    v["ports"].as_array_mut().unwrap().push(duplicate);
    assert!(serde_json::from_value::<SymbolDefinition>(v).is_err());
}

#[test]
fn all_primitives_contribute_finite_negative_local_bounds() {
    let shapes = vec![
        SymbolPrimitive::Line {
            start: pt(-8., -6.),
            end: pt(-4., -2.),
        },
        SymbolPrimitive::Polyline {
            points: vec![pt(-3., -3.), pt(3., -3.), pt(0., 4.)],
            closed: true,
        },
        SymbolPrimitive::Rectangle {
            rect: Rect2D::new(pt(5., 2.), Size2D::new(3., 4.).unwrap()).unwrap(),
        },
        SymbolPrimitive::Circle {
            center: pt(0., 0.),
            radius: 2.,
        },
        SymbolPrimitive::Anchor {
            position: pt(12., -9.),
        },
    ];
    let d = definition(shapes);
    assert_eq!(d.bounds().origin(), pt(-8., -9.));
    assert_eq!(d.bounds().max(), pt(12., 6.));
    let t = Transform2D::new(pt(-20., 10.), 90., true, false).unwrap();
    let b = t.transform_bounds(d.bounds()).unwrap();
    assert_eq!(b.origin(), pt(-26., -2.));
    assert_eq!(b.max(), pt(-11., 18.));
}
#[test]
fn explicit_orientation_rules_normalize_and_reject_nonfinite_angles() {
    let rules = SymbolOrientations::new(vec![450., 0., 90.], false, true).unwrap();
    assert_eq!(rules.rotations(), &[0., 90.]);
    assert!(rules.allows(Transform2D::new(pt(0., 0.), 90., false, true).unwrap()));
    assert!(!rules.allows(Transform2D::new(pt(0., 0.), 90., true, false).unwrap()));
    assert!(!rules.allows(Transform2D::new(pt(0., 0.), 45., false, false).unwrap()));
    assert!(SymbolOrientations::new(vec![f64::NAN], true, true).is_err());
    for v in [
        json!({"id":"1","position":{"x":0.,"y":0.},"orientation_degrees":0.,"identification":"","role":null}),
        json!({"id":"","position":{"x":0.,"y":0.},"orientation_degrees":0.,"identification":"1","role":null}),
    ] {
        assert!(serde_json::from_value::<SymbolPort>(v).is_err());
    }
}

#[test]
fn tiny_negative_orientation_matches_transform_zero_and_roundtrips_canonically() {
    let transform = Transform2D::new(pt(0., 0.), -1e-15, false, false).unwrap();
    let rules = SymbolOrientations::new(vec![-1e-15], false, false).unwrap();
    assert!(
        rules.allows(transform),
        "rotation rules must use the same canonical zero as Transform2D"
    );
    assert_eq!(rules.rotations(), &[0.]);
    let bytes = serde_json::to_string(&rules).unwrap();
    let loaded: SymbolOrientations = serde_json::from_str(&bytes).unwrap();
    assert_eq!(loaded, rules);
    assert_eq!(serde_json::to_string(&loaded).unwrap(), bytes);
}
#[test]
fn tiny_negative_port_orientation_is_zero_before_and_after_snapshot() {
    let port = SymbolPort::new("p", pt(0., 0.), -1e-15, "1", None).unwrap();
    assert_eq!(port.orientation_degrees(), 0.);
    let bytes = serde_json::to_string(&port).unwrap();
    let loaded: SymbolPort = serde_json::from_str(&bytes).unwrap();
    assert_eq!(loaded, port);
    assert_eq!(serde_json::to_string(&loaded).unwrap(), bytes);
}
