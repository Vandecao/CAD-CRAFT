// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Edições de identificação e referências derivadas; nenhum nome cria conectividade.
use super::*;

#[derive(Clone, Debug)]
pub enum DeviceEdit {
    SetDesignationLocked {
        device: DeviceId,
        locked: bool,
    },
    Update {
        device: DeviceId,
        designation: Designation,
        classification: Text,
        properties: TechnicalProperties,
    },
    Rename {
        device: DeviceId,
        designation: Designation,
    },
    SetClassification {
        device: DeviceId,
        classification: Text,
    },
    RenameFunction {
        function: FunctionId,
        designation: Designation,
    },
    AssociateFunction {
        function: FunctionId,
        device: DeviceId,
        role: FunctionRole,
    },
    SetMainFunction {
        device: DeviceId,
        function: Option<FunctionId>,
    },
    MoveRepresentation {
        representation: RepresentationId,
        page: PageId,
        transform: Transform2D,
    },
    PageName {
        page: PageId,
        name: Text,
    },
    PageNumber {
        page: PageId,
        number: Text,
    },
    PageStructure {
        page: PageId,
        structure: DocumentStructure,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::domain) struct IdentificationChange<T> {
    pub before: T,
    pub after: T,
}

/// Somente registros tocados. Replay exige pré-estado e restaura IDs originais.
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
    pub fn len(&self) -> usize {
        self.devices.len() + self.functions.len() + self.representations.len() + self.pages.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Fotografia consultada por IDs. Não é serializada nem autoritativa.
#[derive(Clone, Debug, PartialEq)]
pub struct DeviceReference {
    pub device: DeviceId,
    pub designation: Designation,
    pub function: Option<FunctionId>,
    pub function_designation: Option<Designation>,
    pub function_kind: Option<FunctionKind>,
    pub role: FunctionRole,
    pub representation: RepresentationId,
    pub page: PageId,
    pub position: Point2D,
    pub page_number: String,
    pub document_structure: DocumentStructure,
    pub terminals: Vec<String>,
    /// Etapa 13: `/página.coluna` com moldura; senão página e coordenadas.
    pub reference: String,
}

/// Entrada derivada do cadastro de contatos. Ausência de ocorrência mantém a linha sem referência.
#[derive(Clone, Debug, PartialEq)]
pub struct ContactMirrorRow {
    pub function: FunctionId,
    pub kind: FunctionKind,
    pub terminals: Vec<String>,
    pub references: Vec<DeviceReference>,
}
