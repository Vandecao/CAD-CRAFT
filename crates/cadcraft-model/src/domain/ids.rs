// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Typed entity and library identifiers; start with the ID definitions below.
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use uuid::{Uuid, Variant};

macro_rules! typed_ids {
    ($($name:ident),+ $(,)?) => {$ (
        /// Strongly typed entity identifier backed by an RFC 4122 variant UUID v7.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(try_from = "Uuid", into = "Uuid")]
        pub struct $name(Uuid);

        impl $name {
            /// Generate a fresh RFC 4122 variant UUID v7 identifier.
            pub fn new() -> Self { Self(Uuid::now_v7()) }
            /// Return the underlying UUID without changing identity.
            pub fn as_uuid(self) -> Uuid { self.0 }
        }
        impl Default for $name { fn default() -> Self { Self::new() } }
        impl TryFrom<Uuid> for $name {
            type Error = DomainError;
            fn try_from(value: Uuid) -> Result<Self> {
                if value.get_version_num() != 7 || value.get_variant() != Variant::RFC4122 {
                    return Err(DomainError::InvalidId);
                }
                Ok(Self(value))
            }
        }
        impl From<$name> for Uuid { fn from(value: $name) -> Self { value.0 } }
        impl FromStr for $name {
            type Err = DomainError;
            fn from_str(value: &str) -> Result<Self> {
                Self::try_from(Uuid::parse_str(value).map_err(|_| DomainError::InvalidId)?)
            }
        }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
        }
    )+};
}

typed_ids!(
    ProjectId,
    DocumentId,
    PageId,
    ComponentId,
    DeviceId,
    FunctionId,
    TerminalId,
    ConnectionPointId,
    ConnectionId,
    ConnectionJunctionId,
    ReferenceGroupId,
    ConductorId,
    CableId,
    CableCoreId,
    TerminalStripId,
    RepresentationId,
    ConnectorId,
    ConnectorMatingId,
    TerminalLevelId,
    FrameTemplateId,
    AssetId,
    ReportId,
    SingleLineId
);

/// Typed structural reference resolved within a project. IDs of different entity kinds cannot be
/// implicitly interchanged.
/// ```compile_fail
/// # use cadcraft_model as cad_craft;
/// use cad_craft::domain::{DeviceId, FunctionId};
/// let function: FunctionId = DeviceId::new();
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EntityRef {
    /// Explicit reference to a project.
    Project(ProjectId),
    /// Explicit reference to a document.
    Document(DocumentId),
    /// Explicit reference to a page.
    Page(PageId),
    /// Explicit reference to a component.
    Component(ComponentId),
    /// Explicit reference to a device.
    Device(DeviceId),
    /// Explicit reference to a logical function.
    Function(FunctionId),
    /// Explicit reference to a terminal.
    Terminal(TerminalId),
    /// Explicit reference to a connection point.
    ConnectionPoint(ConnectionPointId),
    /// Explicit reference to a conductor.
    Conductor(ConductorId),
    /// Explicit reference to a cable.
    Cable(CableId),
    /// Explicit reference to a cable core.
    CableCore(CableCoreId),
    /// Explicit reference to a terminal strip.
    TerminalStrip(TerminalStripId),
    /// Explicit reference to a connector.
    Connector(ConnectorId),
}
