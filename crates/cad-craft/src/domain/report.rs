// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::{
    DocumentId, FrameTemplateId, Page, PageId, ProjectId, ReportId, RepresentationId, Size2D,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportKind {
    PageIndex,
    Devices,
    Materials,
    Connections,
    Terminals,
    Cables,
}
impl ReportKind {
    pub const ALL: [Self; 6] = [
        Self::PageIndex,
        Self::Devices,
        Self::Materials,
        Self::Connections,
        Self::Terminals,
        Self::Cables,
    ];
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportFilter {
    pub document: Option<DocumentId>,
    pub page: Option<PageId>,
    pub function_path: Vec<String>,
    pub location_path: Vec<String>,
    /// false: agrupa por peça; true: uma linha por dispositivo e peça.
    #[serde(default)]
    pub materials_by_device: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportSort {
    #[default]
    Default,
    Reverse,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportDefinition {
    pub group: ReportId,
    pub kind: ReportKind,
    pub filter: ReportFilter,
    pub sort: ReportSort,
    pub columns: Vec<String>,
    pub index: u32,
}

#[derive(Clone, Debug)]
pub struct ReportRequest {
    pub kind: ReportKind,
    pub filter: ReportFilter,
    pub sort: ReportSort,
    pub columns: Vec<String>,
    pub document: DocumentId,
    pub frame: Option<FrameTemplateId>,
    pub size: Size2D,
}
impl ReportRequest {
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

#[derive(Clone, Debug, PartialEq)]
pub struct ReportPreview {
    pub rows: usize,
    pub rows_per_page: usize,
    pub pages: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReportDelta {
    pub(crate) project: ProjectId,
    pub(crate) before: Vec<Page>,
    pub(crate) after: Vec<Page>,
}
impl ReportDelta {
    pub fn created_page(&self) -> Option<PageId> {
        self.after.first().map(Page::id)
    }
    pub fn len(&self) -> usize {
        self.before.len().max(self.after.len())
    }
    pub fn is_empty(&self) -> bool {
        self.before == self.after
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ReportValue {
    Text(String),
    Number(f64),
}
impl ReportValue {
    pub fn text(&self) -> String {
        match self {
            Self::Text(v) => v.clone(),
            Self::Number(v) => v.to_string(),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ReportCell {
    pub value: ReportValue,
    pub target: Option<RepresentationId>,
}
impl ReportCell {
    pub fn text(&self) -> String {
        self.value.text()
    }
    pub fn plain(value: impl Into<String>) -> Self {
        Self {
            value: ReportValue::Text(value.into()),
            target: None,
        }
    }
    pub fn number(value: f64) -> Self {
        Self {
            value: ReportValue::Number(value),
            target: None,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ReportRow {
    pub cells: Vec<ReportCell>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ReportColumn {
    pub title: &'static str,
    pub width_mm: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ReportTable {
    pub kind: ReportKind,
    pub columns: Vec<ReportColumn>,
    pub rows: Vec<ReportRow>,
}
