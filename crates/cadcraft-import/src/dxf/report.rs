// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Resolves drawing units and reports missing or unsupported unit metadata; start at `drawing_unit`.
use crate::ImportReport;

fn unit_mm(code: i64) -> Option<f64> {
    cadcraft_model::dxf::unit_mm(code)
}

pub(super) fn drawing_unit(
    specified: Option<f64>,
    header_unit: impl FnOnce() -> Option<i64>,
    report: &mut ImportReport,
) -> f64 {
    match specified {
        Some(u) => u,
        None => match header_unit() {
            Some(code) => match unit_mm(code) {
                Some(u) => u,
                None => {
                    report.info("unidade do desenho não informada: usado milímetro");
                    1.0
                }
            },
            None => {
                report.info("unidade do desenho não informada: usado milímetro");
                1.0
            }
        },
    }
}
