// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Sheet reference texts for cross-references, contact mirrors and interruptions; start with
//! SheetReferenceText. The core builds them so screen and PDF show identical text. Each segment
//! retains its target occurrence.
use super::*;
use std::collections::BTreeMap;

/// Height of sheet reference text in millimetres.
pub const SHEET_REFERENCE_HEIGHT: f64 = 2.5;

/// Purpose of a reference annotation drawn on a sheet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SheetReferenceKind {
    /// Below an occurrence: the main one lists the others, while others refer to the main one.
    CrossReference,
    /// Contact mirror below the main function.
    ContactMirror,
    /// Potential interruptions in the same group.
    Interruption,
}

/// Text segment; `target` is its navigation destination occurrence.
#[derive(Clone, Debug, PartialEq)]
pub struct SheetReferenceSegment {
    /// Text displayed to the user.
    pub text: String,
    /// Explicit target of this record or operation.
    pub target: Option<RepresentationId>,
}

/// Reference text anchored to an occurrence; `origin` is its top-left corner, using TopLeft
/// alignment and SHEET_REFERENCE_HEIGHT.
#[derive(Clone, Debug, PartialEq)]
pub struct SheetReference {
    /// Occurrence to which this reference text is anchored.
    pub anchor: RepresentationId,
    /// Declared category of this record or operation.
    pub kind: SheetReferenceKind,
    /// Origin in millimetre coordinates.
    pub origin: Point2D,
    /// Ordered text segments and their navigation targets.
    pub segments: Vec<SheetReferenceSegment>,
}

impl SheetReference {
    /// Render this value as display text.
    pub fn text(&self) -> String {
        self.segments.iter().map(|s| s.text.as_str()).collect()
    }
}

fn plain(text: impl Into<String>) -> SheetReferenceSegment {
    SheetReferenceSegment {
        text: text.into(),
        target: None,
    }
}
fn link(text: impl Into<String>, target: RepresentationId) -> SheetReferenceSegment {
    SheetReferenceSegment {
        text: text.into(),
        target: Some(target),
    }
}

/// Contact mirror kind inferred from point labels, such as 13-14 normally open and 21-22 normally
/// closed.
pub fn contact_kind_label(kind: &FunctionKind, terminals: &[String]) -> &'static str {
    let ends = |c: char| terminals.iter().any(|s| s.ends_with(c));
    match kind {
        FunctionKind::MainContact => "principal",
        FunctionKind::AuxiliaryContact if ends('3') && ends('4') => "NA",
        FunctionKind::AuxiliaryContact if ends('1') && ends('2') => "NF",
        _ => "contato",
    }
}

impl Project {
    /// Short sheet label: `/page.column`, or `/page` without zones.
    pub fn short_reference(&self, page: PageId, point: Point2D) -> String {
        let label = self.reference_label(page, point);
        if label.starts_with('/') {
            label
        } else {
            format!(
                "/{}",
                self.page(page).map(|p| p.number().as_str()).unwrap_or("?")
            )
        }
    }

    fn short_reference_of(&self, id: RepresentationId) -> Option<String> {
        let r = self.representation(id)?;
        Some(self.short_reference(r.page_id(), r.transform().translation()))
    }

    /// All reference texts on the page in drawing order.
    pub fn sheet_references(&self, page: PageId) -> Vec<SheetReference> {
        let mut out = self.cross_references_on(page);
        out.extend(self.contact_mirrors_on(page));
        out.extend(self.interruptions_on(page));
        out
    }

    fn below(&self, rep: &Representation, offset: f64) -> Option<Point2D> {
        let bounds = rep
            .local_bounds()
            .and_then(|b| rep.transform().transform_bounds(b).ok())?;
        Point2D::new(bounds.origin().x(), bounds.origin().y() - offset).ok()
    }

