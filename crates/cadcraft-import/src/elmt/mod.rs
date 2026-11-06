// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! QElectroTech symbol import; start at `import`, then read, convert and report.
//!
//! Coordenadas QET: px relativos ao ponto de inserção (hotspot), Y para
//! baixo. CAD-CRAFT: mm, Y para cima. Escala padrão aprovada: 0,5 mm/px.
//! Os elementos da coleção QElectroTech (CC-BY 3.0) servem só de entrada do
//! usuário; não são versionados nem usados como dados de aprendizado.
use crate::xml::{self, Element};
use crate::{ImportReport, ImportResult, check_size, fail};
use cadcraft_model::domain::{
    LineDash, Point2D, PrimitiveStyle, Rect2D, Size2D, SymbolDefinition, SymbolDefinitionId,
    SymbolPort, SymbolPrimitive, SymbolTextField, TextAlign,
};
use std::collections::{BTreeMap, BTreeSet};

/// Escala padrão (mm por px do QElectroTech), decisão da Etapa 17.
pub const DEFAULT_MM_PER_PX: f64 = 0.5;

#[derive(Debug, Clone)]
pub struct ElmtOptions {
    pub mm_per_px: f64,
    /// Categoria na biblioteca (ex.: nome da pasta de origem).
    pub category: String,
    /// Base do ID quando o elemento não tem UUID (ex.: nome do arquivo).
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
