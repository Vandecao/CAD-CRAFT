// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Symbol definitions, primitives and ports; start with SymbolDefinition.
use super::{Metadata, Point2D, Rect2D, Size2D, TextAlign, Transform2D};
use crate::domain::geometry::transform::canonical_degrees;
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

fn invalid() -> DomainError {
    DomainError::InvalidRelation("definição de símbolo inválida")
}
fn nonempty(value: &str) -> Result<()> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        Err(invalid())
    } else {
        Ok(())
    }
}

/// Textual library identity independent of occurrence UUIDs.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SymbolDefinitionId(String);
impl SymbolDefinitionId {
    /// Validate a stable textual symbol library key.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        nonempty(&value)?;
        Ok(Self(value))
    }
    /// Borrow the validated textual value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl TryFrom<String> for SymbolDefinitionId {
    type Error = DomainError;
    fn try_from(v: String) -> Result<Self> {
        Self::new(v)
    }
}
impl From<SymbolDefinitionId> for String {
    fn from(v: SymbolDefinitionId) -> Self {
        v.0
    }
}
impl std::fmt::Display for SymbolDefinitionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Local millimetre geometry. Text has monospaced cells 0.6 times its height; origin is the
/// bottom-left corner of the nominal box without padding.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SymbolPrimitive {
    /// Straight line primitive.
    Line {
        /// Starting value or geometric endpoint.
        start: Point2D,
        /// Final endpoint in local millimetre coordinates.
        end: Point2D,
    },
    /// Ordered polyline vertices with optional closure.
    Polyline {
        /// Connection points or authored geometric vertices.
        points: Vec<Point2D>,
        /// Whether the polyline closes its final segment.
        closed: bool,
    },
    /// Rectangular primitive.
    Rectangle {
        /// Rectangle in local millimetre coordinates.
        rect: Rect2D,
    },
    /// Circular primitive.
    Circle {
        /// Center in local millimetre coordinates.
        center: Point2D,
        /// Radius in millimetres.
        radius: f64,
    },
    /// Circular arc primitive.
    Arc {
        /// Center in local millimetre coordinates.
        center: Point2D,
        /// Radius in millimetres.
        radius: f64,
        /// Initial arc angle in degrees.
        start_degrees: f64,
        /// Signed angular extent of the arc in degrees.
        sweep_degrees: f64,
    },
    /// Text content.
    Text {
        /// Origin in millimetre coordinates.
        origin: Point2D,
        /// Text displayed to the user.
        text: String,
        /// Text or geometry height in millimetres.
        height: f64,
    },
    /// Anchor without drawable geometry.
    Anchor {
        /// Position in the containing page or terminal.
        position: Point2D,
    },
    /// Ellipse with local-axis radii rotated by `rotation_degrees`.
    Ellipse {
        /// Center in local millimetre coordinates.
        center: Point2D,
        /// Radius along the local X axis in millimetres.
        radius_x: f64,
        /// Radius along the local Y axis in millimetres.
        radius_y: f64,
        /// Counterclockwise rotation in degrees.
        rotation_degrees: f64,
    },
    /// Text resolved for an occurrence, such as designation or port label.
    DynamicText {
        /// Origin in millimetre coordinates.
        origin: Point2D,
        /// Dynamic text field resolved from the current project context.
        field: SymbolTextField,
        /// Text or geometry height in millimetres.
        height: f64,
        /// Text alignment relative to the origin.
        align: TextAlign,
        /// Counterclockwise rotation in degrees.
        rotation_degrees: f64,
    },
}

/// Value displayed by dynamic text in a symbol occurrence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolTextField {
    /// Device designation, such as `-K1`.
    Designation,
    /// Kind of the represented function.
    FunctionKind,
    /// Label of the connection point bound to the port.
    PortLabel(String),
    /// Occurrence cross-reference, formatted as `/page.column`.
    Reference,
    /// Device technical property selected by key.
    DeviceProperty(String),
}
impl SymbolTextField {
    /// Return the pt-BR display label.
    pub fn label(&self) -> String {
        match self {
            Self::Designation => "Designação".into(),
            Self::FunctionKind => "Função".into(),
            Self::PortLabel(p) => format!("Ponto da porta {p}"),
            Self::Reference => "Referência".into(),
            Self::DeviceProperty(k) => format!("Propriedade: {k}"),
        }
    }
    /// Placeholder when no occurrence exists, used by the editor and library.
    pub fn placeholder(&self) -> String {
        match self {
            Self::Designation => "-K?".into(),
            Self::FunctionKind => "<função>".into(),
            Self::PortLabel(p) => p.clone(),
            Self::Reference => "/?.?".into(),
            Self::DeviceProperty(k) => format!("<{k}>"),
        }
    }
}

