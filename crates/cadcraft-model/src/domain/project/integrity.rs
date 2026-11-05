// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Project relation validation; start with the Project integrity checks.
use super::*;

impl Project {
    pub fn contains(&self, reference: EntityRef) -> bool {
        match reference {
            EntityRef::Project(id) => self.id == id,
            EntityRef::Document(id) => self.documents.contains_key(&id),
            EntityRef::Page(id) => self.pages.contains_key(&id),
            EntityRef::Device(id) => self.devices.contains_key(&id),
            EntityRef::Function(id) => self.functions.contains_key(&id),
            EntityRef::Component(id) => self.components.contains_key(&id),
            EntityRef::Terminal(id) => self.terminals.contains_key(&id),
            EntityRef::ConnectionPoint(id) => self.connection_points.contains_key(&id),
            EntityRef::Conductor(id) => self.conductors.contains_key(&id),
            EntityRef::Cable(id) => self.cables.contains_key(&id),
            EntityRef::CableCore(id) => self.cable_cores.contains_key(&id),
            EntityRef::TerminalStrip(id) => self.terminal_strips.contains_key(&id),
            EntityRef::Connector(id) => self.connectors.contains_key(&id),
        }
    }
    pub(super) fn require(&self, reference: EntityRef) -> Result<()> {
        if self.contains(reference) {
            Ok(())
        } else {
            Err(DomainError::MissingEntity(reference))
        }
    }
    pub(super) fn check_project(&self, found: ProjectId) -> Result<()> {
        if found == self.id {
            Ok(())
        } else {
            Err(DomainError::ForeignProject {
                expected: self.id,
                found,
            })
        }
    }
    pub(super) fn check_properties(&self, properties: &TechnicalProperties) -> Result<()> {
        for reference in properties.references() {
            self.require(reference)?;
        }
        Ok(())
    }
    pub(super) fn check_document(&self, _: &Document) -> Result<()> {
        Ok(())
    }
    pub(super) fn check_device(&self, _: &Device) -> Result<()> {
        Ok(())
    }
    pub(super) fn check_page(&self, page: &Page) -> Result<()> {
        self.require_frame(page.frame())?;
        self.require(EntityRef::Document(page.document_id()))?;
        if let Some(report) = page.report() {
            if !matches!(page.kind(), PageKind::TechnicalDocumentation) {
                return Err(DomainError::InvalidRelation(
                    "relatório deve ser página técnica",
                ));
            }
            if let Some(id) = report.filter.document {
                self.require(EntityRef::Document(id))?;
            }
            if let Some(id) = report.filter.page {
                self.require(EntityRef::Page(id))?;
            }
        }
        for element in page.elements() {
            self.require(element.into())?;
        }
        Ok(())
    }
    pub(super) fn check_function(&self, function: &Function) -> Result<()> {
        self.require(EntityRef::Device(function.device_id()))?;
        if function.role() == FunctionRole::Main
            && self
                .functions_for_device(function.device_id())
                .any(|other| other.id() != function.id() && other.role() == FunctionRole::Main)
        {
            return Err(DomainError::InvalidRelation(
                "dispositivo já possui função principal",
            ));
        }
        Ok(())
    }
    pub(super) fn check_component(&self, component: &Component) -> Result<()> {
        if component.quantity() == 0 {
            return Err(DomainError::InvalidNumber("quantidade da peça"));
        }
        if let Some(part) = component.part_id()
            && (component.device_id().is_none() || !self.parts.contains_key(part))
        {
            return Err(DomainError::InvalidRelation(
                "peça atribuída sem dispositivo ou cadastro",
            ));
        }
        if let Some(device) = component.device_id() {
            self.require(EntityRef::Device(device))?;
        }
        if let Some(function_id) = component.function_id() {
            let function = self
                .function(function_id)
                .ok_or(DomainError::MissingEntity(EntityRef::Function(function_id)))?;
            if component.device_id() != Some(function.device_id()) {
                return Err(DomainError::InvalidRelation(
                    "função e componente devem pertencer ao mesmo dispositivo",
                ));
            }
        }
        Ok(())
    }
    pub(super) fn check_terminal(&self, terminal: &Terminal) -> Result<()> {
        self.require(terminal.owner().into())?;
        terminal.check_specialization()
    }
    pub(super) fn check_connection_point(&self, point: &ConnectionPoint) -> Result<()> {
        let function = self
            .function(point.function_id())
            .ok_or(DomainError::MissingEntity(EntityRef::Function(
                point.function_id(),
            )))?;
        if point.terminal_id().is_none() && point.terminal_level().is_some() {
            return Err(DomainError::InvalidRelation(
                "nível de borne sem borne associado",
            ));
        }
        if let Some(terminal_id) = point.terminal_id() {
            let terminal = self
                .terminal(terminal_id)
                .ok_or(DomainError::MissingEntity(EntityRef::Terminal(terminal_id)))?;
            // Multilevel terminals keep each level an independent electrical
            // node: a point must say which level it represents.
            match (terminal.kind(), point.terminal_level()) {
                (TerminalKind::MultiLevel, Some(level)) if terminal.level(level).is_some() => {}
                (TerminalKind::MultiLevel, _) => {
                    return Err(DomainError::InvalidRelation(
                        "ponto de borne multinível exige nível existente",
                    ));
                }
                (_, Some(_)) => {
                    return Err(DomainError::InvalidRelation(
                        "nível informado para borne sem níveis",
                    ));
                }
                _ => {}
            }
            match terminal.owner() {
                TerminalOwner::Device(device) if device == function.device_id() => {}
                TerminalOwner::Component(id) => {
                    let component = self
                        .component(id)
                        .ok_or(DomainError::MissingEntity(EntityRef::Component(id)))?;
                    if component.device_id() != Some(function.device_id())
                        || component
                            .function_id()
                            .is_some_and(|id| id != function.id())
                    {
                        return Err(DomainError::InvalidRelation(
                            "terminal do componente incompatível com a função",
                        ));
                    }
                }
                _ => {
                    return Err(DomainError::InvalidRelation(
                        "terminal e função pertencem a dispositivos diferentes",
                    ));
                }
            }
        }
        Ok(())
    }
    /// Integridade estrutural e do índice de conectividade, sem cálculos elétricos/ERC.
    pub fn validate(&self) -> Result<()> {
        if self.numbering_scheme.start == 0 || self.numbering_scheme.step == 0 {
            return Err(DomainError::InvalidRelation("regra de numeração inválida"));
        }
        self.check_scheme.validate()?;
        self.validate_function_roles()?;
        self.validate_frames()?;
        self.validate_markers()?;
        self.validate_connection_drawing()?;
        self.validate_terminal_bridges()?;
        self.check_properties(&self.properties)?;
        for part in self.parts.values() {
            part.validate()?;
            self.check_properties(part.properties())?;
        }
        for entity in self.documents.values() {
            self.check_project(entity.project_id())?;
            self.check_document(entity)?;
            self.check_properties(entity.properties())?;
        }
        for entity in self.pages.values() {
            self.check_project(entity.project_id())?;
            self.check_page(entity)?;
            self.check_properties(entity.properties())?;
        }
        for entity in self.devices.values() {
            self.check_project(entity.project_id())?;
            self.check_device(entity)?;
            self.check_properties(entity.properties())?;
        }
        for entity in self.functions.values() {
            self.check_project(entity.project_id())?;
            self.check_function(entity)?;
            self.check_properties(entity.properties())?;
        }
        for entity in self.components.values() {
            self.check_project(entity.project_id())?;
            self.check_component(entity)?;
            self.check_properties(entity.properties())?;
        }
        for entity in self.terminals.values() {
            self.check_project(entity.project_id())?;
            self.check_terminal(entity)?;
            self.check_properties(entity.properties())?;
        }
        for entity in self.connection_points.values() {
            self.check_project(entity.project_id())?;
            self.check_connection_point(entity)?;
            self.check_properties(entity.properties())?;
        }
        if self.build_graph()? != self.graph {
            return Err(DomainError::InvalidRelation(
                "índice de conectividade inconsistente",
            ));
        }
        if self.build_physical_index()? != self.physical_index {
            return Err(DomainError::InvalidRelation("índice físico inconsistente"));
        }
        if self.build_document_index()? != self.document_index {
            return Err(DomainError::InvalidRelation(
                "índice documental inconsistente",
            ));
        }
        Ok(())
    }
}
