// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Millimetre comparison helpers; start with EPS_MM and nearly_equal.

/// Absolute millimetre tolerance shared by geometric boundary checks.
pub const EPS_MM: f64 = 1e-6;

/// Compare finite coordinates with an inclusive absolute millimetre tolerance.
pub fn nearly_equal(a: f64, b: f64) -> bool {
    is_zero(a - b)
}

/// Test a finite millimetre offset against the inclusive zero tolerance.
pub fn is_zero(value: f64) -> bool {
    value.abs() <= EPS_MM
}
