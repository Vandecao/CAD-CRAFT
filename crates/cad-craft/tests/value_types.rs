// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

use cad_craft::domain::*;
use std::collections::HashSet;

#[test]
fn identifiers_are_unique_and_round_trip_through_text_and_serde() {
    let ids: HashSet<_> = (0..2048).map(|_| ProjectId::new()).collect();
    assert_eq!(ids.len(), 2048);
    for id in ids {
        assert_eq!(id.as_uuid().get_version_num(), 7);
        assert_eq!(id.to_string().parse::<ProjectId>().unwrap(), id);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(serde_json::from_str::<ProjectId>(&json).unwrap(), id);
    }
}

#[test]
fn invalid_uuid_versions_variants_and_text_are_rejected() {
    for value in [
        "",
        "bad",
        "00000000-0000-0000-0000-000000000000",
        "550e8400-e29b-41d4-a716-446655440000",
        "01890abc-1234-7000-0000-000000000001",
    ] {
        assert!(value.parse::<ProjectId>().is_err());
        assert!(serde_json::from_str::<ProjectId>(&format!("\"{value}\"")).is_err());
    }
}

#[test]
fn all_entity_ids_preserve_explicit_valid_uuid_conversion() {
    let uuid = ProjectId::new().as_uuid();
    macro_rules! check { ($($ty:ty),+) => { $(
        assert_eq!(<$ty>::try_from(uuid).unwrap().as_uuid(), uuid);
        assert!(<$ty>::try_from(uuid::Uuid::nil()).is_err());
    )+ }; }
    check!(
        ProjectId,
        DocumentId,
        PageId,
        ComponentId,
        DeviceId,
        FunctionId,
        TerminalId,
        ConnectionPointId
    );
}

#[test]
fn names_and_structured_designations_validate_segments() {
    for invalid in ["", "  ", "line\nbreak", "bad\0text"] {
        assert!(Text::new(invalid).is_err());
    }
    assert_eq!(Text::new(" Painel A ").unwrap().as_str(), "Painel A");
    for invalid in ["", "-K1", "A B", "K1:1", "a/b", "=F", "+L"] {
        assert!(DesignationPart::new(invalid).is_err());
    }
    let designation = Designation::new("K1")
        .unwrap()
        .with_functional_path(vec![DesignationPart::new("F1").unwrap()])
        .with_location_path(vec![DesignationPart::new("P1").unwrap()])
        .with_terminal(DesignationPart::new("A1").unwrap());
    assert_eq!(designation.to_string(), "=F1+P1-K1:A1");
    assert_eq!(
        serde_json::from_str::<Designation>(&serde_json::to_string(&designation).unwrap()).unwrap(),
        designation
    );
    assert!(serde_json::from_str::<DesignationPart>("\"-K1\"").is_err());
}

#[test]
fn geometry_uses_cartesian_millimeters_and_inclusive_boundaries() {
    let p = Point2D::new(10.0, 20.0).unwrap();
    let q = p.translated(-3.0, 4.0).unwrap();
    assert_eq!((q.x(), q.y()), (7.0, 24.0));
    assert_eq!(p.distance_to(q).unwrap(), 5.0);
    let rect = Rect2D::new(p, Size2D::new(30.0, 40.0).unwrap()).unwrap();
    assert!(rect.contains(Point2D::new(40.0, 60.0).unwrap()));
    assert!(!rect.contains(Point2D::new(9.0, 21.0).unwrap()));
    let other = Rect2D::new(
        Point2D::new(20.0, 30.0).unwrap(),
        Size2D::new(40.0, 40.0).unwrap(),
    )
    .unwrap();
    let intersection = rect.intersection(other).unwrap();
    assert_eq!(intersection.origin(), Point2D::new(20.0, 30.0).unwrap());
    assert_eq!(intersection.size(), Size2D::new(20.0, 30.0).unwrap());
    assert!(rect.intersects(other));
    assert!(
        rect.intersection(
            Rect2D::new(
                Point2D::new(-5.0, -5.0).unwrap(),
                Size2D::new(0.0, 0.0).unwrap()
            )
            .unwrap()
        )
        .is_none()
    );
}

#[test]
fn invalid_geometry_and_arithmetic_overflow_return_errors() {
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(Point2D::new(invalid, 0.0).is_err());
        assert!(Size2D::new(1.0, invalid).is_err());
    }
    assert!(Size2D::new(-1.0, 1.0).is_err());
    assert!(Size2D::new(0.0, 0.0).is_ok());
    let max = Point2D::new(f64::MAX, 0.0).unwrap();
    assert!(max.translated(f64::MAX, 0.0).is_err());
    assert!(
        max.distance_to(Point2D::new(-f64::MAX, 0.0).unwrap())
            .is_err()
    );
    assert!(Rect2D::new(max, Size2D::new(f64::MAX, 0.0).unwrap()).is_err());
    assert!(serde_json::from_str::<Size2D>(r#"{"width":-1.0,"height":5.0}"#).is_err());
    assert!(serde_json::from_str::<Point2D>(r#"{"x":1e400,"y":0}"#).is_err());
}

#[test]
fn typed_properties_separate_known_units_and_custom_values() {
    let mut properties = TechnicalProperties::default();
    properties
        .set_known(KnownProperty::RatedVoltage, 230.0)
        .unwrap();
    assert_eq!(
        properties.known()[&KnownProperty::RatedVoltage].get(),
        230.0
    );
    let before = properties.clone();
    assert!(
        properties
            .set_known(KnownProperty::RatedVoltage, -1.0)
            .is_err()
    );
    assert_eq!(properties, before);
    let reference = EntityRef::Device(DeviceId::new());
    for (key, value) in [
        ("text", PropertyValue::Text("AC".into())),
        ("integer", PropertyValue::Integer(3)),
        (
            "decimal",
            PropertyValue::Decimal(Decimal::new(-2.5).unwrap()),
        ),
        ("boolean", PropertyValue::Boolean(true)),
        ("reference", PropertyValue::Reference(reference)),
        (
            "enum",
            PropertyValue::Enumeration(EnumValue::new("supply", ["AC", "DC"], "AC").unwrap()),
        ),
    ] {
        properties.set_custom(key, value).unwrap();
    }
    let json = serde_json::to_string(&properties).unwrap();
    assert_eq!(
        serde_json::from_str::<TechnicalProperties>(&json).unwrap(),
        properties
    );
    assert_eq!(properties.references().collect::<Vec<_>>(), vec![reference]);
    assert!(
        properties
            .set_custom(" ", PropertyValue::Boolean(false))
            .is_err()
    );
}

#[test]
fn property_value_validation_cannot_be_bypassed_with_serde() {
    assert!(Decimal::new(f64::NAN).is_err());
    assert!(EnumValue::new("supply", ["AC", "DC"], "invalid").is_err());
    assert!(EnumValue::new("supply", ["AC", "AC"], "AC").is_err());
    assert!(EnumValue::new("supply", [], "AC").is_err());
    assert!(
        serde_json::from_str::<TechnicalProperties>(
            r#"{"known":{"RatedVoltage":-1.0},"custom":{}}"#
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<EnumValue>(
            r#"{"name":"supply","choices":["AC","DC"],"selected":"bad"}"#
        )
        .is_err()
    );
}
