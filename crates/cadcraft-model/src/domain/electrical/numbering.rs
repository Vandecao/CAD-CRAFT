// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Numbering schemes, filters and preview rows; start with NumberingScheme.
use super::{DeviceId, DocumentId, PageId};
use serde::{Deserialize, Serialize};

/// Fields used to generate a device designation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceNumberFormat {
    /// Sequential numbering without positional fields.
    #[default]
    Sequential,
    /// Number using page and column.
    PageColumn,
}

/// Spatial ordering used when assigning sequential numbers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumberingOrder {
    /// Order by page, then increasing column and descending row.
    #[default]
    PageColumnRow,
    /// Order by page, then decreasing Y and increasing X.
    PositionXY,
}

/// Fields used to generate a logical connection number.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WireNumberFormat {
    /// Sequential numbering without positional fields.
    #[default]
    Sequential,
    /// Number using page and column.
    PageColumn,
}

/// Boundary at which a numbering sequence restarts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumberingScope {
    /// The entire project.
    #[default]
    Project,
    /// Restart numbering within each document.
    Document,
    /// Restart numbering within each page.
    Pages,
    /// Restrict to the explicit selection.
    Selection,
}

/// Project numbering formats, order, sequence and lock policy.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumberingScheme {
    /// Format used for automatic device designations.
    pub device_format: DeviceNumberFormat,
    /// Format used for automatic wire numbers.
    #[serde(default)]
    pub wire_format: WireNumberFormat,
    /// Boundary for restarting the numbering sequence.
    #[serde(default)]
    pub scope: NumberingScope,
    /// Explicit ordering used by this operation.
    pub order: NumberingOrder,
    /// First integer assigned by a numbering sequence.
    pub start: u32,
    /// Increment between consecutive generated numbers.
    pub step: u32,
    /// Whether insertion uses the configured automatic numbering.
    pub use_on_insert: bool,
    /// Whether automatic numbering preserves locked values.
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

/// Subset of devices selected for a numbering preview.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NumberingFilter {
    /// The entire project.
    Project,
    /// Include devices represented within the specified document.
    Document(DocumentId),
    /// Include devices represented on the specified pages.
    Pages(Vec<PageId>),
    /// Restrict to the explicit selection.
    Selection(Vec<DeviceId>),
}
