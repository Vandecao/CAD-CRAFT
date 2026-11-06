// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Graphical occurrences independent of logical entities; start with Representation.
use super::*;
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};

/// Explicit electrical assignment, optional only for symbol occurrences. Connection has neither
/// generic technical properties nor EntityRef support.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RepresentationTarget {
    /// No electrical target association.
    Unassociated,
    /// Explicit reference to a device.
    Device(DeviceId),
    /// Explicit reference to a logical function.
    Function(FunctionId),
    /// Explicit reference to a component.
    Component(ComponentId),
    /// Explicit reference to a terminal.
    Terminal(TerminalId),
    /// Explicit reference to a connection point.
    ConnectionPoint(ConnectionPointId),
    /// Explicit reference to a connection.
    Connection(ConnectionId),
    /// Explicit reference to a conductor.
    Conductor(ConductorId),
    /// Explicit reference to a cable.
    Cable(CableId),
    /// Explicit reference to a cable core.
    CableCore(CableCoreId),
    /// Explicit reference to a terminal strip.
    TerminalStrip(TerminalStripId),
}

/// Document style independent of graphical libraries and electrical properties. Higher layers draw
/// later; IDs can break ties.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisualProperties {
    /// Whether this occurrence is drawn.
    pub visible: bool,
    /// Drawing layer; larger values draw later.
    pub layer: i32,
    /// Optional documentary label.
    pub label: Option<Text>,
}
impl Default for VisualProperties {
    fn default() -> Self {
        Self {
            visible: true,
            layer: 0,
            label: None,
        }
    }
}

