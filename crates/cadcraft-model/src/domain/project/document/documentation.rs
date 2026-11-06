// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Page metadata, ordering and removal; start with rename_page and ordered_pages.
use super::*;
use std::collections::BTreeSet;

impl Project {
    /// Return the optional identification; absence represents an unset value or relation.
    pub fn identification(&self) -> Option<&Text> {
        self.identification.as_ref()
    }
    /// Replace or clear the project documentary identification.
    pub fn set_identification(&mut self, identification: Option<Text>) {
        self.identification = identification;
    }

    /// Change the page name after validating the supplied text.
    pub fn rename_page(&mut self, id: PageId, name: impl Into<String>) -> Result<()> {
        let name = Text::new(name)?;
        self.page_mut(id)?.name = name;
        Ok(())
    }
    /// Change the documentary page number after validating the supplied text.
    pub fn identify_page(&mut self, id: PageId, number: impl Into<String>) -> Result<()> {
        let number = Text::new(number)?;
        self.page_mut(id)?.number = number;
        Ok(())
    }
    /// Replace the page kind and sheet dimensions.
    pub fn set_page_format(&mut self, id: PageId, kind: PageKind, size: Size2D) -> Result<()> {
        let page = self.page_mut(id)?;
        page.kind = kind;
        page.size = size;
        Ok(())
    }
    /// Replace the page functional and location classification.
    pub fn set_page_structure(&mut self, id: PageId, structure: DocumentStructure) -> Result<()> {
        self.page_mut(id)?.structure = structure;
        Ok(())
    }
    pub(in crate::domain::project) fn page_mut(&mut self, id: PageId) -> Result<&mut Page> {
        self.pages
            .get_mut(&id)
            .ok_or(DomainError::MissingEntity(EntityRef::Page(id)))
    }
    /// Explicit order followed by PageId; unordered pages come last, sorted by ID.
    pub fn ordered_pages(&self, document: DocumentId) -> Result<Vec<&Page>> {
        self.require(EntityRef::Document(document))?;
        let mut pages: Vec<_> = self.pages_in_document(document).collect();
        pages.sort_by_key(|p| (p.order().is_none(), p.order(), p.id()));
        Ok(pages)
    }
    /// Complete permutation within one document; validate everything before writing.
    pub fn reorder_pages(&mut self, document: DocumentId, order: &[PageId]) -> Result<()> {
        self.require(EntityRef::Document(document))?;
        let expected: BTreeSet<_> = self.pages_in_document(document).map(Page::id).collect();
        let given: BTreeSet<_> = order.iter().copied().collect();
        if expected != given || given.len() != order.len() || u32::try_from(order.len()).is_err() {
            return Err(DomainError::InvalidRelation(
                "ordenação exige todas as páginas do documento exatamente uma vez",
            ));
        }
        for (index, id) in order.iter().enumerate() {
            self.page_mut(*id)?.order = Some(index as u32);
        }
        Ok(())
    }
    /// Derived tree without records or identity for path groups.
    pub fn page_tree(&self, document: DocumentId, axis: StructureAxis) -> Result<Vec<PageGroup>> {
        let mut groups: BTreeMap<Vec<DesignationPart>, Vec<PageId>> = BTreeMap::new();
        for page in self.ordered_pages(document)? {
            let path = page.structure().path(axis);
            for depth in 0..path.len() {
                groups.entry(path[..depth].to_vec()).or_default();
            }
            groups.entry(path.to_vec()).or_default().push(page.id());
        }
        Ok(groups
            .into_iter()
            .map(|(path, pages)| PageGroup { path, pages })
            .collect())
    }
    /// No cascading removal. References owned by the page disappear with it.
    pub fn remove_page(&mut self, id: PageId) -> Result<Page> {
        self.require(EntityRef::Page(id))?;
        let reference = EntityRef::Page(id);
        if self.document_index.by_page.contains_key(&id) || self.external_page_reference(id) {
            return Err(DomainError::EntityInUse(reference));
        }
        self.pages
            .remove(&id)
            .ok_or(DomainError::MissingEntity(reference))
    }
    fn external_page_reference(&self, id: PageId) -> bool {
        self.properties_reference(EntityRef::Page(id), Some(id))
            || self
                .pages
                .values()
                .any(|p| p.id() != id && p.report().is_some_and(|r| r.filter.page == Some(id)))
    }
    /// Technical properties referring to `target`, excluding those of the ignored page.
    pub(in crate::domain::project) fn properties_reference(
        &self,
        target: EntityRef,
        skip_page: Option<PageId>,
    ) -> bool {
        std::iter::once(&self.properties)
            .chain(self.documents.values().map(Document::properties))
            .chain(
                self.pages
                    .values()
                    .filter(|p| Some(p.id()) != skip_page)
                    .map(Page::properties),
            )
            .chain(self.devices.values().map(Device::properties))
            .chain(self.functions.values().map(Function::properties))
            .chain(self.components.values().map(Component::properties))
            .chain(self.terminals.values().map(Terminal::properties))
            .chain(
                self.connection_points
                    .values()
                    .map(ConnectionPoint::properties),
            )
            .chain(self.conductors.values().map(Conductor::properties))
            .chain(self.cables.values().map(Cable::properties))
            .chain(self.cable_cores.values().map(CableCore::properties))
            .chain(self.terminal_strips.values().map(TerminalStrip::properties))
            .any(|props| props.references().any(|r| r == target))
    }
}
