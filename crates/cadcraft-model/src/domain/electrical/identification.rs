// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Identification edits and derived references; start with DeviceDelta. Names never create
//! connectivity.
use super::*;

/// Reversible changes to device identification and its documentary associations.
#[derive(Clone, Debug)]
pub enum DeviceEdit {
    /// Set designation locked.
    SetDesignationLocked {
        /// Associated logical device.
        device: DeviceId,
        /// Whether automatic numbering preserves the current value.
        locked: bool,
    },
    /// Replace the selected record with validated data.
    Update {
        /// Associated logical device.
        device: DeviceId,
        /// Documentary designation; does not imply electrical connectivity.
        designation: Designation,
        /// Documentary classification text.
        classification: Text,
        /// Typed technical properties.
        properties: TechnicalProperties,
    },
    /// Change the documentary name or designation.
    Rename {
        /// Associated logical device.
        device: DeviceId,
        /// Documentary designation; does not imply electrical connectivity.
        designation: Designation,
    },
    /// Set classification.
    SetClassification {
        /// Associated logical device.
        device: DeviceId,
        /// Documentary classification text.
        classification: Text,
    },
    /// Rename function.
    RenameFunction {
        /// Associated electrical function.
        function: FunctionId,
        /// Documentary designation; does not imply electrical connectivity.
        designation: Designation,
    },
    /// Associate function.
    AssociateFunction {
        /// Associated electrical function.
        function: FunctionId,
        /// Associated logical device.
        device: DeviceId,
        /// Explicitly declared functional role.
        role: FunctionRole,
    },
    /// Set main function.
    SetMainFunction {
        /// Associated logical device.
        device: DeviceId,
        /// Associated electrical function.
        function: Option<FunctionId>,
    },
    /// Move representation.
    MoveRepresentation {
        /// Graphical occurrence associated with this operation.
        representation: RepresentationId,
        /// Page identifier used by this operation or filter.
        page: PageId,
        /// Validated placement transform in page coordinates.
        transform: Transform2D,
    },
    /// Page name.
    PageName {
        /// Page identifier used by this operation or filter.
        page: PageId,
        /// Human-readable name.
        name: Text,
    },
    /// Page documentary number.
    PageNumber {
        /// Page identifier used by this operation or filter.
        page: PageId,
        /// Documentary sheet or wire number.
        number: Text,
    },
    /// Page structure.
    PageStructure {
        /// Page identifier used by this operation or filter.
        page: PageId,
        /// Functional and location document classification.
        structure: DocumentStructure,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::domain) struct IdentificationChange<T> {
    pub before: T,
    pub after: T,
}

/// Touched records only. Replay requires the expected prior state and restores original IDs.
#[derive(Clone, Debug, PartialEq)]
pub struct DeviceDelta {
    pub(in crate::domain) project: ProjectId,
    pub(in crate::domain) devices: Vec<IdentificationChange<Device>>,
    pub(in crate::domain) functions: Vec<IdentificationChange<Function>>,
    pub(in crate::domain) representations: Vec<IdentificationChange<Representation>>,
    pub(in crate::domain) pages: Vec<IdentificationChange<Page>>,
    pub(in crate::domain) legacy_conflicts: Vec<(DeviceId, DeviceId)>,
}
impl DeviceDelta {
    pub(in crate::domain) fn new(project: ProjectId) -> Self {
        Self {
            project,
            devices: vec![],
            functions: vec![],
            representations: vec![],
            pages: vec![],
            legacy_conflicts: vec![],
        }
    }
    /// Return the number of contained entries or entity changes recorded by this value.
    pub fn len(&self) -> usize {
        self.devices.len() + self.functions.len() + self.representations.len() + self.pages.len()
    }
    /// Whether this collection or recorded edit contains no effective entries or changes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Snapshot queried by IDs; neither serialized nor authoritative.
#[derive(Clone, Debug, PartialEq)]
pub struct DeviceReference {
    /// Associated logical device.
    pub device: DeviceId,
    /// Documentary designation; does not imply electrical connectivity.
    pub designation: Designation,
    /// Associated electrical function.
    pub function: Option<FunctionId>,
    /// Designation of the associated function, when present.
    pub function_designation: Option<Designation>,
    /// Kind of the associated electrical function.
    pub function_kind: Option<FunctionKind>,
    /// Explicitly declared functional role.
    pub role: FunctionRole,
    /// Graphical occurrence associated with this operation.
    pub representation: RepresentationId,
    /// Page identifier used by this operation or filter.
    pub page: PageId,
    /// Position in the containing page or terminal.
    pub position: Point2D,
    /// Documentary page number for display.
    pub page_number: String,
    /// Functional and location classification of the occurrence page.
    pub document_structure: DocumentStructure,
    /// Terminal entries associated with this record.
    pub terminals: Vec<String>,
    /// `/page.column` with a frame; otherwise page and coordinates.
    pub reference: String,
}

/// Derived contact registry entry. Missing occurrences leave the row without a reference.
#[derive(Clone, Debug, PartialEq)]
pub struct ContactMirrorRow {
    /// Associated electrical function.
    pub function: FunctionId,
    /// Declared category of this record or operation.
    pub kind: FunctionKind,
    /// Terminal entries associated with this record.
    pub terminals: Vec<String>,
    /// Derived graphical references to associated occurrences.
    pub references: Vec<DeviceReference>,
}
