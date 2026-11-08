// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Document Project operations; start with the subject modules and delta types below.
use super::*;
pub(in crate::domain::project) mod clipboard;
pub(in crate::domain::project) mod document_edit;
pub(in crate::domain::project) mod document_queries;
pub(in crate::domain::project) mod documentation;
pub(in crate::domain::project) mod frames;
pub(in crate::domain::project) mod single_lines;
pub use clipboard::{ClipItem, MAX_CLIPBOARD_ITEMS, SheetEditDelta};
pub use document_edit::{DocumentDelta, DocumentEdit};
pub use frames::{FrameDelta, FrameEdit, FramePackage};
pub use single_lines::{SingleLineDelta, SingleLineEdit};
