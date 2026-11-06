// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! part editor entry point. Read state for data, input for actions, and draw for UI.

use crate::error::{GuiError, GuiResult};
use crate::{editing::GraphicEdit, preferences::Preferences, state::Session};
use cad_craft::{
    domain::*,
    interchange::{export_parts_csv, import_parts_csv},
    persistence,
};
use eframe::egui::{self, Context, Ui};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

mod draw;
mod input;
mod state;

pub use state::PartEditor;
use state::*;
