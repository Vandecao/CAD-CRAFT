// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Physical wiring insertion and removal; start with the Project wiring methods.
use super::*;

impl Project {
    /// Find the conductor by its stable identifier; return None when absent.
    pub fn conductor(&self, id: ConductorId) -> Option<&Conductor> {
        self.conductors.get(&id)
    }
    /// Iterate conductors without exposing mutable access.
    pub fn conductors(&self) -> impl Iterator<Item = &Conductor> {
        self.conductors.values()
    }
    /// Find the cable by its stable identifier; return None when absent.
    pub fn cable(&self, id: CableId) -> Option<&Cable> {
        self.cables.get(&id)
    }
    /// Iterate cables without exposing mutable access.
    pub fn cables(&self) -> impl Iterator<Item = &Cable> {
        self.cables.values()
    }
    /// Find the cable core by its stable identifier; return None when absent.
    pub fn cable_core(&self, id: CableCoreId) -> Option<&CableCore> {
        self.cable_cores.get(&id)
    }
    /// Iterate cable cores without exposing mutable access.
    pub fn cable_cores(&self) -> impl Iterator<Item = &CableCore> {
        self.cable_cores.values()
    }
    /// Find the terminal strip by its stable identifier; return None when absent.
    pub fn terminal_strip(&self, id: TerminalStripId) -> Option<&TerminalStrip> {
        self.terminal_strips.get(&id)
    }
    /// Iterate terminal strips without exposing mutable access.
    pub fn terminal_strips(&self) -> impl Iterator<Item = &TerminalStrip> {
        self.terminal_strips.values()
    }

