// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! state support for the physical editor; start at the parent module.

use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum PhysicalTab {
    #[default]
    Strips,
    Connectors,
    Cables,
    Wiring,
    Lists,
}

#[derive(Default)]
pub(in crate::physical_editor) struct PhysicalCache {
    pub(in crate::physical_editor) key: Option<(ProjectId, u64)>,
    pub(in crate::physical_editor) devices: Vec<(DeviceId, String)>,
    pub(in crate::physical_editor) strips: Vec<TerminalStrip>,
    pub(in crate::physical_editor) terminals: Vec<Terminal>,
    pub(in crate::physical_editor) points: Vec<ConnectionPoint>,
    pub(in crate::physical_editor) cables: Vec<Cable>,
    pub(in crate::physical_editor) cores: Vec<CableCore>,
    pub(in crate::physical_editor) wires: Vec<Conductor>,
    pub(in crate::physical_editor) bridges: Vec<TerminalBridge>,
    pub(in crate::physical_editor) connectors: Vec<Connector>,
    pub(in crate::physical_editor) matings: Vec<ConnectorMating>,
    pub(in crate::physical_editor) lists: PhysicalLists,
}

#[derive(Default)]
pub(crate) struct PhysicalEditor {
    pub open: bool,
    pub tab: PhysicalTab,
    pub(in crate::physical_editor) cache: PhysicalCache,
    pub(in crate::physical_editor) feedback: String,
    pub(in crate::physical_editor) selected_strip: Option<TerminalStripId>,
    pub(in crate::physical_editor) selected_terminal: Option<TerminalId>,
    pub(in crate::physical_editor) attach_candidate: Option<TerminalId>,
    pub(in crate::physical_editor) new_terminal_name: String,
    pub(in crate::physical_editor) selected_cable: Option<CableId>,
    pub(in crate::physical_editor) selected_core: Option<CableCoreId>,
    pub(in crate::physical_editor) selected_wire: Option<ConductorId>,
    pub(in crate::physical_editor) selected_device: Option<DeviceId>,
    pub(in crate::physical_editor) selected_connection: Option<ConnectionId>,
    pub(in crate::physical_editor) bridge_source: Option<ConnectionPointId>,
    pub(in crate::physical_editor) bridge_target: Option<ConnectionPointId>,
    pub(in crate::physical_editor) bridge_kind: bool,
    pub(in crate::physical_editor) new_strip: String,
    pub(in crate::physical_editor) strip_name: String,
    pub(in crate::physical_editor) terminal_name: String,
    pub(in crate::physical_editor) terminal_position: String,
    pub(in crate::physical_editor) cable_name: String,
    pub(in crate::physical_editor) new_cable_name: String,
    pub(in crate::physical_editor) new_cable_count: String,
    pub(in crate::physical_editor) cable_kind_index: usize,
    pub(in crate::physical_editor) cable_description: String,
    pub(in crate::physical_editor) cable_manufacturer: String,
    pub(in crate::physical_editor) cable_catalog: String,
    pub(in crate::physical_editor) cable_count: String,
    pub(in crate::physical_editor) core_name: String,
    pub(in crate::physical_editor) core_marking: String,
    pub(in crate::physical_editor) core_section: String,
    pub(in crate::physical_editor) wire_name: String,
    pub(in crate::physical_editor) wire_section: String,
    pub(in crate::physical_editor) wire_edit_name: String,
    pub(in crate::physical_editor) wire_edit_section: String,
    pub(in crate::physical_editor) wire_source: Option<(TerminalId, DesignationPart)>,
    pub(in crate::physical_editor) wire_target: Option<(TerminalId, DesignationPart)>,
    pub(in crate::physical_editor) wire_marking: String,
    pub(in crate::physical_editor) wire_to_link: Option<ConductorId>,
    pub(in crate::physical_editor) list_tab: usize,
    pub(in crate::physical_editor) renumber_start: String,
    pub(in crate::physical_editor) locate: Option<RepresentationId>,
    pub(in crate::physical_editor) list_selected: Option<RepresentationTarget>,
    pub(in crate::physical_editor) connector: connectors::ConnectorDraft,
    pub(in crate::physical_editor) kind: connectors::KindDraft,
}
