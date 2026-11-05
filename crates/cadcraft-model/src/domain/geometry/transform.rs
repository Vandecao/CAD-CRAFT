// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Validated occurrence transforms; start with Transform2D.
use super::{Point2D, Rect2D, Size2D};
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};

/// `rem_euclid` pode arredondar negativos próximos de zero para o divisor.
/// Toda orientação documental deve compartilhar a mesma forma canônica.
pub(in crate::domain) fn canonical_degrees(degrees: f64) -> f64 {
    let angle = degrees.rem_euclid(360.0);
    if angle == 360.0 { 0.0 } else { angle }
}

/// Coordenadas locais em mm: espelhar componentes X/Y, girar anti-horário,
/// transladar. Sem escala, pixels, inversão de Y ou restrição à folha.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "TransformData")]
pub struct Transform2D {
    translation: Point2D,
    rotation_degrees: f64,
    mirror_x: bool,
    mirror_y: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TransformData {
    translation: Point2D,
    rotation_degrees: f64,
    mirror_x: bool,
    mirror_y: bool,
}

impl TryFrom<TransformData> for Transform2D {
    type Error = DomainError;
    fn try_from(t: TransformData) -> Result<Self> {
        Self::new(t.translation, t.rotation_degrees, t.mirror_x, t.mirror_y)
    }
}

impl Transform2D {
    pub fn new(
        translation: Point2D,
        rotation_degrees: f64,
        mirror_x: bool,
        mirror_y: bool,
    ) -> Result<Self> {
        if !rotation_degrees.is_finite() {
            return Err(DomainError::InvalidNumber("rotação"));
        }
        Ok(Self {
            translation,
            rotation_degrees: canonical_degrees(rotation_degrees),
            mirror_x,
            mirror_y,
        })
    }
    pub fn translation(self) -> Point2D {
        self.translation
    }
    pub fn rotation_degrees(self) -> f64 {
        self.rotation_degrees
    }
    pub fn mirror_x(self) -> bool {
        self.mirror_x
    }
    pub fn mirror_y(self) -> bool {
        self.mirror_y
    }

    fn sin_cos(self) -> (f64, f64) {
        // Ângulos ortogonais exatos evitam resíduos nos limites e no hit testing.
        match self.rotation_degrees {
            0.0 => (0.0, 1.0),
            90.0 => (1.0, 0.0),
            180.0 => (0.0, -1.0),
            270.0 => (-1.0, 0.0),
            _ => self.rotation_degrees.to_radians().sin_cos(),
        }
    }
    pub fn apply(self, point: Point2D) -> Result<Point2D> {
        let x = if self.mirror_x { -point.x() } else { point.x() };
        let y = if self.mirror_y { -point.y() } else { point.y() };
        let (sin, cos) = self.sin_cos();
        Point2D::new(x * cos - y * sin, x * sin + y * cos)?
            .translated(self.translation.x(), self.translation.y())
    }
    pub fn inverse_apply(self, point: Point2D) -> Result<Point2D> {
        let p = point.translated(-self.translation.x(), -self.translation.y())?;
        let (sin, cos) = self.sin_cos();
        let x = p.x() * cos + p.y() * sin;
        let y = -p.x() * sin + p.y() * cos;
        Point2D::new(
            if self.mirror_x { -x } else { x },
            if self.mirror_y { -y } else { y },
        )
    }
    /// Etapa 19: composição `outer ∘ self` (aplica `self` e depois `outer`).
    /// Um espelho em um só eixo inverte o sentido da rotação interna.
    pub fn then(self, outer: Transform2D) -> Result<Self> {
        let reflects = outer.mirror_x != outer.mirror_y;
        let inner = if reflects {
            -self.rotation_degrees
        } else {
            self.rotation_degrees
        };
        let (mirror_x, mirror_y) = (
            self.mirror_x != outer.mirror_x,
            self.mirror_y != outer.mirror_y,
        );
        // Espelhar os dois eixos equivale a girar 180°: forma canônica sem espelho.
        let (rotation, mirror_x, mirror_y) = if mirror_x && mirror_y {
            (outer.rotation_degrees + inner + 180.0, false, false)
        } else {
            (outer.rotation_degrees + inner, mirror_x, mirror_y)
        };
        Self::new(outer.apply(self.translation)?, rotation, mirror_x, mirror_y)
    }
    /// Gira 90° ou 270° (troca os eixos horizontal e vertical).
    pub fn swaps_axes(self) -> bool {
        self.rotation_degrees == 90.0 || self.rotation_degrees == 270.0
    }
    pub fn transform_bounds(self, bounds: Rect2D) -> Result<Rect2D> {
        let a = bounds.origin();
        let b = bounds.max();
        let points = [
            self.apply(a)?,
            self.apply(b)?,
            self.apply(Point2D::new(a.x(), b.y())?)?,
            self.apply(Point2D::new(b.x(), a.y())?)?,
        ];
        let min_x = points.iter().map(|p| p.x()).fold(f64::INFINITY, f64::min);
        let min_y = points.iter().map(|p| p.y()).fold(f64::INFINITY, f64::min);
        let max_x = points
            .iter()
            .map(|p| p.x())
            .fold(f64::NEG_INFINITY, f64::max);
        let max_y = points
            .iter()
            .map(|p| p.y())
            .fold(f64::NEG_INFINITY, f64::max);
        Rect2D::new(
            Point2D::new(min_x, min_y)?,
            Size2D::new(max_x - min_x, max_y - min_y)?,
        )
    }
}
