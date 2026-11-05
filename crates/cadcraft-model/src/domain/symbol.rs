// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::transform::canonical_degrees;
use super::{Metadata, Point2D, Rect2D, Size2D, TextAlign, Transform2D};
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

/// Identidade textual de biblioteca, independente de ocorrências UUID.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SymbolDefinitionId(String);
impl SymbolDefinitionId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        nonempty(&value)?;
        Ok(Self(value))
    }
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

/// Geometria local em milímetros. Texto tem células monoespaçadas 0,6 × altura;
/// sua origem é o canto inferior esquerdo da caixa nominal, sem padding.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SymbolPrimitive {
    Line {
        start: Point2D,
        end: Point2D,
    },
    Polyline {
        points: Vec<Point2D>,
        closed: bool,
    },
    Rectangle {
        rect: Rect2D,
    },
    Circle {
        center: Point2D,
        radius: f64,
    },
    Arc {
        center: Point2D,
        radius: f64,
        start_degrees: f64,
        sweep_degrees: f64,
    },
    Text {
        origin: Point2D,
        text: String,
        height: f64,
    },
    Anchor {
        position: Point2D,
    },
    /// Etapa 17: elipse com raios nos eixos locais girados de `rotation_degrees`.
    Ellipse {
        center: Point2D,
        radius_x: f64,
        radius_y: f64,
        rotation_degrees: f64,
    },
    /// Etapa 17: texto preenchido na ocorrência (designação, porta, …).
    DynamicText {
        origin: Point2D,
        field: SymbolTextField,
        height: f64,
        align: TextAlign,
        rotation_degrees: f64,
    },
}

/// Etapa 17: o que um texto dinâmico mostra na ocorrência do símbolo.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolTextField {
    /// Designação do dispositivo (ex.: `-K1`).
    Designation,
    /// Tipo da função representada.
    FunctionKind,
    /// Rótulo do ponto de conexão ligado à porta.
    PortLabel(String),
    /// Referência cruzada da ocorrência (`/página.coluna`).
    Reference,
    /// Propriedade técnica do dispositivo pela chave.
    DeviceProperty(String),
}
impl SymbolTextField {
    pub fn label(&self) -> String {
        match self {
            Self::Designation => "Designação".into(),
            Self::FunctionKind => "Função".into(),
            Self::PortLabel(p) => format!("Ponto da porta {p}"),
            Self::Reference => "Referência".into(),
            Self::DeviceProperty(k) => format!("Propriedade: {k}"),
        }
    }
    /// Texto mostrado quando não há ocorrência (editor e biblioteca).
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

/// Etapa 17: padrão do traço.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineDash {
    #[default]
    Solid,
    Dashed,
    DashDot,
    Dotted,
}
impl LineDash {
    pub const ALL: [Self; 4] = [Self::Solid, Self::Dashed, Self::DashDot, Self::Dotted];
    pub fn label(self) -> &'static str {
        match self {
            Self::Solid => "Contínuo",
            Self::Dashed => "Tracejado",
            Self::DashDot => "Traço-ponto",
            Self::Dotted => "Pontilhado",
        }
    }
    /// Comprimentos traço/vão em múltiplos da espessura nominal (mm).
    pub fn pattern_mm(self) -> &'static [f64] {
        match self {
            Self::Solid => &[],
            Self::Dashed => &[3.0, 1.5],
            Self::DashDot => &[4.0, 1.0, 0.5, 1.0],
            Self::Dotted => &[0.5, 1.0],
        }
    }
}

/// Etapa 17: aparência de uma primitiva (padrão = traço contínuo, espessura
/// do desenho, sem preenchimento).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrimitiveStyle {
    #[serde(default, skip_serializing_if = "is_solid")]
    pub dash: LineDash,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width_mm: Option<f64>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub fill: bool,
}
fn is_solid(d: &LineDash) -> bool {
    *d == LineDash::Solid
}
impl PrimitiveStyle {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
    /// Confere o estilo para a primitiva (preenchimento só em formas fechadas).
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

/// Largura nominal (caracteres) usada para os limites de um texto dinâmico.
const DYNAMIC_TEXT_CHARS: f64 = 4.0;

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
fn bounds_points(points: impl IntoIterator<Item = Point2D>) -> Result<Rect2D> {
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
    /// Uma lista vazia permite qualquer rotação finita.
    pub fn any() -> Self {
        Self {
            rotations: vec![],
            mirror_x: true,
            mirror_y: true,
        }
    }
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
    pub fn rotations(&self) -> &[f64] {
        &self.rotations
    }
    pub fn allows(&self, t: Transform2D) -> bool {
        (!t.mirror_x() || self.mirror_x)
            && (!t.mirror_y() || self.mirror_y)
            && (self.rotations.is_empty() || self.rotations.contains(&t.rotation_degrees()))
    }
}
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
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn position(&self) -> Point2D {
        self.position
    }
    pub fn orientation_degrees(&self) -> f64 {
        self.orientation_degrees
    }
    pub fn identification(&self) -> &str {
        &self.identification
    }
    pub fn role(&self) -> Option<&str> {
        self.role.as_deref()
    }
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
    /// Etapa 17: estilo por índice de primitiva (omitido quando vazio).
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
    pub fn with_orientations(mut self, orientations: SymbolOrientations) -> Self {
        self.orientations = orientations;
        self
    }
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
    pub fn id(&self) -> &SymbolDefinitionId {
        &self.id
    }
    pub fn version(&self) -> u32 {
        self.version
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn category(&self) -> &str {
        &self.category
    }
    pub fn primitives(&self) -> &[SymbolPrimitive] {
        &self.primitives
    }
    pub fn ports(&self) -> &[SymbolPort] {
        &self.ports
    }
    pub fn orientations(&self) -> &SymbolOrientations {
        &self.orientations
    }
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    pub fn bounds(&self) -> Rect2D {
        self.bounds
    }
    /// Etapa 17: estilo da primitiva `index` (padrão quando não definido).
    pub fn style(&self, index: usize) -> PrimitiveStyle {
        self.styles.get(&index).copied().unwrap_or_default()
    }
    pub fn styles(&self) -> &BTreeMap<usize, PrimitiveStyle> {
        &self.styles
    }
    /// Etapa 17: mesmos dados com estilos novos (validados).
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
    /// Etapa 17: nova versão editada (mesmo ID, versão + 1).
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
    /// Etapa 17: cópia com outra identidade (versão 1), para duplicar.
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
