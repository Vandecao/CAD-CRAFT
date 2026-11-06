// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! state support for the physical editor/connectors; start at the parent module.

use super::*;

#[derive(Default)]
pub(in crate::physical_editor) struct ConnectorDraft {
    pub selected: Option<ConnectorId>,
    pub contact: Option<TerminalId>,
    pub(in crate::physical_editor) new_device: Option<DeviceId>,
    pub(in crate::physical_editor) new_name: String,
    pub(in crate::physical_editor) new_gender: ConnectorGender,
    pub(in crate::physical_editor) new_count: String,
    pub(in crate::physical_editor) new_position: String,
    pub(in crate::physical_editor) name: String,
    pub(in crate::physical_editor) gender: ConnectorGender,
    pub(in crate::physical_editor) contact_name: String,
    pub(in crate::physical_editor) add_contact: String,
    pub(in crate::physical_editor) renumber_start: String,
    pub(in crate::physical_editor) partner: Option<ConnectorId>,
    pub(in crate::physical_editor) pairs: Vec<ContactPair>,
    pub(in crate::physical_editor) pair_first: Option<TerminalId>,
    pub(in crate::physical_editor) pair_second: Option<TerminalId>,
}

/// Draft of the specialization of the selected terminal, re-seeded from the
/// Core after every revision so Undo/Redo never leaves stale values.
#[derive(Default)]
pub(in crate::physical_editor) struct KindDraft {
    pub(in crate::physical_editor) seeded: Option<TerminalId>,
    pub(in crate::physical_editor) kind: TerminalKind,
    pub(in crate::physical_editor) positions: String,
    pub(in crate::physical_editor) side_a: String,
    pub(in crate::physical_editor) side_b: String,
    pub(in crate::physical_editor) levels: Vec<(Option<TerminalLevelId>, String, String)>,
    pub(in crate::physical_editor) disconnect_open: bool,
    pub(in crate::physical_editor) point: Option<ConnectionPointId>,
    pub(in crate::physical_editor) point_level: Option<TerminalLevelId>,
}

pub(in crate::physical_editor) fn parts(text: &str) -> Result<Vec<DesignationPart>> {
    text.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(DesignationPart::new)
        .collect()
}
pub(in crate::physical_editor) fn joined<'a>(
    values: impl Iterator<Item = &'a DesignationPart>,
) -> String {
    values
        .map(DesignationPart::as_str)
        .collect::<Vec<_>>()
        .join(", ")
}
/// Pure builder used by the GUI and its tests: draft → candidate terminal.
/// The Core validates the result; nothing is assumed from the kind.
pub(in crate::physical_editor) fn terminal_from_draft(
    base: &Terminal,
    d: &KindDraft,
) -> Result<Terminal> {
    let mut sides = BTreeMap::new();
    for p in parts(&d.side_a)? {
        sides.insert(p, TerminalSide::A);
    }
    for p in parts(&d.side_b)? {
        if sides.insert(p, TerminalSide::B).is_some() {
            return Err(DomainError::InvalidRelation(
                "posição declarada nos dois lados",
            ));
        }
    }
    let mut t = base.clone().with_kind(d.kind);
    if d.kind == TerminalKind::MultiLevel {
        let levels = d
            .levels
            .iter()
            .map(|(id, name, positions)| {
                let name = DesignationPart::new(name.trim())?;
                match id.and_then(|id| base.level(id)) {
                    Some(level) => level
                        .clone()
                        .with_designation(name)
                        .with_positions(parts(positions)?),
                    None => TerminalLevel::new(name, parts(positions)?),
                }
            })
            .collect::<Result<Vec<_>>>()?;
        t = t.with_levels(levels);
    } else {
        t = t.with_levels(vec![]).with_positions(parts(&d.positions)?)?;
    }
    let state = (d.kind == TerminalKind::Disconnect).then_some(if d.disconnect_open {
        DisconnectState::Open
    } else {
        DisconnectState::Closed
    });
    Ok(t.with_sides(sides).with_disconnect_state(state))
}

impl KindDraft {
    pub(in crate::physical_editor) fn seed(&mut self, t: &Terminal) {
        self.seeded = Some(t.id());
        self.kind = t.kind();
        self.positions = joined(t.positions());
        let side = |s| joined(t.sides().iter().filter(|(_, v)| **v == s).map(|(p, _)| p));
        self.side_a = side(TerminalSide::A);
        self.side_b = side(TerminalSide::B);
        self.levels = t
            .levels()
            .iter()
            .map(|l| {
                (
                    Some(l.id()),
                    l.designation().as_str().to_owned(),
                    joined(l.positions()),
                )
            })
            .collect();
        self.disconnect_open = t.disconnect_state() == Some(DisconnectState::Open);
        if self.point_level.is_some_and(|id| t.level(id).is_none()) {
            self.point_level = None;
        }
    }
}
