// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use crate::domain::{
    CableCoreId, CableId, ConductorId, ConnectionId, ConnectionPointId, EntityRef,
    PhysicalTermination, ProjectId, RepresentationId, TerminalId,
};
use std::fmt;

/// Erros esperados de entrada e integridade, sem panics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DomainError {
    InvalidId,
    InvalidText,
    InvalidDesignation,
    InvalidNumber(&'static str),
    InvalidEnumeration,
    DuplicateEntity(EntityRef),
    MissingEntity(EntityRef),
    EntityInUse(EntityRef),
    DuplicateRepresentation(RepresentationId),
    MissingRepresentation(RepresentationId),
    ForeignProject {
        expected: ProjectId,
        found: ProjectId,
    },
    InvalidRelation(&'static str),
    DuplicateReference(EntityRef),
    DuplicateConnectionId(ConnectionId),
    MissingConnection(ConnectionId),
    SelfConnection(ConnectionPointId),
    DuplicateConnection {
        source: ConnectionPointId,
        target: ConnectionPointId,
    },
    MissingTermination(PhysicalTermination),
    DuplicateConductorDesignation(ConductorId),
    CableCapacityExceeded(CableId),
    DuplicateCoreIdentification(CableId),
    CoreAlreadyAssigned(CableCoreId),
    ConductorAlreadyAssigned(ConductorId),
    IncompatibleCore(CableCoreId),
    TerminalAlreadyAssigned(TerminalId),
    ConnectionInUse(ConnectionId),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidId => write!(f, "identificador deve ser UUID v7 com variante RFC 4122"),
            Self::InvalidText => write!(f, "texto obrigatório vazio ou com caracteres de controle"),
            Self::InvalidDesignation => write!(f, "parte de designação inválida"),
            Self::InvalidNumber(field) => write!(f, "número inválido: {field}"),
            Self::InvalidEnumeration => {
                write!(f, "enumeração vazia, duplicada ou seleção inexistente")
            }
            Self::DuplicateEntity(id) => write!(f, "entidade duplicada: {id:?}"),
            Self::MissingEntity(id) => write!(f, "entidade inexistente: {id:?}"),
            Self::EntityInUse(id) => write!(f, "entidade possui referências: {id:?}"),
            Self::DuplicateRepresentation(id) => write!(f, "representação duplicada: {id}"),
            Self::MissingRepresentation(id) => write!(f, "representação inexistente: {id}"),
            Self::ForeignProject { expected, found } => write!(
                f,
                "projeto incompatível: esperado {expected}, recebido {found}"
            ),
            Self::InvalidRelation(reason) => write!(f, "relação inválida: {reason}"),
            Self::DuplicateReference(id) => write!(f, "referência duplicada: {id:?}"),
            Self::DuplicateConnectionId(id) => write!(f, "ID de conexão duplicado: {id}"),
            Self::MissingConnection(id) => write!(f, "conexão inexistente: {id}"),
            Self::SelfConnection(id) => write!(f, "conexão autorreferente: {id}"),
            Self::DuplicateConnection { source, target } => {
                write!(f, "conexão duplicada: {source}–{target}")
            }
            Self::MissingTermination(end) => write!(f, "terminação física inexistente: {end:?}"),
            Self::DuplicateConductorDesignation(id) => {
                write!(f, "designação física já usada no par pelo condutor {id}")
            }
            Self::CableCapacityExceeded(id) => {
                write!(f, "quantidade nominal de veias excedida: {id}")
            }
            Self::DuplicateCoreIdentification(id) => {
                write!(f, "identificação de veia duplicada no cabo {id}")
            }
            Self::CoreAlreadyAssigned(id) => write!(f, "veia já instalada: {id}"),
            Self::ConductorAlreadyAssigned(id) => write!(f, "condutor já associado: {id}"),
            Self::IncompatibleCore(id) => write!(f, "condutor incompatível com a veia {id}"),
            Self::TerminalAlreadyAssigned(id) => write!(f, "terminal já associado a régua: {id}"),
            Self::ConnectionInUse(id) => {
                write!(f, "conexão possui condutor ou representação: {id}")
            }
        }
    }
}

impl std::error::Error for DomainError {}
pub type Result<T> = std::result::Result<T, DomainError>;
