// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Domain validation errors and result alias; start with DomainError.
use crate::domain::{
    CableCoreId, CableId, ConductorId, ConnectionId, ConnectionPointId, EntityRef,
    PhysicalTermination, ProjectId, RepresentationId, TerminalId,
};
use std::fmt;

/// Expected input and integrity errors, reported without panics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DomainError {
    /// Identifier is not an RFC 4122 variant UUID v7.
    InvalidId,
    /// Required text is empty or contains control characters.
    InvalidText,
    /// Designation contains a forbidden segment.
    InvalidDesignation,
    /// The named numeric field violates its value constraints.
    InvalidNumber(&'static str),
    /// Enumeration is empty, duplicated or selects an absent choice.
    InvalidEnumeration,
    /// An entity with the same typed ID is already registered.
    DuplicateEntity(EntityRef),
    /// The referenced entity is not registered.
    MissingEntity(EntityRef),
    /// External references prevent removal of the entity.
    EntityInUse(EntityRef),
    /// A graphical occurrence with the same ID already exists.
    DuplicateRepresentation(RepresentationId),
    /// The graphical occurrence does not exist.
    MissingRepresentation(RepresentationId),
    /// The entity belongs to another project.
    ForeignProject {
        /// Project identifier required by the operation.
        expected: ProjectId,
        /// Project identifier supplied by the entity.
        found: ProjectId,
    },
    /// A relation violates a domain invariant.
    InvalidRelation(&'static str),
    /// A relation contains the same entity reference more than once.
    DuplicateReference(EntityRef),
    /// A logical connection with the same ID already exists.
    DuplicateConnectionId(ConnectionId),
    /// The logical connection does not exist.
    MissingConnection(ConnectionId),
    /// A connection cannot link a point to itself.
    SelfConnection(ConnectionPointId),
    /// A connection already links the supplied endpoints.
    DuplicateConnection {
        /// Explicit source endpoint.
        source: ConnectionPointId,
        /// Explicit target of this record or operation.
        target: ConnectionPointId,
    },
    /// The physical terminal position does not exist.
    MissingTermination(PhysicalTermination),
    /// Conductor designation is already used within this endpoint pair.
    DuplicateConductorDesignation(ConductorId),
    /// Registered cores exceed the declared cable capacity.
    CableCapacityExceeded(CableId),
    /// Core identification is already used within this cable.
    DuplicateCoreIdentification(CableId),
    /// The cable core already has a conductor.
    CoreAlreadyAssigned(CableCoreId),
    /// The conductor already has a cable core.
    ConductorAlreadyAssigned(ConductorId),
    /// Conductor properties are incompatible with the cable core.
    IncompatibleCore(CableCoreId),
    /// The terminal already belongs to a strip.
    TerminalAlreadyAssigned(TerminalId),
    /// A conductor or occurrence still references this connection.
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
/// Result of a model operation, reporting a DomainError on invalid input or relations.
pub type Result<T> = std::result::Result<T, DomainError>;
