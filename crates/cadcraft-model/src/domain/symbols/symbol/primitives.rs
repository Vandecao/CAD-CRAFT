// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Symbol primitive bounds and transforms; start with SymbolPrimitive::bounds.
use super::*;

fn rotated_box(origin: Point2D, w: f64, h: f64, align: TextAlign, degrees: f64) -> Result<Rect2D> {
    let (fx, fy) = align.fractions();
    let (s, c) = degrees.to_radians().sin_cos();
    let corners = [(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)].map(|(x, y)| {
        let (lx, ly) = (x - fx * w, y - fy * h);
        origin.translated(lx * c - ly * s, lx * s + ly * c)
    });
    let mut points = Vec::with_capacity(4);
    for p in corners {
        points.push(p?);
    }
    bounds_points(points)
}
pub(super) fn bounds_points(points: impl IntoIterator<Item = Point2D>) -> Result<Rect2D> {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for p in points {
        min_x = min_x.min(p.x());
        min_y = min_y.min(p.y());
        max_x = max_x.max(p.x());
        max_y = max_y.max(p.y());
    }
    Rect2D::new(
        Point2D::new(min_x, min_y)?,
        Size2D::new(max_x - min_x, max_y - min_y)?,
    )
}
fn circle_point(center: Point2D, radius: f64, degrees: f64) -> Result<Point2D> {
    let (s, c) = match canonical_degrees(degrees) {
        0. => (0., 1.),
        90. => (1., 0.),
        180. => (0., -1.),
        270. => (-1., 0.),
        a => a.to_radians().sin_cos(),
    };
    center.translated(c * radius, s * radius)
}
impl SymbolPrimitive {
    pub fn bounds(&self) -> Result<Rect2D> {
        match self {
            Self::Line { start, end } => {
                if start == end {
                    return Err(invalid());
                }
                bounds_points([*start, *end])
            }
            Self::Polyline { points, closed } => {
                if points.len() < if *closed { 3 } else { 2 }
                    || points.windows(2).any(|p| p[0] == p[1])
                {
                    return Err(invalid());
                }
                bounds_points(points.iter().copied())
            }
            Self::Rectangle { rect } => {
                if rect.size().width() <= 0. || rect.size().height() <= 0. {
                    return Err(invalid());
                }
                Ok(*rect)
            }
            Self::Circle { center, radius } => {
                if !radius.is_finite() || *radius <= 0. {
                    return Err(invalid());
                }
                bounds_points([
                    center.translated(-radius, -radius)?,
                    center.translated(*radius, *radius)?,
                ])
            }
            Self::Arc {
                center,
                radius,
                start_degrees,
                sweep_degrees,
            } => {
                if !radius.is_finite()
                    || *radius <= 0.
                    || !start_degrees.is_finite()
                    || !sweep_degrees.is_finite()
                    || *sweep_degrees == 0.
                    || sweep_degrees.abs() > 360.
                {
                    return Err(invalid());
                }
                let start = canonical_degrees(*start_degrees);
                let end = start + sweep_degrees;
                let mut points = vec![
                    circle_point(*center, *radius, start)?,
                    circle_point(*center, *radius, end)?,
                ];
                for cardinal in [0., 90., 180., 270.] {
                    let delta = if *sweep_degrees > 0. {
                        canonical_degrees(cardinal - start)
                    } else {
                        canonical_degrees(start - cardinal)
                    };
                    if delta <= sweep_degrees.abs() {
                        points.push(circle_point(*center, *radius, cardinal)?);
                    }
                }
                bounds_points(points)
            }
            Self::Text {
                origin,
                text,
                height,
            } => {
                nonempty(text)?;
                if !height.is_finite() || *height <= 0. {
                    return Err(invalid());
                }
                Rect2D::new(
                    *origin,
                    Size2D::new(*height * 0.6 * text.chars().count() as f64, *height)?,
                )
            }
            Self::Anchor { position } => Rect2D::new(*position, Size2D::new(0., 0.)?),
            Self::Ellipse {
                center,
                radius_x,
                radius_y,
                rotation_degrees,
            } => {
                if !radius_x.is_finite()
                    || !radius_y.is_finite()
                    || *radius_x <= 0.
                    || *radius_y <= 0.
                    || !rotation_degrees.is_finite()
                {
                    return Err(invalid());
                }
                let (s, c) = rotation_degrees.to_radians().sin_cos();
                let hx = ((radius_x * c).powi(2) + (radius_y * s).powi(2)).sqrt();
                let hy = ((radius_x * s).powi(2) + (radius_y * c).powi(2)).sqrt();
                bounds_points([center.translated(-hx, -hy)?, center.translated(hx, hy)?])
            }
            Self::DynamicText {
                origin,
                field,
                height,
                align,
                rotation_degrees,
            } => {
                if let SymbolTextField::PortLabel(v) | SymbolTextField::DeviceProperty(v) = field {
                    nonempty(v)?;
                }
                if !height.is_finite() || *height <= 0. || !rotation_degrees.is_finite() {
                    return Err(invalid());
                }
                rotated_box(
                    *origin,
                    height * 0.6 * DYNAMIC_TEXT_CHARS,
                    *height,
                    *align,
                    *rotation_degrees,
                )
            }
        }
    }
    /// Forma fechada (aceita preenchimento).
    pub fn is_closed(&self) -> bool {
        matches!(
            self,
            Self::Rectangle { .. }
                | Self::Circle { .. }
                | Self::Ellipse { .. }
                | Self::Polyline { closed: true, .. }
        )
    }
    /// Nome curto do tipo (editor e relatórios).
    pub fn kind_label(&self) -> &'static str {
        match self {
            Self::Line { .. } => "Linha",
            Self::Polyline { closed: true, .. } => "Polígono",
            Self::Polyline { .. } => "Polilinha",
            Self::Rectangle { .. } => "Retângulo",
            Self::Circle { .. } => "Círculo",
            Self::Arc { .. } => "Arco",
            Self::Text { .. } => "Texto",
            Self::Anchor { .. } => "Âncora",
            Self::Ellipse { .. } => "Elipse",
            Self::DynamicText { .. } => "Texto dinâmico",
        }
    }
    /// Mesma primitiva deslocada (mm).
    pub fn translated(&self, dx: f64, dy: f64) -> Result<Self> {
        let m = |p: &Point2D| p.translated(dx, dy);
        Ok(match self.clone() {
            Self::Line { start, end } => Self::Line {
                start: m(&start)?,
                end: m(&end)?,
            },
            Self::Polyline { points, closed } => Self::Polyline {
                points: points.iter().map(m).collect::<Result<_>>()?,
                closed,
            },
            Self::Rectangle { rect } => Self::Rectangle {
                rect: Rect2D::new(m(&rect.origin())?, rect.size())?,
            },
            Self::Circle { center, radius } => Self::Circle {
                center: m(&center)?,
                radius,
            },
            Self::Arc {
                center,
                radius,
                start_degrees,
                sweep_degrees,
            } => Self::Arc {
                center: m(&center)?,
                radius,
                start_degrees,
                sweep_degrees,
            },
            Self::Text {
                origin,
                text,
                height,
            } => Self::Text {
                origin: m(&origin)?,
                text,
                height,
            },
            Self::Anchor { position } => Self::Anchor {
                position: m(&position)?,
            },
            Self::Ellipse {
                center,
                radius_x,
                radius_y,
                rotation_degrees,
            } => Self::Ellipse {
                center: m(&center)?,
                radius_x,
                radius_y,
                rotation_degrees,
            },
            Self::DynamicText {
                origin,
                field,
                height,
                align,
                rotation_degrees,
            } => Self::DynamicText {
                origin: m(&origin)?,
                field,
                height,
                align,
                rotation_degrees,
            },
        })
    }
}
