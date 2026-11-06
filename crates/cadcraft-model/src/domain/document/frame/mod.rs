// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Sheet frames, usable areas, zones, title blocks and special texts; start with FrameTemplate.
//! Local millimetre coordinates use a bottom-left origin, X right and Y up. Zones divide the usable
//! area: columns run left to right and rows top to bottom. Intervals are half-open except the final
//! interval, which includes its end.
use super::*;
use crate::domain::tolerance::EPS_MM as EPS;
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

fn invalid(reason: &'static str) -> DomainError {
    DomainError::InvalidRelation(reason)
}
const MAX_ZONES: usize = 1000;
const MAX_ELEMENTS: usize = 100_000;

/// Zone labels: numbers starting at `start`, or letters A through Z, then AA, AB and so on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneLabelStyle {
    /// Decimal numbering beginning at the supplied value.
    Numbers {
        /// Number assigned to the first zone.
        start: u32,
    },
    /// Alphabetic labels A through Z, then AA, AB and so on.
    Letters {
        /// Zero-based alphabetic offset of the first zone.
        start: u32,
    },
}
impl ZoneLabelStyle {
    /// Format the zone label at the zero-based axis index.
    pub fn label(self, index: usize) -> String {
        match self {
            Self::Numbers { start } => (start as u64 + index as u64).to_string(),
            Self::Letters { start } => {
                let mut n = start as u64 + index as u64 + 1;
                let mut out = Vec::new();
                while n > 0 {
                    let r = ((n - 1) % 26) as u8;
                    out.push(b'A' + r);
                    n = (n - 1) / 26;
                }
                out.reverse();
                String::from_utf8(out).expect("ASCII")
            }
        }
    }
}

/// Division of one usable-area axis. Empty `sizes` means no zones on that axis.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZoneAxis {
    /// Zone lengths in millimetres; empty means no division.
    pub sizes: Vec<f64>,
    /// Rendering or zone-label style.
    pub style: ZoneLabelStyle,
    /// Column labels above; row labels to the left.
    pub labels_start: bool,
    /// Column labels below; row labels to the right.
    pub labels_end: bool,
}
impl ZoneAxis {
    /// Create an undivided zone axis.
    pub fn none() -> Self {
        Self {
            sizes: vec![],
            style: ZoneLabelStyle::Numbers { start: 1 },
            labels_start: false,
            labels_end: false,
        }
    }
    /// Create equally sized zones using the supplied label style.
    pub fn uniform(count: usize, size: f64, style: ZoneLabelStyle) -> Self {
        Self {
            sizes: vec![size; count],
            style,
            labels_start: true,
            labels_end: false,
        }
    }
    /// Return the combined zone extent in millimetres.
    pub fn total(&self) -> f64 {
        self.sizes.iter().sum()
    }
    /// Index of the zone containing `offset`, measured from the axis origin.
    pub fn index_at(&self, offset: f64) -> Option<usize> {
        if self.sizes.is_empty() || !offset.is_finite() || offset < -EPS {
            return None;
        }
        let mut end = 0.0;
        for (i, size) in self.sizes.iter().enumerate() {
            end += size;
            let last = i + 1 == self.sizes.len();
            if offset < end - EPS || (last && offset <= end + EPS) {
                return Some(i);
            }
        }
        None
    }
    /// Start and end bounds of each zone measured from the axis origin.
    pub fn bounds(&self) -> Vec<(f64, f64)> {
        let mut start = 0.0;
        self.sizes
            .iter()
            .map(|s| {
                let b = (start, start + s);
                start += s;
                b
            })
            .collect()
    }
    fn validate(&self, available: f64) -> Result<()> {
        if self.sizes.len() > MAX_ZONES
            || self.sizes.iter().any(|s| !s.is_finite() || *s <= 0.0)
            || self.total() > available + EPS
        {
            return Err(invalid("zonas da moldura inválidas ou fora da área útil"));
        }
        Ok(())
    }
}

