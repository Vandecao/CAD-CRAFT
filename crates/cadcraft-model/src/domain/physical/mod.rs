// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Physical model types; start with the re-exported entities below.
use super::*;
pub(in crate::domain) mod cable;
pub(in crate::domain) mod cable_core;
pub(in crate::domain) mod conductor;
pub(in crate::domain) mod connector;
pub(in crate::domain) mod part;
pub(in crate::domain) mod physical_elements;
pub(in crate::domain) mod physical_values;
pub(in crate::domain) mod terminal;
pub(in crate::domain) mod terminal_bridge;
pub(in crate::domain) mod terminal_strip;
pub use cable::*;
pub use cable_core::*;
pub use conductor::*;
pub use connector::*;
pub use part::*;
pub use physical_elements::*;
pub use physical_values::*;
pub use terminal::*;
pub use terminal_bridge::*;
pub use terminal_strip::*;