    fn cross_references_on(&self, page: PageId) -> Vec<SheetReference> {
        type Occurrence = (RepresentationId, PageId, FunctionRole);
        let mut by_device: BTreeMap<DeviceId, Vec<Occurrence>> = BTreeMap::new();
        for r in self.representations() {
            if r.symbol().is_none() || self.marker(r.id()).is_some() {
                continue;
            }
            let Some(device) = self.device_for_target(r.target()) else {
                continue;
            };
            let role = self
                .function_for_target(r.target())
                .and_then(|f| self.function(f))
                .map_or(FunctionRole::Unspecified, Function::role);
            by_device
                .entry(device)
                .or_default()
                .push((r.id(), r.page_id(), role));
        }
        let mut out = vec![];
        for occurrences in by_device.values().filter(|o| o.len() > 1) {
            for (id, _, role) in occurrences.iter().filter(|o| o.1 == page) {
                let main: Vec<_> = occurrences
                    .iter()
                    .filter(|o| o.0 != *id && o.2 == FunctionRole::Main)
                    .collect();
                let targets: Vec<_> = if *role != FunctionRole::Main && !main.is_empty() {
                    main
                } else {
                    occurrences.iter().filter(|o| o.0 != *id).collect()
                };
                let mut segments = vec![];
                for target in targets.iter().take(4) {
                    if !segments.is_empty() {
                        segments.push(plain("  "));
                    }
                    if let Some(label) = self.short_reference_of(target.0) {
                        segments.push(link(label, target.0));
                    }
                }
                if targets.len() > 4 {
                    segments.push(plain("  …"));
                }
                let Some(rep) = self.representation(*id) else {
                    continue;
                };
                if let Some(origin) = self.below(rep, 1.0) {
                    out.push(SheetReference {
                        anchor: *id,
                        kind: SheetReferenceKind::CrossReference,
                        origin,
                        segments,
                    });
                }
            }
        }
        out
    }

    fn contact_mirrors_on(&self, page: PageId) -> Vec<SheetReference> {
        let mut out = Vec::new();
        for rep in self.representations_on_page(page).unwrap_or_default() {
            let Some(function) = self
                .function_for_target(rep.target())
                .and_then(|id| self.function(id))
            else {
                continue;
            };
            if function.role() != FunctionRole::Main {
                continue;
            }
            let rows = self
                .contact_mirror(function.device_id())
                .unwrap_or_default();
            if rows.is_empty() {
                continue;
            }
            let mut segments = vec![];
            for (i, row) in rows.iter().enumerate() {
                if i > 0 {
                    segments.push(plain("  |  "));
                }
                let kind = contact_kind_label(&row.kind, &row.terminals);
                segments.push(plain(format!("{kind} {} ", row.terminals.join("-"))));
                if row.references.is_empty() {
                    segments.push(plain("—"));
                }
                for (j, reference) in row.references.iter().enumerate() {
                    if j > 0 {
                        segments.push(plain(", "));
                    }
                    segments.push(link(reference.reference.clone(), reference.representation));
                }
            }
            if let Some(origin) = self.below(rep, 5.0) {
                out.push(SheetReference {
                    anchor: rep.id(),
                    kind: SheetReferenceKind::ContactMirror,
                    origin,
                    segments,
                });
            }
        }
        out
    }

    fn interruptions_on(&self, page: PageId) -> Vec<SheetReference> {
        let mut out = vec![];
        for marker in self
            .markers()
            .filter(|m| matches!(m.kind(), MarkerKind::Interruption))
        {
            let Some(rep) = self
                .representation(marker.id())
                .filter(|r| r.page_id() == page)
            else {
                continue;
            };
            let refs = self.marker_references(marker.id()).unwrap_or_default();
            let mut segments = vec![];
            for id in refs {
                let Some(label) = self.short_reference_of(id) else {
                    continue;
                };
                if !segments.is_empty() {
                    segments.push(plain("  "));
                }
                segments.push(link(label, id));
            }
            if segments.is_empty() {
                continue;
            }
            let at = rep.transform().translation();
            if let Ok(origin) = Point2D::new(at.x(), at.y() - 5.0) {
                out.push(SheetReference {
                    anchor: rep.id(),
                    kind: SheetReferenceKind::Interruption,
                    origin,
                    segments,
                });
            }
        }
        out
    }
}
