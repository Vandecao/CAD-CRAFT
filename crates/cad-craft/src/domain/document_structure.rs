// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::{DesignationPart, PageId};
use serde::{Deserialize, Serialize};

/// Classificação documental, independente de FunctionId/DeviceId.
/// Caminhos vazios indicam ausência de classificação.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentStructure {
    functional_path: Vec<DesignationPart>,
    location_path: Vec<DesignationPart>,
}
impl DocumentStructure {
    pub fn new(functional_path: Vec<DesignationPart>, location_path: Vec<DesignationPart>) -> Self {
        Self {
            functional_path,
            location_path,
        }
    }
    pub fn functional_path(&self) -> &[DesignationPart] {
        &self.functional_path
    }
    pub fn location_path(&self) -> &[DesignationPart] {
        &self.location_path
    }
    pub fn path(&self, axis: StructureAxis) -> &[DesignationPart] {
        match axis {
            StructureAxis::Function => self.functional_path(),
            StructureAxis::Location => self.location_path(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructureAxis {
    Function,
    Location,
}

/// Nó de árvore em sequência pré-ordenada. O pai é o caminho sem a última parte.
/// `pages` contém somente folhas diretamente classificadas nesse caminho.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageGroup {
    pub path: Vec<DesignationPart>,
    pub pages: Vec<PageId>,
}