/// Independent column and row divisions of a frame usable area.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZoneLayout {
    /// Ordered report columns.
    pub columns: ZoneAxis,
    /// Report rows or their count before pagination.
    pub rows: ZoneAxis,
}

/// Text alignment point relative to its origin.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlign {
    /// Bottom left text alignment.
    #[default]
    BottomLeft,
    /// Bottom center text alignment.
    BottomCenter,
    /// Bottom right text alignment.
    BottomRight,
    /// Middle left text alignment.
    MiddleLeft,
    /// Center text alignment.
    Center,
    /// Middle right text alignment.
    MiddleRight,
    /// Top left text alignment.
    TopLeft,
    /// Top center text alignment.
    TopCenter,
    /// Top right text alignment.
    TopRight,
}
impl TextAlign {
    /// Horizontal and bottom-up vertical fractions of the text box.
    pub fn fractions(self) -> (f64, f64) {
        match self {
            Self::BottomLeft => (0.0, 0.0),
            Self::BottomCenter => (0.5, 0.0),
            Self::BottomRight => (1.0, 0.0),
            Self::MiddleLeft => (0.0, 0.5),
            Self::Center => (0.5, 0.5),
            Self::MiddleRight => (1.0, 0.5),
            Self::TopLeft => (0.0, 1.0),
            Self::TopCenter => (0.5, 1.0),
            Self::TopRight => (1.0, 1.0),
        }
    }
}

/// Text height, alignment, rotation and watermark rendering settings.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameTextStyle {
    /// Text or geometry height in millimetres.
    pub height: f64,
    /// Text alignment relative to the origin.
    pub align: TextAlign,
    /// Counterclockwise rotation in degrees.
    pub rotation_degrees: f64,
    /// Watermark drawn before content in a light color.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub watermark: bool,
}
impl FrameTextStyle {
    /// Create text settings with the supplied height and alignment, zero rotation and no watermark.
    pub fn new(height: f64, align: TextAlign) -> Self {
        Self {
            height,
            align,
            rotation_degrees: 0.0,
            watermark: false,
        }
    }
    fn validate(&self) -> Result<()> {
        if !self.height.is_finite() || self.height <= 0.0 || !self.rotation_degrees.is_finite() {
            return Err(invalid("estilo de texto da moldura inválido"));
        }
        Ok(())
    }
}

/// Fixed text with a pt-BR default and language variants, such as en-US.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalizedText {
    /// Fallback text when no language variant matches.
    pub default: String,
    /// Text variants indexed by language key.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub translations: BTreeMap<String, String>,
}
impl LocalizedText {
    /// Create fallback text without language-specific variants.
    pub fn new(default: impl Into<String>) -> Self {
        Self {
            default: default.into(),
            translations: BTreeMap::new(),
        }
    }
    /// Return the requested language variant, falling back to the default text.
    pub fn get(&self, language: Option<&str>) -> &str {
        language
            .and_then(|l| self.translations.get(l))
            .map(String::as_str)
            .unwrap_or(&self.default)
    }
}

