// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Logical edges between connection points; start with Connection::new.
use super::{ConnectionId, ConnectionPointId, ProjectId, Text};
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};

/// Aresta lógica não direcionada; não é linha gráfica nem condutor físico.
/// A existência dos extremos e o pertencimento são verificados pelo `Project`.
/// Não há propriedades físicas ou referências de `EntityRef` para conexões.
/// IDs de conexão não são intercambiáveis com IDs de pontos:
/// ```compile_fail
/// use cad_craft::domain::{ConnectionId, ConnectionPointId};
/// let point: ConnectionPointId = ConnectionId::new();
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "ConnectionData")]
pub struct Connection {
    id: ConnectionId,
    project_id: ProjectId,
    source: ConnectionPointId,
    target: ConnectionPointId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(in crate::domain) number: Option<Text>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(in crate::domain) number_locked: bool,
}

impl Connection {
    pub fn new(
        project_id: ProjectId,
        source: ConnectionPointId,
        target: ConnectionPointId,
    ) -> Result<Self> {
        Self::try_from(ConnectionData {
            id: ConnectionId::new(),
            project_id,
            source,
            target,
            number: None,
            number_locked: false,
        })
    }

    pub fn id(&self) -> ConnectionId {
        self.id
    }
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Extremos em ordem crescente de ID; não expressam sentido de corrente.
    pub fn endpoints(&self) -> (ConnectionPointId, ConnectionPointId) {
        (self.source, self.target)
    }
    pub fn number(&self) -> Option<&Text> {
        self.number.as_ref()
    }
    pub fn number_locked(&self) -> bool {
        self.number_locked
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConnectionData {
    id: ConnectionId,
    project_id: ProjectId,
    source: ConnectionPointId,
    target: ConnectionPointId,
    #[serde(default)]
    number: Option<Text>,
    #[serde(default)]
    number_locked: bool,
}

impl TryFrom<ConnectionData> for Connection {
    type Error = DomainError;

    fn try_from(data: ConnectionData) -> Result<Self> {
        if data.source == data.target {
            return Err(DomainError::SelfConnection(data.source));
        }
        Ok(Self {
            id: data.id,
            project_id: data.project_id,
            source: data.source.min(data.target),
            target: data.source.max(data.target),
            number: data.number,
            number_locked: data.number_locked,
        })
    }
}
