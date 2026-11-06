// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! state support for the part editor; start at the parent module.

use super::*;

#[derive(Default)]
pub struct PartEditor {
    pub open: bool,
    pub(in crate::part_editor) catalog: BTreeMap<PartId, Part>,
    pub(in crate::part_editor) seen_folder: String,
    pub(in crate::part_editor) selected_part: Option<PartId>,
    pub(in crate::part_editor) selected_device: Option<DeviceId>,
    pub(in crate::part_editor) selected_function: Option<FunctionId>,
    pub(in crate::part_editor) selected_point: Option<ConnectionPointId>,
    pub(in crate::part_editor) device_name: String,
    pub(in crate::part_editor) device_classification: String,
    pub(in crate::part_editor) device_properties: TechnicalProperties,
    pub(in crate::part_editor) device_known_key: usize,
    pub(in crate::part_editor) device_known_value: String,
    pub(in crate::part_editor) device_custom_key: String,
    pub(in crate::part_editor) device_custom_value: String,
    pub(in crate::part_editor) selected_terminal: Option<TerminalId>,
    pub(in crate::part_editor) selected_terminal_level: Option<TerminalLevelId>,
    pub(in crate::part_editor) id: String,
    pub(in crate::part_editor) manufacturer: String,
    pub(in crate::part_editor) code: String,
    pub(in crate::part_editor) description: String,
    pub(in crate::part_editor) classification: String,
    pub(in crate::part_editor) properties: TechnicalProperties,
    pub(in crate::part_editor) translations: BTreeMap<String, String>,
    pub(in crate::part_editor) known_key: usize,
    pub(in crate::part_editor) known_value: String,
    pub(in crate::part_editor) custom_key: String,
    pub(in crate::part_editor) custom_value: String,
    pub(in crate::part_editor) language: String,
    pub(in crate::part_editor) translated_description: String,
    pub(in crate::part_editor) model: Vec<PartFunction>,
    pub(in crate::part_editor) model_kind: usize,
    pub(in crate::part_editor) model_role: FunctionRole,
    pub(in crate::part_editor) model_points: String,
    pub(in crate::part_editor) quantity: u32,
    pub(in crate::part_editor) csv_path: String,
    pub(in crate::part_editor) bom_path: String,
    pub(in crate::part_editor) function_name: String,
    pub(in crate::part_editor) function_kind: usize,
    pub(in crate::part_editor) function_role: FunctionRole,
    pub(in crate::part_editor) point_name: String,
    pub(in crate::part_editor) message: String,
    pub(in crate::part_editor) locate: Option<RepresentationId>,
}

#[derive(Clone, Copy)]
pub(in crate::part_editor) enum Action {
    Save,
    ImportCsv,
    ExportCsv,
    ExportBom,
    Reload,
    UpdateProject,
    Assign,
    Remove(ComponentId),
    UpdateDevice,
    CreateFunction,
    UpdateFunction,
    RemoveFunction,
    CreatePoint,
    RenamePoint,
    RemovePoint,
    AssociateTerminal,
    Undo,
    Redo,
    Locate(RepresentationId),
}

pub(in crate::part_editor) fn kind(index: usize) -> FunctionKind {
    match index {
        0 => FunctionKind::Coil,
        1 => FunctionKind::MainContact,
        2 => FunctionKind::AuxiliaryContact,
        _ => FunctionKind::Other(Text::new("Outra").expect("constant")),
    }
}
pub(in crate::part_editor) fn kind_index(kind: &FunctionKind) -> usize {
    match kind {
        FunctionKind::Coil => 0,
        FunctionKind::MainContact => 1,
        FunctionKind::AuxiliaryContact => 2,
        FunctionKind::Other(_) => 3,
    }
}
pub(in crate::part_editor) fn kind_label(index: usize) -> &'static str {
    ["Bobina", "Contato principal", "Contato auxiliar", "Outra"][index.min(3)]
}
/// Readable function type; manufacturer parts may define custom types.
pub(in crate::part_editor) fn kind_text(kind: &FunctionKind) -> String {
    match kind {
        FunctionKind::Coil => "Bobina".into(),
        FunctionKind::MainContact => "Contato principal".into(),
        FunctionKind::AuxiliaryContact => "Contato auxiliar".into(),
        FunctionKind::Other(t) => t.as_str().to_owned(),
    }
}
impl PartEditor {
    pub(crate) fn install_catalog(&mut self, parts: Vec<Part>, folder: &str) {
        self.seen_folder = folder.to_owned();
        self.catalog = parts
            .into_iter()
            .map(|part| (part.id().clone(), part))
            .collect();
        if self
            .selected_part
            .as_ref()
            .is_some_and(|id| !self.catalog.contains_key(id))
        {
            self.selected_part = None;
        }
    }
    pub fn catalog(&self) -> impl Iterator<Item = &Part> {
        self.catalog.values()
    }
    pub fn take_locate(&mut self) -> Option<RepresentationId> {
        self.locate.take()
    }
}
