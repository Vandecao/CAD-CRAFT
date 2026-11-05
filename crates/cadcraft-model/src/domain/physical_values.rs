// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::{DesignationPart, TerminalId};
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};

macro_rules! quantity {
    ($name:ident, $invalid:expr, $unit:literal) => {
        #[doc = $unit]
        #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
        #[serde(try_from = "f64", into = "f64")]
        pub struct $name(f64);
        impl $name {
            pub fn new(value: f64) -> Result<Self> {
                if !value.is_finite() || ($invalid)(value) {
                    return Err(DomainError::InvalidNumber($unit));
                }
                Ok(Self(value))
            }
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
    "Seção nominal estritamente positiva em mm²."
);
quantity!(
    LengthMm,
    |v: f64| v < 0.0,
    "Comprimento não negativo em milímetros; nunca derivado do esquema."
);
impl LengthMm {
    pub fn from_meters(meters: f64) -> Result<Self> {
        Self::new(meters * 1000.0)
    }
}

/// Cor física, sem inferência de função elétrica ou conformidade normativa.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConductorColor {
    Black,
    Brown,
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Violet,
    Grey,
    White,
    Pink,
    Turquoise,
    GreenYellow,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConductorMaterial {
    Copper,
    Aluminium,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConductorFunction {
    #[default]
    Unspecified,
    Power,
    Control,
    Signal,
    Neutral,
    ProtectiveEarth,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CableKind {
    Power,
    Control,
    Instrumentation,
    Data,
    Unspecified,
}

/// Identificação física: ao menos cor ou código alfanumérico validado.
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
    pub fn color(color: ConductorColor) -> Self {
        Self {
            color: Some(color),
            identification: None,
        }
    }
    pub fn identified(identification: DesignationPart) -> Self {
        Self {
            color: None,
            identification: Some(identification),
        }
    }
    pub fn color_value(&self) -> Option<ConductorColor> {
        self.color
    }
    pub fn identification(&self) -> Option<&DesignationPart> {
        self.identification.as_ref()
    }
}

/// Posição cadastrada em um terminal físico. Não é ponto funcional nem cria continuidade.
/// A existência da posição é verificada pelo Project.
/// ```compile_fail
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
    pub fn new(terminal_id: TerminalId, position: DesignationPart) -> Self {
        Self {
            terminal_id,
            position,
        }
    }
    pub fn terminal_id(&self) -> TerminalId {
        self.terminal_id
    }
    pub fn position(&self) -> &DesignationPart {
        &self.position
    }
}
