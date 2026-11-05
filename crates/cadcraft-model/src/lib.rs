// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Neutral project model of CAD-CRAFT, independent of files and GUI.
//!
//! The [`domain::Project`] aggregate protects every relation between entities
//! and derives networks from logical connectivity. Everything that CAD-CRAFT
//! imports or exports passes through this model. Start at [`domain::Project`].

mod document_index;
pub mod domain;
pub mod error;
mod graph;
mod physical_index;
