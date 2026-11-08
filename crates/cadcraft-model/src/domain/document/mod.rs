// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Document model types; start with the re-exported entities below.
use super::*;
pub(in crate::domain) mod asset;
pub(in crate::domain) mod definition;
pub(in crate::domain) mod document_structure;
pub(in crate::domain) mod frame;
pub(in crate::domain) mod page;
pub(in crate::domain) mod page_graphic;
pub(in crate::domain) mod single_line;
pub use asset::*;
pub use definition::*;
pub use document_structure::*;
pub use frame::*;
pub use page::*;
pub use page_graphic::*;
pub use single_line::*;
