// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Macro capture and insertion as single reversible transactions; start with
//! Project::capture_macro. Capture includes selected symbols, free content and markers, devices
//! with all functions and points, assigned parts, symbol definitions, port bindings, internal wires
//! and junctions whose routes are all included. Exclusions are reported. Insertion assigns fresh
//! UUID v7 IDs and free designations, transforms geometry and adds missing symbols and parts.
//! Conflicting project versions are retained and reported. Validate the whole project before
//! accepting.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

/// Content excluded or adapted during capture or insertion.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MacroReport {
    /// Wires with one endpoint outside the selection.
    pub dropped_routes: usize,
    /// Junctions with any route outside the selection.
    pub dropped_junctions: usize,
    /// Markers bound to points outside the selection.
    pub dropped_markers: usize,
    /// Physical engineering occurrences, including terminals, cables and conductors.
    pub dropped_physical: usize,
    /// Discarded point-to-terminal bindings.
    pub cleared_terminals: usize,
    /// Project symbols retained despite differing from the macro version.
    pub kept_symbols: Vec<SymbolDefinitionId>,
    /// Project parts retained despite differing from macro data.
    pub kept_parts: Vec<PartId>,
}
impl MacroReport {
    /// Whether the capture or insertion report contains no exclusions or adaptations.
    pub fn is_clean(&self) -> bool {
        *self == Self::default()
    }
    /// Short pt-BR text for the user interface.
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        let mut count = |n: usize, what: &str| {
            if n > 0 {
                parts.push(format!("{n} {what}"));
            }
        };
        count(self.dropped_routes, "fio(s) com ponta fora da seleção");
        count(self.dropped_junctions, "junção(ões) parciais");
        count(self.dropped_markers, "marcador(es) sem ponto na seleção");
        count(self.dropped_physical, "elemento(s) de engenharia física");
        count(self.cleared_terminals, "vínculo(s) de borne");
        let mut text = if parts.is_empty() {
            String::new()
        } else {
            format!("Descartados: {}.", parts.join(", "))
        };
        if !self.kept_symbols.is_empty() {
            text.push_str(&format!(
                " Mantida a versão do projeto de {} símbolo(s): {}.",
                self.kept_symbols.len(),
                self.kept_symbols
                    .iter()
                    .map(SymbolDefinitionId::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !self.kept_parts.is_empty() {
            text.push_str(&format!(
                " Mantidos os dados do projeto de {} peça(s): {}.",
                self.kept_parts.len(),
                self.kept_parts
                    .iter()
                    .map(PartId::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        text.trim().to_owned()
    }
}

/// Insertion location and transform. Apply `placement` after moving the base point to the origin;
/// its translation is the insertion point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MacroPlacement {
    /// Page identifier used by this operation or filter.
    pub page: PageId,
    /// Source-coordinate base point placed at the insertion point.
    pub base: Point2D,
    /// Validated placement transform in page coordinates.
    pub transform: Transform2D,
}

/// Entities written by insertion in their final form with new IDs.
#[derive(Clone, Debug, Default, PartialEq)]
struct Batch {
    definitions: Vec<SymbolDefinition>,
    parts: Vec<Part>,
    devices: Vec<Device>,
    functions: Vec<Function>,
    points: Vec<ConnectionPoint>,
    components: Vec<Component>,
    representations: Vec<Representation>,
    bindings: Vec<(PortRef, ConnectionPointId)>,
    markers: Vec<ElectricalMarker>,
    connections: Vec<Connection>,
    routes: Vec<ConnectionRoute>,
    junctions: Vec<ConnectionJunction>,
}
impl Batch {
    fn len(&self) -> usize {
        self.definitions.len()
            + self.parts.len()
            + self.devices.len()
            + self.functions.len()
            + self.points.len()
            + self.components.len()
            + self.representations.len()
            + self.bindings.len()
            + self.markers.len()
            + self.connections.len()
            + self.routes.len()
            + self.junctions.len()
    }
}

/// Reversible macro insertion or paste-with-wires result.
#[derive(Clone, Debug, PartialEq)]
pub struct MacroInsertDelta {
    project: ProjectId,
    page: Option<DocumentDelta>,
    batch: Batch,
}
impl MacroInsertDelta {
    /// Return the number of contained entries or entity changes recorded by this value.
    pub fn len(&self) -> usize {
        self.batch.len() + usize::from(self.page.is_some())
    }
    /// Whether this collection or recorded edit contains no effective entries or changes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Page created by a page macro.
    pub fn created_page(&self) -> Option<PageId> {
        match &self.page {
            Some(DocumentDelta::PageInserted(page)) => Some(page.id()),
            _ => None,
        }
    }
    /// Inserted visible occurrences, including symbols, free content, markers and wires, for selection
    /// after insertion.
    pub fn representation_ids(&self) -> Vec<RepresentationId> {
        self.batch
            .representations
            .iter()
            .map(Representation::id)
            .collect()
    }
    /// Return the inserted device IDs.
    pub fn device_ids(&self) -> Vec<DeviceId> {
        self.batch.devices.iter().map(Device::id).collect()
    }
    /// Return the inserted route occurrence IDs.
    pub fn route_ids(&self) -> Vec<RepresentationId> {
        self.batch.routes.iter().map(ConnectionRoute::id).collect()
    }
}

fn macro_error(reason: &'static str) -> DomainError {
    DomainError::InvalidRelation(reason)
}
fn json_error(error: serde_json::Error) -> DomainError {
    let _ = error;
    DomainError::InvalidRelation("conteúdo de macro inconsistente")
}

/// Designation prefix, such as K from K12.
fn prefix_of(device: &str) -> String {
    let prefix = device.trim_end_matches(|c: char| c.is_ascii_digit());
    if prefix.is_empty() {
        "D".to_owned()
    } else {
        prefix.to_owned()
    }
}

/// Recursively replace textual IDs according to the map.
fn remap(value: &mut serde_json::Value, map: &BTreeMap<String, String>) {
    match value {
        serde_json::Value::String(s) => {
            if let Some(new) = map.get(s.as_str()) {
                *s = new.clone();
            }
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(|v| remap(v, map)),
        serde_json::Value::Object(fields) => fields.values_mut().for_each(|v| remap(v, map)),
        _ => {}
    }
}

mod capture;
mod insertion;
