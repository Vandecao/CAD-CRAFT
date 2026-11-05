// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::{ComponentId, ConnectionId, ConnectionPointId, DeviceId, FunctionId};

/// Fotografia derivada de uma componente conexa do grafo lógico.
/// Sem identidade persistente, serialização ou mutadores. Consulte novamente o
/// `Project` após uma alteração. Todos os membros estão ordenados e sem repetição.
/// Funções/dispositivos/componentes são associações; não criam conexões internas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Network {
    pub(super) points: Vec<ConnectionPointId>,
    pub(super) connections: Vec<ConnectionId>,
    pub(super) functions: Vec<FunctionId>,
    pub(super) devices: Vec<DeviceId>,
    pub(super) components: Vec<ComponentId>,
}

impl Network {
    pub fn points(&self) -> &[ConnectionPointId] {
        &self.points
    }
    pub fn connections(&self) -> &[ConnectionId] {
        &self.connections
    }
    pub fn functions(&self) -> &[FunctionId] {
        &self.functions
    }
    pub fn devices(&self) -> &[DeviceId] {
        &self.devices
    }

    /// Componentes com associação explícita à função ou proprietários de terminais
    /// mapeados pelos pontos. Associação apenas ao dispositivo não basta.
    pub fn components(&self) -> &[ComponentId] {
        &self.components
    }

    /// Em uma componente conexa simples, há ciclo quando |E| >= |V|.
    /// Ciclo topológico é permitido e não equivale a diagnóstico elétrico/ERC.
    pub fn has_cycle(&self) -> bool {
        self.connections.len() >= self.points.len()
    }
}
