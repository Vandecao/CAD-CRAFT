// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Pin millimetre tolerance boundaries; start with the boundary test.
use cad_craft::domain::tolerance::{EPS_MM, is_zero, nearly_equal};

#[test]
fn millimetre_tolerance_is_inclusive_and_rejects_nonfinite_values() {
    assert_eq!(EPS_MM, 1e-6);
    for value in [0.0, -0.0, EPS_MM, -EPS_MM] {
        assert!(is_zero(value));
        assert!(nearly_equal(value, 0.0));
    }
    let outside = f64::from_bits(EPS_MM.to_bits() + 1);
    assert!(!is_zero(outside));
    assert!(!is_zero(-outside));
    assert!(nearly_equal(10.0, 10.0 + EPS_MM / 2.0));
    assert!(!nearly_equal(10.0, 10.0 + EPS_MM * 2.0));
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(!is_zero(value));
        assert!(!nearly_equal(value, value));
        assert!(!nearly_equal(value, 0.0));
    }
}

#[test]
fn frame_zone_boundaries_keep_the_existing_millimetre_tolerance() {
    use cad_craft::domain::{ZoneAxis, ZoneLabelStyle};
    let axis = ZoneAxis::uniform(2, 10.0, ZoneLabelStyle::Numbers { start: 1 });
    let outside = f64::from_bits(EPS_MM.to_bits() + 1);
    assert_eq!(axis.index_at(-EPS_MM), Some(0));
    assert_eq!(axis.index_at(-outside), None);
    assert_eq!(axis.index_at(10.0 - EPS_MM * 2.0), Some(0));
    assert_eq!(axis.index_at(10.0 - EPS_MM), Some(1));
    assert_eq!(axis.index_at(20.0 + EPS_MM), Some(1));
    assert_eq!(axis.index_at(20.0 + EPS_MM * 2.0), None);
    assert_eq!(axis.index_at(f64::NAN), None);
}
