// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Catalog parts and function templates; start with Part and PartFunction.
use super::{FunctionKind, FunctionRole, SymbolDefinitionId, TechnicalProperties, Text};
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Chave textual estável da peça, independente dos IDs UUID do projeto.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PartId(String);
impl PartId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.trim() != value
            || value.is_empty()
            || value.len() > 160
            || value.chars().any(|c| c.is_control() || c.is_whitespace())
        {
            return Err(DomainError::InvalidText);
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl TryFrom<String> for PartId {
    type Error = DomainError;
    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}
impl From<PartId> for String {
    fn from(value: PartId) -> Self {
        value.0
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "PartFunctionData")]
pub struct PartFunction {
    kind: FunctionKind,
    role: FunctionRole,
    points: Vec<Text>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suggested_symbol: Option<SymbolDefinitionId>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PartFunctionData {
    kind: FunctionKind,
    role: FunctionRole,
    points: Vec<Text>,
    #[serde(default)]
    suggested_symbol: Option<SymbolDefinitionId>,
}
impl TryFrom<PartFunctionData> for PartFunction {
    type Error = DomainError;
    fn try_from(data: PartFunctionData) -> Result<Self> {
        let mut result = Self::new(data.kind, data.role, data.points.iter().map(Text::as_str))?;
        result.suggested_symbol = data.suggested_symbol;
        Ok(result)
    }
}
impl PartFunction {
    pub fn new<'a>(
        kind: FunctionKind,
        role: FunctionRole,
        points: impl IntoIterator<Item = &'a str>,
    ) -> Result<Self> {
        let points: Vec<Text> = points.into_iter().map(Text::new).collect::<Result<_>>()?;
        if points.is_empty()
            || points.len() > 128
            || points.iter().collect::<BTreeSet<_>>().len() != points.len()
        {
            return Err(DomainError::InvalidRelation(
                "pontos da função ausentes ou duplicados",
            ));
        }
        Ok(Self {
            kind,
            role,
            points,
            suggested_symbol: None,
        })
    }
    pub fn kind(&self) -> &FunctionKind {
        &self.kind
    }
    pub fn role(&self) -> FunctionRole {
        self.role
    }
    pub fn points(&self) -> &[Text] {
        &self.points
    }
    pub fn suggested_symbol(&self) -> Option<&SymbolDefinitionId> {
        self.suggested_symbol.as_ref()
    }
    pub fn with_suggested_symbol(mut self, symbol: SymbolDefinitionId) -> Self {
        self.suggested_symbol = Some(symbol);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "PartData")]
pub struct Part {
    id: PartId,
    manufacturer: Text,
    code: Text,
    description: Text,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    translations: BTreeMap<Text, Text>,
    classification: Text,
    #[serde(default)]
    properties: TechnicalProperties,
    #[serde(default)]
    functions: Vec<PartFunction>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PartData {
    id: PartId,
    manufacturer: Text,
    code: Text,
    description: Text,
    #[serde(default)]
    translations: BTreeMap<Text, Text>,
    classification: Text,
    #[serde(default)]
    properties: TechnicalProperties,
    #[serde(default)]
    functions: Vec<PartFunction>,
}
impl TryFrom<PartData> for Part {
    type Error = DomainError;
    fn try_from(data: PartData) -> Result<Self> {
        let part = Self {
            id: data.id,
            manufacturer: data.manufacturer,
            code: data.code,
            description: data.description,
            translations: data.translations,
            classification: data.classification,
            properties: data.properties,
            functions: data.functions,
        };
        part.validate()?;
        Ok(part)
    }
}
impl Part {
    pub fn validate(&self) -> Result<()> {
        if self.functions.len() > 128 {
            return Err(DomainError::InvalidRelation("funções demais na peça"));
        }
        if self.properties.references().next().is_some() {
            return Err(DomainError::InvalidRelation(
                "peça da biblioteca não pode referenciar entidades do projeto",
            ));
        }
        for (index, function) in self.functions.iter().enumerate() {
            if self.functions[..index]
                .iter()
                .any(|earlier| earlier.kind == function.kind && earlier.points == function.points)
            {
                return Err(DomainError::InvalidRelation(
                    "função duplicada no modelo da peça",
                ));
            }
        }
        Ok(())
    }
    pub fn new(
        id: impl Into<String>,
        manufacturer: impl Into<String>,
        code: impl Into<String>,
        description: impl Into<String>,
        classification: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            id: PartId::new(id)?,
            manufacturer: Text::new(manufacturer)?,
            code: Text::new(code)?,
            description: Text::new(description)?,
            translations: BTreeMap::new(),
            classification: Text::new(classification)?,
            properties: TechnicalProperties::default(),
            functions: Vec::new(),
        })
    }
    pub fn id(&self) -> &PartId {
        &self.id
    }
    pub fn manufacturer(&self) -> &Text {
        &self.manufacturer
    }
    pub fn code(&self) -> &Text {
        &self.code
    }
    pub fn description(&self) -> &Text {
        &self.description
    }
    pub fn classification(&self) -> &Text {
        &self.classification
    }
    pub fn properties(&self) -> &TechnicalProperties {
        &self.properties
    }
    pub fn functions(&self) -> &[PartFunction] {
        &self.functions
    }
    pub fn translations(&self) -> &BTreeMap<Text, Text> {
        &self.translations
    }
    pub fn with_properties(mut self, properties: TechnicalProperties) -> Self {
        self.properties = properties;
        self
    }
    pub fn with_translation(
        mut self,
        language: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<Self> {
        self.translations
            .insert(Text::new(language)?, Text::new(description)?);
        Ok(self)
    }
    pub fn with_functions(mut self, functions: Vec<PartFunction>) -> Result<Self> {
        self.functions = functions;
        self.validate()?;
        Ok(self)
    }
}
