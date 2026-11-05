// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 12: edições documentais reversíveis (documentos e páginas).
//!
//! Cada delta guarda a entidade completa ou as ordens anteriores/posteriores;
//! a reversão confere o estado esperado antes de escrever, sem clonar Project.
use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum DocumentEdit {
    InsertDocument(Document),
    InsertPage(Box<Page>),
    RemovePage(PageId),
    ReorderPages {
        document: DocumentId,
        order: Vec<PageId>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum DocumentDelta {
    DocumentInserted(Document),
    PageInserted(Page),
    PageRemoved(Page),
    Order {
        document: DocumentId,
        before: Vec<(PageId, Option<u32>)>,
        after: Vec<(PageId, Option<u32>)>,
    },
}
impl DocumentDelta {
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Order { before, after, .. } if before == after)
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Order { after, .. } => after.len().max(1),
            _ => 1,
        }
    }
}

fn history_error() -> DomainError {
    DomainError::InvalidRelation("histórico documental não corresponde ao documento atual")
}

impl Project {
    /// Sem cascata: exige documento sem páginas e sem citações em propriedades.
    pub fn remove_document(&mut self, id: DocumentId) -> Result<Document> {
        let reference = EntityRef::Document(id);
        self.require(reference)?;
        if self.pages_in_document(id).next().is_some() || self.properties_reference(reference, None)
        {
            return Err(DomainError::EntityInUse(reference));
        }
        self.documents
            .remove(&id)
            .ok_or(DomainError::MissingEntity(reference))
    }

    fn page_orders(&self, document: DocumentId) -> Vec<(PageId, Option<u32>)> {
        self.pages_in_document(document)
            .map(|p| (p.id(), p.order()))
            .collect()
    }

    pub fn edit_document(&mut self, edit: DocumentEdit) -> Result<DocumentDelta> {
        match edit {
            DocumentEdit::InsertDocument(document) => {
                self.insert_document(document.clone())?;
                Ok(DocumentDelta::DocumentInserted(document))
            }
            DocumentEdit::InsertPage(page) => {
                self.insert_page((*page).clone())?;
                Ok(DocumentDelta::PageInserted(*page))
            }
            DocumentEdit::RemovePage(id) => self.remove_page(id).map(DocumentDelta::PageRemoved),
            DocumentEdit::ReorderPages { document, order } => {
                let before = self.page_orders(document);
                self.reorder_pages(document, &order)?;
                Ok(DocumentDelta::Order {
                    document,
                    before,
                    after: self.page_orders(document),
                })
            }
        }
    }

    pub fn restore_document_edit(&mut self, delta: &DocumentDelta, forward: bool) -> Result<()> {
        match delta {
            DocumentDelta::DocumentInserted(document) => {
                if forward {
                    self.insert_document(document.clone()).map(|_| ())
                } else {
                    if self.document(document.id()) != Some(document) {
                        return Err(history_error());
                    }
                    self.remove_document(document.id()).map(|_| ())
                }
            }
            DocumentDelta::PageInserted(page) | DocumentDelta::PageRemoved(page) => {
                let insert = forward == matches!(delta, DocumentDelta::PageInserted(_));
                if insert {
                    self.insert_page(page.clone()).map(|_| ())
                } else {
                    if self.page(page.id()) != Some(page) {
                        return Err(history_error());
                    }
                    self.remove_page(page.id()).map(|_| ())
                }
            }
            DocumentDelta::Order {
                document,
                before,
                after,
            } => {
                let (expected, target) = if forward {
                    (before, after)
                } else {
                    (after, before)
                };
                if &self.page_orders(*document) != expected {
                    return Err(history_error());
                }
                for (id, order) in target {
                    self.page_mut(*id)?.order = *order;
                }
                Ok(())
            }
        }
    }
}
