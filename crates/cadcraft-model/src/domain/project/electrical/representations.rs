// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Representation insertion and spatial queries; start with insert_representation.
use super::*;
use crate::document_index::DocumentIndex;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq)]
pub struct RepresentationChange {
    pub before: Option<Representation>,
    pub after: Option<Representation>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RepresentationDelta {
    project: ProjectId,
    changes: Vec<RepresentationChange>,
}
impl RepresentationDelta {
    pub fn len(&self) -> usize {
        self.changes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}

impl Project {
    /// Valida o lote completo antes de alterar o documento ou seu índice.
    pub fn edit_representations(
        &mut self,
        changes: Vec<RepresentationChange>,
    ) -> Result<RepresentationDelta> {
        let mut ids = BTreeSet::new();
        for change in &changes {
            let id = change
                .after
                .as_ref()
                .or(change.before.as_ref())
                .ok_or(DomainError::InvalidRelation(
                    "alteração de representação vazia",
                ))?
                .id();
            if !ids.insert(id) {
                return Err(DomainError::DuplicateRepresentation(id));
            }
            if change.before.as_ref().is_some_and(|r| r.id() != id)
                || change.after.as_ref().is_some_and(|r| r.id() != id)
            {
                return Err(DomainError::InvalidRelation("IDs diferentes na alteração"));
            }
            if self.representation(id) != change.before.as_ref() {
                return Err(DomainError::InvalidRelation(
                    "estado anterior da representação divergente",
                ));
            }
            if self.connection_route(id).is_some()
                || self.marker(id).is_some()
                || self.drawing_references_representation(id)
            {
                return Err(DomainError::InvalidRelation(
                    "rota, junção ou marcador não admitido no lote",
                ));
            }
            if let Some(after) = &change.after {
                self.check_representation(after)?;
            }
        }
        let changes: Vec<_> = changes
            .into_iter()
            .filter(|c| c.before != c.after)
            .collect();
        for change in &changes {
            if let Some(before) = &change.before {
                self.document_index.remove(before);
                self.representations.remove(&before.id());
            }
            if let Some(after) = &change.after {
                self.document_index.insert(after);
                self.representations.insert(after.id(), after.clone());
            }
        }
        Ok(RepresentationDelta {
            project: self.id,
            changes,
        })
    }
    pub fn restore_representations(
        &mut self,
        delta: &RepresentationDelta,
        forward: bool,
    ) -> Result<()> {
        self.check_project(delta.project)?;
        self.edit_representations(
            delta
                .changes
                .iter()
                .map(|c| {
                    if forward {
                        c.clone()
                    } else {
                        RepresentationChange {
                            before: c.after.clone(),
                            after: c.before.clone(),
                        }
                    }
                })
                .collect(),
        )?;
        Ok(())
    }
    /// Atomic graphical edit. Validate the complete batch before writing any
    /// transform; page/target IDs and the derived indices remain unchanged.
    pub fn transform_representations(
        &mut self,
        transforms: &[(RepresentationId, Transform2D)],
    ) -> Result<()> {
        self.transform_representations_with_drawing(transforms)?;
        Ok(())
    }
    pub fn representation(&self, id: RepresentationId) -> Option<&Representation> {
        self.representations.get(&id)
    }
    pub fn representations(&self) -> impl Iterator<Item = &Representation> {
        self.representations.values()
    }
    pub fn insert_representation(
        &mut self,
        representation: Representation,
    ) -> Result<RepresentationId> {
        let id = representation.id();
        if self.representations.contains_key(&id) {
            return Err(DomainError::DuplicateRepresentation(id));
        }
        self.check_representation(&representation)?;
        self.document_index.insert(&representation);
        self.representations.insert(id, representation);
        Ok(id)
    }
    pub fn remove_representation(&mut self, id: RepresentationId) -> Result<Representation> {
        if self.drawing_references_representation(id) || self.marker(id).is_some() {
            return Err(drawing_error());
        }
        let removed = self
            .representations
            .remove(&id)
            .ok_or(DomainError::MissingRepresentation(id))?;
        self.document_index.remove(&removed);
        Ok(removed)
    }
    /// Movimento entre páginas do mesmo projeto, validado antes de alterar índices.
    pub fn move_representation(
        &mut self,
        id: RepresentationId,
        page: PageId,
        transform: Transform2D,
    ) -> Result<()> {
        if self.drawing_references_representation(id) {
            return Err(drawing_error());
        }
        let mut updated = self
            .representation(id)
            .ok_or(DomainError::MissingRepresentation(id))?
            .clone();
        updated.page_id = page;
        updated.transform = transform;
        self.replace_representation(updated)
    }
    pub fn set_representation_bounds(
        &mut self,
        id: RepresentationId,
        bounds: Option<Rect2D>,
    ) -> Result<()> {
        if self.connection_routes.contains_key(&id) {
            return Err(drawing_error());
        }
        let mut updated = self
            .representation(id)
            .ok_or(DomainError::MissingRepresentation(id))?
            .clone();
        updated.local_bounds = bounds;
        self.replace_representation(updated)
    }
    pub fn set_representation_visual(
        &mut self,
        id: RepresentationId,
        visual: VisualProperties,
    ) -> Result<()> {
        if self
            .marker(id)
            .is_some_and(|m| visual.label.as_ref() != Some(m.name()))
        {
            return Err(DomainError::InvalidRelation(
                "altere o nome pelo comando de marcador",
            ));
        }
        let representation = self
            .representations
            .get_mut(&id)
            .ok_or(DomainError::MissingRepresentation(id))?;
        representation.visual = visual;
        Ok(())
    }
    fn replace_representation(&mut self, updated: Representation) -> Result<()> {
        self.check_representation(&updated)?;
        let old = self
            .representations
            .get(&updated.id())
            .ok_or(DomainError::MissingRepresentation(updated.id()))?;
        self.document_index.remove(old);
        self.document_index.insert(&updated);
        self.representations.insert(updated.id(), updated);
        Ok(())
    }
    pub(in crate::domain::project) fn require_representation_target(
        &self,
        target: RepresentationTarget,
    ) -> Result<()> {
        let reference = match target {
            RepresentationTarget::Unassociated => return Ok(()),
            RepresentationTarget::Device(id) => EntityRef::Device(id),
            RepresentationTarget::Function(id) => EntityRef::Function(id),
            RepresentationTarget::Component(id) => EntityRef::Component(id),
            RepresentationTarget::Terminal(id) => EntityRef::Terminal(id),
            RepresentationTarget::ConnectionPoint(id) => EntityRef::ConnectionPoint(id),
            RepresentationTarget::Connection(id) => {
                return self
                    .connection(id)
                    .map(|_| ())
                    .ok_or(DomainError::MissingConnection(id));
            }
            RepresentationTarget::Conductor(id) => EntityRef::Conductor(id),
            RepresentationTarget::Cable(id) => EntityRef::Cable(id),
            RepresentationTarget::CableCore(id) => EntityRef::CableCore(id),
            RepresentationTarget::TerminalStrip(id) => EntityRef::TerminalStrip(id),
        };
        self.require(reference)
    }
    pub(in crate::domain::project) fn check_representation(
        &self,
        representation: &Representation,
    ) -> Result<()> {
        self.check_project(representation.project_id())?;
        self.require(EntityRef::Page(representation.page_id()))?;
        self.require_representation_target(representation.target())?;
        if let Some(graphic) = representation.graphic() {
            let expected = graphic.bounds()?;
            let close = representation.local_bounds().is_some_and(|actual| {
                let a = actual.origin();
                let b = expected.origin();
                let asize = actual.size();
                let bsize = expected.size();
                (a.x() - b.x()).abs() < 1e-9
                    && (a.y() - b.y()).abs() < 1e-9
                    && (asize.width() - bsize.width()).abs() < 1e-9
                    && (asize.height() - bsize.height()).abs() < 1e-9
            });
            if representation.symbol().is_some()
                || representation.target() != RepresentationTarget::Unassociated
                || !close
            {
                return Err(DomainError::InvalidRelation("conteúdo livre inconsistente"));
            }
        }
        match representation.symbol() {
            Some(id) => {
                let definition = self
                    .symbol_definition(id)
                    .ok_or(DomainError::InvalidRelation("definição de símbolo ausente"))?;
                self.check_representation_definition(representation, definition)?;
            }
            None if representation.target() == RepresentationTarget::Unassociated
                && representation.graphic().is_none() =>
            {
                return Err(DomainError::InvalidRelation(
                    "representação sem alvo exige símbolo",
                ));
            }
            None => {}
        }
        representation.bounds()?;
        Ok(())
    }
    pub(in crate::domain::project) fn build_document_index(&self) -> Result<DocumentIndex> {
        for definition in self.symbol_definitions.values() {
            definition.validate()?;
        }
        let mut index = DocumentIndex::default();
        for representation in self.representations.values() {
            self.check_representation(representation)?;
            index.insert(representation);
        }
        Ok(index)
    }
}
