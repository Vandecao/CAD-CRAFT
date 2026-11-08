// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Function kind and designation prefix recorded in symbol metadata; start
//! with SymbolDefinition::function_kind_hint.
//!
//! The values live in the existing metadata map, so symbol files keep their
//! format; symbols without them behave as before (the insertion falls back to
//! the symbol name and the "D" prefix).
use super::*;

/// Metadata key holding the function kind of a new function.
pub const METADATA_FUNCTION_KIND: &str = "function-kind";
/// Metadata key holding the designation prefix of a new device.
pub const METADATA_PREFIX: &str = "designation-prefix";
/// Stored value for a coil function.
pub const KIND_COIL: &str = "coil";
/// Stored value for a main contact function.
pub const KIND_MAIN_CONTACT: &str = "main-contact";
/// Stored value for an auxiliary contact function.
pub const KIND_AUXILIARY_CONTACT: &str = "auxiliary-contact";

/// Function kind choice offered by the symbol editor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SymbolFunctionKind {
    /// No kind recorded: the function is named after the symbol.
    #[default]
    Other,
    /// Coil.
    Coil,
    /// Main contact (power pole).
    MainContact,
    /// Auxiliary contact.
    AuxiliaryContact,
}
impl SymbolFunctionKind {
    /// Every choice, in the order shown to the user.
    pub const ALL: [Self; 4] = [
        Self::Other,
        Self::Coil,
        Self::MainContact,
        Self::AuxiliaryContact,
    ];
    /// Return the pt-BR display label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Other => "Outro",
            Self::Coil => "Bobina",
            Self::MainContact => "Contato principal",
            Self::AuxiliaryContact => "Contato auxiliar",
        }
    }
    fn stored(self) -> Option<&'static str> {
        match self {
            Self::Other => None,
            Self::Coil => Some(KIND_COIL),
            Self::MainContact => Some(KIND_MAIN_CONTACT),
            Self::AuxiliaryContact => Some(KIND_AUXILIARY_CONTACT),
        }
    }
    fn parse(value: &str) -> Self {
        match value {
            KIND_COIL => Self::Coil,
            KIND_MAIN_CONTACT => Self::MainContact,
            KIND_AUXILIARY_CONTACT => Self::AuxiliaryContact,
            _ => Self::Other,
        }
    }
}

fn key(name: &str) -> Text {
    Text::new(name).expect("static metadata key")
}

impl SymbolDefinition {
    /// Function kind recorded in the symbol, `Other` when absent.
    pub fn symbol_function_kind(&self) -> SymbolFunctionKind {
        self.metadata()
            .get(&key(METADATA_FUNCTION_KIND))
            .map_or(SymbolFunctionKind::Other, |v| SymbolFunctionKind::parse(v))
    }
    /// Function kind for a new function placed with this symbol, if recorded.
    pub fn function_kind_hint(&self) -> Option<FunctionKind> {
        match self.symbol_function_kind() {
            SymbolFunctionKind::Other => None,
            SymbolFunctionKind::Coil => Some(FunctionKind::Coil),
            SymbolFunctionKind::MainContact => Some(FunctionKind::MainContact),
            SymbolFunctionKind::AuxiliaryContact => Some(FunctionKind::AuxiliaryContact),
        }
    }
    /// Designation prefix recorded in the symbol (K, KT, Q…), if any.
    pub fn designation_prefix(&self) -> Option<&str> {
        self.metadata()
            .get(&key(METADATA_PREFIX))
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
    }
    /// Copy of the metadata with the function kind and prefix replaced; an
    /// empty prefix or `Other` removes the entry.
    pub fn metadata_with_hints(&self, kind: SymbolFunctionKind, prefix: &str) -> Metadata {
        let mut metadata = self.metadata().clone();
        match kind.stored() {
            Some(value) => {
                metadata.insert(key(METADATA_FUNCTION_KIND), value.into());
            }
            None => {
                metadata.remove(&key(METADATA_FUNCTION_KIND));
            }
        }
        let prefix = prefix.trim();
        if prefix.is_empty() {
            metadata.remove(&key(METADATA_PREFIX));
        } else {
            metadata.insert(key(METADATA_PREFIX), prefix.into());
        }
        metadata
    }
}
