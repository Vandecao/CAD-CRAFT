// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Typed technical properties and entity references; start with TechnicalProperties.
use super::{EntityRef, Text};
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Finite f64 technical value; does not provide exact decimal arithmetic.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "f64", into = "f64")]
pub struct Decimal(f64);
impl Decimal {
    /// Create a technical numeric value, rejecting NaN and infinity.
    pub fn new(value: f64) -> Result<Self> {
        if !value.is_finite() {
            return Err(DomainError::InvalidNumber("decimal"));
        }
        Ok(Self(value))
    }
    /// Return the validated numeric value.
    pub fn get(self) -> f64 {
        self.0
    }
}
impl TryFrom<f64> for Decimal {
    type Error = DomainError;
    fn try_from(v: f64) -> Result<Self> {
        Self::new(v)
    }
}
impl From<Decimal> for f64 {
    fn from(v: Decimal) -> Self {
        v.0
    }
}

/// Named technical enumeration with validated choices and selected value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "EnumData")]
pub struct EnumValue {
    name: Text,
    choices: Vec<Text>,
    selected: Text,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EnumData {
    name: Text,
    choices: Vec<Text>,
    selected: Text,
}
impl TryFrom<EnumData> for EnumValue {
    type Error = DomainError;
    fn try_from(v: EnumData) -> Result<Self> {
        let unique: BTreeSet<_> = v.choices.iter().collect();
        if unique.len() != v.choices.len() || !unique.contains(&v.selected) {
            return Err(DomainError::InvalidEnumeration);
        }
        Ok(Self {
            name: v.name,
            choices: v.choices,
            selected: v.selected,
        })
    }
}
impl EnumValue {
    /// Create an enumeration with a name, unique choices and an existing selected value.
    pub fn new<'a>(
        name: &str,
        choices: impl IntoIterator<Item = &'a str>,
        selected: &str,
    ) -> Result<Self> {
        Self::try_from(EnumData {
            name: Text::new(name)?,
            choices: choices.into_iter().map(Text::new).collect::<Result<_>>()?,
            selected: Text::new(selected)?,
        })
    }
    /// Return the name.
    pub fn name(&self) -> &Text {
        &self.name
    }
    /// Return the choices in their stored order.
    pub fn choices(&self) -> &[Text] {
        &self.choices
    }
    /// Return the selected.
    pub fn selected(&self) -> &Text {
        &self.selected
    }
}

/// Typed value of a custom technical property.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PropertyValue {
    /// Text content.
    Text(String),
    /// Signed integer property.
    Integer(i64),
    /// Finite numeric property.
    Decimal(Decimal),
    /// Boolean property.
    Boolean(bool),
    /// Validated enumeration property.
    Enumeration(EnumValue),
    /// Typed reference to a project entity.
    Reference(EntityRef),
}

/// Known properties: voltage in V, current in A, power in W, frequency in Hz, resistance in ohms
/// and cross section in mm². All known values are nonnegative.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KnownProperty {
    /// Rated voltage in volts.
    RatedVoltage,
    /// Rated current in amperes.
    RatedCurrent,
    /// Rated power in watts.
    RatedPower,
    /// Frequency in hertz.
    Frequency,
    /// Resistance in ohms.
    Resistance,
    /// Cross section in square millimetres.
    CrossSection,
}

/// Known numeric properties and named typed custom properties.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "PropertiesData")]
pub struct TechnicalProperties {
    known: BTreeMap<KnownProperty, Decimal>,
    custom: BTreeMap<Text, PropertyValue>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PropertiesData {
    known: BTreeMap<KnownProperty, Decimal>,
    custom: BTreeMap<Text, PropertyValue>,
}
impl TryFrom<PropertiesData> for TechnicalProperties {
    type Error = DomainError;
    fn try_from(v: PropertiesData) -> Result<Self> {
        if v.known.values().any(|d| d.get() < 0.0) {
            return Err(DomainError::InvalidNumber("propriedade conhecida"));
        }
        Ok(Self {
            known: v.known,
            custom: v.custom,
        })
    }
}
impl TechnicalProperties {
    /// Validate and set a nonnegative numeric property using its known key and units.
    pub fn set_known(&mut self, key: KnownProperty, value: f64) -> Result<()> {
        let value = Decimal::new(value)?;
        if value.get() < 0.0 {
            return Err(DomainError::InvalidNumber("propriedade conhecida"));
        }
        self.known.insert(key, value);
        Ok(())
    }
    /// Insert or replace a named custom property after validating its key.
    pub fn set_custom(&mut self, key: impl Into<String>, value: PropertyValue) -> Result<()> {
        self.custom.insert(Text::new(key)?, value);
        Ok(())
    }
    /// Return the known.
    pub fn known(&self) -> &BTreeMap<KnownProperty, Decimal> {
        &self.known
    }
    /// Return the custom.
    pub fn custom(&self) -> &BTreeMap<Text, PropertyValue> {
        &self.custom
    }
    /// Iterate typed entity references stored in custom properties.
    pub fn references(&self) -> impl Iterator<Item = EntityRef> + '_ {
        self.custom.values().filter_map(|v| match v {
            PropertyValue::Reference(id) => Some(*id),
            _ => None,
        })
    }
}