/// Value resolved from the project, page or frame.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecialField {
    /// Project name.
    ProjectName,
    /// Project description.
    ProjectDescription,
    /// Project documentary identification.
    ProjectIdentification,
    /// Project metadata selected by key, such as author or customer.
    ProjectProperty(Text),
    /// Containing document name.
    DocumentName,
    /// Page documentary number.
    PageNumber,
    /// Page name.
    PageName,
    /// Page functional classification path.
    PageFunctionalPath,
    /// Page location classification path.
    PageLocationPath,
    /// One-based page position within its document.
    PagePosition,
    /// Number of pages in the containing document.
    PageCountInDocument,
    /// Total project page count.
    PageCountInProject,
    /// Previous page number in document order.
    PreviousPageNumber,
    /// Next page number in document order.
    NextPageNumber,
    /// Page metadata selected by key, such as date or drafter.
    PageProperty(Text),
    /// Frame template name.
    FrameName,
}
impl SpecialField {
    /// Return the pt-BR display label.
    pub fn label(&self) -> String {
        match self {
            Self::ProjectName => "Nome do projeto".into(),
            Self::ProjectDescription => "Descrição do projeto".into(),
            Self::ProjectIdentification => "Identificação do projeto".into(),
            Self::ProjectProperty(k) => format!("Projeto: {}", k.as_str()),
            Self::DocumentName => "Documento".into(),
            Self::PageNumber => "Número da página".into(),
            Self::PageName => "Descrição da página".into(),
            Self::PageFunctionalPath => "Função (=)".into(),
            Self::PageLocationPath => "Localização (+)".into(),
            Self::PagePosition => "Posição da página".into(),
            Self::PageCountInDocument => "Total de páginas do documento".into(),
            Self::PageCountInProject => "Total de páginas do projeto".into(),
            Self::PreviousPageNumber => "Página anterior".into(),
            Self::NextPageNumber => "Página seguinte".into(),
            Self::PageProperty(k) => format!("Página: {}", k.as_str()),
            Self::FrameName => "Nome da moldura".into(),
        }
    }
}

/// Drawable line, shape, text, field or image in a sheet frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum FrameElement {
    /// Straight line primitive.
    Line {
        /// Starting value or geometric endpoint.
        start: Point2D,
        /// Final endpoint in local millimetre coordinates.
        end: Point2D,
        /// Stroke or geometry width in millimetres.
        width: f64,
    },
    /// Rectangular primitive.
    Rectangle {
        /// Rectangle in local millimetre coordinates.
        rect: Rect2D,
        /// Stroke or geometry width in millimetres.
        width: f64,
    },
    /// Circular primitive.
    Circle {
        /// Center in local millimetre coordinates.
        center: Point2D,
        /// Radius in millimetres.
        radius: f64,
        /// Stroke or geometry width in millimetres.
        width: f64,
    },
    /// Circular arc primitive.
    Arc {
        /// Center in local millimetre coordinates.
        center: Point2D,
        /// Radius in millimetres.
        radius: f64,
        /// Initial arc angle in degrees.
        start_degrees: f64,
        /// Signed angular extent of the arc in degrees.
        sweep_degrees: f64,
        /// Stroke or geometry width in millimetres.
        width: f64,
    },
    /// Text content.
    Text {
        /// Origin in millimetre coordinates.
        origin: Point2D,
        /// Text displayed to the user.
        text: LocalizedText,
        /// Rendering or zone-label style.
        style: FrameTextStyle,
    },
    /// Special text; `index` selects `key.n` in indexed properties, such as revision rows.
    Special {
        /// Origin in millimetre coordinates.
        origin: Point2D,
        /// Dynamic text field resolved from the current project context.
        field: SpecialField,
        /// Position or indexed-field selector within the containing record.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        index: Option<u32>,
        /// Rendering or zone-label style.
        style: FrameTextStyle,
    },
    /// Project image fitted into a millimetre rectangle.
    Image {
        /// Rectangle in local millimetre coordinates.
        rect: Rect2D,
        /// Project image asset identifier.
        asset: AssetId,
    },
}
impl FrameElement {
    fn validate(&self) -> Result<()> {
        let width_ok = |w: &f64| w.is_finite() && *w >= 0.0;
        let ok = match self {
            Self::Line { start, end, width } => start != end && width_ok(width),
            Self::Rectangle { rect, width } => {
                rect.size().width() > 0.0 && rect.size().height() > 0.0 && width_ok(width)
            }
            Self::Circle { radius, width, .. } => {
                radius.is_finite() && *radius > 0.0 && width_ok(width)
            }
            Self::Arc {
                radius,
                start_degrees,
                sweep_degrees,
                width,
                ..
            } => {
                radius.is_finite()
                    && *radius > 0.0
                    && start_degrees.is_finite()
                    && sweep_degrees.is_finite()
                    && *sweep_degrees != 0.0
                    && sweep_degrees.abs() <= 360.0
                    && width_ok(width)
            }
            Self::Text { text, style, .. } => {
                style.validate()?;
                !text.default.trim().is_empty()
                    && text
                        .translations
                        .iter()
                        .all(|(k, v)| !k.trim().is_empty() && !v.trim().is_empty())
            }
            Self::Special { style, index, .. } => {
                style.validate()?;
                index.is_none_or(|i| i > 0)
            }
            Self::Image { rect, .. } => rect.size().width() > 0.0 && rect.size().height() > 0.0,
        };
        if ok {
            Ok(())
        } else {
            Err(invalid("elemento de moldura inválido"))
        }
    }
}