/// Graphical occurrence on a page with an explicit target and validated transform.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RepresentationData")]
pub struct Representation {
    pub(in crate::domain) id: RepresentationId,
    project_id: ProjectId,
    pub(in crate::domain) page_id: PageId,
    pub(in crate::domain) target: RepresentationTarget,
    symbol: Option<SymbolDefinitionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    graphic: Option<PageGraphic>,
    pub(in crate::domain) transform: Transform2D,
    pub(in crate::domain) local_bounds: Option<Rect2D>,
    pub(in crate::domain) visual: VisualProperties,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RepresentationData {
    id: RepresentationId,
    project_id: ProjectId,
    page_id: PageId,
    target: RepresentationTarget,
    #[serde(default)]
    symbol: Option<SymbolDefinitionId>,
    #[serde(default)]
    graphic: Option<PageGraphic>,
    transform: Transform2D,
    local_bounds: Option<Rect2D>,
    visual: VisualProperties,
}
impl TryFrom<RepresentationData> for Representation {
    type Error = DomainError;
    fn try_from(data: RepresentationData) -> Result<Self> {
        let local_bounds = if let Some(graphic) = &data.graphic {
            let expected = graphic.bounds()?;
            let actual = data.local_bounds.ok_or(DomainError::InvalidRelation(
                "limites do conteúdo livre ausentes",
            ))?;
            let a = actual.origin();
            let b = expected.origin();
            let asize = actual.size();
            let bsize = expected.size();
            if (a.x() - b.x()).abs() > 1e-9
                || (a.y() - b.y()).abs() > 1e-9
                || (asize.width() - bsize.width()).abs() > 1e-9
                || (asize.height() - bsize.height()).abs() > 1e-9
            {
                return Err(DomainError::InvalidRelation(
                    "limites do conteúdo livre divergentes",
                ));
            }
            Some(expected)
        } else {
            data.local_bounds
        };
        let value = Self {
            id: data.id,
            project_id: data.project_id,
            page_id: data.page_id,
            target: data.target,
            symbol: data.symbol,
            graphic: data.graphic,
            transform: data.transform,
            local_bounds,
            visual: data.visual,
        };
        value.bounds()?;
        Ok(value)
    }
}
impl Representation {
    /// Create a graphical occurrence with fresh identity, explicit target and page placement.
    pub fn new(
        project_id: ProjectId,
        page_id: PageId,
        target: RepresentationTarget,
        transform: Transform2D,
    ) -> Self {
        Self {
            id: RepresentationId::new(),
            project_id,
            page_id,
            target,
            symbol: None,
            graphic: None,
            transform,
            local_bounds: None,
            visual: VisualProperties::default(),
        }
    }
    /// Symbol occurrence without implicit electrical entity creation.
    pub fn new_symbol(
        project_id: ProjectId,
        page_id: PageId,
        definition: &SymbolDefinition,
        target: Option<RepresentationTarget>,
        transform: Transform2D,
    ) -> Result<Self> {
        if !definition.orientations().allows(transform) {
            return Err(DomainError::InvalidRelation(
                "orientação de símbolo não permitida",
            ));
        }
        let mut value = Self::new(
            project_id,
            page_id,
            target.unwrap_or(RepresentationTarget::Unassociated),
            transform,
        );
        value.symbol = Some(definition.id().clone());
        value.local_bounds = Some(definition.bounds());
        value.bounds()?;
        Ok(value)
    }
    /// Return the optional symbol; absence represents an unset value or relation.
    pub fn symbol(&self) -> Option<&SymbolDefinitionId> {
        self.symbol.as_ref()
    }
    /// Return the optional graphic; absence represents an unset value or relation.
    pub fn graphic(&self) -> Option<&PageGraphic> {
        self.graphic.as_ref()
    }
    /// Return the new graphic.
    pub fn new_graphic(
        project_id: ProjectId,
        page_id: PageId,
        graphic: PageGraphic,
        transform: Transform2D,
    ) -> Result<Self> {
        let mut value = Self::new(
            project_id,
            page_id,
            RepresentationTarget::Unassociated,
            transform,
        );
        value.local_bounds = Some(graphic.bounds()?);
        value.graphic = Some(graphic);
        value.bounds()?;
        Ok(value)
    }
    /// Create an independent occurrence on a page with fresh identity and the supplied transform.
    pub fn duplicate_on(&self, page: PageId, transform: Transform2D) -> Result<Self> {
        let mut value = self.clone();
        value.id = RepresentationId::new();
        value.page_id = page;
        value.transform = transform;
        value.bounds()?;
        Ok(value)
    }
    /// Return the same occurrence identity on another page with the supplied transform.
    pub fn relocated(&self, page: PageId, transform: Transform2D) -> Result<Self> {
        let mut value = self.clone();
        value.page_id = page;
        value.transform = transform;
        value.bounds()?;
        Ok(value)
    }
    /// Replace free-content geometry after validating its drawable bounds.
    pub fn with_graphic(mut self, graphic: PageGraphic) -> Result<Self> {
        if self.graphic.is_none() {
            return Err(DomainError::InvalidRelation(
                "representação não é conteúdo livre",
            ));
        }
        self.local_bounds = Some(graphic.bounds()?);
        self.graphic = Some(graphic);
        self.bounds()?;
        Ok(self)
    }
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> RepresentationId {
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
    /// Return the target.
    pub fn target(&self) -> RepresentationTarget {
        self.target
    }
    /// Return the transform.
    pub fn transform(&self) -> Transform2D {
        self.transform
    }
    /// Return the optional local bounds; absence represents an unset value or relation.
    pub fn local_bounds(&self) -> Option<Rect2D> {
        self.local_bounds
    }
    /// Return the visual.
    pub fn visual(&self) -> &VisualProperties {
        &self.visual
    }
    /// Return this value with visual replaced.
    pub fn with_visual(mut self, visual: VisualProperties) -> Self {
        self.visual = visual;
        self
    }
    /// Return this value with local bounds replaced, rejecting invalid input.
    pub fn with_local_bounds(mut self, bounds: Rect2D) -> Result<Self> {
        self.local_bounds = Some(bounds);
        self.bounds()?;
        Ok(self)
    }
    /// Axis-aligned bounds in page coordinates. Without geometry only the anchor is known.
    pub fn bounds(&self) -> Result<Rect2D> {
        match self.local_bounds {
            Some(bounds) => self.transform.transform_bounds(bounds),
            None => Rect2D::new(self.transform.translation(), Size2D::new(0.0, 0.0)?),
        }
    }
}

/// Sheet and content bounds, independent of viewport pan/zoom limits. Content includes invisible
/// occurrences and may extend beyond the sheet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DocumentBounds {
    /// Sheet extent in millimetres.
    pub sheet: Rect2D,
    /// Bounds of graphical content, absent when empty.
    pub content: Option<Rect2D>,
}
