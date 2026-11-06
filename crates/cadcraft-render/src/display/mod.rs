// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Exportable page display lists; start with DisplayItem, DisplayList and
//! PageDisplayList, then follow the builder for frames, symbols and references.
//!
//! Coordinates are millimetres, origin bottom left, Y up. Shared sheet content
//! feeds PDF and page DXF without GUI dependencies. Editing aids such as the grid,
//! port handles, snap hints and selection are excluded.
use crate::font;
use cadcraft_model::domain::*;
use cadcraft_model::error::Result;

/// Espessura (mm) do traço de símbolos sem espessura própria.
pub const SYMBOL_STROKE: f64 = 0.25;
/// Espessura (mm) dos fios.
pub const ROUTE_STROKE: f64 = 0.35;
/// Raio (mm) do ponto de junção.
pub const JUNCTION_RADIUS: f64 = 0.6;
/// Altura (mm) dos rótulos de zona e de textos auxiliares da folha.
pub const LABEL_HEIGHT: f64 = 2.5;
/// Altura da linha de texto em relação ao corpo (caixa de alinhamento).
pub const LINE_HEIGHT: f64 = 1.2;
/// Linha de base acima do pé da caixa, em relação ao corpo.
pub const BASELINE: f64 = 0.25;
/// Erro máximo (mm) da tesselagem de curvas.
const CURVE_TOLERANCE: f64 = 0.01;

/// Cor lógica; quem escreve escolhe o valor (colorido ou preto e branco).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayColor {
    /// Desenho principal.
    Ink,
    /// Referências cruzadas, espelho e interrupções.
    Reference,
    /// Marca d'água da moldura.
    Watermark,
    /// Guias claras (margem sem moldura, caixa sem geometria, imagem ausente).
    Guide,
}

/// Categoria do item (camada no DXF; o PDF a ignora).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DisplayLayer {
    Frame,
    Symbols,
    Wires,
    Texts,
    References,
    Junctions,
    Guides,
}

impl DisplayLayer {
    pub const ALL: [Self; 7] = [
        Self::Frame,
        Self::Symbols,
        Self::Wires,
        Self::Texts,
        Self::References,
        Self::Junctions,
        Self::Guides,
    ];
    /// Nome da camada (sem acentos, aceito por qualquer leitor DXF).
    pub fn name(self) -> &'static str {
        match self {
            Self::Frame => "Moldura",
            Self::Symbols => "Simbolos",
            Self::Wires => "Fios",
            Self::Texts => "Textos",
            Self::References => "Referencias",
            Self::Junctions => "Juncoes",
            Self::Guides => "Guias",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DisplayStroke {
    pub width: f64,
    /// Traço/vão em mm; vazio = contínuo.
    pub dash: Vec<f64>,
    pub color: DisplayColor,
}

/// Texto numa caixa local em mm. `matrix` leva a caixa para a página:
/// `x' = a·x + c·y + e`, `y' = b·x + d·y + f` (`[a, b, c, d, e, f]`).
/// O corpo da fonte é `size`; a linha de base fica em `y = baseline`.
/// Com `width`, o texto é esticado/comprimido até essa largura.
#[derive(Clone, Debug, PartialEq)]
pub struct DisplayText {
    pub text: String,
    pub matrix: [f64; 6],
    pub size: f64,
    pub baseline: f64,
    pub width: Option<f64>,
    pub color: DisplayColor,
    pub layer: DisplayLayer,
}

impl DisplayText {
    /// Largura natural (Helvetica) na unidade da caixa.
    pub fn natural_width(&self) -> f64 {
        font::text_width(&self.text, self.size)
    }
    /// Ponto da página correspondente ao ponto `(x, y)` da caixa.
    pub fn map(&self, x: f64, y: f64) -> (f64, f64) {
        let [a, b, c, d, e, f] = self.matrix;
        (a * x + c * y + e, b * x + d * y + f)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DisplayItem {
    Path {
        points: Vec<Point2D>,
        closed: bool,
        stroke: Option<DisplayStroke>,
        fill: Option<DisplayColor>,
        layer: DisplayLayer,
    },
    Text(DisplayText),
    Image {
        asset: AssetId,
        rect: Rect2D,
        layer: DisplayLayer,
    },
    /// Área clicável que leva à ocorrência `target`.
    Link {
        rect: Rect2D,
        target: RepresentationId,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct DisplayList {
    pub page: PageId,
    pub width: f64,
    pub height: f64,
    pub items: Vec<DisplayItem>,
}

impl DisplayList {
    pub fn texts(&self) -> impl Iterator<Item = &DisplayText> {
        self.items.iter().filter_map(|i| match i {
            DisplayItem::Text(t) => Some(t),
            _ => None,
        })
    }
    pub fn links(&self) -> impl Iterator<Item = (Rect2D, RepresentationId)> + '_ {
        self.items.iter().filter_map(|i| match i {
            DisplayItem::Link { rect, target } => Some((*rect, *target)),
            _ => None,
        })
    }
}

struct Builder<'a> {
    project: &'a Project,
    items: Vec<DisplayItem>,
    /// Camada dos itens acrescentados a seguir.
    layer: DisplayLayer,
}

mod connection;
mod frame;
mod geometry;
mod page;
mod report;
mod symbol;
mod text;

pub use connection::{WIRE_NUMBER_GAP, wire_number_placement};
pub use frame::frame_zone_labels;
use geometry::{arc_points, ellipse_points, rect_corners, world_curve};
pub use page::PageDisplayList;
