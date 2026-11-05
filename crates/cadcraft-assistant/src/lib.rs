// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Assistant core of CAD-CRAFT: help, project queries, change proposals and
//! the conversation loop. No network and no GUI: the provider (ChatGPT) and
//! the panel live in the application.
//!
//! The model answers in text. Actions come only from fenced ` ```cadcraft `
//! blocks in that text ([`protocol`]); CAD-CRAFT validates every one against
//! a closed catalogue ([`tools`]), runs queries itself and turns changes into
//! [`proposal::Proposal`]s that the user must apply. Data from the project is
//! always sent as delimited, neutralised data ([`prompt`]). Start at
//! [`conversation::Conversation`].

pub mod conversation;
pub mod help;
pub mod prompt;
pub mod proposal;
pub mod protocol;
pub mod tools;
