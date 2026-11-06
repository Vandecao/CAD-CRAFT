// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! EPLAN frame import; start at `import`, then read, convert and report.
//!
//! O nome dos elementos (`O178`, `O179`…) muda de arquivo para arquivo; o
//! tipo estável é o primeiro número de `A19="tipo/id"`. Códigos conferidos
//! nas molduras do usuário e na prévia SVG embutida pelo próprio EPLAN:
//! 21 moldura, 26 grupo, 30 texto, 31 linha, 34 polilinha, 53 texto
//! especial, 76 configuração, 89 retângulo, 91 imagem, 212 campo de usuário.
use crate::build::{Builder, round3};
use crate::xml::{self, Element};
use crate::{ImportResult, ImportedFrame, check_size, fail};
use cadcraft_model::domain::{
    FrameElement, FrameTextStyle, LocalizedText, SpecialField, Text, TextAlign, ZoneAxis,
    ZoneLabelStyle, ZoneLayout,
};

/// Espessura usada quando o EPLAN indica "da camada" (`L`).
const DEFAULT_WIDTH: f64 = 0.25;
const DEFAULT_TEXT_HEIGHT: f64 = 2.5;
/// Classe de estrutura do identificador 1001/1003: função (=) e local (+).
const STRUCTURE_FUNCTION: &str = "20132";
const STRUCTURE_LOCATION: &str = "20134";
/// Rótulos de coluna e de linha desenhados pela moldura EPLAN.
const COLUMN_LABEL: u32 = 19005;
const ROW_LABEL: u32 = 19006;

/// Imagens referenciadas (`$(MD_IMG)\logo.jpg`) são pedidas ao chamador,
/// que sabe onde fica a pasta de imagens do EPLAN.
pub type ImageResolver<'a> = dyn FnMut(&str) -> Option<Vec<u8>> + 'a;

mod convert;
mod read;
mod report;

pub use convert::eplan_property;
pub(crate) use convert::localized;

/// Imports an EPLAN frame using the supplied image resolver.
pub fn import(bytes: &[u8], resolve: &mut ImageResolver<'_>) -> ImportResult<ImportedFrame> {
    convert::convert(&read::read(bytes)?, resolve)
}
