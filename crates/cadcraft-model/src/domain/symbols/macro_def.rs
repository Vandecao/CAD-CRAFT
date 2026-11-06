// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Reusable, self-contained schematic macros; start with MacroContent. Captures entities and IDs
//! from their source project. Insertion assigns fresh IDs and free device designations, then
//! applies placement; see Project::insert_macro_content. A macro belongs to no project.
use super::*;
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Entity limit per macro, matching the clipboard limit.
pub const MAX_MACRO_ITEMS: usize = 10_000;

/// Stable textual macro key, such as `PARTIDA.DIRETA`, following PartId rules.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct MacroId(String);
impl MacroId {
    /// Validate a stable textual macro library key.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.trim() != value
            || value.is_empty()
            || value.len() > 160
            || value.chars().any(|c| c.is_control() || c.is_whitespace())
        {
            return Err(DomainError::InvalidText);
        }
        Ok(Self(value))
    }
    /// Borrow the validated textual value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl TryFrom<String> for MacroId {
    type Error = DomainError;
    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}
impl From<MacroId> for String {
    fn from(value: MacroId) -> Self {
        value.0
    }
}

/// Port-to-point binding stored in a macro.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MacroBinding {
    /// Explicit symbol occurrence and local port.
    pub port: PortRef,
    /// Explicit functional connection point.
    pub point: ConnectionPointId,
}

