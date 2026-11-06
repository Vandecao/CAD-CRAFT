// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! input support for the physical editor; start at the parent module.

use super::*;

impl PhysicalEditor {
    pub fn open(&mut self, tab: PhysicalTab) {
        self.tab = tab;
        self.open = true;
    }
    pub(in crate::physical_editor) fn refresh(&mut self, session: &Session) {
        let Some(project) = session.project() else {
            self.cache = PhysicalCache::default();
            return;
        };
        let key = (project.id(), session.revision);
        if self.cache.key == Some(key) {
            return;
        }
        self.cache = PhysicalCache {
            key: Some(key),
            devices: project
                .devices()
                .map(|d| (d.id(), d.designation().to_string()))
                .collect(),
            strips: project.terminal_strips().cloned().collect(),
            terminals: project.terminals().cloned().collect(),
            points: project.connection_points().cloned().collect(),
            cables: project.cables().cloned().collect(),
            cores: project.cable_cores().cloned().collect(),
            wires: project.conductors().cloned().collect(),
            bridges: project.terminal_bridges().cloned().collect(),
            connectors: project.connectors().cloned().collect(),
            matings: project.connector_matings().cloned().collect(),
            lists: project.physical_lists(),
        };
        self.reseed();
    }
    /// Re-seeds edit drafts from the current Core state after any revision
    /// change (edit, Undo, Redo), so a stale text field never re-applies an
    /// undone value. Vanished selections are cleared.
    pub(in crate::physical_editor) fn reseed(&mut self) {
        match self
            .selected_strip
            .and_then(|id| self.cache.strips.iter().find(|s| s.id() == id))
        {
            Some(s) => self.strip_name = s.designation().as_str().into(),
            None => self.selected_strip = None,
        }
        match self
            .selected_terminal
            .and_then(|id| self.cache.terminals.iter().find(|t| t.id() == id))
        {
            Some(t) => self.terminal_name = t.designation().as_str().into(),
            None => self.selected_terminal = None,
        }
        match self
            .selected_cable
            .and_then(|id| self.cache.cables.iter().find(|x| x.id() == id))
            .cloned()
        {
            Some(cable) => self.seed_cable(&cable),
            None => self.selected_cable = None,
        }
        match self
            .selected_core
            .and_then(|id| self.cache.cores.iter().find(|x| x.id() == id))
            .cloned()
        {
            Some(core) => self.seed_core(&core),
            None => self.selected_core = None,
        }
        match self
            .selected_wire
            .and_then(|id| self.cache.wires.iter().find(|w| w.id() == id))
        {
            Some(w) => {
                self.wire_edit_name = w.designation().as_str().into();
                self.wire_edit_section = w.cross_section().get().to_string();
            }
            None => self.selected_wire = None,
        }
        if self.selected_connection.is_some_and(|id| {
            !self
                .cache
                .lists
                .assignments
                .iter()
                .any(|a| a.connection == id)
        }) {
            self.selected_connection = None;
        }
        self.reseed_connectors();
    }
    pub(in crate::physical_editor) fn seed_cable(&mut self, cable: &Cable) {
        self.cable_name = cable.designation().as_str().into();
        self.cable_description = cable
            .description()
            .map(|v| v.as_str().to_owned())
            .unwrap_or_default();
        self.cable_manufacturer = cable
            .manufacturer()
            .map(|v| v.as_str().to_owned())
            .unwrap_or_default();
        self.cable_catalog = cable
            .catalog_code()
            .map(|v| v.as_str().to_owned())
            .unwrap_or_default();
        self.cable_count = cable.nominal_core_count().to_string();
        self.cable_kind_index = match cable.kind() {
            CableKind::Unspecified => 0,
            CableKind::Power => 1,
            CableKind::Control => 2,
            CableKind::Instrumentation => 3,
            CableKind::Data => 4,
        };
    }
    pub(in crate::physical_editor) fn seed_core(&mut self, core: &CableCore) {
        self.core_name = core.identification().as_str().into();
        self.core_marking = core
            .marking()
            .identification()
            .map(|v| v.as_str().to_owned())
            .unwrap_or_default();
        self.core_section = core
            .cross_section()
            .map(|v| v.get().to_string())
            .unwrap_or_default();
    }
    pub(in crate::physical_editor) fn apply(
        &mut self,
        session: &mut Session,
        edit: Result<PhysicalEdit>,
    ) {
        match edit.and_then(|edit| session.dispatch_physical(edit)) {
            Ok(true) => {
                self.feedback = "Alteração registrada no histórico.".into();
                self.cache.key = None;
            }
            Ok(false) => self.feedback = "Nenhuma alteração.".into(),
            Err(error) => self.feedback = error.to_string(),
        }
    }
}
impl PhysicalEditor {
    /// Preselects a logical connection (for example the selected drawn route).
    pub fn focus_connection(&mut self, id: ConnectionId) {
        if self.selected_connection != Some(id) {
            self.selected_connection = Some(id);
            self.wire_source = None;
            self.wire_target = None;
            self.wire_to_link = None;
        }
    }
    /// Graphic occurrence requested by the editor, consumed by the app navigator.
    pub fn take_locate(&mut self) -> Option<RepresentationId> {
        self.locate.take()
    }
    pub(in crate::physical_editor) fn request_locate(
        &mut self,
        session: &Session,
        target: RepresentationTarget,
    ) {
        match session
            .project()
            .and_then(|p| p.physical_occurrence(target))
        {
            Some(id) => self.locate = Some(id),
            None => self.feedback = "Objeto sem ocorrência gráfica explícita.".into(),
        }
    }
}
impl PhysicalEditor {
    pub(in crate::physical_editor) fn build_wire(
        &self,
        project_id: ProjectId,
    ) -> Result<Conductor> {
        let name = DesignationPart::new(&self.wire_name)?;
        let area = optional_section(&self.wire_section)?.ok_or(DomainError::InvalidNumber(
            "seção nominal obrigatória em mm²",
        ))?;
        let marking = if self.wire_marking.trim().is_empty() {
            ConductorMarking::identified(name.clone())
        } else {
            ConductorMarking::identified(DesignationPart::new(&self.wire_marking)?)
        };
        let (source, target) = self
            .wire_source
            .clone()
            .zip(self.wire_target.clone())
            .ok_or(DomainError::InvalidRelation(
                "selecione origem e destino físicos",
            ))?;
        Conductor::new(
            project_id,
            name,
            PhysicalTermination::new(source.0, source.1),
            PhysicalTermination::new(target.0, target.1),
            area,
            marking,
        )
    }
}
pub(in crate::physical_editor) fn optional_text(value: &str) -> Result<Option<Text>> {
    if value.trim().is_empty() {
        Ok(None)
    } else {
        Text::new(value).map(Some)
    }
}
pub(in crate::physical_editor) fn optional_section(value: &str) -> Result<Option<CrossSectionMm2>> {
    if value.trim().is_empty() {
        Ok(None)
    } else {
        let number = value
            .parse::<f64>()
            .map_err(|_| DomainError::InvalidNumber("seção em mm²"))?;
        CrossSectionMm2::new(number).map(Some)
    }
}
