// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Single-line link edits and integrity; start with Project::edit_single_line.
use super::*;

/// Create or delete a single-line link.
#[derive(Clone, Debug, PartialEq)]
pub enum SingleLineEdit {
    /// Register a new link.
    Create(SingleLineLink),
    /// Remove a link.
    Delete(SingleLineId),
}

/// Link before and after an edit, for exact undo.
#[derive(Clone, Debug, PartialEq)]
pub struct SingleLineDelta {
    before: Option<SingleLineLink>,
    after: Option<SingleLineLink>,
}
impl SingleLineDelta {
    /// Link created by the edit, if any.
    pub fn created(&self) -> Option<SingleLineId> {
        self.after.as_ref().map(SingleLineLink::id)
    }
}

impl Project {
    /// Single-line links in ID order.
    pub fn single_lines(&self) -> impl Iterator<Item = &SingleLineLink> {
        self.single_lines.values()
    }
    /// Find a single-line link by ID.
    pub fn single_line(&self, id: SingleLineId) -> Option<&SingleLineLink> {
        self.single_lines.get(&id)
    }
    /// Apply a validated, reversible single-line edit.
    pub fn edit_single_line(&mut self, edit: SingleLineEdit) -> Result<SingleLineDelta> {
        let delta = match edit {
            SingleLineEdit::Create(link) => {
                if self.single_lines.contains_key(&link.id()) {
                    return Err(DomainError::InvalidRelation("conexão unifilar duplicada"));
                }
                SingleLineDelta {
                    before: None,
                    after: Some(link),
                }
            }
            SingleLineEdit::Delete(id) => SingleLineDelta {
                before: Some(
                    self.single_line(id)
                        .ok_or(DomainError::InvalidRelation("conexão unifilar inexistente"))?
                        .clone(),
                ),
                after: None,
            },
        };
        self.restore_single_line(&delta, true)?;
        Ok(delta)
    }
    /// Replay a single-line edit; forward reapplies it and false restores the
    /// prior state. Reject a mismatched expected state.
    pub fn restore_single_line(&mut self, delta: &SingleLineDelta, forward: bool) -> Result<()> {
        let (expected, target) = if forward {
            (&delta.before, &delta.after)
        } else {
            (&delta.after, &delta.before)
        };
        let id = expected
            .as_ref()
            .or(target.as_ref())
            .map(SingleLineLink::id)
            .ok_or(DomainError::InvalidRelation("edição unifilar vazia"))?;
        if self.single_lines.get(&id) != expected.as_ref() {
            return Err(DomainError::InvalidRelation(
                "conexão unifilar divergente do histórico",
            ));
        }
        let mut candidate = self.clone();
        match target {
            Some(link) => candidate.single_lines.insert(id, link.clone()),
            None => candidate.single_lines.remove(&id),
        };
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }
    /// Links reference existing representations on their own page.
    pub(in crate::domain::project) fn validate_single_lines(&self) -> Result<()> {
        for link in self.single_lines.values() {
            self.check_project(link.project_id())?;
            link.validate()?;
            self.require(EntityRef::Page(link.page_id()))?;
            for end in [link.source(), link.target()] {
                let r = self
                    .representation(end)
                    .ok_or(DomainError::MissingRepresentation(end))?;
                if r.page_id() != link.page_id() {
                    return Err(DomainError::InvalidRelation(
                        "conexão unifilar liga representações de outra página",
                    ));
                }
            }
        }
        Ok(())
    }
    /// Links that end on representation `id`; they block its removal.
    pub fn single_lines_of(&self, id: RepresentationId) -> Vec<SingleLineId> {
        self.single_lines
            .values()
            .filter(|l| l.source() == id || l.target() == id)
            .map(SingleLineLink::id)
            .collect()
    }
}
