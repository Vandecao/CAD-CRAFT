// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! QElectroTech titleblock import; start at `import`, then read, convert and report.
//!
//! A titleblock is a grid (`cols`: fixed `Npx`, `tN%` of total, `rN%` of
//! remaining width; `rows` in pixels) with field cells (label and value using
//! `%var`/`%{var}` variables) and logo cells. It sits at the bottom of a
//! user-selected sheet inside a border with a margin.
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
    /// Border margin in mm.
    pub margin: f64,
    /// QElectroTech pixel scale in mm per pixel.
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
