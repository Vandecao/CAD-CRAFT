// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Derived document occurrence index; start with DocumentIndex.
use crate::domain::{PageId, Representation, RepresentationId, RepresentationTarget};
use std::collections::{BTreeMap, BTreeSet};

/// Rebuildable indexes; never serialized or used as an identity source.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct DocumentIndex {
    pub(crate) by_page: BTreeMap<PageId, BTreeSet<RepresentationId>>,
    pub(crate) by_target: BTreeMap<RepresentationTarget, BTreeSet<RepresentationId>>,
}
impl DocumentIndex {
    pub(crate) fn insert(&mut self, representation: &Representation) {
        self.by_page
            .entry(representation.page_id())
            .or_default()
            .insert(representation.id());
        if representation.target() != RepresentationTarget::Unassociated {
            self.by_target
                .entry(representation.target())
                .or_default()
                .insert(representation.id());
        }
    }
    pub(crate) fn remove(&mut self, representation: &Representation) {
        if let Some(ids) = self.by_page.get_mut(&representation.page_id()) {
            ids.remove(&representation.id());
            if ids.is_empty() {
                self.by_page.remove(&representation.page_id());
            }
        }
        if let Some(ids) = self.by_target.get_mut(&representation.target()) {
            ids.remove(&representation.id());
            if ids.is_empty() {
                self.by_target.remove(&representation.target());
            }
        }
    }
}
