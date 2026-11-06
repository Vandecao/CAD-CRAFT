// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! QElectroTech symbol import; start at `import`, then read, convert and report.
//!
//! QET coordinates are pixels relative to the insertion hotspot, with Y
//! downward. CAD-CRAFT uses mm and Y upward. Default scale: 0.5 mm/px.
//! QElectroTech collection elements (CC-BY 3.0) are user-provided inputs;
//! they are neither versioned here nor used as training data.
use crate::xml::{self, Element};
use crate::{ImportReport, ImportResult, check_size, fail};
use cadcraft_model::domain::{
    LineDash, Point2D, PrimitiveStyle, Rect2D, Size2D, SymbolDefinition, SymbolDefinitionId,
    SymbolPort, SymbolPrimitive, SymbolTextField, TextAlign,
};
use std::collections::{BTreeMap, BTreeSet};

/// Default scale in mm per QElectroTech pixel.
pub const DEFAULT_MM_PER_PX: f64 = 0.5;

#[derive(Debug, Clone)]
pub struct ElmtOptions {
    pub mm_per_px: f64,
    /// Library category, such as the source folder name.
    pub category: String,
    /// ID base when the element has no UUID, such as the filename.
    pub id_hint: String,
}
impl ElmtOptions {
    pub fn new(id_hint: impl Into<String>) -> Self {
        Self {
            mm_per_px: DEFAULT_MM_PER_PX,
            category: "QElectroTech".into(),
            id_hint: id_hint.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImportedSymbol {
    pub definition: SymbolDefinition,
    pub report: ImportReport,
}

mod convert;
mod read;
mod report;

pub(crate) use convert::import_definition;
pub(crate) use convert::{align_of, font_height};

pub fn import(bytes: &[u8], options: &ElmtOptions) -> ImportResult<ImportedSymbol> {
    import_definition(&read::read(bytes)?, options)
}
