// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt};

/// Rótulo não vazio, sem controles; espaços externos são removidos.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Text(String);

impl Text {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.trim().is_empty() || value.chars().any(char::is_control) {
            return Err(DomainError::InvalidText);
        }
        Ok(Self(value.trim().to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl TryFrom<String> for Text {
    type Error = DomainError;
    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}
impl From<Text> for String {
    fn from(value: Text) -> Self {
        value.0
    }
}

/// Uma parte sem prefixos/separadores, composta por letras, números, `_` ou `.`.
/// Política básica do aplicativo; não constitui conformidade IEC 81346.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct DesignationPart(String);
impl DesignationPart {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.is_empty()
            || !value
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
        {
            return Err(DomainError::InvalidDesignation);
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl TryFrom<String> for DesignationPart {
    type Error = DomainError;
    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}
impl From<DesignationPart> for String {
    fn from(value: DesignationPart) -> Self {
        value.0
    }
}

/// Aspectos funcionais e de localização como caminhos; sem parser normativo.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Designation {
    functional_path: Vec<DesignationPart>,
    location_path: Vec<DesignationPart>,
    device: DesignationPart,
    terminal: Option<DesignationPart>,
}
impl Designation {
    pub fn new(device: impl Into<String>) -> Result<Self> {
        Ok(Self {
            functional_path: vec![],
            location_path: vec![],
            device: DesignationPart::new(device)?,
            terminal: None,
        })
    }
    pub fn with_functional_path(mut self, path: Vec<DesignationPart>) -> Self {
        self.functional_path = path;
        self
    }
    pub fn with_location_path(mut self, path: Vec<DesignationPart>) -> Self {
        self.location_path = path;
        self
    }
    pub fn with_terminal(mut self, terminal: DesignationPart) -> Self {
        self.terminal = Some(terminal);
        self
    }
    pub fn functional_path(&self) -> &[DesignationPart] {
        &self.functional_path
    }
    pub fn location_path(&self) -> &[DesignationPart] {
        &self.location_path
    }
    pub fn device(&self) -> &DesignationPart {
        &self.device
    }
    pub fn terminal(&self) -> Option<&DesignationPart> {
        self.terminal.as_ref()
    }
}
impl fmt::Display for Designation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for part in &self.functional_path {
            write!(f, "={}", part.as_str())?;
        }
        for part in &self.location_path {
            write!(f, "+{}", part.as_str())?;
        }
        write!(f, "-{}", self.device.as_str())?;
        if let Some(terminal) = &self.terminal {
            write!(f, ":{}", terminal.as_str())?;
        }
        Ok(())
    }
}

/// Metadados descritivos. Relações e valores elétricos ficam nos tipos próprios.
pub type Metadata = BTreeMap<Text, String>;
