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

/// Stroke width in mm for symbols without an explicit width.
pub const SYMBOL_STROKE: f64 = 0.25;
/// Wire stroke width in mm.
pub const ROUTE_STROKE: f64 = 0.35;
/// Junction dot radius in mm.
pub const JUNCTION_RADIUS: f64 = 0.6;
/// Height in mm of zone labels and auxiliary sheet texts.
pub const LABEL_HEIGHT: f64 = 2.5;
/// Text line height relative to font size, used for the alignment box.
pub const LINE_HEIGHT: f64 = 1.2;
/// Baseline offset above the box bottom relative to font size.
pub const BASELINE: f64 = 0.25;
/// Maximum curve tessellation error in mm.
const CURVE_TOLERANCE: f64 = 0.01;

/// Logical color; the writer chooses its color or monochrome value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayColor {
    /// Main drawing content.
    Ink,
    /// Cross-references, contact mirrors and interruptions.
    Reference,
    /// Frame watermark.
    Watermark,
    /// Light guides: unframed margin, geometry-free box or missing image.
    Guide,
}

/// Item category: DXF layer, ignored by PDF.
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
    /// Layer name without accents, accepted by DXF readers.
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
    /// Dash/gap lengths in mm; empty means continuous.
    pub dash: Vec<f64>,
    pub color: DisplayColor,
}

/// Text in a local box in mm. `matrix` maps the box onto the page:
/// `x' = a·x + c·y + e`, `y' = b·x + d·y + f` (`[a, b, c, d, e, f]`).
/// Font size is `size`; the baseline lies at `y = baseline`.
/// When `width` is set, stretch or compress the text to that width.
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
    /// Natural Helvetica width in the box's units.
    pub fn natural_width(&self) -> f64 {
        font::text_width(&self.text, self.size)
    }
    /// Page point corresponding to local box point `(x, y)`.
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
    /// Clickable area navigating to the `target` occurrence.
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
    /// Layer assigned to subsequent items.
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
