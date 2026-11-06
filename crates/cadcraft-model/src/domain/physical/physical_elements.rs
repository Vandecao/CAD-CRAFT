// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Related physical entity IDs; start with PhysicalElements and its read-only accessors.
use super::*;

/// Snapshot of related physical IDs, sorted and unique. Query again after Project edits; this is
/// not an electrical network.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PhysicalElements {
    pub(in crate::domain) conductors: Vec<ConductorId>,
    pub(in crate::domain) cables: Vec<CableId>,
    pub(in crate::domain) cable_cores: Vec<CableCoreId>,
    pub(in crate::domain) terminals: Vec<TerminalId>,
    pub(in crate::domain) terminal_strips: Vec<TerminalStripId>,
}
impl PhysicalElements {
    /// Return the conductors in their stored order.
    pub fn conductors(&self) -> &[ConductorId] {
        &self.conductors
    }
    /// Return the cables in their stored order.
    pub fn cables(&self) -> &[CableId] {
        &self.cables
    }
    /// Return the cable cores in their stored order.
    pub fn cable_cores(&self) -> &[CableCoreId] {
        &self.cable_cores
    }
    /// Return the terminals in their stored order.
    pub fn terminals(&self) -> &[TerminalId] {
        &self.terminals
    }
    /// Return the terminal strips in their stored order.
    pub fn terminal_strips(&self) -> &[TerminalStripId] {
        &self.terminal_strips
    }
}
