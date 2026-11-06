// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Document ordering and structural paths; start with the value types below.
use super::{DesignationPart, PageId};
use serde::{Deserialize, Serialize};

/// Document classification independent of FunctionId and DeviceId. Empty paths mean no
/// classification.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentStructure {
    functional_path: Vec<DesignationPart>,
    location_path: Vec<DesignationPart>,
}
impl DocumentStructure {
    /// Create a document classification from functional and location paths.
    pub fn new(functional_path: Vec<DesignationPart>, location_path: Vec<DesignationPart>) -> Self {
        Self {
            functional_path,
            location_path,
        }
    }
    /// Return the functional path in their stored order.
    pub fn functional_path(&self) -> &[DesignationPart] {
        &self.functional_path
    }
    /// Return the location path in their stored order.
    pub fn location_path(&self) -> &[DesignationPart] {
        &self.location_path
    }
    /// Return the classification path for the selected axis.
    pub fn path(&self, axis: StructureAxis) -> &[DesignationPart] {
        match axis {
            StructureAxis::Function => self.functional_path(),
            StructureAxis::Location => self.location_path(),
        }
    }
}

/// Functional or location axis of a document classification path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructureAxis {
    /// Functional classification axis.
    Function,
    /// Location classification axis.
    Location,
}

/// Tree node in preorder. Its parent is the path without the final segment. `pages` contains only
/// sheets classified directly at this path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageGroup {
    /// Ordered classification path segments.
    pub path: Vec<DesignationPart>,
    /// Pages belonging to this group or classification.
    pub pages: Vec<PageId>,
}
