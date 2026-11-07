// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Project sessions, pages and snapshots; start with Session::replace.
use super::*;

impl Session {
    pub fn replace(&mut self, project: Project, demonstration: bool) -> Result<()> {
        project.validate()?;
        self.history.clear();
        self.reset_graphics();
        self.clipboard = None;
        self.project = Some(project);
        self.selection = Selection::Project;
        self.tabs.clear();
        self.active = None;
        self.dirty = false;
        self.demonstration = demonstration;
        // Releases the previous file lock; callers attach the new file afterwards.
        self.file = None;
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }
    pub fn close(&mut self) {
        self.history.clear();
        self.reset_graphics();
        self.clipboard = None;
        self.project = None;
        self.selection = Selection::None;
        self.tabs.clear();
        self.active = None;
        self.dirty = false;
        self.demonstration = false;
        self.file = None;
        self.revision = self.revision.wrapping_add(1);
    }
    pub fn select(&mut self, selection: Selection) {
        let valid = self.project.as_ref().is_some_and(|p| match selection {
            Selection::None | Selection::Project => true,
            Selection::Document(id) => p.document(id).is_some(),
            Selection::Page(id) => p.page(id).is_some(),
            Selection::Representation(id) => p.representation(id).is_some(),
            Selection::Electrical(target) => p.representations_for_entity(target).is_ok(),
        });
        if valid || selection == Selection::None {
            self.selection_revision = self.selection_revision.wrapping_add(1);
            self.graphics.cancel();
            self.graphics.selection.replace(match selection {
                Selection::Representation(id) => Some(id),
                _ => None,
            });
            self.selection = selection;
        }
    }
    pub fn selected_page(&self) -> Option<PageId> {
        match self.selection {
            Selection::Page(id) => Some(id),
            Selection::Representation(id) => self
                .project()?
                .representation(id)
                .map(Representation::page_id),
            _ => None,
        }
    }
    pub fn selected_document(&self) -> Option<DocumentId> {
        match self.selection {
            Selection::Document(id) => Some(id),
            _ => self
                .selected_page()
                .and_then(|id| self.project()?.page(id).map(Page::document_id))
                .or_else(|| self.project()?.documents().next().map(Document::id)),
        }
    }
    pub fn open_page(&mut self, id: PageId) -> Result<()> {
        if self.project().and_then(|p| p.page(id)).is_none() {
            return Err(DomainError::MissingEntity(EntityRef::Page(id)));
        }
        if !self.tabs.iter().any(|t| t.page == id) {
            self.tabs.push(PageTab {
                page: id,
                camera: Camera::default(),
                previous: None,
                needs_fit: true,
            });
        }
        self.active = Some(id);
        self.reset_graphics();
        self.selection = Selection::Page(id);
        Ok(())
    }
    pub fn close_tab(&mut self, id: PageId) {
        let index = self.tabs.iter().position(|t| t.page == id);
        self.tabs.retain(|t| t.page != id);
        if self.active == Some(id) {
            self.reset_graphics();
            self.active = index
                .and_then(|i| self.tabs.get(i.min(self.tabs.len().saturating_sub(1))))
                .map(|t| t.page);
            self.selection = self.active.map_or(
                if self.project.is_some() {
                    Selection::Project
                } else {
                    Selection::None
                },
                Selection::Page,
            );
        }
    }
    pub fn active_tab_mut(&mut self) -> Option<&mut PageTab> {
        let id = self.active?;
        self.tabs.iter_mut().find(|t| t.page == id)
    }
    pub fn apply(&mut self, edit: Edit) -> Result<()> {
        // A transform on the representation's own page goes through the
        // graphic path below, which reroutes attached wires; only a move to
        // another page uses the device edit (refused when wires reference it).
        let same_page = matches!(&edit, Edit::RepresentationTransform(id, page, _)
            if self
                .project()
                .and_then(|p| p.representation(*id))
                .is_some_and(|r| r.page_id() == *page));
        let edit = match edit {
            Edit::PageName(page, name) => {
                return self
                    .dispatch_graphic(GraphicEdit::Device(DeviceEdit::PageName {
                        page,
                        name: Text::new(name)?,
                    }))
                    .map(|_| ());
            }
            Edit::PageNumber(page, number) => {
                return self
                    .dispatch_graphic(GraphicEdit::Device(DeviceEdit::PageNumber {
                        page,
                        number: Text::new(number)?,
                    }))
                    .map(|_| ());
            }
            Edit::PageStructure(page, structure) => {
                return self
                    .dispatch_graphic(GraphicEdit::Device(DeviceEdit::PageStructure {
                        page,
                        structure,
                    }))
                    .map(|_| ());
            }
            Edit::RepresentationTransform(representation, page, transform) if !same_page => {
                return self
                    .dispatch_graphic(GraphicEdit::Device(DeviceEdit::MoveRepresentation {
                        representation,
                        page,
                        transform,
                    }))
                    .map(|_| ());
            }
            other => other,
        };
        if let Edit::RepresentationVisual(id, visual) = edit {
            return self
                .dispatch_graphic(GraphicEdit::Visual(id, visual))
                .map(|_| ());
        }
        if let Edit::RepresentationTransform(id, page, t) = &edit
            && self
                .project()
                .and_then(|p| p.representation(*id))
                .is_some_and(|r| r.page_id() == *page)
        {
            return self
                .dispatch_graphic(GraphicEdit::Set(vec![(*id, *t)]))
                .map(|_| ());
        }
        edit.apply(self.project_mut()?)?;
        self.history.clear();
        self.graphics.cancel();
        let ids: Vec<_> = self.graphics.selection.ids().iter().copied().collect();
        if !ids.is_empty() {
            self.select_graphics(ids);
        }
        self.changed();
        Ok(())
    }
    /// Documents and pages are reversible through the shared history.
    pub fn new_document(&mut self, name: &str) -> Result<DocumentId> {
        let p = self.project_mut()?;
        let document = Document::new(p.id(), name)?;
        let id = document.id();
        self.dispatch_graphic(GraphicEdit::Document(DocumentEdit::InsertDocument(
            document,
        )))?;
        self.selection = Selection::Document(id);
        Ok(id)
    }
    pub fn new_page(
        &mut self,
        document: DocumentId,
        name: &str,
        number: &str,
        kind: PageKind,
        size: Size2D,
    ) -> Result<PageId> {
        let p = self.project_mut()?;
        // Without a compatible default frame, use one matching the sheet size.
        let default_fits = p
            .default_frame()
            .and_then(|id| p.frame(id))
            .is_none_or(|f| frame_for_size(p, size).is_none_or(|m| m == f.id()));
        let own = (!default_fits).then(|| frame_for_size(p, size)).flatten();
        let page = Page::new(p.id(), document, name, number, kind, size)?.with_frame(own);
        let id = page.id();
        self.dispatch_graphic(GraphicEdit::Document(DocumentEdit::InsertPage(Box::new(
            page,
        ))))?;
        self.selection = Selection::Page(id);
        Ok(id)
    }
    pub fn delete_page(&mut self, id: PageId) -> Result<()> {
        self.dispatch_graphic(GraphicEdit::Document(DocumentEdit::RemovePage(id)))?;
        if self.selection == Selection::Page(id) {
            self.selection = Selection::Project;
        }
        Ok(())
    }
    pub fn reorder(&mut self, document: DocumentId, order: &[PageId]) -> Result<()> {
        self.dispatch_graphic(GraphicEdit::Document(DocumentEdit::ReorderPages {
            document,
            order: order.to_vec(),
        }))?;
        Ok(())
    }
    pub fn move_page(&mut self, id: PageId, direction: i32) -> Result<()> {
        let p = self.project_mut()?;
        let doc = p
            .page(id)
            .ok_or(DomainError::MissingEntity(EntityRef::Page(id)))?
            .document_id();
        let mut order: Vec<_> = p.ordered_pages(doc)?.iter().map(|p| p.id()).collect();
        if let Some(index) = order.iter().position(|p| *p == id) {
            let next = index
                .saturating_add_signed(direction as isize)
                .min(order.len() - 1);
            order.swap(index, next);
            self.reorder(doc, &order)?;
        }
        Ok(())
    }
    pub fn snapshot(&self) -> std::result::Result<String, String> {
        serde_json::to_string_pretty(
            self.project()
                .ok_or(crate::strings::NO_OPEN_PROJECT_TITLE)?,
        )
        .map_err(|e| e.to_string())
    }
    pub fn import_snapshot(&mut self, json: &str) -> std::result::Result<(), String> {
        if json.len() > 8 * 1024 * 1024 {
            return Err("Snapshot técnico excede o limite de 8 MiB".into());
        }
        let project: Project = serde_json::from_str(json).map_err(|e| e.to_string())?;
        self.replace(project, false).map_err(|e| e.to_string())
    }
}
