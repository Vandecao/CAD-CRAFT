// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Symbols Project operations; start with the subject modules and delta types below.
use super::*;
pub(in crate::domain::project) mod macros;
pub(in crate::domain::project) mod operations;
pub(in crate::domain::project) mod placement;
pub(in crate::domain::project) mod symbol_texts;
pub use macros::{MacroInsertDelta, MacroPlacement, MacroReport};
pub use operations::SymbolUpdateDelta;
pub use placement::{
    ExistingFunctionDelta, ExistingFunctionPlacement, PlacementDelta, PlacementDevice,
    SymbolPlacement,
};
