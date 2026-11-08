// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Physical Project operations; start with the subject modules and delta types below.
use super::*;
pub(in crate::domain::project) mod connectors;
pub(in crate::domain::project) mod part_matching;
pub(in crate::domain::project) mod parts;
pub(in crate::domain::project) mod physical_edit;
pub(in crate::domain::project) mod physical_integrity;
pub(in crate::domain::project) mod physical_lists;
pub(in crate::domain::project) mod physical_queries;
pub(in crate::domain::project) mod physical_wiring;
pub(in crate::domain::project) mod strip_terminals;
pub use connectors::ContactStatus;
pub use part_matching::{PartFunctionMatch, PartFunctionPreview};
pub use parts::{BomRow, PartAssignmentDelta, PartRemovalDelta, PartUpdateDelta};
pub use physical_edit::{PhysicalDelta, PhysicalEdit};
pub use physical_lists::{
    CableListRow, ConductorListRow, ConnectorContactRow, ConnectorListRow, CoreListRow,
    PhysicalLists, StripListRow, TerminalListRow, WiringAssignmentRow,
};
pub use strip_terminals::{StripTerminalDelta, StripTerminalPlacement};
