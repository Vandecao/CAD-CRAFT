// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! physical editor/connectors entry point. Read state for data, input for actions, and draw for UI.

use super::*;
use std::collections::BTreeMap;

mod draw;
mod input;
mod mating;
mod state;
mod terminals;

use state::*;
pub(in crate::physical_editor) use state::{ConnectorDraft, KindDraft, terminal_from_draft};
