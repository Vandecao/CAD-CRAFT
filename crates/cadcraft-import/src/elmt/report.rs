// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Records the converted symbol summary at its original point in conversion; start at `summarize`.
use super::*;

pub(super) fn summarize(
    report: &mut ImportReport,
    primitive_count: usize,
    port_count: usize,
    s: f64,
) {
    report.info(format!(
        "{} primitivas, {} portas, escala {s} mm/px",
        primitive_count, port_count
    ));
}
