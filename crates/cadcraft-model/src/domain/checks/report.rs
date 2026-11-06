// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Report definitions and row data; start with ReportKind and ReportDefinition.
use super::{
    DocumentId, FrameTemplateId, Page, PageId, ProjectId, ReportId, RepresentationId, Size2D,
};
use serde::{Deserialize, Serialize};

/// Technical report category supported by the project.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportKind {
    /// Page index report.
    PageIndex,
    /// Device list report.
    Devices,
    /// Bill of materials report.
    Materials,
    /// Logical connection list report.
    Connections,
    /// Terminal list report.
    Terminals,
    /// Cable list report.
    Cables,
}
impl ReportKind {
    /// All supported variants in display order.
    pub const ALL: [Self; 6] = [
        Self::PageIndex,
        Self::Devices,
        Self::Materials,
        Self::Connections,
        Self::Terminals,
        Self::Cables,
    ];
    /// Return the pt-BR display label.
    pub fn label(self) -> &'static str {
        match self {
            Self::PageIndex => "Índice de páginas",
            Self::Devices => "Lista de dispositivos",
            Self::Materials => "Lista de materiais",
            Self::Connections => "Lista de conexões",
            Self::Terminals => "Lista de bornes",
            Self::Cables => "Lista de cabos",
        }
    }
}

/// Optional document, page and structural-path constraints on report rows.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportFilter {
    /// Document identifier used by this operation or filter.
    pub document: Option<DocumentId>,
    /// Page identifier used by this operation or filter.
    pub page: Option<PageId>,
    /// Functional-path constraint on included records.
    pub function_path: Vec<String>,
    /// Location-path constraint on included records.
    pub location_path: Vec<String>,
    /// False groups by part; true produces one row per device and part.
    #[serde(default)]
    pub materials_by_device: bool,
}

/// Ordering applied to generated report rows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportSort {
    /// Default forward ordering.
    #[default]
    Default,
    /// Reverse ordering.
    Reverse,
}

/// Persistent report configuration attached to a generated page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportDefinition {
    /// Stable identity of the generated report group.
    pub group: ReportId,
    /// Declared category of this record or operation.
    pub kind: ReportKind,
    /// Constraints selecting the records included in the report.
    pub filter: ReportFilter,
    /// Ordering applied to report rows.
    pub sort: ReportSort,
    /// Ordered report columns.
    pub columns: Vec<String>,
    /// Position or indexed-field selector within the containing record.
    pub index: u32,
}

/// Configuration and destination for generating report pages.
#[derive(Clone, Debug)]
pub struct ReportRequest {
    /// Declared category of this record or operation.
    pub kind: ReportKind,
    /// Constraints selecting the records included in the report.
    pub filter: ReportFilter,
    /// Ordering applied to report rows.
    pub sort: ReportSort,
    /// Ordered report columns.
    pub columns: Vec<String>,
    /// Document identifier used by this operation or filter.
    pub document: DocumentId,
    /// Frame template used by this sheet or operation.
    pub frame: Option<FrameTemplateId>,
    /// Sheet dimensions in millimetres.
    pub size: Size2D,
}
impl ReportRequest {
    /// Create a report request with default filtering, sorting and column selection.
    pub fn new(kind: ReportKind, document: DocumentId, size: Size2D) -> Self {
        Self {
            kind,
            filter: ReportFilter::default(),
            sort: ReportSort::Default,
            columns: Vec::new(),
            document,
            frame: None,
            size,
        }
    }
}

/// Row and page counts calculated before report generation.
#[derive(Clone, Debug, PartialEq)]
pub struct ReportPreview {
    /// Report rows or their count before pagination.
    pub rows: usize,
    /// Maximum data rows fitted on each generated page.
    pub rows_per_page: usize,
    /// Pages belonging to this group or classification.
    pub pages: usize,
}

/// Before and after report pages for exact undo and redo.
#[derive(Clone, Debug, PartialEq)]
pub struct ReportDelta {
    pub(crate) project: ProjectId,
    pub(crate) before: Vec<Page>,
    pub(crate) after: Vec<Page>,
}
impl ReportDelta {
    /// Return the first generated page ID, if any.
    pub fn created_page(&self) -> Option<PageId> {
        self.after.first().map(Page::id)
    }
    /// Return the number of contained entries or entity changes recorded by this value.
    pub fn len(&self) -> usize {
        self.before.len().max(self.after.len())
    }
    /// Whether this collection or recorded edit contains no effective entries or changes.
    pub fn is_empty(&self) -> bool {
        self.before == self.after
    }
}

/// Typed text or numeric report cell content.
#[derive(Clone, Debug, PartialEq)]
pub enum ReportValue {
    /// Text content.
    Text(String),
    /// Numeric content.
    Number(f64),
}
impl ReportValue {
    /// Render this value as display text.
    pub fn text(&self) -> String {
        match self {
            Self::Text(v) => v.clone(),
            Self::Number(v) => v.to_string(),
        }
    }
}
/// Report value with an optional graphical navigation target.
#[derive(Clone, Debug, PartialEq)]
pub struct ReportCell {
    /// Typed content assigned by this record.
    pub value: ReportValue,
    /// Explicit target of this record or operation.
    pub target: Option<RepresentationId>,
}
impl ReportCell {
    /// Render this value as display text.
    pub fn text(&self) -> String {
        self.value.text()
    }
    /// Create a text cell without a navigation target.
    pub fn plain(value: impl Into<String>) -> Self {
        Self {
            value: ReportValue::Text(value.into()),
            target: None,
        }
    }
    /// Create a numeric cell without a navigation target.
    pub fn number(value: f64) -> Self {
        Self {
            value: ReportValue::Number(value),
            target: None,
        }
    }
}
/// Ordered cells in one technical-report row.
#[derive(Clone, Debug, PartialEq)]
pub struct ReportRow {
    /// Cell values in column order.
    pub cells: Vec<ReportCell>,
}
/// Report column title and physical layout width.
#[derive(Clone, Debug, PartialEq)]
pub struct ReportColumn {
    /// Human-readable column or rule title.
    pub title: &'static str,
    /// Width in millimetres.
    pub width_mm: f64,
}
/// Derived report columns and rows before pagination.
#[derive(Clone, Debug, PartialEq)]
pub struct ReportTable {
    /// Declared category of this record or operation.
    pub kind: ReportKind,
    /// Ordered report columns.
    pub columns: Vec<ReportColumn>,
    /// Report rows or their count before pagination.
    pub rows: Vec<ReportRow>,
}
