// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Geometry model types; start with the re-exported entities below.
pub mod tolerance;
pub(in crate::domain) mod transform;
pub(in crate::domain) mod values;
pub use transform::*;
pub use values::*;
