// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Typed technical properties and entity references; start with TechnicalProperties.
use super::{EntityRef, Text};
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Decimal técnico em f64 finito; não representa aritmética decimal exata.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "f64", into = "f64")]
pub struct Decimal(f64);
impl Decimal {
    pub fn new(value: f64) -> Result<Self> {
        if !value.is_finite() {
            return Err(DomainError::InvalidNumber("decimal"));
        }
        Ok(Self(value))
    }
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
    pub fn name(&self) -> &Text {
        &self.name
    }
    pub fn choices(&self) -> &[Text] {
        &self.choices
    }
    pub fn selected(&self) -> &Text {
        &self.selected
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PropertyValue {
    Text(String),
    Integer(i64),
    Decimal(Decimal),
    Boolean(bool),
    Enumeration(EnumValue),
    Reference(EntityRef),
}

/// Propriedades conhecidas: tensão em V, corrente em A, potência em W,
/// frequência em Hz, resistência em ohm e seção transversal em mm².
/// Todos os valores conhecidos são não negativos.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KnownProperty {
    RatedVoltage,
    RatedCurrent,
    RatedPower,
    Frequency,
    Resistance,
    CrossSection,
}

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
    pub fn set_known(&mut self, key: KnownProperty, value: f64) -> Result<()> {
        let value = Decimal::new(value)?;
        if value.get() < 0.0 {
            return Err(DomainError::InvalidNumber("propriedade conhecida"));
        }
        self.known.insert(key, value);
        Ok(())
    }
    pub fn set_custom(&mut self, key: impl Into<String>, value: PropertyValue) -> Result<()> {
        self.custom.insert(Text::new(key)?, value);
        Ok(())
    }
    pub fn known(&self) -> &BTreeMap<KnownProperty, Decimal> {
        &self.known
    }
    pub fn custom(&self) -> &BTreeMap<Text, PropertyValue> {
        &self.custom
    }
    pub fn references(&self) -> impl Iterator<Item = EntityRef> + '_ {
        self.custom.values().filter_map(|v| match v {
            PropertyValue::Reference(id) => Some(*id),
            _ => None,
        })
    }
}
