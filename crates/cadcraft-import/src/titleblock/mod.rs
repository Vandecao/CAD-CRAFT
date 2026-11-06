// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! QElectroTech titleblock import; start at `import`, then read, convert and report.
//!
//! O carimbo é uma grade (`cols`: `Npx` fixas, `tN%` do total, `rN%` do
//! restante; `rows` em px) com células de campo (rótulo, valor com variáveis
//! `%var`/`%{var}`) e de logotipo. Ele é colocado na base de uma folha
//! escolhida pelo usuário, dentro de uma borda com margem.
use crate::build::{Builder, round3};
use crate::xml::{self, Element};
use crate::{ImportResult, ImportedFrame, base64_decode, check_size, fail, qet_variable};
use cadcraft_model::domain::{
    FrameTextStyle, LocalizedText, Size2D, TextAlign, ZoneAxis, ZoneLayout, inspect_image,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy)]
pub struct TitleblockOptions {
    pub sheet: Size2D,
    /// Margem da borda (mm).
    pub margin: f64,
    /// Escala das medidas em px do QElectroTech (mm por px).
    pub mm_per_px: f64,
}
impl TitleblockOptions {
    pub fn new(sheet: Size2D) -> Self {
        Self {
            sheet,
            margin: 5.0,
            mm_per_px: 0.25,
        }
    }
}

mod convert;
mod read;
mod report;

pub(crate) use convert::import_template;

pub fn import(bytes: &[u8], options: TitleblockOptions) -> ImportResult<ImportedFrame> {
    import_template(&read::read(bytes)?, options)
}
