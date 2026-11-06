// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! frame editor entry point. Read state for data, input for actions, and draw for UI.

use crate::error::{GuiError, GuiResult};
use crate::{
    app::{CadApp, Level},
    camera::{Camera, grid_points, snap},
    editing::GraphicEdit,
    frame_view, theme,
};
use cad_craft::domain::*;
use eframe::egui::{self, Key, PointerButton, Rect, Sense, Stroke, Ui};
use std::collections::BTreeSet;

mod draw;
mod files;
mod forms;
mod geometry;
mod input;
mod state;

pub(crate) use files::read_external_frame;
use files::*;
use forms::*;
use geometry::*;
use state::*;
pub use state::{FrameEditor, FrameTool};
