// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! CAD-CRAFT core as one dependency: a facade that re-exports the crates.
//!
//! - [`domain`] and [`error`]: neutral project model (`cadcraft-model`).
//! - [`persistence`]: native file formats and migrations (`cadcraft-format`).
//! - [`interchange`]: importers for other platforms (`cadcraft-import`).
//! - [`display`] and [`font`]: display list of a page (`cadcraft-render`).
//! - [`pdf`], [`dxf_export`] and [`report_export`]: exporters (`cadcraft-export`).
//!
//! Everything goes into the `Project` model and everything comes out of it.

/// Neutral project model; also brings [`display::PageDisplayList`] into scope,
/// so `project.page_display_list(page)` works with `use cad_craft::domain::*`.
pub mod domain {
    pub use cadcraft_model::domain::*;
    pub use cadcraft_render::display::PageDisplayList;
}
pub use cadcraft_export::{dxf_export, pdf, report_export};
pub use cadcraft_format as persistence;
pub use cadcraft_import as interchange;
pub use cadcraft_model::error;
pub use cadcraft_render::{display, font};
