// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Finite millimetre geometry values; start with Point2D, Size2D and Rect2D.
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};

/// Coordenada cartesiana em milímetros; origem inferior esquerda, X direita/Y cima.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "PointData")]
pub struct Point2D {
    x: f64,
    y: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PointData {
    x: f64,
    y: f64,
}
impl TryFrom<PointData> for Point2D {
    type Error = DomainError;
    fn try_from(p: PointData) -> Result<Self> {
        Self::new(p.x, p.y)
    }
}
impl Point2D {
    pub fn new(x: f64, y: f64) -> Result<Self> {
        if !x.is_finite() || !y.is_finite() {
            return Err(DomainError::InvalidNumber("coordenada"));
        }
        Ok(Self { x, y })
    }
    pub fn x(self) -> f64 {
        self.x
    }
    pub fn y(self) -> f64 {
        self.y
    }
    pub fn translated(self, dx: f64, dy: f64) -> Result<Self> {
        Self::new(self.x + dx, self.y + dy)
    }
    pub fn distance_to(self, other: Self) -> Result<f64> {
        let distance = (self.x - other.x).hypot(self.y - other.y);
        if !distance.is_finite() {
            return Err(DomainError::InvalidNumber("distância"));
        }
        Ok(distance)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "SizeData")]
pub struct Size2D {
    width: f64,
    height: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SizeData {
    width: f64,
    height: f64,
}
impl TryFrom<SizeData> for Size2D {
    type Error = DomainError;
    fn try_from(s: SizeData) -> Result<Self> {
        Self::new(s.width, s.height)
    }
}
impl Size2D {
    pub fn new(width: f64, height: f64) -> Result<Self> {
        if !width.is_finite() || !height.is_finite() || width < 0.0 || height < 0.0 {
            return Err(DomainError::InvalidNumber("dimensão"));
        }
        Ok(Self { width, height })
    }
    pub fn width(self) -> f64 {
        self.width
    }
    pub fn height(self) -> f64 {
        self.height
    }
}

/// Retângulo fechado; tamanhos zero são permitidos. Limites precisam ser finitos.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RectData")]
pub struct Rect2D {
    origin: Point2D,
    size: Size2D,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RectData {
    origin: Point2D,
    size: Size2D,
}
impl TryFrom<RectData> for Rect2D {
    type Error = DomainError;
    fn try_from(r: RectData) -> Result<Self> {
        Self::new(r.origin, r.size)
    }
}
impl Rect2D {
    pub fn new(origin: Point2D, size: Size2D) -> Result<Self> {
        origin.translated(size.width, size.height)?;
        Ok(Self { origin, size })
    }
    pub fn origin(self) -> Point2D {
        self.origin
    }
    pub fn size(self) -> Size2D {
        self.size
    }
    pub fn max(self) -> Point2D {
        Point2D {
            x: self.origin.x + self.size.width,
            y: self.origin.y + self.size.height,
        }
    }
    pub fn contains(self, point: Point2D) -> bool {
        point.x >= self.origin.x
            && point.y >= self.origin.y
            && point.x <= self.max().x
            && point.y <= self.max().y
    }
    pub fn intersects(self, other: Self) -> bool {
        self.intersection(other).is_some()
    }
    pub fn intersection(self, other: Self) -> Option<Self> {
        let x = self.origin.x.max(other.origin.x);
        let y = self.origin.y.max(other.origin.y);
        let max_x = self.max().x.min(other.max().x);
        let max_y = self.max().y.min(other.max().y);
        if max_x < x || max_y < y {
            return None;
        }
        // A interseção está contida nos retângulos finitos já validados.
        Some(Self {
            origin: Point2D { x, y },
            size: Size2D {
                width: max_x - x,
                height: max_y - y,
            },
        })
    }
}
