// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! PDF export options, reports and shared state; start with export_pdf in page.

use cadcraft_model::domain::*;
use cadcraft_model::error::DomainError;
use cadcraft_render::display::PageDisplayList;
use cadcraft_render::display::{DisplayColor, DisplayItem, DisplayList, DisplayText};
use cadcraft_render::font;
use pdf_writer::types::{ActionType, AnnotationType, LineCapStyle, LineJoinStyle, PageMode};
use pdf_writer::{Content, Date, Filter, Finish, Name, Pdf, Rect, Ref, Str, TextStr};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

/// Millimetres to PDF points.
const PT_PER_MM: f64 = 72.0 / 25.4;
const FONT: Name<'static> = Name(b"F1");

/// Pages to export.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum PdfScope {
    #[default]
    Project,
    Document(DocumentId),
    /// Selected pages, emitted in project order.
    Pages(Vec<PageId>),
}

/// Scale on printed paper.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PaperFit {
    /// Shrink or enlarge to fit the paper, centered.
    #[default]
    Fit,
    /// Centered at 1:1; content outside the paper is clipped.
    Actual,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PdfOptions {
    pub scope: PdfScope,
    /// Monochrome: render vector content in black and gray only.
    pub monochrome: bool,
    pub bookmarks: bool,
    /// Print paper size in mm and scale; `None` uses the actual sheet size.
    pub paper: Option<(Size2D, PaperFit)>,
}

impl Default for PdfOptions {
    fn default() -> Self {
        Self {
            scope: PdfScope::Project,
            monochrome: false,
            bookmarks: true,
            paper: None,
        }
    }
}

/// Export summary and required adaptations.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PdfReport {
    pub pages: usize,
    pub links: usize,
    /// Links targeting pages outside the exported range.
    pub omitted_links: usize,
    /// Characters unavailable in Helvetica WinAnsi, replaced with `?`.
    pub replaced: BTreeSet<char>,
    /// Missing or unreadable images, rendered as boxes.
    pub missing_images: usize,
}

impl PdfReport {
    pub fn summary(&self) -> String {
        let mut text = format!(
            "{} página(s), {} link(s) de referência.",
            self.pages, self.links
        );
        if self.omitted_links > 0 {
            text.push_str(&format!(
                " {} referência(s) para páginas fora do alcance ficaram sem link.",
                self.omitted_links
            ));
        }
        if !self.replaced.is_empty() {
            let chars: String = self.replaced.iter().collect();
            text.push_str(&format!(
                " Caracteres sem suporte na fonte, trocados por '?': {chars}."
            ));
        }
        if self.missing_images > 0 {
            text.push_str(&format!(
                " {} imagem(ns) sem conteúdo desenhada(s) como caixa.",
                self.missing_images
            ));
        }
        text
    }
}

#[derive(Clone, Debug)]
pub struct PdfOutput {
    pub bytes: Vec<u8>,
    pub report: PdfReport,
}

#[derive(Debug)]
pub enum PdfError {
    /// No pages in the selected range.
    Empty,
    Cancelled,
    Domain(DomainError),
}

impl std::fmt::Display for PdfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "nenhuma página no alcance escolhido"),
            Self::Cancelled => write!(f, "exportação cancelada"),
            Self::Domain(e) => write!(f, "{e}"),
        }
    }
}
impl std::error::Error for PdfError {}
impl From<DomainError> for PdfError {
    fn from(e: DomainError) -> Self {
        Self::Domain(e)
    }
}

/// Pages in the range, in document and page order.
pub fn pages_in_scope(project: &Project, scope: &PdfScope) -> Result<Vec<PageId>, PdfError> {
    let mut out = vec![];
    for document in project.documents() {
        if let PdfScope::Document(id) = scope
            && document.id() != *id
        {
            continue;
        }
        for page in project.ordered_pages(document.id())? {
            if let PdfScope::Pages(ids) = scope
                && !ids.contains(&page.id())
            {
                continue;
            }
            out.push(page.id());
        }
    }
    if let PdfScope::Pages(ids) = scope
        && let Some(missing) = ids.iter().find(|id| !out.contains(id))
    {
        return Err(DomainError::MissingEntity(EntityRef::Page(*missing)).into());
    }
    Ok(out)
}

/// Affine `[a, b, c, d, e, f]`: `x' = a·x + c·y + e`, `y' = b·x + d·y + f`.
type Affine = [f64; 6];

/// Sheet placement on PDF paper in points.
#[derive(Clone, Copy)]
struct Placement {
    media: (f64, f64),
    /// Sheet mm to PDF points.
    matrix: Affine,
}
/// Sequential object references.
struct Refs(i32);
impl Refs {
    fn next(&mut self) -> Ref {
        self.0 += 1;
        Ref::new(self.0)
    }
}

/// Image already embedded in the PDF: resource name and reference.
type ImageEntry = Option<(String, Ref)>;

struct Writer<'a> {
    project: &'a Project,
    options: &'a PdfOptions,
    pdf: Pdf,
    refs: Refs,
    images: BTreeMap<AssetId, ImageEntry>,
    report: PdfReport,
}

mod content;
mod geometry;
mod images;
mod page;
mod text;

use geometry::{compose, f32s, set_fill, set_stroke};
use images::deflate;
pub use page::export_pdf;
