// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! EPLAN frame import; start at `import`, then read, convert and report.
//!
//! Element names (`O178`, `O179`…) vary between files; the stable
//! type is the first number in `A19="type/id"`. Codes were checked
//! against user frames and EPLAN's own embedded SVG preview:
//! 21 frame, 26 group, 30 text, 31 line, 34 polyline, 53 special
//! text, 76 configuration, 89 rectangle, 91 image, 212 user field.
use crate::build::{Builder, round3};
use crate::xml::{self, Element};
use crate::{ImportResult, ImportedFrame, check_size, fail};
use cadcraft_model::domain::{
    FrameElement, FrameTextStyle, LocalizedText, SpecialField, Text, TextAlign, ZoneAxis,
    ZoneLabelStyle, ZoneLayout,
};

/// Stroke width used when EPLAN specifies the layer default (`L`).
const DEFAULT_WIDTH: f64 = 0.25;
const DEFAULT_TEXT_HEIGHT: f64 = 2.5;
/// Structure class for identifiers 1001/1003: function (=) and location (+).
const STRUCTURE_FUNCTION: &str = "20132";
const STRUCTURE_LOCATION: &str = "20134";
/// Column and row labels drawn by the EPLAN frame.
const COLUMN_LABEL: u32 = 19005;
const ROW_LABEL: u32 = 19006;

/// Referenced images (`$(MD_IMG)\logo.jpg`) are requested from the caller,
/// which knows where EPLAN's image directory is located.
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