/// Captured entities retaining their source project IDs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MacroContent {
    pub(in crate::domain) project: ProjectId,
    pub(in crate::domain) page: PageId,
    #[serde(default)]
    pub(in crate::domain) devices: Vec<Device>,
    #[serde(default)]
    pub(in crate::domain) functions: Vec<Function>,
    #[serde(default)]
    pub(in crate::domain) points: Vec<ConnectionPoint>,
    #[serde(default)]
    pub(in crate::domain) components: Vec<Component>,
    #[serde(default)]
    pub(in crate::domain) parts: Vec<Part>,
    #[serde(default)]
    pub(in crate::domain) definitions: Vec<SymbolDefinition>,
    /// Symbols, free content and markers; route occurrences are derived from `routes` during insertion.
    #[serde(default)]
    pub(in crate::domain) representations: Vec<Representation>,
    #[serde(default)]
    pub(in crate::domain) bindings: Vec<MacroBinding>,
    #[serde(default)]
    pub(in crate::domain) markers: Vec<ElectricalMarker>,
    #[serde(default)]
    pub(in crate::domain) connections: Vec<Connection>,
    #[serde(default)]
    pub(in crate::domain) routes: Vec<ConnectionRoute>,
    #[serde(default)]
    pub(in crate::domain) junctions: Vec<ConnectionJunction>,
}
impl MacroContent {
    /// Return the number of contained entries or entity changes recorded by this value.
    pub fn len(&self) -> usize {
        self.devices.len()
            + self.functions.len()
            + self.points.len()
            + self.components.len()
            + self.parts.len()
            + self.definitions.len()
            + self.representations.len()
            + self.bindings.len()
            + self.markers.len()
            + self.connections.len()
            + self.routes.len()
            + self.junctions.len()
    }
    /// Whether there are no visible occurrences or routes to insert.
    pub fn is_empty(&self) -> bool {
        self.representations.is_empty() && self.routes.is_empty()
    }
    /// Return the devices in their stored order.
    pub fn devices(&self) -> &[Device] {
        &self.devices
    }
    /// Return the functions in their stored order.
    pub fn functions(&self) -> &[Function] {
        &self.functions
    }
    /// Return the representations in their stored order.
    pub fn representations(&self) -> &[Representation] {
        &self.representations
    }
    /// Return the routes in their stored order.
    pub fn routes(&self) -> &[ConnectionRoute] {
        &self.routes
    }
    /// Return the junctions in their stored order.
    pub fn junctions(&self) -> &[ConnectionJunction] {
        &self.junctions
    }
    /// Return the definitions in their stored order.
    pub fn definitions(&self) -> &[SymbolDefinition] {
        &self.definitions
    }
    /// Return the parts in their stored order.
    pub fn parts(&self) -> &[Part] {
        &self.parts
    }
    /// Find the definition by its stable identifier; return None when absent.
    pub fn definition(&self, id: &SymbolDefinitionId) -> Option<&SymbolDefinition> {
        self.definitions.iter().find(|d| d.id() == id)
    }
    /// Bounds of symbols, free content and wires in source coordinates.
    pub fn bounds(&self) -> Result<Rect2D> {
        let mut min = (f64::INFINITY, f64::INFINITY);
        let mut max = (f64::NEG_INFINITY, f64::NEG_INFINITY);
        let mut grow = |p: Point2D| {
            min = (min.0.min(p.x()), min.1.min(p.y()));
            max = (max.0.max(p.x()), max.1.max(p.y()));
        };
        for r in &self.representations {
            let b = r.bounds()?;
            grow(b.origin());
            grow(b.max());
        }
        for route in &self.routes {
            for p in route.points() {
                grow(*p);
            }
        }
        if !min.0.is_finite() {
            return Err(DomainError::InvalidRelation("macro sem conteúdo"));
        }
        Rect2D::new(
            Point2D::new(min.0, min.1)?,
            Size2D::new(max.0 - min.0, max.1 - min.1)?,
        )
    }
    /// Pure preview without fresh IDs: occurrences and wire polylines transformed from `base` to
    /// `placement`.
    pub fn preview(
        &self,
        base: Point2D,
        placement: Transform2D,
    ) -> Result<(Vec<Representation>, Vec<Vec<Point2D>>)> {
        let total = placement_from_base(base, placement)?;
        let representations = self
            .representations
            .iter()
            .map(|r| {
                let mut next = r.clone();
                next.transform = r.transform.then(total)?;
                Ok(next)
            })
            .collect::<Result<Vec<_>>>()?;
        let routes = self
            .routes
            .iter()
            .map(|r| Ok(r.transformed(total)?.points().to_vec()))
            .collect::<Result<Vec<_>>>()?;
        Ok((representations, routes))
    }
    /// Cheap structural checks; full consistency is checked by inserting into a scratch project via
    /// Project::check_macro_content.
    fn check_structure(&self) -> Result<()> {
        if self.len() > MAX_MACRO_ITEMS {
            return Err(DomainError::InvalidRelation("macro grande demais"));
        }
        if self.is_empty() {
            return Err(DomainError::InvalidRelation("macro sem conteúdo"));
        }
        let mut definitions = BTreeSet::new();
        if self
            .definitions
            .iter()
            .any(|d| !definitions.insert(d.id().clone()))
        {
            return Err(DomainError::InvalidRelation("símbolo repetido na macro"));
        }
        let mut parts = BTreeSet::new();
        if self.parts.iter().any(|p| !parts.insert(p.id().clone())) {
            return Err(DomainError::InvalidRelation("peça repetida na macro"));
        }
        for definition in &self.definitions {
            definition.validate()?;
        }
        for part in &self.parts {
            part.validate()?;
        }
        Ok(())
    }
}

/// `p ↦ placement(p − base)`.
pub(in crate::domain) fn placement_from_base(
    base: Point2D,
    placement: Transform2D,
) -> Result<Transform2D> {
    Transform2D::new(Point2D::new(-base.x(), -base.y())?, 0.0, false, false)?.then(placement)
}

/// Window macro inserted into a page, or page macro creating a new page.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MacroKind {
    /// Reusable content inserted into an existing page.
    Window,
    /// Reusable content that creates a new page with the supplied metadata and size.
    Page {
        /// Human-readable name.
        name: Text,
        /// Declared category of this record or operation.
        kind: PageKind,
        /// Sheet dimensions in millimetres.
        size: Size2D,
    },
}

