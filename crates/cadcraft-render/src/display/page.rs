// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Page display list orchestration; start with the PageDisplayList extension trait.

use super::*;

/// Display list of a page. An extension trait because methods of [`Project`]
/// can only be declared in the model crate; the method name is unchanged.
pub trait PageDisplayList {
    /// Page display list: frame, content, references and junctions,
    /// in screen drawing order.
    fn page_display_list(&self, page: PageId) -> Result<DisplayList>;
}
impl PageDisplayList for Project {
    /// Page display list: frame, content, references and junctions,
    /// in screen drawing order.
    fn page_display_list(&self, page: PageId) -> Result<DisplayList> {
        let p = self
            .page(page)
            .ok_or(cadcraft_model::error::DomainError::MissingEntity(
                EntityRef::Page(page),
            ))?;
        let mut builder = Builder {
            project: self,
            items: vec![],
            layer: DisplayLayer::Frame,
        };
        match self.page_frame(page) {
            Some(frame) => builder.frame(page, frame),
            None => builder.provisional_frame(p),
        }
        if let Some(definition) = p.report() {
            builder.report(p, definition)?;
        }
        builder.representations(page);
        builder.references(page);
        builder.junctions(page);
        Ok(DisplayList {
            page,
            width: p.size().width(),
            height: p.size().height(),
            items: builder.items,
        })
    }
}
