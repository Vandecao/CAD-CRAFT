// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Reports EPLAN zone labels replaced by neutral zone rendering; start at `summarize`.
use super::convert::Labels;
use crate::ImportReport;

pub(super) fn summarize(report: &mut ImportReport, labels: &Labels) {
    if !labels.columns.is_empty() || !labels.rows.is_empty() {
        report.info(format!(
            "{} rótulos de coluna/linha do EPLAN substituídos pelos rótulos da divisão",
            labels.columns.len() + labels.rows.len()
        ));
    }
}
