// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Single-line links between symbol occurrences; start with SingleLineLink.
//!
//! A single-line link is documentation only: it joins two representations
//! on a single-line page with a polyline and a phase-count mark, and never
//! creates logical connections, networks or wire numbers.
use super::*;
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};

/// Highest phase count drawn as individual strokes; more use a number.
pub const SINGLE_LINE_MAX_STROKES: u8 = 4;

/// Polyline between two representations on a single-line page.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SingleLineLink {
    pub(in crate::domain) id: SingleLineId,
    pub(in crate::domain) project_id: ProjectId,
    pub(in crate::domain) page_id: PageId,
    pub(in crate::domain) source: RepresentationId,
    pub(in crate::domain) target: RepresentationId,
    pub(in crate::domain) points: Vec<Point2D>,
    pub(in crate::domain) phases: u8,
}
impl SingleLineLink {
    /// Create a link with fresh identity: at least two distinct points and
    /// one to nine phases.
    pub fn new(
        project_id: ProjectId,
        page_id: PageId,
        source: RepresentationId,
        target: RepresentationId,
        points: Vec<Point2D>,
        phases: u8,
    ) -> Result<Self> {
        let link = Self {
            id: SingleLineId::new(),
            project_id,
            page_id,
            source,
            target,
            points,
            phases,
        };
        link.validate()?;
        Ok(link)
    }
    /// Check local constraints.
    pub fn validate(&self) -> Result<()> {
        if self.points.len() < 2
            || self.points.windows(2).any(|w| w[0] == w[1])
            || !(1..=9).contains(&self.phases)
            || self.source == self.target
        {
            return Err(DomainError::InvalidRelation(
                "conexão unifilar precisa de dois pontos, duas representações e de 1 a 9 fases",
            ));
        }
        Ok(())
    }
    /// Return this entity's stable identifier.
    pub fn id(&self) -> SingleLineId {
        self.id
    }
    /// Return the project identifier.
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }
    /// Return the page identifier.
    pub fn page_id(&self) -> PageId {
        self.page_id
    }
    /// Return the source representation.
    pub fn source(&self) -> RepresentationId {
        self.source
    }
    /// Return the target representation.
    pub fn target(&self) -> RepresentationId {
        self.target
    }
    /// Return the polyline points in millimetres.
    pub fn points(&self) -> &[Point2D] {
        &self.points
    }
    /// Return the number of phases marked on the link.
    pub fn phases(&self) -> u8 {
        self.phases
    }
    /// Phase marks: short oblique strokes across the middle of the longest
    /// segment, one per phase up to [`SINGLE_LINE_MAX_STROKES`]; above that a
    /// single stroke and the returned label position for the count.
    pub fn phase_marks(&self) -> (Vec<[Point2D; 2]>, Option<Point2D>) {
        let Some(pair) = self.points.windows(2).max_by(|a, b| {
            let len = |p: &[Point2D]| (p[1].x() - p[0].x()).hypot(p[1].y() - p[0].y());
            len(a).total_cmp(&len(b))
        }) else {
            return (vec![], None);
        };
        let (a, b) = (pair[0], pair[1]);
        let length = (b.x() - a.x()).hypot(b.y() - a.y());
        let (ux, uy) = ((b.x() - a.x()) / length, (b.y() - a.y()) / length);
        let (nx, ny) = (-uy, ux);
        let (mx, my) = ((a.x() + b.x()) / 2.0, (a.y() + b.y()) / 2.0);
        let strokes = if self.phases > SINGLE_LINE_MAX_STROKES {
            1
        } else {
            self.phases
        };
        let spacing = 1.5;
        let first = -(f64::from(strokes) - 1.0) * spacing / 2.0;
        let mut marks = Vec::with_capacity(usize::from(strokes));
        for i in 0..strokes {
            let s = first + f64::from(i) * spacing;
            let (cx, cy) = (mx + ux * s, my + uy * s);
            // 45° stroke, 3 mm long: half along the line, half across it.
            let (dx, dy) = ((ux + nx) * 1.06, (uy + ny) * 1.06);
            if let (Ok(p), Ok(q)) = (
                Point2D::new(cx - dx, cy - dy),
                Point2D::new(cx + dx, cy + dy),
            ) {
                marks.push([p, q]);
            }
        }
        let label = (self.phases > SINGLE_LINE_MAX_STROKES)
            .then(|| Point2D::new(mx + nx * 3.0, my + ny * 3.0).ok())
            .flatten();
        (marks, label)
    }
}