/// Versioned sheet frame with validated geometry, zones and title-block elements.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "FrameData")]
pub struct FrameTemplate {
    id: FrameTemplateId,
    name: Text,
    version: u32,
    description: String,
    sheet: Size2D,
    usable: Rect2D,
    zones: ZoneLayout,
    elements: Vec<FrameElement>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FrameData {
    id: FrameTemplateId,
    name: Text,
    version: u32,
    description: String,
    sheet: Size2D,
    usable: Rect2D,
    zones: ZoneLayout,
    elements: Vec<FrameElement>,
}
impl TryFrom<FrameData> for FrameTemplate {
    type Error = DomainError;
    fn try_from(d: FrameData) -> Result<Self> {
        let frame = Self {
            id: d.id,
            name: d.name,
            version: d.version,
            description: d.description,
            sheet: d.sheet,
            usable: d.usable,
            zones: d.zones,
            elements: d.elements,
        };
        frame.validate()?;
        Ok(frame)
    }
}
impl FrameTemplate {
    /// Create a frame template with fresh identity, version one and validated sheet geometry.
    pub fn new(
        name: impl Into<String>,
        sheet: Size2D,
        usable: Rect2D,
        zones: ZoneLayout,
        elements: Vec<FrameElement>,
    ) -> Result<Self> {
        let frame = Self {
            id: FrameTemplateId::new(),
            name: Text::new(name)?,
            version: 1,
            description: String::new(),
            sheet,
            usable,
            zones,
            elements,
        };
        frame.validate()?;
        Ok(frame)
    }
    /// Return this value with description replaced.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
    /// Validated next version of the same template, retaining its ID.
    pub fn revised(
        &self,
        name: Text,
        zones: ZoneLayout,
        elements: Vec<FrameElement>,
    ) -> Result<Self> {
        let frame = Self {
            name,
            version: self.version.saturating_add(1),
            zones,
            elements,
            ..self.clone()
        };
        frame.validate()?;
        Ok(frame)
    }
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> FrameTemplateId {
        self.id
    }
    /// Return the name.
    pub fn name(&self) -> &Text {
        &self.name
    }
    /// Return the version.
    pub fn version(&self) -> u32 {
        self.version
    }
    /// Return the description.
    pub fn description(&self) -> &str {
        &self.description
    }
    /// Return the sheet.
    pub fn sheet(&self) -> Size2D {
        self.sheet
    }
    /// Return the usable.
    pub fn usable(&self) -> Rect2D {
        self.usable
    }
    /// Return the zones.
    pub fn zones(&self) -> &ZoneLayout {
        &self.zones
    }
    /// Iterate or borrow the explicitly stored document elements.
    pub fn elements(&self) -> &[FrameElement] {
        &self.elements
    }
    /// Check local value constraints and report the first violated invariant.
    pub fn validate(&self) -> Result<()> {
        let u = self.usable;
        let max = u.max();
        if u.origin().x() < -EPS
            || u.origin().y() < -EPS
            || max.x() > self.sheet.width() + EPS
            || max.y() > self.sheet.height() + EPS
            || u.size().width() <= 0.0
            || u.size().height() <= 0.0
            || self.sheet.width() <= 0.0
            || self.sheet.height() <= 0.0
            || self.elements.len() > MAX_ELEMENTS
        {
            return Err(invalid("folha ou área útil da moldura inválida"));
        }
        self.zones.columns.validate(u.size().width())?;
        self.zones.rows.validate(u.size().height())?;
        for e in &self.elements {
            e.validate()?;
        }
        Ok(())
    }
    /// Column and row at the sheet point, or `None` outside the division.
    pub fn zone_at(&self, point: Point2D) -> Option<Zone> {
        let u = self.usable;
        let column = self.zones.columns.index_at(point.x() - u.origin().x());
        let row = self.zones.rows.index_at(u.max().y() - point.y());
        let inside_y = point.y() >= u.origin().y() - EPS && point.y() <= u.max().y() + EPS;
        let inside_x = point.x() >= u.origin().x() - EPS && point.x() <= u.max().x() + EPS;
        let column = column.filter(|_| inside_y);
        let row = row.filter(|_| inside_x);
        if column.is_none() && row.is_none() {
            return None;
        }
        Some(Zone {
            column: column.map(|i| self.zones.columns.style.label(i)),
            row: row.map(|i| self.zones.rows.style.label(i)),
        })
    }
}

/// Labels of the located zone; either axis may have no division.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Zone {
    /// Column-zone label, absent when the axis is undivided.
    pub column: Option<String>,
    /// Row-zone label, absent when the axis is undivided.
    pub row: Option<String>,
}

