// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Domain report, numbering and library operations; start with Session dispatch methods.
use super::*;

impl Session {
    pub fn apply_report_delta(&mut self, delta: ReportDelta) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.apply_report_delta(p, delta)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    /// Etapa 24: troca o esquema de verificação (com Desfazer).
    pub fn set_check_scheme(&mut self, scheme: CheckScheme) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.check_scheme(p, scheme)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn generate_report(&mut self, request: &ReportRequest) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.generate_report(p, request)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn update_reports(&mut self, groups: &[ReportId]) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.update_reports(p, groups)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn set_numbering_scheme(&mut self, scheme: NumberingScheme) -> Result<()> {
        self.project_mut()?.set_numbering_scheme(scheme)?;
        self.changed();
        Ok(())
    }
    pub fn number_devices(&mut self, preview: &DeviceNumberPreview) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.number_devices(p, preview)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn number_connections(&mut self, preview: &ConnectionNumberPreview) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.number_connections(p, preview)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn set_connection_number(
        &mut self,
        id: ConnectionId,
        number: Option<Text>,
        locked: bool,
    ) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.connection_number(p, id, number, locked)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn number_pages(&mut self, preview: &PageNumberPreview) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.number_pages(p, preview)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    /// Etapa 17: símbolo pelo ID — versão do projeto, depois biblioteca
    /// externa, depois embutidos.
    pub fn find_symbol(&self, id: &SymbolDefinitionId) -> Option<SymbolDefinition> {
        self.project()
            .and_then(|p| p.symbol_definition(id))
            .or_else(|| self.symbol_library.iter().find(|d| d.id() == id))
            .or_else(|| SymbolLibrary::builtin().get(id))
            .cloned()
    }
    /// Etapa 17: catálogo para o Centro de inserção (sem repetir IDs).
    pub fn symbol_catalog(&self) -> Vec<SymbolDefinition> {
        let mut seen = std::collections::BTreeSet::new();
        let project = self
            .project()
            .into_iter()
            .flat_map(|p| p.symbol_definitions());
        project
            .chain(self.symbol_library.iter())
            .chain(SymbolLibrary::builtin().definitions())
            .filter(|d| seen.insert(d.id().clone()))
            .cloned()
            .collect()
    }
    /// Duplicar (Ctrl+D): cola a seleção deslocada, sem usar a área de
    /// transferência.
    /// Etapa 17: troca definições do projeto por versões novas (um passo).
    pub fn update_symbols(&mut self, definitions: Vec<SymbolDefinition>) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.symbol_updates(p, definitions)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn assign_part(&mut self, device: DeviceId, part: Part, quantity: u32) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.assign_part(p, device, part, quantity)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn remove_part_assignment(&mut self, component: ComponentId) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.remove_part(p, component)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn update_part(&mut self, part: Part) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.update_part(p, part)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn update_parts(&mut self, parts: Vec<Part>) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.update_parts(p, parts)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn edit_function_point(&mut self, edit: FunctionPointEdit) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.function_point(p, edit)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
}
