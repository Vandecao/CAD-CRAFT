// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Sheet metadata and presentation settings; start with Page::new.
use super::*;
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Documentary purpose of a project sheet.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageKind {
    /// Multiline electrical schematic.
    MultiLine,
    /// Single-line electrical schematic.
    SingleLine,
    /// Overview documentation.
    Overview,
    /// Physical panel layout.
    PanelLayout,
    /// General technical documentation.
    TechnicalDocumentation,
    /// User-defined documentary category.
    Other(Text),
}

/// Orientation derived from a sheet width and height.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageOrientation {
    /// Height exceeds width.
    Portrait,
    /// Width exceeds height.
    Landscape,
    /// Width equals height.
    Square,
}

/// Electrical reference on a page, without symbol, position or graphical occurrence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PageElement {
    /// Explicit reference to a component.
    Component(ComponentId),
    /// Explicit reference to a logical function.
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

/// Project sheet with document membership, format and documentary metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "PageData")]
pub struct Page {
    pub(in crate::domain) id: PageId,
    pub(in crate::domain) project_id: ProjectId,
    document_id: DocumentId,
    pub(in crate::domain) name: Text,
    pub(in crate::domain) number: Text,
    pub(in crate::domain) kind: PageKind,
    pub(in crate::domain) size: Size2D,
    pub(in crate::domain) order: Option<u32>,
    pub(in crate::domain) structure: DocumentStructure,
    pub(in crate::domain) metadata: Metadata,
    pub(in crate::domain) elements: BTreeSet<PageElement>,
    pub(in crate::domain) properties: TechnicalProperties,
    /// Page frame override; `None` uses the project's default frame.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(in crate::domain) frame: Option<FrameTemplateId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(in crate::domain) report: Option<ReportDefinition>,
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
    /// Create a detached sheet with fresh identity, document membership and validated name and number.
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
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> PageId {
        self.id
    }
    /// Return the project identifier.
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }
    /// Return the document identifier.
    pub fn document_id(&self) -> DocumentId {
        self.document_id
    }
    /// Return the name.
    pub fn name(&self) -> &Text {
        &self.name
    }
    /// Borrow the documentary page number.
    pub fn number(&self) -> &Text {
        &self.number
    }
    /// Return the kind.
    pub fn kind(&self) -> &PageKind {
        &self.kind
    }
    /// Return dimensions in millimetres.
    pub fn size(&self) -> Size2D {
        self.size
    }
    /// Return the optional order; absence represents an unset value or relation.
    pub fn order(&self) -> Option<u32> {
        self.order
    }
    /// Return the structure.
    pub fn structure(&self) -> &DocumentStructure {
        &self.structure
    }
    /// Return the orientation.
    pub fn orientation(&self) -> PageOrientation {
        if self.size.width() > self.size.height() {
            PageOrientation::Landscape
        } else if self.size.width() < self.size.height() {
            PageOrientation::Portrait
        } else {
            PageOrientation::Square
        }
    }
    /// Page frame override, validated when registered in Project.
    pub fn with_frame(mut self, frame: Option<FrameTemplateId>) -> Self {
        self.frame = frame;
        self
    }
    /// Return the optional frame; absence represents an unset value or relation.
    pub fn frame(&self) -> Option<FrameTemplateId> {
        self.frame
    }
    /// Return the optional report; absence represents an unset value or relation.
    pub fn report(&self) -> Option<&ReportDefinition> {
        self.report.as_ref()
    }
    /// Return the metadata.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    /// Iterate or borrow the explicitly stored document elements.
    pub fn elements(&self) -> impl Iterator<Item = PageElement> + '_ {
        self.elements.iter().copied()
    }
    /// Return the properties.
    pub fn properties(&self) -> &TechnicalProperties {
        &self.properties
    }
    /// Return this value with properties replaced.
    pub fn with_properties(mut self, properties: TechnicalProperties) -> Self {
        self.properties = properties;
        self
    }
    /// Insert or replace a validated metadata entry.
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) -> Result<()> {
        self.metadata.insert(Text::new(key)?, value.into());
        Ok(())
    }
}
