// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Graphical occurrences independent of logical entities; start with Representation.
use super::*;
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};

/// Associação elétrica explícita, opcional somente para ocorrências de símbolos.
/// Connection continua sem propriedades técnicas genéricas/EntityRef.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RepresentationTarget {
    Unassociated,
    Device(DeviceId),
    Function(FunctionId),
    Component(ComponentId),
    Terminal(TerminalId),
    ConnectionPoint(ConnectionPointId),
    Connection(ConnectionId),
    Conductor(ConductorId),
    Cable(CableId),
    CableCore(CableCoreId),
    TerminalStrip(TerminalStripId),
}

/// Estilo documental sem biblioteca gráfica ou propriedades elétricas.
/// Camadas maiores são desenhadas depois; empate pode ser desfeito por ID.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisualProperties {
    pub visible: bool,
    pub layer: i32,
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
    /// Símbolo é uma ocorrência documental, sem criação implícita elétrica.
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
    pub fn symbol(&self) -> Option<&SymbolDefinitionId> {
        self.symbol.as_ref()
    }
    pub fn graphic(&self) -> Option<&PageGraphic> {
        self.graphic.as_ref()
    }
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
    pub fn duplicate_on(&self, page: PageId, transform: Transform2D) -> Result<Self> {
        let mut value = self.clone();
        value.id = RepresentationId::new();
        value.page_id = page;
        value.transform = transform;
        value.bounds()?;
        Ok(value)
    }
    pub fn relocated(&self, page: PageId, transform: Transform2D) -> Result<Self> {
        let mut value = self.clone();
        value.page_id = page;
        value.transform = transform;
        value.bounds()?;
        Ok(value)
    }
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
    pub fn id(&self) -> RepresentationId {
        self.id
    }
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }
    pub fn page_id(&self) -> PageId {
        self.page_id
    }
    pub fn target(&self) -> RepresentationTarget {
        self.target
    }
    pub fn transform(&self) -> Transform2D {
        self.transform
    }
    pub fn local_bounds(&self) -> Option<Rect2D> {
        self.local_bounds
    }
    pub fn visual(&self) -> &VisualProperties {
        &self.visual
    }
    pub fn with_visual(mut self, visual: VisualProperties) -> Self {
        self.visual = visual;
        self
    }
    pub fn with_local_bounds(mut self, bounds: Rect2D) -> Result<Self> {
        self.local_bounds = Some(bounds);
        self.bounds()?;
        Ok(self)
    }
    /// AABB em coordenadas da página. Sem geometria, somente a âncora é conhecida.
    pub fn bounds(&self) -> Result<Rect2D> {
        match self.local_bounds {
            Some(bounds) => self.transform.transform_bounds(bounds),
            None => Rect2D::new(self.transform.translation(), Size2D::new(0.0, 0.0)?),
        }
    }
}

/// Folha e conteúdo separados. Não definem limites de pan/zoom do viewport.
/// O conteúdo inclui representações invisíveis e pode estar fora da folha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DocumentBounds {
    pub sheet: Rect2D,
    pub content: Option<Rect2D>,
}
