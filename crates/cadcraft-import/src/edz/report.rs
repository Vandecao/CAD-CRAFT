// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Defines EDZ conversion results with loss reports and unmapped property codes; start at `EdzImport`.
use super::*;

#[derive(Debug, Clone)]
pub struct EdzImport {
    pub container: EdzContainer,
    pub parts: Vec<Part>,
    pub macros: Vec<MacroDefinition>,
    pub report: ImportReport,
    pub unknown_codes: BTreeSet<String>,
}
