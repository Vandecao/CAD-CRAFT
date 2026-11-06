// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Typed physical wiring values; start with endpoint and terminal value types.
use super::{DesignationPart, TerminalId};
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};

macro_rules! quantity {
    ($name:ident, $invalid:expr, $unit:literal, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
        #[serde(try_from = "f64", into = "f64")]
        pub struct $name(f64);
        impl $name {
            /// Validate a finite quantity in its declared unit and range.
            pub fn new(value: f64) -> Result<Self> {
                if !value.is_finite() || ($invalid)(value) {
                    return Err(DomainError::InvalidNumber($unit));
                }
                Ok(Self(value))
            }
            /// Return the validated scalar in the quantity's declared unit.
            pub fn get(self) -> f64 {
                self.0
            }
        }
        impl TryFrom<f64> for $name {
            type Error = DomainError;
            fn try_from(value: f64) -> Result<Self> {
                Self::new(value)
            }
        }
        impl From<$name> for f64 {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}
quantity!(
    CrossSectionMm2,
    |v: f64| v <= 0.0,
    "Seção nominal estritamente positiva em mm².",
    "Finite, strictly positive nominal cross section in square millimetres."
);
quantity!(
    LengthMm,
    |v: f64| v < 0.0,
    "Comprimento não negativo em milímetros; nunca derivado do esquema.",
    "Finite, nonnegative length in millimetres, never inferred from schematic geometry."
);
impl LengthMm {
    /// Convert metres to the internal millimetre value and validate the result.
    pub fn from_meters(meters: f64) -> Result<Self> {
        Self::new(meters * 1000.0)
    }
}

/// Physical color implying neither electrical function nor normative compliance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConductorColor {
    /// Black physical conductor color.
    Black,
    /// Brown physical conductor color.
    Brown,
    /// Red physical conductor color.
    Red,
    /// Orange physical conductor color.
    Orange,
    /// Yellow physical conductor color.
    Yellow,
    /// Green physical conductor color.
    Green,
    /// Blue physical conductor color.
    Blue,
    /// Violet physical conductor color.
    Violet,
    /// Grey physical conductor color.
    Grey,
    /// White physical conductor color.
    White,
    /// Pink physical conductor color.
    Pink,
    /// Turquoise physical conductor color.
    Turquoise,
    /// Green yellow physical conductor color.
    GreenYellow,
}
/// Declared physical conductor material.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConductorMaterial {
    /// Copper conductor.
    Copper,
    /// Aluminium conductor.
    Aluminium,
}
/// Declared conductor purpose without inferred connectivity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConductorFunction {
    /// No explicit classification supplied.
    #[default]
    Unspecified,
    /// Power wiring.
    Power,
    /// Control wiring.
    Control,
    /// Signal wiring.
    Signal,
    /// Neutral conductor.
    Neutral,
    /// Protective earth conductor.
    ProtectiveEarth,
}
/// Documentary classification of a physical cable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CableKind {
    /// Power wiring.
    Power,
    /// Control wiring.
    Control,
    /// Instrumentation cable.
    Instrumentation,
    /// Data cable.
    Data,
    /// No explicit classification supplied.
    Unspecified,
}

/// Physical identification with at least a color or a validated alphanumeric code.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "MarkingData")]
pub struct ConductorMarking {
    color: Option<ConductorColor>,
    identification: Option<DesignationPart>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MarkingData {
    color: Option<ConductorColor>,
    identification: Option<DesignationPart>,
}
impl TryFrom<MarkingData> for ConductorMarking {
    type Error = DomainError;
    fn try_from(data: MarkingData) -> Result<Self> {
        Self::new(data.color, data.identification)
    }
}
impl ConductorMarking {
    /// Create a physical marking, requiring at least a color or identification code.
    pub fn new(
        color: Option<ConductorColor>,
        identification: Option<DesignationPart>,
    ) -> Result<Self> {
        if color.is_none() && identification.is_none() {
            return Err(DomainError::InvalidText);
        }
        Ok(Self {
            color,
            identification,
        })
    }
    /// Create a marking containing only a physical color.
    pub fn color(color: ConductorColor) -> Self {
        Self {
            color: Some(color),
            identification: None,
        }
    }
    /// Create a marking containing only the validated identification code.
    pub fn identified(identification: DesignationPart) -> Self {
        Self {
            color: None,
            identification: Some(identification),
        }
    }
    /// Return the optional color value; absence represents an unset value or relation.
    pub fn color_value(&self) -> Option<ConductorColor> {
        self.color
    }
    /// Return the optional identification; absence represents an unset value or relation.
    pub fn identification(&self) -> Option<&DesignationPart> {
        self.identification.as_ref()
    }
}

/// Registered physical terminal position. It is not a functional point and creates no continuity;
/// Project verifies its existence.
/// ```compile_fail
/// # use cadcraft_model as cad_craft;
/// use cad_craft::domain::{ConductorId, CableCoreId};
/// let core: CableCoreId = ConductorId::new();
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalTermination {
    terminal_id: TerminalId,
    position: DesignationPart,
}
impl PhysicalTermination {
    /// Declare a terminal and landing position; Project validates their existence.
    pub fn new(terminal_id: TerminalId, position: DesignationPart) -> Self {
        Self {
            terminal_id,
            position,
        }
    }
    /// Return the terminal identifier.
    pub fn terminal_id(&self) -> TerminalId {
        self.terminal_id
    }
    /// Borrow the terminal landing-position designation.
    pub fn position(&self) -> &DesignationPart {
        &self.position
    }
}
