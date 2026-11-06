// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Shared DXF group protocol and units; start with parse_groups and write_group.
//!
//! These low-level interoperability values have no Project or rendering dependency.
//! Writers retain their own numeric rounding and text encoding policies.
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DxfError {
    InvalidGroupCode(String),
    Truncated,
    TooLarge,
}
impl fmt::Display for DxfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidGroupCode(code) => write!(f, "DXF inválido: código de grupo '{code}'"),
            Self::Truncated => f.write_str("DXF truncado (valor ausente)"),
            Self::TooLarge => f.write_str("DXF grande demais"),
        }
    }
}
impl std::error::Error for DxfError {}

/// Decode UTF-8 (with optional BOM), falling back to the legacy Latin-1 policy.
pub fn decode(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.trim_start_matches('\u{feff}').to_owned(),
        Err(_) => bytes.iter().map(|&b| b as char).collect(),
    }
}

/// Read bounded code/value pairs without interpreting drawing entities.
pub fn parse_groups(text: &str) -> Result<Vec<(i32, String)>, DxfError> {
    let mut lines = text.lines();
    let mut out = Vec::new();
    while let Some(code) = lines.next() {
        let code = code.trim();
        if code.is_empty() && out.is_empty() {
            continue;
        }
        let code: i32 = code
            .parse()
            .map_err(|_| DxfError::InvalidGroupCode(code.to_owned()))?;
        let value = lines.next().ok_or(DxfError::Truncated)?;
        out.push((code, value.trim_end_matches('\r').trim().to_owned()));
        if out.len() > 20_000_000 {
            return Err(DxfError::TooLarge);
        }
    }
    Ok(out)
}

/// Write the existing R12 group layout, shared by frame and page exporters.
pub fn write_group(out: &mut impl fmt::Write, code: i32, value: &str) -> fmt::Result {
    writeln!(out, "{code:>3}\n{value}")
}

/// Convert supported INSUNITS codes to millimetres per drawing unit.
pub fn unit_mm(code: i64) -> Option<f64> {
    Some(match code {
        1 => 25.4,
        2 => 304.8,
        4 => 1.0,
        5 => 10.0,
        6 => 1000.0,
        8 => 25.4e-6,
        9 => 25.4e-3,
        10 => 914.4,
        13 => 1e-3,
        14 => 100.0,
        _ => return None,
    })
}