/// Versioned reusable schematic content with placement base and library metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "MacroData")]
pub struct MacroDefinition {
    id: MacroId,
    name: Text,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    description: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    translations: BTreeMap<Text, Text>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    category: String,
    version: u32,
    kind: MacroKind,
    /// Insertion point in source coordinates, positioned under the cursor.
    base: Point2D,
    content: MacroContent,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MacroData {
    id: MacroId,
    name: Text,
    #[serde(default)]
    description: String,
    #[serde(default)]
    translations: BTreeMap<Text, Text>,
    #[serde(default)]
    category: String,
    version: u32,
    kind: MacroKind,
    base: Point2D,
    content: MacroContent,
}
impl TryFrom<MacroData> for MacroDefinition {
    type Error = DomainError;
    fn try_from(d: MacroData) -> Result<Self> {
        let definition = Self {
            id: d.id,
            name: d.name,
            description: d.description,
            translations: d.translations,
            category: d.category,
            version: d.version,
            kind: d.kind,
            base: d.base,
            content: d.content,
        };
        definition.check_fields()?;
        definition.content.check_structure()?;
        Ok(definition)
    }
}
impl MacroDefinition {
    /// Create a reusable macro after validating identity, metadata, base point and captured content.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        kind: MacroKind,
        base: Point2D,
        content: MacroContent,
    ) -> Result<Self> {
        let definition = Self {
            id: MacroId::new(id)?,
            name: Text::new(name)?,
            description: String::new(),
            translations: BTreeMap::new(),
            category: String::new(),
            version: 1,
            kind,
            base,
            content,
        };
        definition.validate()?;
        Ok(definition)
    }
    fn check_fields(&self) -> Result<()> {
        if self.version == 0 {
            return Err(DomainError::InvalidRelation("versão da macro deve ser ≥ 1"));
        }
        if self.description.len() > 4096
            || self.category.len() > 160
            || self.category.chars().any(char::is_control)
        {
            return Err(DomainError::InvalidText);
        }
        Ok(())
    }
    /// Full validation of fields, structure and scratch-project insertion, requiring closed internal
    /// references and consistent geometry.
    pub fn validate(&self) -> Result<()> {
        self.check_fields()?;
        self.content.check_structure()?;
        Project::check_macro_content(&self.content)
    }
    /// Return this value with description replaced, rejecting invalid input.
    pub fn with_description(mut self, description: impl Into<String>) -> Result<Self> {
        self.description = description.into();
        self.check_fields()?;
        Ok(self)
    }
    /// Return this value with category replaced, rejecting invalid input.
    pub fn with_category(mut self, category: impl Into<String>) -> Result<Self> {
        self.category = category.into().trim().to_owned();
        self.check_fields()?;
        Ok(self)
    }
    /// Return this value with translation replaced.
    pub fn with_translation(
        mut self,
        language: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<Self> {
        self.translations
            .insert(Text::new(language)?, Text::new(description)?);
        Ok(self)
    }
    /// New version of this macro, incrementing the version by one.
    pub fn next_version(mut self) -> Self {
        self.version = self.version.saturating_add(1);
        self
    }
    /// Return this value with version replaced, rejecting invalid input.
    pub fn with_version(mut self, version: u32) -> Result<Self> {
        self.version = version;
        self.check_fields()?;
        Ok(self)
    }
    /// Return this value with name replaced, rejecting invalid input.
    pub fn with_name(mut self, name: impl Into<String>) -> Result<Self> {
        self.name = Text::new(name)?;
        Ok(self)
    }
    /// Return this value with id replaced, rejecting invalid input.
    pub fn with_id(mut self, id: impl Into<String>) -> Result<Self> {
        self.id = MacroId::new(id)?;
        Ok(self)
    }
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> &MacroId {
        &self.id
    }
    /// Return the name.
    pub fn name(&self) -> &Text {
        &self.name
    }
    /// Return the description.
    pub fn description(&self) -> &str {
        &self.description
    }
    /// Borrow localized descriptions indexed by language key.
    pub fn translations(&self) -> &BTreeMap<Text, Text> {
        &self.translations
    }
    /// Return the category.
    pub fn category(&self) -> &str {
        &self.category
    }
    /// Return the version.
    pub fn version(&self) -> u32 {
        self.version
    }
    /// Return the kind.
    pub fn kind(&self) -> &MacroKind {
        &self.kind
    }
    /// Return the base.
    pub fn base(&self) -> Point2D {
        self.base
    }
    /// Return the content.
    pub fn content(&self) -> &MacroContent {
        &self.content
    }
    /// Whether this definition creates a new page when inserted.
    pub fn is_page(&self) -> bool {
        matches!(self.kind, MacroKind::Page { .. })
    }
}
