// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Reversible document and page edits; start with DocumentDelta. Each delta stores full entities or
//! before/after orders. Replay checks expected state before writing and never clones Project.
use super::*;

/// Document or page insertion, removal or ordering operation.
#[derive(Clone, Debug, PartialEq)]
pub enum DocumentEdit {
    /// Insert document.
    InsertDocument(Document),
    /// Insert page.
    InsertPage(Box<Page>),
    /// Remove page.
    RemovePage(PageId),
    /// Reorder pages.
    ReorderPages {
        /// Document identifier used by this operation or filter.
        document: DocumentId,
        /// Explicit ordering used by this operation.
        order: Vec<PageId>,
    },
}

/// Recorded document edit suitable for exact undo and redo.
#[derive(Clone, Debug, PartialEq)]
pub enum DocumentDelta {
    /// Recorded insertion of a document.
    DocumentInserted(Document),
    /// Recorded insertion of a page.
    PageInserted(Page),
    /// Recorded removal of a page.
    PageRemoved(Page),
    /// Recorded order state before and after the operation.
    Order {
        /// Document identifier used by this operation or filter.
        document: DocumentId,
        /// Recorded state before the operation.
        before: Vec<(PageId, Option<u32>)>,
        /// Recorded state after the operation.
        after: Vec<(PageId, Option<u32>)>,
    },
}
impl DocumentDelta {
    /// Whether this collection or recorded edit contains no effective entries or changes.
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Order { before, after, .. } if before == after)
    }
    /// Return the number of contained entries or entity changes recorded by this value.
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
    /// No cascading removal: the document must have no pages or property references.
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

    /// Validate and apply the document operation, returning the recorded reversible change.
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

    /// Replay the recorded document edit change; forward reapplies it and false restores the prior
    /// state. Reject a mismatched expected state.
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
