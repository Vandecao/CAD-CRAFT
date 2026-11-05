// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Related physical entity IDs; start with PhysicalElements and its read-only accessors.
use super::*;

/// Fotografia de IDs físicos relacionados; membros únicos e ordenados por ID.
/// Reconsultar após alterações no Project; não representa uma rede elétrica.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PhysicalElements {
    pub(in crate::domain) conductors: Vec<ConductorId>,
    pub(in crate::domain) cables: Vec<CableId>,
    pub(in crate::domain) cable_cores: Vec<CableCoreId>,
    pub(in crate::domain) terminals: Vec<TerminalId>,
    pub(in crate::domain) terminal_strips: Vec<TerminalStripId>,
}
impl PhysicalElements {
    pub fn conductors(&self) -> &[ConductorId] {
        &self.conductors
    }
    pub fn cables(&self) -> &[CableId] {
        &self.cables
    }
    pub fn cable_cores(&self) -> &[CableCoreId] {
        &self.cable_cores
    }
    pub fn terminals(&self) -> &[TerminalId] {
        &self.terminals
    }
    pub fn terminal_strips(&self) -> &[TerminalStripId] {
        &self.terminal_strips
    }
}
