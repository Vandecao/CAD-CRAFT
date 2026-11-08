// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! symbol editor entry point. Read state for data, input for actions, and draw for UI.

use crate::{
    app::{CadApp, Level},
    camera::{Camera, grid_points, snap},
    symbol_view, theme,
};
use cad_craft::domain::*;
use eframe::egui::{self, Key, PointerButton, Rect, Sense, Stroke, Ui};
use std::collections::BTreeMap;

mod draw;
mod fields;
mod input;
mod state;

use state::*;
pub use state::{FieldChoice, SymbolEditor, SymbolItem, SymbolTool};
