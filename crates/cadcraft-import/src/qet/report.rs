// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Finalizes the unchanged QElectroTech conversion report; start at `finish`.
use super::*;

pub(super) fn finish(mut importer: Importer<'_>) -> ImportResult<QetImport> {
    let st = &importer.stats;
    importer.report.info(format!(
        "{} folha(s), {} símbolo(s), {} ocorrência(s), {} dispositivo(s), {} fio(s), {} relatório(s) de folha, {} texto(s), {} desenho(s) sem terminal",
        st.pages, st.symbols, st.occurrences, st.devices, st.conductors, st.sheet_reports, st.texts, st.drawings
    ));
    Ok(QetImport {
        project: importer.project,
        report: importer.report,
        stats: importer.stats,
    })
}
