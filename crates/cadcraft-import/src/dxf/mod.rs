// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! ASCII DXF frame exchange; start at `import` or `export` and follow the phase modules.
//!
//! Reads `LINE`, `LWPOLYLINE`/`POLYLINE` (including bulge arcs),
//! `CIRCLE`, `ARC`, `TEXT`, simple `MTEXT`, `ATTDEF`/`ATTRIB` and blocks
//! with expanded `INSERT`s. `<<code>>` or `%{code}` texts and attributes become
//! special texts. Writes DXF R12 (AC1009) in mm; stroke width is encoded in
//! the layer name (`CAD-CRAFT 0.35mm`), with usable area and zones in a `999` comment,
//! allowing CAD-CRAFT to reimport frames without losing geometry.
use crate::build::{Builder, round3};
use crate::{
    ImportReport, ImportResult, ImportedFrame, check_size, fail, parse_special_code, qet_variable,
    special_code,
};
use cadcraft_model::domain::{
    FrameElement, FrameTemplate, FrameTextStyle, LocalizedText, Size2D, SpecialField, Text,
    TextAlign, ZoneAxis, ZoneLayout,
};
use std::collections::BTreeMap;
use std::fmt::Write as _;

const DEFAULT_WIDTH: f64 = 0.25;
const MAX_BLOCK_DEPTH: usize = 8;
const MAX_ELEMENTS: usize = 100_000;
const META_PREFIX: &str = "CAD-CRAFT moldura ";
const LAYER_PREFIX: &str = "CAD-CRAFT ";
const WATERMARK_LAYER: &str = "CAD-CRAFT marca d'agua";

#[derive(Debug, Clone)]
pub struct DxfImportOptions {
    pub name: String,
    /// Millimetres per drawing unit; `None` uses `$INSUNITS`, falling back to mm.
    pub unit_mm: Option<f64>,
    /// Explicit sheet; `None` uses drawing limits or the bounding box.
    pub sheet: Option<Size2D>,
}
impl DxfImportOptions {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            unit_mm: None,
            sheet: None,
        }
    }
}

mod convert;
mod read;
mod report;
mod write;
use read::{Block, Entity, Parsed};

#[derive(serde::Serialize, serde::Deserialize)]
struct Meta {
    name: String,
    description: String,
    usable: [f64; 4],
    zones: ZoneLayout,
}

pub fn import(bytes: &[u8], options: &DxfImportOptions) -> ImportResult<ImportedFrame> {
    convert::convert(read::parse(bytes)?, options)
}

/// Exports a frame as R12 DXF, reporting unsupported elements.
pub fn export(frame: &FrameTemplate) -> (Vec<u8>, ImportReport) {
    write::export(frame)
}

#[cfg(test)]
mod tests {
    use super::{convert::clean_text, write::escape};

    #[test]
    fn text_escapes_and_mtext_codes_are_cleaned() {
        assert_eq!(clean_text("%%c10 %%d %%p1", false), "Ø10 ° ±1");
        assert_eq!(clean_text("\\U+00E7\\U+00E3o", false), "ção");
        assert_eq!(
            clean_text("{\\fArial|b0;Linha}\\PDois", true),
            "Linha\nDois"
        );
        assert_eq!(escape("Ação"), "A\\U+00E7\\U+00E3o");
    }
}
