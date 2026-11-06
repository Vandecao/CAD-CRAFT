// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Validated structured designations; start with Designation and DesignationPart.
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt};

/// Nonempty label without control characters; trims surrounding whitespace.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Text(String);

impl Text {
    /// Trim and validate required text, rejecting empty content and control characters.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.trim().is_empty() || value.chars().any(char::is_control) {
            return Err(DomainError::InvalidText);
        }
        Ok(Self(value.trim().to_owned()))
    }
    /// Borrow the validated textual value.
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

/// Single part without prefixes or separators, containing letters, numbers, `_` or `.`. This
/// application policy does not claim IEC 81346 compliance.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct DesignationPart(String);
impl DesignationPart {
    /// Validate one nonempty designation segment without prefixes or separators.
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
    /// Borrow the validated textual value.
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

/// Functional and location aspects as paths, without a normative parser.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Designation {
    functional_path: Vec<DesignationPart>,
    location_path: Vec<DesignationPart>,
    device: DesignationPart,
    terminal: Option<DesignationPart>,
}
impl Designation {
    /// Create a device designation from a validated segment, initially without paths.
    pub fn new(device: impl Into<String>) -> Result<Self> {
        Ok(Self {
            functional_path: vec![],
            location_path: vec![],
            device: DesignationPart::new(device)?,
            terminal: None,
        })
    }
    /// Return this value with functional path replaced.
    pub fn with_functional_path(mut self, path: Vec<DesignationPart>) -> Self {
        self.functional_path = path;
        self
    }
    /// Return this value with location path replaced.
    pub fn with_location_path(mut self, path: Vec<DesignationPart>) -> Self {
        self.location_path = path;
        self
    }
    /// Return this value with terminal replaced.
    pub fn with_terminal(mut self, terminal: DesignationPart) -> Self {
        self.terminal = Some(terminal);
        self
    }
    /// Return the functional path in their stored order.
    pub fn functional_path(&self) -> &[DesignationPart] {
        &self.functional_path
    }
    /// Return the location path in their stored order.
    pub fn location_path(&self) -> &[DesignationPart] {
        &self.location_path
    }
    /// Return the device.
    pub fn device(&self) -> &DesignationPart {
        &self.device
    }
    /// Return the optional terminal; absence represents an unset value or relation.
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

/// Descriptive metadata; relations and electrical values use dedicated types.
pub type Metadata = BTreeMap<Text, String>;
