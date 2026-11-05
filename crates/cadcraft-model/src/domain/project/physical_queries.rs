// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::*;
use std::collections::BTreeSet;

impl Project {
    /// Veias em ordem de ID, sem clonagem do registro. Cabo ausente produz iterador vazio.
    pub fn cores_for_cable(&self, cable: CableId) -> impl Iterator<Item = &CableCore> {
        self.physical_index
            .cores_by_cable
            .get(&cable)
            .into_iter()
            .flatten()
            .filter_map(|id| self.cable_core(*id))
    }
    pub fn core_for_conductor(&self, conductor: ConductorId) -> Option<&CableCore> {
        self.physical_index
            .core_by_conductor
            .get(&conductor)
            .and_then(|id| self.cable_core(*id))
    }
    /// Referência opcional de cabo do condutor, derivada da única associação na veia.
    pub fn cable_for_conductor(&self, conductor: ConductorId) -> Option<&Cable> {
        self.core_for_conductor(conductor)
            .and_then(|core| self.cable(core.cable_id()))
    }
    pub fn strip_for_terminal(&self, terminal: TerminalId) -> Option<&TerminalStrip> {
        self.physical_index
            .strip_by_terminal
            .get(&terminal)
            .and_then(|id| self.terminal_strip(*id))
    }
    /// Pontos funcionais na ordem source/target do condutor; None é engenharia incompleta válida.
    pub fn functional_points_for_conductor(
        &self,
        conductor: ConductorId,
    ) -> Result<Option<(ConnectionPointId, ConnectionPointId)>> {
        let wire = self
            .conductor(conductor)
            .ok_or(DomainError::MissingEntity(EntityRef::Conductor(conductor)))?;
        wire.connection()
            .map(|link| self.check_conductor_connection(wire, link))
            .transpose()
    }
    pub fn physical_for_connection(&self, connection: ConnectionId) -> Result<PhysicalElements> {
        self.connection(connection)
            .ok_or(DomainError::MissingConnection(connection))?;
        let wires: BTreeSet<_> = self
            .physical_index
            .conductors_by_connection
            .get(&connection)
            .into_iter()
            .flatten()
            .copied()
            .collect();
        let terminals = wires
            .iter()
            .filter_map(|id| self.conductor(*id))
            .flat_map(|c| [c.endpoints().0.terminal_id(), c.endpoints().1.terminal_id()])
            .collect();
        Ok(self.describe_physical(wires, terminals))
    }
    /// Terminais do dispositivo (inclusive componentes explicitamente associados),
    /// condutores incidentes, suas veias/cabos e réguas desses terminais.
    /// Não inclui terminais do outro dispositivo só porque existe um condutor entre eles.
    pub fn physical_for_device(&self, device: DeviceId) -> Result<PhysicalElements> {
        self.require(EntityRef::Device(device))?;
        let terminals: BTreeSet<_> = self
            .terminals
            .values()
            .filter(|terminal| match terminal.owner() {
                TerminalOwner::Device(id) => id == device,
                TerminalOwner::Component(id) => self
                    .component(id)
                    .is_some_and(|c| c.device_id() == Some(device)),
            })
            .map(Terminal::id)
            .collect();
        let wires = terminals
            .iter()
            .filter_map(|id| self.physical_index.conductors_by_terminal.get(id))
            .flatten()
            .copied()
            .collect();
        Ok(self.describe_physical(wires, terminals))
    }
    fn describe_physical(
        &self,
        wires: BTreeSet<ConductorId>,
        terminals: BTreeSet<TerminalId>,
    ) -> PhysicalElements {
        let cores: BTreeSet<_> = wires
            .iter()
            .filter_map(|id| self.core_for_conductor(*id))
            .map(CableCore::id)
            .collect();
        let cables: BTreeSet<_> = cores
            .iter()
            .filter_map(|id| self.cable_core(*id))
            .map(CableCore::cable_id)
            .collect();
        let strips: BTreeSet<_> = terminals
            .iter()
            .filter_map(|id| self.strip_for_terminal(*id))
            .map(TerminalStrip::id)
            .collect();
        PhysicalElements {
            conductors: wires.into_iter().collect(),
            cables: cables.into_iter().collect(),
            cable_cores: cores.into_iter().collect(),
            terminals: terminals.into_iter().collect(),
            terminal_strips: strips.into_iter().collect(),
        }
    }
}