    /// Validate and register conductor, rejecting duplicate IDs, foreign ownership or invalid
    /// relations.
    pub fn insert_conductor(&mut self, conductor: Conductor) -> Result<ConductorId> {
        let id = conductor.id();
        if self.conductors.contains_key(&id) {
            return Err(DomainError::DuplicateEntity(EntityRef::Conductor(id)));
        }
        self.check_conductor(&conductor)?;
        self.physical_index.insert_conductor(&conductor)?;
        self.conductors.insert(id, conductor);
        Ok(id)
    }
    /// Validate and register cable, rejecting duplicate IDs, foreign ownership or invalid relations.
    pub fn insert_cable(&mut self, cable: Cable) -> Result<CableId> {
        let id = cable.id();
        if self.cables.contains_key(&id) {
            return Err(DomainError::DuplicateEntity(EntityRef::Cable(id)));
        }
        self.check_cable(&cable)?;
        self.cables.insert(id, cable);
        Ok(id)
    }
    /// Validate and register cable core, rejecting duplicate IDs, foreign ownership or invalid
    /// relations.
    pub fn insert_cable_core(&mut self, core: CableCore) -> Result<CableCoreId> {
        let id = core.id();
        if self.cable_cores.contains_key(&id) {
            return Err(DomainError::DuplicateEntity(EntityRef::CableCore(id)));
        }
        let capacity = self.check_core(&core)?;
        self.physical_index.insert_core(&core, capacity)?;
        self.cable_cores.insert(id, core);
        Ok(id)
    }
    /// Validate and register terminal strip, rejecting duplicate IDs, foreign ownership or invalid
    /// relations.
    pub fn insert_terminal_strip(&mut self, strip: TerminalStrip) -> Result<TerminalStripId> {
        let id = strip.id();
        if self.terminal_strips.contains_key(&id) {
            return Err(DomainError::DuplicateEntity(EntityRef::TerminalStrip(id)));
        }
        self.check_strip(&strip)?;
        self.physical_index.insert_strip(&strip)?;
        self.terminal_strips.insert(id, strip);
        Ok(id)
    }
    /// Append a position without implicit continuity. Positions cannot be removed.
    pub fn add_terminal_position(
        &mut self,
        terminal: TerminalId,
        position: DesignationPart,
    ) -> Result<()> {
        let target = self
            .terminals
            .get_mut(&terminal)
            .ok_or(DomainError::MissingEntity(EntityRef::Terminal(terminal)))?;
        if !target.positions.insert(position) {
            return Err(DomainError::InvalidRelation("posição física duplicada"));
        }
        Ok(())
    }
    /// Append to the strip; a terminal belongs to at most one strip.
    pub fn attach_terminal(&mut self, strip: TerminalStripId, terminal: TerminalId) -> Result<()> {
        self.require(EntityRef::Terminal(terminal))?;
        if self
            .physical_index
            .strip_by_terminal
            .contains_key(&terminal)
        {
            return Err(DomainError::TerminalAlreadyAssigned(terminal));
        }
        let target = self
            .terminal_strips
            .get_mut(&strip)
            .ok_or(DomainError::MissingEntity(EntityRef::TerminalStrip(strip)))?;
        target.terminals.push(terminal);
        self.physical_index
            .strip_by_terminal
            .insert(terminal, strip);
        Ok(())
    }
    /// Require both core and conductor to be free; replacement requires explicit unbinding first.
    pub fn assign_core_conductor(
        &mut self,
        core: CableCoreId,
        conductor: ConductorId,
    ) -> Result<()> {
        let target = self
            .cable_core(core)
            .ok_or(DomainError::MissingEntity(EntityRef::CableCore(core)))?;
        let wire = self
            .conductor(conductor)
            .ok_or(DomainError::MissingEntity(EntityRef::Conductor(conductor)))?;
        if target.conductor_id().is_some() {
            return Err(DomainError::CoreAlreadyAssigned(core));
        }
        if self
            .physical_index
            .core_by_conductor
            .contains_key(&conductor)
        {
            return Err(DomainError::ConductorAlreadyAssigned(conductor));
        }
        self.check_core_conductor(target, wire)?;
        self.cable_cores
            .get_mut(&core)
            .ok_or(DomainError::MissingEntity(EntityRef::CableCore(core)))?
            .conductor_id = Some(conductor);
        self.physical_index
            .core_by_conductor
            .insert(conductor, core);
        Ok(())
    }
    /// Free the core without deleting the conductor; an already free core returns None.
    pub fn unassign_core_conductor(&mut self, core: CableCoreId) -> Result<Option<ConductorId>> {
        let target = self
            .cable_cores
            .get_mut(&core)
            .ok_or(DomainError::MissingEntity(EntityRef::CableCore(core)))?;
        let previous = target.conductor_id.take();
        if let Some(id) = previous {
            self.physical_index.core_by_conductor.remove(&id);
        }
        Ok(previous)
    }
    /// Document an existing connection without creating edges or changing networks.
    pub fn assign_conductor_connection(
        &mut self,
        conductor: ConductorId,
        link: ConductorConnection,
    ) -> Result<()> {
        if self.terminal_bridges.contains_key(&link.connection_id()) {
            return Err(DomainError::InvalidRelation(
                "ponte não é ligação física externa",
            ));
        }
        let wire = self
            .conductor(conductor)
            .ok_or(DomainError::MissingEntity(EntityRef::Conductor(conductor)))?;
        if wire.connection().is_some() {
            return Err(DomainError::ConductorAlreadyAssigned(conductor));
        }
        self.check_conductor_connection(wire, link)?;
        self.conductors
            .get_mut(&conductor)
            .ok_or(DomainError::MissingEntity(EntityRef::Conductor(conductor)))?
            .connection = Some(link);
        self.physical_index
            .link_connection(conductor, link.connection_id());
        Ok(())
    }
    /// Remove only the binding, retaining both conductor and connection.
    pub fn unassign_conductor_connection(
        &mut self,
        conductor: ConductorId,
    ) -> Result<Option<ConductorConnection>> {
        let wire = self
            .conductors
            .get_mut(&conductor)
            .ok_or(DomainError::MissingEntity(EntityRef::Conductor(conductor)))?;
        let previous = wire.connection.take();
        if let Some(link) = previous {
            self.physical_index
                .unlink_connection(conductor, link.connection_id());
        }
        Ok(previous)
    }
}
