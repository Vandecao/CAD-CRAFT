// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::*;
use std::collections::BTreeSet;

impl Project {
    pub fn identification(&self) -> Option<&Text> {
        self.identification.as_ref()
    }
    pub fn set_identification(&mut self, identification: Option<Text>) {
        self.identification = identification;
    }

    pub fn rename_page(&mut self, id: PageId, name: impl Into<String>) -> Result<()> {
        let name = Text::new(name)?;
        self.page_mut(id)?.name = name;
        Ok(())
    }
    pub fn identify_page(&mut self, id: PageId, number: impl Into<String>) -> Result<()> {
        let number = Text::new(number)?;
        self.page_mut(id)?.number = number;
        Ok(())
    }
    pub fn set_page_format(&mut self, id: PageId, kind: PageKind, size: Size2D) -> Result<()> {
        let page = self.page_mut(id)?;
        page.kind = kind;
        page.size = size;
        Ok(())
    }
    pub fn set_page_structure(&mut self, id: PageId, structure: DocumentStructure) -> Result<()> {
        self.page_mut(id)?.structure = structure;
        Ok(())
    }
    pub(super) fn page_mut(&mut self, id: PageId) -> Result<&mut Page> {
        self.pages
            .get_mut(&id)
            .ok_or(DomainError::MissingEntity(EntityRef::Page(id)))
    }
    /// Ordem explícita, depois PageId; páginas sem ordem vêm ao final por ID.
    pub fn ordered_pages(&self, document: DocumentId) -> Result<Vec<&Page>> {
        self.require(EntityRef::Document(document))?;
        let mut pages: Vec<_> = self.pages_in_document(document).collect();
        pages.sort_by_key(|p| (p.order().is_none(), p.order(), p.id()));
        Ok(pages)
    }
    /// Permutação completa de um único documento; valida tudo antes de escrever.
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
    /// Árvore derivada, sem criar registros ou identidade para grupos de caminhos.
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
    /// Sem cascata. Referências da própria página desaparecem com ela.
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
    /// Propriedades técnicas que citam `target`, exceto as da página ignorada.
    pub(super) fn properties_reference(
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
