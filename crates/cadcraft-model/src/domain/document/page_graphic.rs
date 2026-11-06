// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Free page text and geometry; start with PageGraphic.
use super::{Point2D, PrimitiveStyle, Rect2D, Size2D, SymbolPrimitive};
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};

/// Horizontal alignment of free page text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlignment {
    /// Left text alignment.
    #[default]
    Left,
    /// Center text alignment.
    Center,
    /// Right text alignment.
    Right,
}

/// Document geometry independent of electrical entities.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "PageGraphicData")]
pub struct PageGraphic {
    primitive: SymbolPrimitive,
    stroke_mm: f64,
    alignment: TextAlignment,
    /// Stroke and fill; stroke width remains in `stroke_mm`.
    #[serde(default, skip_serializing_if = "PrimitiveStyle::is_default")]
    style: PrimitiveStyle,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PageGraphicData {
    primitive: SymbolPrimitive,
    stroke_mm: f64,
    alignment: TextAlignment,
    #[serde(default)]
    style: PrimitiveStyle,
}

impl TryFrom<PageGraphicData> for PageGraphic {
    type Error = DomainError;
    fn try_from(value: PageGraphicData) -> Result<Self> {
        Self::new(value.primitive, value.stroke_mm, value.alignment)?.with_style(value.style)
    }
}

impl PageGraphic {
    /// Create free page content, validating its primitive, stroke and alignment.
    pub fn new(
        primitive: SymbolPrimitive,
        stroke_mm: f64,
        alignment: TextAlignment,
    ) -> Result<Self> {
        if !stroke_mm.is_finite() || stroke_mm <= 0.0 {
            return Err(DomainError::InvalidNumber("espessura do traço"));
        }
        if !matches!(primitive, SymbolPrimitive::Text { .. }) && alignment != TextAlignment::Left {
            return Err(DomainError::InvalidRelation(
                "alinhamento exige texto livre",
            ));
        }
        if !matches!(
            primitive,
            SymbolPrimitive::Text { .. }
                | SymbolPrimitive::Line { .. }
                | SymbolPrimitive::Rectangle { .. }
                | SymbolPrimitive::Circle { .. }
                | SymbolPrimitive::Arc { .. }
                | SymbolPrimitive::Ellipse { .. }
        ) {
            return Err(DomainError::InvalidRelation(
                "primitiva de conteúdo livre não admitida",
            ));
        }
        let value = Self {
            primitive,
            stroke_mm,
            alignment,
            style: PrimitiveStyle::default(),
        };
        value.bounds()?;
        Ok(value)
    }
    /// Return the primitive.
    pub fn primitive(&self) -> &SymbolPrimitive {
        &self.primitive
    }
    /// Return the stroke width in millimetres.
    pub fn stroke_mm(&self) -> f64 {
        self.stroke_mm
    }
    /// Return the alignment.
    pub fn alignment(&self) -> TextAlignment {
        self.alignment
    }
    /// Return the style.
    pub fn style(&self) -> PrimitiveStyle {
        self.style
    }
    /// Stroke and fill; stroke width remains in `stroke_mm`.
    pub fn with_style(mut self, style: PrimitiveStyle) -> Result<Self> {
        let style = PrimitiveStyle {
            width_mm: None,
            ..style
        };
        style.check(&self.primitive)?;
        self.style = style;
        Ok(self)
    }
    /// Resolve text alignment into geometry suitable for drawing.
    pub fn drawable(&self) -> Result<SymbolPrimitive> {
        match &self.primitive {
            SymbolPrimitive::Text {
                origin,
                text,
                height,
            } => {
                let width = height * 0.6 * text.chars().count() as f64;
                let offset = match self.alignment {
                    TextAlignment::Left => 0.0,
                    TextAlignment::Center => width / 2.0,
                    TextAlignment::Right => width,
                };
                Ok(SymbolPrimitive::Text {
                    origin: origin.translated(-offset, 0.0)?,
                    text: text.clone(),
                    height: *height,
                })
            }
            other => Ok(other.clone()),
        }
    }
    /// Return axis-aligned geometry bounds in millimetres.
    pub fn bounds(&self) -> Result<Rect2D> {
        self.drawable()?.bounds()
    }
    /// Create geometry from drag endpoints, validating dimensions and stroke settings.
    pub fn from_drag(
        kind: GraphicKind,
        start: Point2D,
        end: Point2D,
        stroke_mm: f64,
    ) -> Result<Self> {
        let min = Point2D::new(start.x().min(end.x()), start.y().min(end.y()))?;
        let size = Size2D::new((start.x() - end.x()).abs(), (start.y() - end.y()).abs())?;
        let primitive = match kind {
            GraphicKind::Line => SymbolPrimitive::Line { start, end },
            GraphicKind::Rectangle => SymbolPrimitive::Rectangle {
                rect: Rect2D::new(min, size)?,
            },
            GraphicKind::Circle => SymbolPrimitive::Circle {
                center: start,
                radius: start.distance_to(end)?,
            },
            GraphicKind::Ellipse => SymbolPrimitive::Ellipse {
                center: Point2D::new((start.x() + end.x()) / 2.0, (start.y() + end.y()) / 2.0)?,
                radius_x: size.width() / 2.0,
                radius_y: size.height() / 2.0,
                rotation_degrees: 0.0,
            },
            GraphicKind::Arc => SymbolPrimitive::Arc {
                center: start,
                radius: start.distance_to(end)?,
                start_degrees: 0.0,
                sweep_degrees: 90.0,
            },
        };
        Self::new(primitive, stroke_mm, TextAlignment::Left)
    }
}

/// Free-geometry tool used to create a page graphic from a drag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphicKind {
    /// Straight line primitive.
    Line,
    /// Rectangular primitive.
    Rectangle,
    /// Circular primitive.
    Circle,
    /// Circular arc primitive.
    Arc,
    /// Ellipse inscribed in the dragged rectangle.
    Ellipse,
}
