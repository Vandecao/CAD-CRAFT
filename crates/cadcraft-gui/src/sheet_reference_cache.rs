// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Revision-scoped derived sheet references; start with SheetReferenceCache::get.
use cad_craft::domain::*;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Default)]
pub(crate) struct SheetReferenceCache {
    scope: Option<(ProjectId, u64)>,
    pages: BTreeMap<PageId, Arc<[SheetReference]>>,
}

impl SheetReferenceCache {
    pub(crate) fn get(
        &mut self,
        project: &Project,
        revision: u64,
        page: PageId,
    ) -> Arc<[SheetReference]> {
        let scope = (project.id(), revision);
        // A page renumbering can change references on every other page. Invalidate
        // the complete derived set on any edit, undo, redo or project replacement.
        if self.scope != Some(scope) {
            self.pages.clear();
            self.scope = Some(scope);
        }
        Arc::clone(
            self.pages
                .entry(page)
                .or_insert_with(|| project.sheet_references(page).into()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        editing::{Edit, GraphicEdit},
        state::Session,
    };

    fn fixture() -> (Session, PageId, PageId) {
        let mut project = crate::connection_demo::project().unwrap();
        let source = project.pages().next().unwrap().id();
        let first = project
            .representations_on_page(source)
            .unwrap()
            .into_iter()
            .find(|r| r.symbol().is_some())
            .unwrap();
        let definition = project
            .symbol_definition(first.symbol().unwrap())
            .unwrap()
            .clone();
        let function = project.function_for_target(first.target()).unwrap();
        let device = project.function(function).unwrap().device_id();
        let target = project
            .insert_page(
                Page::new(
                    project.id(),
                    project.page(source).unwrap().document_id(),
                    "Reference target",
                    "99",
                    PageKind::MultiLine,
                    PaperFormat::A3.size(true),
                )
                .unwrap(),
            )
            .unwrap();
        project
            .place_symbol(SymbolPlacement {
                page: target,
                definition,
                transform: Transform2D::new(Point2D::new(50., 50.).unwrap(), 0., false, false)
                    .unwrap(),
                device: PlacementDevice::Existing(device),
                function_kind: FunctionKind::AuxiliaryContact,
            })
            .unwrap();
        let mut session = Session::default();
        session.replace(project, true).unwrap();
        (session, source, target)
    }

    #[test]
    fn repeated_lookup_reuses_derived_references_without_rebuilding() {
        let (session, source, _) = fixture();
        let mut cache = SheetReferenceCache::default();
        let project = session.project().unwrap();
        let first = cache.get(project, session.revision, source);
        assert!(!first.is_empty());
        assert_eq!(&*first, project.sheet_references(source));
        assert!(
            Arc::ptr_eq(&first, &cache.get(project, session.revision, source)),
            "unchanged frames must reuse the derived reference allocation"
        );
    }

    #[test]
    fn renumbering_and_undo_invalidate_references_to_other_pages() {
        let (mut session, source, target) = fixture();
        let mut cache = SheetReferenceCache::default();
        let original = cache.get(session.project().unwrap(), session.revision, source);
        session
            .apply(Edit::PageNumber(target, "100".into()))
            .unwrap();
        let renamed = cache.get(session.project().unwrap(), session.revision, source);
        assert_ne!(original, renamed);
        assert_eq!(
            &*renamed,
            session.project().unwrap().sheet_references(source)
        );
        session.dispatch_graphic(GraphicEdit::Undo).unwrap();
        let restored = cache.get(session.project().unwrap(), session.revision, source);
        assert_eq!(original, restored);
        session.dispatch_graphic(GraphicEdit::Redo).unwrap();
        assert_eq!(
            renamed,
            cache.get(session.project().unwrap(), session.revision, source)
        );
    }

    #[test]
    fn replacement_project_does_not_reuse_equal_revision_entries() {
        let (session, source, _) = fixture();
        let (other, _, _) = fixture();
        let mut cache = SheetReferenceCache::default();
        assert!(!cache.get(session.project().unwrap(), 1, source).is_empty());
        assert!(cache.get(other.project().unwrap(), 1, source).is_empty());
    }
}
