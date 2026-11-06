// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Bounded EPLAN manufacturer archives; start at `import`, then read, convert and report.
mod archive;
mod parts;
mod pxf;
mod sevenz_guard;
use super::{ImportReport, ImportResult, fail, xml};
use cadcraft_model::domain::{MacroDefinition, Part, PropertyValue};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdzContainer {
    Zip,
    SevenZ,
}
#[derive(Debug, Clone)]
pub struct EdzLimits {
    pub max_input_bytes: u64,
    pub max_entries: usize,
    pub max_item_bytes: u64,
    pub max_total_bytes: u64,
}
impl Default for EdzLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 128 * 1024 * 1024,
            max_entries: 10_000,
            max_item_bytes: 64 * 1024 * 1024,
            max_total_bytes: 256 * 1024 * 1024,
        }
    }
}
/// Identidade estável e segura para o catálogo (não é caminho de disco).
pub(super) fn identifier(raw: &str) -> String {
    let name: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .take(120)
        .collect();
    // Hash FNV-1a estável evita colisões após substituição de caracteres.
    let hash = raw.bytes().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ b as u64).wrapping_mul(0x100000001b3)
    });
    format!("EDZ.{name}.{hash:016x}")
}

mod convert;
mod read;
mod report;
pub use report::EdzImport;

pub fn import(bytes: &[u8]) -> ImportResult<EdzImport> {
    import_with_limits(bytes, &EdzLimits::default())
}
pub fn import_with_limits(bytes: &[u8], limits: &EdzLimits) -> ImportResult<EdzImport> {
    convert::convert(read::read(bytes, limits)?)
}
