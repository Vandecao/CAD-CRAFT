// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Typed GUI edit requests; start with Edit and route graphical edits through Session.
use cad_craft::{domain::*, error::Result};
/// Every graphical entry point uses Session::dispatch_graphic.
#[derive(Clone, Debug)]
pub enum GraphicEdit {
    Device(DeviceEdit),
    Marker(MarkerEdit),
    Drawing(ConnectionDrawingEdit),
    Translate(f64, f64),
    Rotate(f64),
    MirrorHorizontal,
    MirrorVertical,
    Set(Vec<(RepresentationId, Transform2D)>),
    Visual(RepresentationId, VisualProperties),
    InsertSymbol {
        definition: SymbolDefinitionId,
        target: Option<RepresentationTarget>,
        transform: Transform2D,
    },
    /// Etapa 12: inserção que cria dispositivo/função/pontos na mesma transação.
    PlaceSymbol {
        definition: SymbolDefinitionId,
        plan: crate::interaction::DevicePlan,
        transform: Transform2D,
    },
    /// Etapa 12: criação, exclusão e reordenação de documentos/páginas.
    Document(DocumentEdit),
    /// Etapa 13: molduras, atribuição por página e metadados.
    Frame(FrameEdit),
    Undo,
    Redo,
}
#[derive(Clone, Debug)]
pub enum Edit {
    ProjectDescription(String),
    ProjectIdentification(String),
    PageName(PageId, String),
    PageNumber(PageId, String),
    PageFormat(PageId, PageKind, Size2D),
    PageStructure(PageId, DocumentStructure),
    RepresentationTransform(RepresentationId, PageId, Transform2D),
    RepresentationVisual(RepresentationId, VisualProperties),
    Properties(EntityRef, TechnicalProperties),
}
impl Edit {
    pub fn apply(self, project: &mut Project) -> Result<()> {
        match self {
            Self::ProjectDescription(value) => {
                project.set_description(value);
                Ok(())
            }
            Self::ProjectIdentification(value) => {
                let value = if value.trim().is_empty() {
                    None
                } else {
                    Some(Text::new(value)?)
                };
                project.set_identification(value);
                Ok(())
            }
            Self::PageName(id, value) => project.rename_page(id, value),
            Self::PageNumber(id, value) => project.identify_page(id, value),
            Self::PageFormat(id, kind, size) => project.set_page_format(id, kind, size),
            Self::PageStructure(id, structure) => project.set_page_structure(id, structure),
            Self::RepresentationTransform(id, page, transform) => {
                project.move_representation(id, page, transform)
            }
            Self::RepresentationVisual(id, visual) => project.set_representation_visual(id, visual),
            Self::Properties(id, properties) => project.set_properties(id, properties),
        }
    }
}
