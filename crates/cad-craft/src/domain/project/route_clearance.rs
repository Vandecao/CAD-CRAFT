// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Local, deterministic orthogonal clearance around closed symbol bodies.
//! Stroke primitives have no filled interior; their endpoints remain usable.
use super::*;

const CLEARANCE_MM: f64 = 2.0;

#[derive(Clone)]
struct Obstacle {
    polygon: Vec<Point2D>,
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl Obstacle {
    fn new(polygon: Vec<Point2D>) -> Self {
        Self {
            min_x: polygon.iter().map(|p| p.x()).fold(f64::INFINITY, f64::min),
            min_y: polygon.iter().map(|p| p.y()).fold(f64::INFINITY, f64::min),
            max_x: polygon
                .iter()
                .map(|p| p.x())
                .fold(f64::NEG_INFINITY, f64::max),
            max_y: polygon
                .iter()
                .map(|p| p.y())
                .fold(f64::NEG_INFINITY, f64::max),
            polygon,
        }
    }

    fn contains(&self, p: Point2D) -> bool {
        let mut inside = false;
        for i in 0..self.polygon.len() {
            let a = self.polygon[i];
            let b = self.polygon[(i + 1) % self.polygon.len()];
            let cross = (b.x() - a.x()) * (p.y() - a.y()) - (b.y() - a.y()) * (p.x() - a.x());
            if cross == 0.0
                && p.x() >= a.x().min(b.x())
                && p.x() <= a.x().max(b.x())
                && p.y() >= a.y().min(b.y())
                && p.y() <= a.y().max(b.y())
            {
                return false;
            }
            if (a.y() > p.y()) != (b.y() > p.y())
                && p.x() < (b.x() - a.x()) * (p.y() - a.y()) / (b.y() - a.y()) + a.x()
            {
                inside = !inside;
            }
        }
        inside
    }

    fn crosses(&self, a: Point2D, b: Point2D) -> bool {
        if a.x().min(b.x()) >= self.max_x
            || a.x().max(b.x()) <= self.min_x
            || a.y().min(b.y()) >= self.max_y
            || a.y().max(b.y()) <= self.min_y
        {
            return false;
        }
        let horizontal = a.y() == b.y();
        let mut cuts = vec![0.0, 1.0];
        for i in 0..self.polygon.len() {
            let p = self.polygon[i];
            let q = self.polygon[(i + 1) % self.polygon.len()];
            let t = if horizontal && p.y() != q.y() {
                let u = (a.y() - p.y()) / (q.y() - p.y());
                if (0.0..=1.0).contains(&u) && a.x() != b.x() {
                    Some((p.x() + u * (q.x() - p.x()) - a.x()) / (b.x() - a.x()))
                } else {
                    None
                }
            } else if !horizontal && p.x() != q.x() {
                let u = (a.x() - p.x()) / (q.x() - p.x());
                if (0.0..=1.0).contains(&u) && a.y() != b.y() {
                    Some((p.y() + u * (q.y() - p.y()) - a.y()) / (b.y() - a.y()))
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(t) = t
                && (0.0..=1.0).contains(&t)
            {
                cuts.push(t);
            }
        }
        cuts.sort_by(f64::total_cmp);
        cuts.windows(2).any(|pair| {
            let t = (pair[0] + pair[1]) / 2.0;
            let p = if horizontal {
                Point2D::new(a.x() + (b.x() - a.x()) * t, a.y())
            } else {
                Point2D::new(a.x(), a.y() + (b.y() - a.y()) * t)
            };
            p.is_ok_and(|p| self.contains(p))
        })
    }
}

fn obstacles(def: &SymbolDefinition, t: Transform2D) -> Result<Vec<Obstacle>> {
    let mut result = Vec::new();
    for primitive in def.primitives() {
        let polygon = match primitive {
            SymbolPrimitive::Rectangle { rect } => {
                let a = rect.origin();
                let b = rect.max();
                Some(vec![
                    a,
                    Point2D::new(b.x(), a.y())?,
                    b,
                    Point2D::new(a.x(), b.y())?,
                ])
            }
            SymbolPrimitive::Polyline {
                points,
                closed: true,
            } => Some(points.clone()),
            SymbolPrimitive::Circle { center, radius } => Some(
                (0..128)
                    .map(|i| {
                        let angle = std::f64::consts::TAU * i as f64 / 128.0;
                        let (sin, cos) = angle.sin_cos();
                        center.translated(radius * cos, radius * sin)
                    })
                    .collect::<Result<Vec<_>>>()?,
            ),
            _ => None,
        };
        if let Some(polygon) = polygon {
            result.push(Obstacle::new(
                polygon
                    .into_iter()
                    .map(|p| t.apply(p))
                    .collect::<Result<Vec<_>>>()?,
            ));
        }
    }
    Ok(result)
}

fn intersects(points: &[Point2D], obstacle: &Obstacle) -> bool {
    points.windows(2).any(|p| obstacle.crosses(p[0], p[1]))
}

fn clean(mut points: Vec<Point2D>) -> Vec<Point2D> {
    points.dedup();
    let mut i = 1;
    while i + 1 < points.len() {
        let (a, b, c) = (points[i - 1], points[i], points[i + 1]);
        if (a.x() == b.x() && b.x() == c.x()) || (a.y() == b.y() && b.y() == c.y()) {
            points.remove(i);
        } else {
            i += 1;
        }
    }
    points
}

fn detour(from: Point2D, to: Point2D, degrees: f64, obstacle: &Obstacle) -> Result<Vec<Point2D>> {
    let mut candidates = Vec::new();
    for positive in [true, false] {
        let (escape, side_a, side_b) = match degrees {
            0.0 | 180.0 => {
                let x = if degrees == 0.0 {
                    obstacle.max_x.max(from.x()) + CLEARANCE_MM
                } else {
                    obstacle.min_x.min(from.x()) - CLEARANCE_MM
                };
                let y = if positive {
                    obstacle.max_y + CLEARANCE_MM
                } else {
                    obstacle.min_y - CLEARANCE_MM
                };
                (
                    Point2D::new(x, from.y())?,
                    Point2D::new(x, y)?,
                    Point2D::new(to.x(), y)?,
                )
            }
            90.0 | 270.0 => {
                let y = if degrees == 90.0 {
                    obstacle.max_y.max(from.y()) + CLEARANCE_MM
                } else {
                    obstacle.min_y.min(from.y()) - CLEARANCE_MM
                };
                let x = if positive {
                    obstacle.max_x + CLEARANCE_MM
                } else {
                    obstacle.min_x - CLEARANCE_MM
                };
                (
                    Point2D::new(from.x(), y)?,
                    Point2D::new(x, y)?,
                    Point2D::new(x, to.y())?,
                )
            }
            _ => return Err(drawing_error()),
        };
        let candidate = clean(vec![from, escape, side_a, side_b, to]);
        if !intersects(&candidate, obstacle) {
            let length: f64 = candidate
                .windows(2)
                .map(|p| (p[1].x() - p[0].x()).abs() + (p[1].y() - p[0].y()).abs())
                .sum();
            candidates.push((length, candidate));
        }
    }
    candidates.sort_by(|a, b| a.0.total_cmp(&b.0));
    candidates
        .into_iter()
        .next()
        .map(|(_, points)| points)
        .ok_or_else(drawing_error)
}

fn detour_segment(a: Point2D, b: Point2D, obstacle: &Obstacle) -> Result<Vec<Point2D>> {
    let mut candidates = Vec::new();
    for positive in [true, false] {
        let candidate = if a.y() == b.y() {
            let (near, far) = if a.x() < b.x() {
                (obstacle.min_x - CLEARANCE_MM, obstacle.max_x + CLEARANCE_MM)
            } else {
                (obstacle.max_x + CLEARANCE_MM, obstacle.min_x - CLEARANCE_MM)
            };
            let side = if positive {
                obstacle.max_y + CLEARANCE_MM
            } else {
                obstacle.min_y - CLEARANCE_MM
            };
            clean(vec![
                a,
                Point2D::new(near, a.y())?,
                Point2D::new(near, side)?,
                Point2D::new(far, side)?,
                Point2D::new(far, a.y())?,
                b,
            ])
        } else {
            let (near, far) = if a.y() < b.y() {
                (obstacle.min_y - CLEARANCE_MM, obstacle.max_y + CLEARANCE_MM)
            } else {
                (obstacle.max_y + CLEARANCE_MM, obstacle.min_y - CLEARANCE_MM)
            };
            let side = if positive {
                obstacle.max_x + CLEARANCE_MM
            } else {
                obstacle.min_x - CLEARANCE_MM
            };
            clean(vec![
                a,
                Point2D::new(a.x(), near)?,
                Point2D::new(side, near)?,
                Point2D::new(side, far)?,
                Point2D::new(a.x(), far)?,
                b,
            ])
        };
        if !intersects(&candidate, obstacle) {
            let length: f64 = candidate
                .windows(2)
                .map(|p| (p[1].x() - p[0].x()).abs() + (p[1].y() - p[0].y()).abs())
                .sum();
            candidates.push((length, candidate));
        }
    }
    candidates.sort_by(|a, b| a.0.total_cmp(&b.0));
    candidates
        .into_iter()
        .next()
        .map(|(_, points)| points)
        .ok_or_else(drawing_error)
}

impl Project {
    pub(super) fn clear_route_geometry(
        &self,
        route: &ConnectionRoute,
        mut points: Vec<Point2D>,
        transforms: &BTreeMap<RepresentationId, Transform2D>,
    ) -> Result<Vec<Point2D>> {
        for (port, reverse) in [(route.source(), false), (route.target(), true)] {
            let rep = self
                .representation(port.representation_id())
                .ok_or_else(drawing_error)?;
            let def = rep
                .symbol()
                .and_then(|id| self.symbol_definition(id))
                .ok_or_else(drawing_error)?;
            let t = transforms
                .get(&rep.id())
                .copied()
                .unwrap_or(rep.transform());
            let direction = def
                .ports()
                .iter()
                .find(|p| p.id() == port.port_id())
                .ok_or_else(drawing_error)?
                .transformed(t)?
                .orientation_degrees();
            let bodies = obstacles(def, t)?;
            if reverse {
                points.reverse();
            }
            for body in &bodies {
                if !intersects(&points, body) {
                    continue;
                }
                let first = *points.first().ok_or_else(drawing_error)?;
                let next = *points.get(1).ok_or_else(drawing_error)?;
                let prefix = detour(first, next, direction, body)?;
                points.splice(0..2, prefix);
                if intersects(&points, body) {
                    return Err(drawing_error());
                }
            }
            if reverse {
                points.reverse();
            }
        }
        // Closed bodies are tested by transformed contours. The bounding box
        // is used only to select clearance coordinates after a true crossing.
        let mut page_obstacles = Vec::new();
        for rep in self.representations.values() {
            if rep.page_id() != route.page_id()
                || rep.id() == route.source().representation_id()
                || rep.id() == route.target().representation_id()
            {
                continue;
            }
            let Some(def) = rep.symbol().and_then(|id| self.symbol_definition(id)) else {
                continue;
            };
            let t = transforms
                .get(&rep.id())
                .copied()
                .unwrap_or(rep.transform());
            page_obstacles.extend(obstacles(def, t)?);
        }
        for body in &page_obstacles {
            let mut attempts = 0;
            while let Some(index) = points.windows(2).position(|p| body.crosses(p[0], p[1])) {
                attempts += 1;
                if attempts > 4096 {
                    return Err(drawing_error());
                }
                let replacement = detour_segment(points[index], points[index + 1], body)?;
                points.splice(index..index + 2, replacement);
            }
        }
        if page_obstacles.iter().any(|body| intersects(&points, body)) {
            return Err(drawing_error());
        }
        validate_route_points(&points)?;
        Ok(points)
    }
}