/// Stroke pattern.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineDash {
    /// Continuous stroke.
    #[default]
    Solid,
    /// Dashed stroke.
    Dashed,
    /// Alternating dash and dot stroke.
    DashDot,
    /// Dotted stroke.
    Dotted,
}
impl LineDash {
    /// All supported variants in display order.
    pub const ALL: [Self; 4] = [Self::Solid, Self::Dashed, Self::DashDot, Self::Dotted];
    /// Return the pt-BR display label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Solid => "Contínuo",
            Self::Dashed => "Tracejado",
            Self::DashDot => "Traço-ponto",
            Self::Dotted => "Pontilhado",
        }
    }
    /// Dash and gap lengths as multiples of nominal stroke width in millimetres.
    pub fn pattern_mm(self) -> &'static [f64] {
        match self {
            Self::Solid => &[],
            Self::Dashed => &[3.0, 1.5],
            Self::DashDot => &[4.0, 1.0, 0.5, 1.0],
            Self::Dotted => &[0.5, 1.0],
        }
    }
}

/// Primitive appearance; defaults to a continuous stroke, drawing stroke width and no fill.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrimitiveStyle {
    /// Stroke dash pattern.
    #[serde(default, skip_serializing_if = "is_solid")]
    pub dash: LineDash,
    /// Width in millimetres.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width_mm: Option<f64>,
    /// Whether the closed primitive interior is filled.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub fill: bool,
}
fn is_solid(d: &LineDash) -> bool {
    *d == LineDash::Solid
}
impl PrimitiveStyle {
    /// Whether the primitive style equals the default drawing style.
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
    /// Validate style compatibility; only closed shapes accept fill.
    pub fn check(&self, primitive: &SymbolPrimitive) -> Result<()> {
        if self
            .width_mm
            .is_some_and(|w| !w.is_finite() || w <= 0.0 || w > 20.0)
        {
            return Err(invalid());
        }
        if self.fill && !primitive.is_closed() {
            return Err(DomainError::InvalidRelation(
                "preenchimento só em formas fechadas",
            ));
        }
        Ok(())
    }
}

/// Nominal character width used for dynamic text bounds.
const DYNAMIC_TEXT_CHARS: f64 = 4.0;

