// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 13: molduras (folha, área útil, zonas, carimbo e textos especiais).
//!
//! Coordenadas locais da folha em milímetros, origem no canto inferior
//! esquerdo, X à direita e Y para cima. Zonas partem da área útil: colunas
//! da esquerda para a direita, linhas de cima para baixo (convenção dos
//! esquemas elétricos). Cada zona é um intervalo semiaberto `[início, fim)`;
//! a última também inclui o fim.
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

/// Rótulos de zona: números a partir de `start` ou letras (A…Z, AA, AB…).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneLabelStyle {
    Numbers { start: u32 },
    Letters { start: u32 },
}
impl ZoneLabelStyle {
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

/// Divisão de um eixo da área útil. `sizes` vazio = eixo sem zonas.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZoneAxis {
    pub sizes: Vec<f64>,
    pub style: ZoneLabelStyle,
    /// Colunas: rótulos acima; linhas: rótulos à esquerda.
    pub labels_start: bool,
    /// Colunas: rótulos abaixo; linhas: rótulos à direita.
    pub labels_end: bool,
}
impl ZoneAxis {
    pub fn none() -> Self {
        Self {
            sizes: vec![],
            style: ZoneLabelStyle::Numbers { start: 1 },
            labels_start: false,
            labels_end: false,
        }
    }
    pub fn uniform(count: usize, size: f64, style: ZoneLabelStyle) -> Self {
        Self {
            sizes: vec![size; count],
            style,
            labels_start: true,
            labels_end: false,
        }
    }
    pub fn total(&self) -> f64 {
        self.sizes.iter().sum()
    }
    /// Índice da zona que contém `offset` (distância desde o início do eixo).
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
    /// Limites `[início, fim]` (desde o início do eixo) de cada zona.
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZoneLayout {
    pub columns: ZoneAxis,
    pub rows: ZoneAxis,
}

/// Ponto de alinhamento do texto em relação à origem.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlign {
    #[default]
    BottomLeft,
    BottomCenter,
    BottomRight,
    MiddleLeft,
    Center,
    MiddleRight,
    TopLeft,
    TopCenter,
    TopRight,
}
impl TextAlign {
    /// Fração (horizontal, vertical a partir de baixo) da caixa do texto.
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

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameTextStyle {
    pub height: f64,
    pub align: TextAlign,
    pub rotation_degrees: f64,
    /// Etapa 14: marca d'água — desenhada antes do conteúdo, em cor clara.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub watermark: bool,
}
impl FrameTextStyle {
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

/// Texto fixo: valor padrão (pt-BR) e variantes por idioma (`en-US` → texto).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalizedText {
    pub default: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub translations: BTreeMap<String, String>,
}
impl LocalizedText {
    pub fn new(default: impl Into<String>) -> Self {
        Self {
            default: default.into(),
            translations: BTreeMap::new(),
        }
    }
    pub fn get(&self, language: Option<&str>) -> &str {
        language
            .and_then(|l| self.translations.get(l))
            .map(String::as_str)
            .unwrap_or(&self.default)
    }
}

/// Valor resolvido a partir do projeto, da página ou da moldura.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecialField {
    ProjectName,
    ProjectDescription,
    ProjectIdentification,
    /// Metadado do projeto pela chave (ex.: "Autor", "Cliente").
    ProjectProperty(Text),
    DocumentName,
    PageNumber,
    PageName,
    PageFunctionalPath,
    PageLocationPath,
    /// Posição da página (1…n) no documento.
    PagePosition,
    PageCountInDocument,
    PageCountInProject,
    PreviousPageNumber,
    NextPageNumber,
    /// Metadado da página pela chave (ex.: "Data", "Desenhista").
    PageProperty(Text),
    FrameName,
}
impl SpecialField {
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum FrameElement {
    Line {
        start: Point2D,
        end: Point2D,
        width: f64,
    },
    Rectangle {
        rect: Rect2D,
        width: f64,
    },
    Circle {
        center: Point2D,
        radius: f64,
        width: f64,
    },
    Arc {
        center: Point2D,
        radius: f64,
        start_degrees: f64,
        sweep_degrees: f64,
        width: f64,
    },
    Text {
        origin: Point2D,
        text: LocalizedText,
        style: FrameTextStyle,
    },
    /// Texto especial; `index` seleciona `chave.n` em propriedades indexadas
    /// (ex.: linhas do quadro de revisões).
    Special {
        origin: Point2D,
        field: SpecialField,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        index: Option<u32>,
        style: FrameTextStyle,
    },
    /// Etapa 14: imagem do projeto ajustada ao retângulo (mm).
    Image {
        rect: Rect2D,
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
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
    /// Nova versão do mesmo modelo (mesmo ID), validada.
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
    pub fn id(&self) -> FrameTemplateId {
        self.id
    }
    pub fn name(&self) -> &Text {
        &self.name
    }
    pub fn version(&self) -> u32 {
        self.version
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn sheet(&self) -> Size2D {
        self.sheet
    }
    pub fn usable(&self) -> Rect2D {
        self.usable
    }
    pub fn zones(&self) -> &ZoneLayout {
        &self.zones
    }
    pub fn elements(&self) -> &[FrameElement] {
        &self.elements
    }
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
    /// Zona (coluna, linha) do ponto da folha; `None` fora da divisão.
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

/// Rótulos da zona encontrada (cada eixo pode não ter divisão).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Zone {
    pub column: Option<String>,
    pub row: Option<String>,
}

/// Formato textual das referências cruzadas quando há moldura com zonas.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferenceFormat {
    /// `/página.coluna`, ex.: `/3.4`.
    #[default]
    PageColumn,
    /// `/página.colunaLinha`, ex.: `/3.4B`.
    PageColumnRow,
}

/// Formatos de folha ISO 216 (mm), paisagem ou retrato.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaperFormat {
    A0,
    A1,
    A2,
    A3,
    A4,
}
impl PaperFormat {
    pub const ALL: [Self; 5] = [Self::A0, Self::A1, Self::A2, Self::A3, Self::A4];
    pub fn label(self) -> &'static str {
        match self {
            Self::A0 => "A0",
            Self::A1 => "A1",
            Self::A2 => "A2",
            Self::A3 => "A3",
            Self::A4 => "A4",
        }
    }
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
    /// Formato e orientação que coincidem com o tamanho (tolerância 0,5 mm).
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

/// Valor de exibição do texto especial; `index` usa a chave `chave.n`.
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