/// Cross-reference text format for frames with zones.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferenceFormat {
    /// `/page.column`, such as `/3.4`.
    #[default]
    PageColumn,
    /// `/page.columnRow`, such as `/3.4B`.
    PageColumnRow,
}

/// ISO 216 sheet sizes in millimetres, landscape or portrait.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaperFormat {
    /// ISO 216 A0 sheet size.
    A0,
    /// ISO 216 A1 sheet size.
    A1,
    /// ISO 216 A2 sheet size.
    A2,
    /// ISO 216 A3 sheet size.
    A3,
    /// ISO 216 A4 sheet size.
    A4,
}
impl PaperFormat {
    /// All supported variants in display order.
    pub const ALL: [Self; 5] = [Self::A0, Self::A1, Self::A2, Self::A3, Self::A4];
    /// Return the pt-BR display label.
    pub fn label(self) -> &'static str {
        match self {
            Self::A0 => "A0",
            Self::A1 => "A1",
            Self::A2 => "A2",
            Self::A3 => "A3",
            Self::A4 => "A4",
        }
    }
    /// Return dimensions in millimetres.
    pub fn size(self, landscape: bool) -> Size2D {
        let (w, h) = match self {
            Self::A0 => (1189.0, 841.0),
            Self::A1 => (841.0, 594.0),
            Self::A2 => (594.0, 420.0),
            Self::A3 => (420.0, 297.0),
            Self::A4 => (297.0, 210.0),
        };
        let (w, h) = if landscape { (w, h) } else { (h, w) };
        Size2D::new(w, h).expect("static paper size")
    }
    /// Format and orientation matching the size within 0.5 mm.
    pub fn matching(size: Size2D) -> Option<(Self, bool)> {
        Self::ALL.into_iter().find_map(|f| {
            [true, false].into_iter().find_map(|landscape| {
                let s = f.size(landscape);
                ((s.width() - size.width()).abs() < 0.5 && (s.height() - size.height()).abs() < 0.5)
                    .then_some((f, landscape))
            })
        })
    }
}

/// Display value of special text; `index` selects `key.n`.
pub(crate) fn indexed_key(key: &Text, index: Option<u32>) -> String {
    match index {
        Some(n) => format!("{}.{n}", key.as_str()),
        None => key.as_str().to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn letter_labels_continue_after_z() {
        let s = ZoneLabelStyle::Letters { start: 0 };
        assert_eq!(s.label(0), "A");
        assert_eq!(s.label(25), "Z");
        assert_eq!(s.label(26), "AA");
        assert_eq!(s.label(27), "AB");
        assert_eq!(ZoneLabelStyle::Numbers { start: 0 }.label(3), "3");
    }
}

mod editing;
mod factory;
pub use editing::FrameDraft;
