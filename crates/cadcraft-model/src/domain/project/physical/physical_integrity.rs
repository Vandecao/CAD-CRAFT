// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Physical wiring relation validation; start with the Project physical integrity checks.
use super::*;

impl Project {
    pub(in crate::domain::project) fn validate_terminal_bridges(&self) -> Result<()> {
        for bridge in self.terminal_bridges.values() {
            self.check_project(bridge.project_id())?;
            self.check_properties(bridge.properties())?;
            let strip = self
                .terminal_strip(bridge.strip_id())
                .ok_or(DomainError::InvalidRelation("régua da ponte ausente"))?;
            let connection = self
                .connection(bridge.id())
                .ok_or(DomainError::MissingConnection(bridge.id()))?;
            // Conductors linked to a bridge are rejected by check_conductor_connection.
            if self.drawing_owns_connection(bridge.id()) || self.marker_owns_connection(bridge.id())
            {
                return Err(DomainError::InvalidRelation(
                    "conexão de ponte com posse incompatível",
                ));
            }
            let (a, b) = connection.endpoints();
            let mut earth = Vec::new();
            for point in [a, b] {
                let terminal = self
                    .connection_point(point)
                    .and_then(ConnectionPoint::terminal_id)
                    .ok_or(DomainError::InvalidRelation(
                        "ponto da ponte sem borne físico",
                    ))?;
                if !strip.terminals().contains(&terminal) {
                    return Err(DomainError::InvalidRelation("borne da ponte fora da régua"));
                }
                earth.push(
                    self.terminal(terminal)
                        .is_some_and(|t| t.kind() == TerminalKind::ProtectiveEarth),
                );
            }
            if earth[0] != earth[1] {
                return Err(DomainError::InvalidRelation(
                    "ponte entre borne PE e borne não PE",
                ));
            }
        }
        Ok(())
    }
    pub(in crate::domain::project) fn check_section_properties(
        &self,
        properties: &TechnicalProperties,
    ) -> Result<()> {
        if properties
            .known()
            .contains_key(&KnownProperty::CrossSection)
        {
            return Err(DomainError::InvalidRelation(
                "seção nominal deve usar somente o campo tipado",
            ));
        }
        Ok(())
    }
    fn check_termination(&self, end: &PhysicalTermination) -> Result<()> {
        let terminal = self
            .terminal(end.terminal_id())
            .ok_or(DomainError::MissingEntity(EntityRef::Terminal(
                end.terminal_id(),
            )))?;
        if !terminal.has_position(end.position()) {
            return Err(DomainError::MissingTermination(end.clone()));
        }
        Ok(())
    }
    /// Declared function of a conductor landing on a PE terminal must be PE
    /// or left unspecified; the classification itself never wires anything.
    fn check_earth_termination(
        &self,
        conductor: &Conductor,
        end: &PhysicalTermination,
    ) -> Result<()> {
        let earth = self
            .terminal(end.terminal_id())
            .is_some_and(|t| t.kind() == TerminalKind::ProtectiveEarth);
        if earth
            && !matches!(
                conductor.function(),
                ConductorFunction::Unspecified | ConductorFunction::ProtectiveEarth
            )
        {
            return Err(DomainError::InvalidRelation(
                "condutor com função não PE em borne PE",
            ));
        }
        Ok(())
    }
    pub(in crate::domain::project) fn check_conductor(&self, conductor: &Conductor) -> Result<()> {
        self.check_project(conductor.project_id())?;
        let (a, b) = conductor.endpoints();
        self.check_termination(a)?;
        self.check_termination(b)?;
        self.check_earth_termination(conductor, a)?;
        self.check_earth_termination(conductor, b)?;
        if a == b {
            return Err(DomainError::InvalidRelation(
                "extremos físicos devem ser distintos",
            ));
        }
        self.check_section_properties(conductor.properties())?;
        self.check_properties(conductor.properties())?;
        if let Some(link) = conductor.connection() {
            self.check_conductor_connection(conductor, link)?;
        }
        Ok(())
    }
    pub(in crate::domain::project) fn check_conductor_connection(
        &self,
        conductor: &Conductor,
        link: ConductorConnection,
    ) -> Result<(ConnectionPointId, ConnectionPointId)> {
        if self.terminal_bridges.contains_key(&link.connection_id()) {
            return Err(DomainError::InvalidRelation(
                "ponte não é ligação física externa",
            ));
        }
        let edge = self
            .connection(link.connection_id())
            .ok_or(DomainError::MissingConnection(link.connection_id()))?;
        let (a, b) = edge.endpoints();
        let source = link.source_point();
        let target = if source == a {
            b
        } else if source == b {
            a
        } else {
            return Err(DomainError::InvalidRelation(
                "ponto funcional não pertence à conexão",
            ));
        };
        for (point_id, end) in [
            (source, conductor.endpoints().0),
            (target, conductor.endpoints().1),
        ] {
            let point = self
                .connection_point(point_id)
                .ok_or(DomainError::MissingEntity(EntityRef::ConnectionPoint(
                    point_id,
                )))?;
            if point.terminal_id() != Some(end.terminal_id()) {
                return Err(DomainError::InvalidRelation(
                    "ponto funcional não mapeia a terminação física",
                ));
            }
            if let Some(level) = point.terminal_level()
                && !self
                    .terminal(end.terminal_id())
                    .and_then(|t| t.level(level))
                    .is_some_and(|l| l.has_position(end.position()))
            {
                return Err(DomainError::InvalidRelation(
                    "terminação física fora do nível do ponto funcional",
                ));
            }
        }
        Ok((source, target))
    }
    pub(in crate::domain::project) fn check_cable(&self, cable: &Cable) -> Result<()> {
        self.check_project(cable.project_id())?;
        self.check_properties(cable.properties())
    }
    pub(in crate::domain::project) fn check_core(&self, core: &CableCore) -> Result<u32> {
        self.check_project(core.project_id())?;
        let cable = self
            .cable(core.cable_id())
            .ok_or(DomainError::MissingEntity(EntityRef::Cable(
                core.cable_id(),
            )))?;
        self.check_section_properties(core.properties())?;
        self.check_properties(core.properties())?;
        if let Some(id) = core.conductor_id() {
            let conductor = self
                .conductor(id)
                .ok_or(DomainError::MissingEntity(EntityRef::Conductor(id)))?;
            self.check_core_conductor(core, conductor)?;
        }
        Ok(cable.nominal_core_count())
    }
    pub(in crate::domain::project) fn check_core_conductor(
        &self,
        core: &CableCore,
        conductor: &Conductor,
    ) -> Result<()> {
        if core
            .cross_section()
            .is_some_and(|area| area != conductor.cross_section())
            || matches!((core.marking().color_value(), conductor.marking().color_value()), (Some(a), Some(b)) if a != b)
        {
            return Err(DomainError::IncompatibleCore(core.id()));
        }
        Ok(())
    }
    pub(in crate::domain::project) fn check_strip(&self, strip: &TerminalStrip) -> Result<()> {
        self.check_project(strip.project_id())?;
        self.check_properties(strip.properties())?;
        let mut numbers = std::collections::BTreeSet::new();
        for &id in strip.terminals() {
            self.require(EntityRef::Terminal(id))?;
            let number = self
                .terminal(id)
                .map(|t| t.designation().clone())
                .ok_or(DomainError::MissingEntity(EntityRef::Terminal(id)))?;
            if !numbers.insert(number) {
                return Err(DomainError::InvalidRelation(
                    "numeração de borne duplicada na régua",
                ));
            }
        }
        Ok(())
    }
    pub(in crate::domain::project) fn build_physical_index(&self) -> Result<PhysicalIndex> {
        let mut index = PhysicalIndex::default();
        for cable in self.cables.values() {
            self.check_cable(cable)?;
        }
        for conductor in self.conductors.values() {
            self.check_conductor(conductor)?;
            index.insert_conductor(conductor)?;
        }
        for core in self.cable_cores.values() {
            let capacity = self.check_core(core)?;
            index.insert_core(core, capacity)?;
        }
        for strip in self.terminal_strips.values() {
            self.check_strip(strip)?;
            index.insert_strip(strip)?;
        }
        for connector in self.connectors.values() {
            self.check_connector(connector)?;
            index.insert_connector(connector)?;
        }
        for mating in self.connector_matings.values() {
            self.check_mating(mating)?;
            index.insert_mating(mating)?;
        }
        Ok(index)
    }
}
