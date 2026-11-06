// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Records the converted titleblock grid summary; start at `summarize`.
use crate::ImportReport;

pub(super) fn summarize(
    report: &mut ImportReport,
    cols: &[f64],
    rows: &[f64],
    height: f64,
    s: f64,
) {
    report.info(format!(
        "carimbo de {} × {} células, {:.1} × {:.1} mm ({} mm por px)",
        cols.len(),
        rows.len(),
        cols.iter().sum::<f64>(),
        height,
        s
    ));
}
