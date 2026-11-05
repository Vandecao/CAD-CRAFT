// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::*;
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageKind {
    MultiLine,
    SingleLine,
    Overview,
    PanelLayout,
    TechnicalDocumentation,
    Other(Text),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageOrientation {
    Portrait,
    Landscape,
    Square,
}

/// Referência elétrica na página; sem símbolo, posição ou representação gráfica.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PageElement {
    Component(ComponentId),
    Function(FunctionId),
}
impl From<PageElement> for EntityRef {
    fn from(e: PageElement) -> Self {
        match e {
            PageElement::Component(id) => Self::Component(id),
            PageElement::Function(id) => Self::Function(id),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "PageData")]
pub struct Page {
    pub(super) id: PageId,
    pub(super) project_id: ProjectId,
    document_id: DocumentId,
    pub(super) name: Text,
    pub(super) number: Text,
    pub(super) kind: PageKind,
    pub(super) size: Size2D,
    pub(super) order: Option<u32>,
    pub(super) structure: DocumentStructure,
    pub(super) metadata: Metadata,
    pub(super) elements: BTreeSet<PageElement>,
    pub(super) properties: TechnicalProperties,
    /// Etapa 13: moldura própria; `None` usa a moldura padrão do projeto.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) frame: Option<FrameTemplateId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) report: Option<ReportDefinition>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PageData {
    id: PageId,
    project_id: ProjectId,
    document_id: DocumentId,
    name: Text,
    number: Text,
    kind: PageKind,
    size: Size2D,
    #[serde(default)]
    order: Option<u32>,
    #[serde(default)]
    structure: DocumentStructure,
    metadata: Metadata,
    elements: Vec<PageElement>,
    properties: TechnicalProperties,
    #[serde(default)]
    frame: Option<FrameTemplateId>,
    #[serde(default)]
    report: Option<ReportDefinition>,
}
impl TryFrom<PageData> for Page {
    type Error = DomainError;
    fn try_from(p: PageData) -> Result<Self> {
        let mut elements = BTreeSet::new();
        for element in p.elements {
            if !elements.insert(element) {
                return Err(DomainError::DuplicateReference(element.into()));
            }
        }
        Ok(Self {
            id: p.id,
            project_id: p.project_id,
            document_id: p.document_id,
            name: p.name,
            number: p.number,
            kind: p.kind,
            size: p.size,
            order: p.order,
            structure: p.structure,
            metadata: p.metadata,
            elements,
            properties: p.properties,
            frame: p.frame,
            report: p.report,
        })
    }
}
impl Page {
    pub fn new(
        project_id: ProjectId,
        document_id: DocumentId,
        name: impl Into<String>,
        number: impl Into<String>,
        kind: PageKind,
        size: Size2D,
    ) -> Result<Self> {
        Ok(Self {
            id: PageId::new(),
            project_id,
            document_id,
            name: Text::new(name)?,
            number: Text::new(number)?,
            kind,
            size,
            order: None,
            structure: DocumentStructure::default(),
            metadata: Metadata::new(),
            elements: BTreeSet::new(),
            properties: TechnicalProperties::default(),
            frame: None,
            report: None,
        })
    }
    pub fn id(&self) -> PageId {
        self.id
    }
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }
    pub fn document_id(&self) -> DocumentId {
        self.document_id
    }
    pub fn name(&self) -> &Text {
        &self.name
    }
    pub fn number(&self) -> &Text {
        &self.number
    }
    pub fn kind(&self) -> &PageKind {
        &self.kind
    }
    pub fn size(&self) -> Size2D {
        self.size
    }
    pub fn order(&self) -> Option<u32> {
        self.order
    }
    pub fn structure(&self) -> &DocumentStructure {
        &self.structure
    }
    pub fn orientation(&self) -> PageOrientation {
        if self.size.width() > self.size.height() {
            PageOrientation::Landscape
        } else if self.size.width() < self.size.height() {
            PageOrientation::Portrait
        } else {
            PageOrientation::Square
        }
    }
    /// Etapa 13: moldura própria da página (validada ao inserir no projeto).
    pub fn with_frame(mut self, frame: Option<FrameTemplateId>) -> Self {
        self.frame = frame;
        self
    }
    pub fn frame(&self) -> Option<FrameTemplateId> {
        self.frame
    }
    pub fn report(&self) -> Option<&ReportDefinition> {
        self.report.as_ref()
    }
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    pub fn elements(&self) -> impl Iterator<Item = PageElement> + '_ {
        self.elements.iter().copied()
    }
    pub fn properties(&self) -> &TechnicalProperties {
        &self.properties
    }
    pub fn with_properties(mut self, properties: TechnicalProperties) -> Self {
        self.properties = properties;
        self
    }
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) -> Result<()> {
        self.metadata.insert(Text::new(key)?, value.into());
        Ok(())
    }
}
