// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! physical editor entry point. Read state for data, input for actions, and draw for UI.

use crate::state::{Selection, Session};
use cad_craft::{
    domain::*,
    error::{DomainError, Result},
};
use eframe::egui::{self, ComboBox, Context, ScrollArea, Ui};

mod cables;
mod connectors;
mod draw;
mod input;
mod lists;
mod state;
mod strips;
mod summary;
mod wiring;

use input::*;
use state::*;
pub(crate) use state::{PhysicalEditor, PhysicalTab};
pub(crate) use summary::{physical_summary, renumber_edits};

use lists::terminal_text;

// Test fixtures for the physical editor.
#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::panels::{NavigatorTab, physical_tree};

    fn project() -> (Project, TerminalStripId, CableId) {
        let mut p = Project::new("Navegador").unwrap();
        let device = p
            .insert_device(Device::new(p.id(), Designation::new("K1").unwrap(), "relé").unwrap())
            .unwrap();
        let strip = TerminalStrip::new(p.id(), DesignationPart::new("X1").unwrap());
        let strip_id = strip.id();
        let mut edits = vec![PhysicalEdit::UpsertStrip(strip.clone())];
        let mut order = vec![];
        for n in ["7", "3", "9"] {
            let t = Terminal::new(
                p.id(),
                TerminalOwner::Device(device),
                DesignationPart::new(n).unwrap(),
            );
            order.push(t.id());
            edits.push(PhysicalEdit::UpsertTerminal(t));
        }
        edits.push(PhysicalEdit::UpsertStrip(strip.with_terminals(order)));
        let cable = Cable::new(
            p.id(),
            DesignationPart::new("W1").unwrap(),
            CableKind::Control,
            2,
        )
        .unwrap();
        let cable_id = cable.id();
        edits.push(PhysicalEdit::UpsertCable(cable));
        for n in ["1", "PE"] {
            edits.push(PhysicalEdit::UpsertCore(CableCore::new(
                p.id(),
                cable_id,
                DesignationPart::new(n).unwrap(),
                ConductorMarking::identified(DesignationPart::new(n).unwrap()),
            )));
        }
        p.edit_physical(PhysicalEdit::Batch(edits)).unwrap();
        (p, strip_id, cable_id)
    }

    #[test]
    fn navigator_tree_is_hierarchical_and_follows_strip_order() {
        let (p, strip, cable) = project();
        let terminals = physical_tree(&p, NavigatorTab::Terminals);
        assert_eq!(
            terminals[0].target,
            Some(RepresentationTarget::TerminalStrip(strip))
        );
        assert_eq!(terminals[0].depth, 0);
        let children: Vec<_> = terminals[1..4].iter().map(|r| r.label.clone()).collect();
        assert!(children[0].starts_with("1. 7") && children[1].starts_with("2. 3"));
        assert!(terminals[1..4].iter().all(|r| r.depth == 1));
        let cables = physical_tree(&p, NavigatorTab::Cables);
        assert_eq!(cables[0].target, Some(RepresentationTarget::Cable(cable)));
        assert_eq!(cables.len(), 3, "cable plus its two cores, no PE invented");
        assert!(
            cables[1..]
                .iter()
                .all(|r| r.depth == 1 && r.label.contains("livre"))
        );
    }

    /// Terminal strip, terminal and levels; connector and contacts.
    #[test]
    fn navigator_shows_levels_and_connector_contacts_hierarchically() {
        let (mut p, strip, _) = project();
        let order = p.terminal_strip(strip).unwrap().terminals().to_vec();
        let device = p.devices().next().unwrap().id();
        let multilevel = p
            .terminal(order[0])
            .unwrap()
            .clone()
            .with_kind(TerminalKind::MultiLevel)
            .with_levels(vec![
                TerminalLevel::new(
                    DesignationPart::new("1").unwrap(),
                    vec![DesignationPart::new("a").unwrap()],
                )
                .unwrap(),
                TerminalLevel::new(
                    DesignationPart::new("2").unwrap(),
                    vec![DesignationPart::new("b").unwrap()],
                )
                .unwrap(),
            ]);
        let contact = Terminal::new(
            p.id(),
            TerminalOwner::Device(device),
            DesignationPart::new("1").unwrap(),
        );
        let contact_id = contact.id();
        p.edit_physical(PhysicalEdit::Batch(vec![
            PhysicalEdit::UpsertTerminal(multilevel),
            PhysicalEdit::UpsertTerminal(contact),
            PhysicalEdit::UpsertConnector(
                Connector::new(p.id(), DesignationPart::new("X5").unwrap())
                    .with_gender(ConnectorGender::Male)
                    .with_device(Some(device))
                    .with_contacts(vec![contact_id]),
            ),
        ]))
        .unwrap();
        let rows = physical_tree(&p, NavigatorTab::Terminals);
        let labels: Vec<_> = rows.iter().map(|r| (r.depth, r.label.as_str())).collect();
        assert!(labels[1].1.starts_with("1. 7 [multinível]"), "{labels:?}");
        assert_eq!(labels[2], (2, "Nível 1 (a)"));
        assert_eq!(labels[3], (2, "Nível 2 (b)"));
        let connector = rows
            .iter()
            .position(|r| {
                r.label
                    .starts_with("Conector X5 · macho · 1 contatos (1 livres)")
            })
            .expect("connector row");
        assert_eq!(rows[connector].depth, 0);
        assert_eq!(
            rows[connector].target,
            Some(RepresentationTarget::Device(device))
        );
        assert_eq!(rows[connector + 1].depth, 1);
        assert_eq!(
            rows[connector + 1].target,
            Some(RepresentationTarget::Terminal(contact_id))
        );
        assert!(
            rows[connector + 1]
                .label
                .contains("Contato 1 · posições - · livre")
        );
        assert!(
            rows.iter()
                .all(|r| !r.label.starts_with("Bornes sem régua")),
            "contacts are listed under their connector, not as loose terminals"
        );
        let summary = physical_summary(&p, RepresentationTarget::Terminal(order[0]));
        assert!(
            summary
                .iter()
                .any(|(k, v)| k == "Especialização" && v == "multinível")
        );
        assert!(summary.iter().any(|(k, v)| k == "Nível 2" && v == "b"));
        let summary = physical_summary(&p, RepresentationTarget::Terminal(contact_id));
        assert!(
            summary
                .iter()
                .any(|(k, v)| k == "Conector" && v == "X5 · macho")
        );
    }

    #[test]
    fn specialization_draft_round_trips_and_rejects_conflicting_sides() {
        let (p, strip, _) = project();
        let id = p.terminal_strip(strip).unwrap().terminals()[0];
        let base = p.terminal(id).unwrap().clone();
        let mut draft = connectors::KindDraft::default();
        draft.kind = TerminalKind::FeedThrough;
        draft.positions = "1, 2".into();
        draft.side_a = "1".into();
        draft.side_b = "2".into();
        let t = connectors::terminal_from_draft(&base, &draft).unwrap();
        assert!(t.check_specialization().is_ok());
        assert_eq!(t.id(), id);
        draft.side_b = "1, 2".into();
        assert!(connectors::terminal_from_draft(&base, &draft).is_err());
    }

    #[test]
    fn sequential_renumbering_keeps_ids_and_is_one_undoable_batch() {
        let (mut p, strip, _) = project();
        let order = p.terminal_strip(strip).unwrap().terminals().to_vec();
        let terminals: Vec<_> = p.terminals().cloned().collect();
        let before = p.clone();
        let delta = p
            .edit_physical(PhysicalEdit::Batch(
                renumber_edits(&terminals, &order, 1).unwrap(),
            ))
            .unwrap();
        let names: Vec<_> = order
            .iter()
            .map(|id| p.terminal(*id).unwrap().designation().as_str().to_owned())
            .collect();
        assert_eq!(names, ["1", "2", "3"]);
        assert_eq!(
            p.terminal_strip(strip).unwrap().terminals(),
            order.as_slice()
        );
        p.restore_physical(&delta, false).unwrap();
        assert_eq!(p, before);
        assert!(renumber_edits(&terminals, &order, u32::MAX).is_err());
    }

    #[test]
    fn properties_summary_reports_physical_layers_without_inventing_values() {
        let (p, strip, cable) = project();
        let rows = physical_summary(&p, RepresentationTarget::Cable(cable));
        assert!(
            rows.iter()
                .any(|(k, v)| k.starts_with("Veias") && v == "2/2/0")
        );
        assert!(rows.iter().any(|(k, v)| k == "Fabricante" && v == "-"));
        let core = p.cores_for_cable(cable).next().unwrap().id();
        let rows = physical_summary(&p, RepresentationTarget::CableCore(core));
        assert!(rows.iter().any(|(_, v)| v == "não informada"));
        assert!(rows.iter().any(|(_, v)| v == "livre"));
        let rows = physical_summary(&p, RepresentationTarget::TerminalStrip(strip));
        assert!(rows.iter().any(|(k, v)| k == "Bornes" && v == "3"));
    }
}