/// Allowed finite symbol rotations and mirror directions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "OrientationsData")]
pub struct SymbolOrientations {
    rotations: Vec<f64>,
    mirror_x: bool,
    mirror_y: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OrientationsData {
    rotations: Vec<f64>,
    mirror_x: bool,
    mirror_y: bool,
}
impl TryFrom<OrientationsData> for SymbolOrientations {
    type Error = DomainError;
    fn try_from(v: OrientationsData) -> Result<Self> {
        Self::new(v.rotations, v.mirror_x, v.mirror_y)
    }
}
impl SymbolOrientations {
    /// An empty list allows any finite rotation.
    pub fn any() -> Self {
        Self {
            rotations: vec![],
            mirror_x: true,
            mirror_y: true,
        }
    }
    /// Create an orientation policy from finite rotations and allowed mirror directions.
    pub fn new(rotations: Vec<f64>, mirror_x: bool, mirror_y: bool) -> Result<Self> {
        if rotations.iter().any(|a| !a.is_finite()) {
            return Err(invalid());
        }
        let mut rotations: Vec<_> = rotations.into_iter().map(canonical_degrees).collect();
        rotations.sort_by(f64::total_cmp);
        rotations.dedup();
        Ok(Self {
            rotations,
            mirror_x,
            mirror_y,
        })
    }
    /// Return the rotations in their stored order.
    pub fn rotations(&self) -> &[f64] {
        &self.rotations
    }
    /// Whether the transform satisfies the allowed rotations and mirror directions.
    pub fn allows(&self, t: Transform2D) -> bool {
        (!t.mirror_x() || self.mirror_x)
            && (!t.mirror_y() || self.mirror_y)
            && (self.rotations.is_empty() || self.rotations.contains(&t.rotation_degrees()))
    }
}
/// Named local connection port with position, orientation and identification.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "PortData")]
pub struct SymbolPort {
    id: String,
    position: Point2D,
    orientation_degrees: f64,
    identification: String,
    role: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PortData {
    id: String,
    position: Point2D,
    orientation_degrees: f64,
    identification: String,
    role: Option<String>,
}
impl TryFrom<PortData> for SymbolPort {
    type Error = DomainError;
    fn try_from(v: PortData) -> Result<Self> {
        Self::new(
            v.id,
            v.position,
            v.orientation_degrees,
            v.identification,
            v.role,
        )
    }
}
impl SymbolPort {
    /// Create a local symbol port with validated identity, orientation and identification.
    pub fn new(
        id: impl Into<String>,
        position: Point2D,
        orientation_degrees: f64,
        identification: impl Into<String>,
        role: Option<String>,
    ) -> Result<Self> {
        let id = id.into();
        let identification = identification.into();
        nonempty(&id)?;
        nonempty(&identification)?;
        if let Some(r) = &role {
            nonempty(r)?;
        }
        if !orientation_degrees.is_finite() {
            return Err(invalid());
        }
        Ok(Self {
            id,
            position,
            orientation_degrees: canonical_degrees(orientation_degrees),
            identification,
            role,
        })
    }
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> &str {
        &self.id
    }
    /// Return the position in millimetre coordinates.
    pub fn position(&self) -> Point2D {
        self.position
    }
    /// Return the orientation degrees.
    pub fn orientation_degrees(&self) -> f64 {
        self.orientation_degrees
    }
    /// Return the identification.
    pub fn identification(&self) -> &str {
        &self.identification
    }
    /// Return the optional role; absence represents an unset value or relation.
    pub fn role(&self) -> Option<&str> {
        self.role.as_deref()
    }
    /// Return the port after applying a validated placement transform.
    pub fn transformed(&self, t: Transform2D) -> Result<Self> {
        let mut angle = self.orientation_degrees;
        if t.mirror_x() {
            angle = 180. - angle;
        }
        if t.mirror_y() {
            angle = -angle;
        }
        Self::new(
            self.id.clone(),
            t.apply(self.position)?,
            angle + t.rotation_degrees(),
            self.identification.clone(),
            self.role.clone(),
        )
    }
}
/// Versioned local symbol geometry, ports, styles and orientation policy.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "DefinitionData")]
pub struct SymbolDefinition {
    id: SymbolDefinitionId,
    version: u32,
    name: String,
    category: String,
    primitives: Vec<SymbolPrimitive>,
    ports: Vec<SymbolPort>,
    orientations: SymbolOrientations,
    metadata: Metadata,
    /// Style by primitive index, omitted when empty.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    styles: BTreeMap<usize, PrimitiveStyle>,
    #[serde(skip)]
    bounds: Rect2D,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DefinitionData {
    id: SymbolDefinitionId,
    version: u32,
    name: String,
    category: String,
    primitives: Vec<SymbolPrimitive>,
    ports: Vec<SymbolPort>,
    orientations: SymbolOrientations,
    metadata: Metadata,
    #[serde(default)]
    styles: BTreeMap<usize, PrimitiveStyle>,
}
impl TryFrom<DefinitionData> for SymbolDefinition {
    type Error = DomainError;
    fn try_from(v: DefinitionData) -> Result<Self> {
        Self::from_data(v)
    }
}
impl SymbolDefinition {
    /// Create a symbol definition after validating its identity, geometry, ports and bounds.
    pub fn new(
        id: SymbolDefinitionId,
        version: u32,
        name: impl Into<String>,
        category: impl Into<String>,
        primitives: Vec<SymbolPrimitive>,
        ports: Vec<SymbolPort>,
    ) -> Result<Self> {
        Self::from_data(DefinitionData {
            id,
            version,
            name: name.into(),
            category: category.into(),
            primitives,
            ports,
            orientations: SymbolOrientations::any(),
            metadata: Metadata::new(),
            styles: BTreeMap::new(),
        })
    }
    /// Return this value with orientations replaced.
    pub fn with_orientations(mut self, orientations: SymbolOrientations) -> Self {
        self.orientations = orientations;
        self
    }
    /// Return this value with metadata replaced.
    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = metadata;
        self
    }
    fn from_data(data: DefinitionData) -> Result<Self> {
        let DefinitionData {
            id,
            version,
            name,
            category,
            primitives,
            ports,
            orientations,
            metadata,
            styles,
        } = data;
        nonempty(&name)?;
        nonempty(&category)?;
        if version == 0 || primitives.is_empty() {
            return Err(invalid());
        }
        let mut port_ids = BTreeSet::new();
        for port in &ports {
            if !port_ids.insert(port.id()) {
                return Err(invalid());
            }
        }
        let mut points = Vec::with_capacity(primitives.len() * 2 + ports.len());
        for primitive in &primitives {
            let b = primitive.bounds()?;
            points.extend([b.origin(), b.max()]);
        }
        for port in &ports {
            points.push(port.position());
        }
        for (index, style) in &styles {
            let primitive = primitives.get(*index).ok_or_else(invalid)?;
            style.check(primitive)?;
        }
        let styles = styles
            .into_iter()
            .filter(|(_, s)| !s.is_default())
            .collect();
        let bounds = bounds_points(points)?;
        Ok(Self {
            id,
            version,
            name,
            category,
            primitives,
            ports,
            orientations,
            metadata,
            styles,
            bounds,
        })
    }
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> &SymbolDefinitionId {
        &self.id
    }
    /// Return the version.
    pub fn version(&self) -> u32 {
        self.version
    }
    /// Return the name.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Return the category.
    pub fn category(&self) -> &str {
        &self.category
    }
    /// Return the primitives in their stored order.
    pub fn primitives(&self) -> &[SymbolPrimitive] {
        &self.primitives
    }
    /// Return the ports in their stored order.
    pub fn ports(&self) -> &[SymbolPort] {
        &self.ports
    }
    /// Return the orientations.
    pub fn orientations(&self) -> &SymbolOrientations {
        &self.orientations
    }
    /// Return the metadata.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    /// Return axis-aligned geometry bounds in millimetres.
    pub fn bounds(&self) -> Rect2D {
        self.bounds
    }
    /// Style of primitive `index`, falling back to the default.
    pub fn style(&self, index: usize) -> PrimitiveStyle {
        self.styles.get(&index).copied().unwrap_or_default()
    }
    /// Return the styles.
    pub fn styles(&self) -> &BTreeMap<usize, PrimitiveStyle> {
        &self.styles
    }
    /// The same data with validated replacement styles.
    pub fn with_styles(self, styles: BTreeMap<usize, PrimitiveStyle>) -> Result<Self> {
        Self::from_data(DefinitionData {
            id: self.id,
            version: self.version,
            name: self.name,
            category: self.category,
            primitives: self.primitives,
            ports: self.ports,
            orientations: self.orientations,
            metadata: self.metadata,
            styles,
        })
    }
    /// Edited next version retaining the ID and incrementing version by one.
    pub fn edited(
        &self,
        name: impl Into<String>,
        category: impl Into<String>,
        primitives: Vec<SymbolPrimitive>,
        ports: Vec<SymbolPort>,
        styles: BTreeMap<usize, PrimitiveStyle>,
    ) -> Result<Self> {
        Self::from_data(DefinitionData {
            id: self.id.clone(),
            version: self.version.saturating_add(1),
            name: name.into(),
            category: category.into(),
            primitives,
            ports,
            orientations: self.orientations.clone(),
            metadata: self.metadata.clone(),
            styles,
        })
    }
    /// Independent duplicate with a different identity and version one.
    pub fn with_identity(&self, id: SymbolDefinitionId, name: impl Into<String>) -> Result<Self> {
        Self::from_data(DefinitionData {
            id,
            version: 1,
            name: name.into(),
            category: self.category.clone(),
            primitives: self.primitives.clone(),
            ports: self.ports.clone(),
            orientations: self.orientations.clone(),
            metadata: self.metadata.clone(),
            styles: self.styles.clone(),
        })
    }
    /// Check local value constraints and report the first violated invariant.
    pub fn validate(&self) -> Result<()> {
        let reconstructed = Self::from_data(DefinitionData {
            id: self.id.clone(),
            version: self.version,
            name: self.name.clone(),
            category: self.category.clone(),
            primitives: self.primitives.clone(),
            ports: self.ports.clone(),
            orientations: self.orientations.clone(),
            metadata: self.metadata.clone(),
            styles: self.styles.clone(),
        })?;
        if reconstructed.bounds != self.bounds {
            return Err(invalid());
        }
        Ok(())
    }
}

mod primitives;
use primitives::bounds_points;
