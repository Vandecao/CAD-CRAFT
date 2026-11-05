// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 19: macros — trecho de esquema reutilizável e autossuficiente.
//!
//! O conteúdo guarda as entidades como estavam no projeto de origem (com os
//! IDs de origem). Inserir troca todos os IDs por novos, dá designações livres
//! aos dispositivos e aplica a transformação de posicionamento (ver
//! `project/macros.rs`). A macro não pertence a nenhum projeto.
use super::*;
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Limite de entidades por macro (o mesmo da área de transferência).
pub const MAX_MACRO_ITEMS: usize = 10_000;

/// Chave textual estável da macro (ex.: `PARTIDA.DIRETA`), como `PartId`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct MacroId(String);
impl MacroId {
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

/// Vínculo porta→ponto guardado na macro.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MacroBinding {
    pub port: PortRef,
    pub point: ConnectionPointId,
}

/// Entidades capturadas, com os IDs do projeto de origem.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MacroContent {
    pub(super) project: ProjectId,
    pub(super) page: PageId,
    #[serde(default)]
    pub(super) devices: Vec<Device>,
    #[serde(default)]
    pub(super) functions: Vec<Function>,
    #[serde(default)]
    pub(super) points: Vec<ConnectionPoint>,
    #[serde(default)]
    pub(super) components: Vec<Component>,
    #[serde(default)]
    pub(super) parts: Vec<Part>,
    #[serde(default)]
    pub(super) definitions: Vec<SymbolDefinition>,
    /// Símbolos, conteúdo livre e marcadores (sem as representações de rota,
    /// que são derivadas de `routes` na inserção).
    #[serde(default)]
    pub(super) representations: Vec<Representation>,
    #[serde(default)]
    pub(super) bindings: Vec<MacroBinding>,
    #[serde(default)]
    pub(super) markers: Vec<ElectricalMarker>,
    #[serde(default)]
    pub(super) connections: Vec<Connection>,
    #[serde(default)]
    pub(super) routes: Vec<ConnectionRoute>,
    #[serde(default)]
    pub(super) junctions: Vec<ConnectionJunction>,
}
impl MacroContent {
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
    pub fn is_empty(&self) -> bool {
        self.representations.is_empty() && self.routes.is_empty()
    }
    pub fn devices(&self) -> &[Device] {
        &self.devices
    }
    pub fn functions(&self) -> &[Function] {
        &self.functions
    }
    pub fn representations(&self) -> &[Representation] {
        &self.representations
    }
    pub fn routes(&self) -> &[ConnectionRoute] {
        &self.routes
    }
    pub fn junctions(&self) -> &[ConnectionJunction] {
        &self.junctions
    }
    pub fn definitions(&self) -> &[SymbolDefinition] {
        &self.definitions
    }
    pub fn parts(&self) -> &[Part] {
        &self.parts
    }
    pub fn definition(&self, id: &SymbolDefinitionId) -> Option<&SymbolDefinition> {
        self.definitions.iter().find(|d| d.id() == id)
    }
    /// Retângulo que envolve símbolos, conteúdo livre e fios (coordenadas de
    /// origem).
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
    /// Prévia pura (sem IDs novos): representações e polilinhas dos fios já
    /// posicionadas por `base` → `placement`.
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
    /// Verificações estruturais baratas; a coerência completa é conferida
    /// inserindo num projeto de rascunho (`Project::check_macro_content`).
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
pub(super) fn placement_from_base(base: Point2D, placement: Transform2D) -> Result<Transform2D> {
    Transform2D::new(Point2D::new(-base.x(), -base.y())?, 0.0, false, false)?.then(placement)
}

/// Macro de janela (trecho inserido na página) ou de página (página nova).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MacroKind {
    Window,
    Page {
        name: Text,
        kind: PageKind,
        size: Size2D,
    },
}

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
    /// Ponto de inserção (coordenadas de origem), que fica sob o cursor.
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
    /// Validação completa: campos, estrutura e inserção num projeto de
    /// rascunho (referências internas fechadas e geometria coerente).
    pub fn validate(&self) -> Result<()> {
        self.check_fields()?;
        self.content.check_structure()?;
        Project::check_macro_content(&self.content)
    }
    pub fn with_description(mut self, description: impl Into<String>) -> Result<Self> {
        self.description = description.into();
        self.check_fields()?;
        Ok(self)
    }
    pub fn with_category(mut self, category: impl Into<String>) -> Result<Self> {
        self.category = category.into().trim().to_owned();
        self.check_fields()?;
        Ok(self)
    }
    pub fn with_translation(
        mut self,
        language: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<Self> {
        self.translations
            .insert(Text::new(language)?, Text::new(description)?);
        Ok(self)
    }
    /// Nova versão da mesma macro (versão + 1).
    pub fn next_version(mut self) -> Self {
        self.version = self.version.saturating_add(1);
        self
    }
    pub fn with_version(mut self, version: u32) -> Result<Self> {
        self.version = version;
        self.check_fields()?;
        Ok(self)
    }
    pub fn with_name(mut self, name: impl Into<String>) -> Result<Self> {
        self.name = Text::new(name)?;
        Ok(self)
    }
    pub fn with_id(mut self, id: impl Into<String>) -> Result<Self> {
        self.id = MacroId::new(id)?;
        Ok(self)
    }
    pub fn id(&self) -> &MacroId {
        &self.id
    }
    pub fn name(&self) -> &Text {
        &self.name
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn translations(&self) -> &BTreeMap<Text, Text> {
        &self.translations
    }
    pub fn category(&self) -> &str {
        &self.category
    }
    pub fn version(&self) -> u32 {
        self.version
    }
    pub fn kind(&self) -> &MacroKind {
        &self.kind
    }
    pub fn base(&self) -> Point2D {
        self.base
    }
    pub fn content(&self) -> &MacroContent {
        &self.content
    }
    pub fn is_page(&self) -> bool {
        matches!(self.kind, MacroKind::Page { .. })
    }
}
