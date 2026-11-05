// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Numbering schemes, filters and preview rows; start with NumberingScheme.
use super::{DeviceId, DocumentId, PageId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceNumberFormat {
    #[default]
    Sequential,
    PageColumn,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumberingOrder {
    #[default]
    PageColumnRow,
    PositionXY,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WireNumberFormat {
    #[default]
    Sequential,
    PageColumn,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumberingScope {
    #[default]
    Project,
    Document,
    Pages,
    Selection,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumberingScheme {
    pub device_format: DeviceNumberFormat,
    #[serde(default)]
    pub wire_format: WireNumberFormat,
    #[serde(default)]
    pub scope: NumberingScope,
    pub order: NumberingOrder,
    pub start: u32,
    pub step: u32,
    pub use_on_insert: bool,
    #[serde(default = "default_keep_locked")]
    pub keep_locked: bool,
}
fn default_keep_locked() -> bool {
    true
}

impl Default for NumberingScheme {
    fn default() -> Self {
        Self {
            device_format: DeviceNumberFormat::Sequential,
            wire_format: WireNumberFormat::Sequential,
            scope: NumberingScope::Project,
            order: NumberingOrder::PageColumnRow,
            start: 1,
            step: 1,
            use_on_insert: false,
            keep_locked: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NumberingFilter {
    Project,
    Document(DocumentId),
    Pages(Vec<PageId>),
    Selection(Vec<DeviceId>),
}
