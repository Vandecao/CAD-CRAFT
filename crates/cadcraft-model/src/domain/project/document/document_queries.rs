// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Occurrence and page geometry queries; start with representations_on_page and document_bounds.
use super::*;

impl Project {
    /// Resultado por RepresentationId, incluindo ocorrências invisíveis.
    pub fn representations_on_page(&self, page: PageId) -> Result<Vec<&Representation>> {
        self.require(EntityRef::Page(page))?;
        Ok(self
            .document_index
            .by_page
            .get(&page)
            .into_iter()
            .flatten()
            .filter_map(|id| self.representation(*id))
            .collect())
    }
    pub fn representations_for_entity(
        &self,
        target: RepresentationTarget,
    ) -> Result<Vec<&Representation>> {
        self.require_representation_target(target)?;
        Ok(self
            .document_index
            .by_target
            .get(&target)
            .into_iter()
            .flatten()
            .filter_map(|id| self.representation(*id))
            .collect())
    }
    /// Outras ocorrências do MESMO alvo; não infere relação por rótulo/designação.
    pub fn related_representations(&self, id: RepresentationId) -> Result<Vec<&Representation>> {
        let target = self
            .representation(id)
            .ok_or(DomainError::MissingRepresentation(id))?
            .target();
        Ok(self
            .representations_for_entity(target)?
            .into_iter()
            .filter(|r| r.id() != id)
            .collect())
    }
    pub fn representation_location(&self, id: RepresentationId) -> Result<(PageId, Point2D)> {
        let r = self
            .representation(id)
            .ok_or(DomainError::MissingRepresentation(id))?;
        Ok((r.page_id(), r.transform().translation()))
    }
    /// A união pode exceder f64 mesmo se cada representação for válida: retorna erro.
    pub fn document_bounds(&self, page: PageId) -> Result<DocumentBounds> {
        let sheet = self
            .page(page)
            .ok_or(DomainError::MissingEntity(EntityRef::Page(page)))?;
        let sheet = Rect2D::new(Point2D::new(0.0, 0.0)?, sheet.size())?;
        let mut content: Option<Rect2D> = None;
        for representation in self.representations_on_page(page)? {
            let next = representation.bounds()?;
            content = Some(match content {
                None => next,
                Some(previous) => {
                    let min_x = previous.origin().x().min(next.origin().x());
                    let min_y = previous.origin().y().min(next.origin().y());
                    let max_x = previous.max().x().max(next.max().x());
                    let max_y = previous.max().y().max(next.max().y());
                    Rect2D::new(
                        Point2D::new(min_x, min_y)?,
                        Size2D::new(max_x - min_x, max_y - min_y)?,
                    )?
                }
            });
        }
        Ok(DocumentBounds { sheet, content })
    }
}
