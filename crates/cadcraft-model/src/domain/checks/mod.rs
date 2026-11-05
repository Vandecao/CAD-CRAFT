// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Checks model types; start with the re-exported entities below.
use super::*;
pub(in crate::domain) mod check;
pub(in crate::domain) mod report;
pub use check::*;
pub use report::*;
