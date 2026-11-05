// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Importação defensiva de pacotes de fabricante EPLAN (.edz).
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
#[derive(Debug, Clone)]
pub struct EdzImport {
    pub container: EdzContainer,
    pub parts: Vec<Part>,
    pub macros: Vec<MacroDefinition>,
    pub report: ImportReport,
    pub unknown_codes: BTreeSet<String>,
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
fn outside_scope(kind: &str, locator: &str) -> bool {
    kind == "picture"
        || kind == "document"
        || (kind == "macro" && locator.to_ascii_lowercase().contains("3d"))
}
pub fn import(bytes: &[u8]) -> ImportResult<EdzImport> {
    import_with_limits(bytes, &EdzLimits::default())
}
pub fn import_with_limits(bytes: &[u8], limits: &EdzLimits) -> ImportResult<EdzImport> {
    let (container, files) = archive::open(bytes, limits)?;
    let manifest = xml::parse(
        files
            .get("manifest.xml")
            .ok_or_else(|| fail("EDZ sem manifest.xml"))?,
    )?;
    if manifest.name != "manifest" || manifest.attr("version") != Some("2.0") {
        return Err(fail("manifesto EDZ inválido ou versão não suportada"));
    }
    let mut output = EdzImport {
        container,
        parts: Vec::new(),
        macros: Vec::new(),
        report: ImportReport::default(),
        unknown_codes: BTreeSet::new(),
    };
    let mut converted_macros: BTreeMap<String, Option<MacroDefinition>> = BTreeMap::new();
    // Validar TODAS as referências antes de converter qualquer dado.
    for item in parts::descendants(&manifest, "item") {
        let kind = item.attr("type").ok_or_else(|| fail("item sem tipo"))?;
        let raw_locator = item
            .attr("locator")
            .ok_or_else(|| fail("item sem locator"))?;
        if raw_locator.is_empty() {
            output
                .report
                .ignored(format!("referência vazia no manifesto: {kind}"));
            continue;
        }
        let locator = archive::path(raw_locator)?;
        let path = archive::path(&format!("items/{kind}/{locator}"))?;
        if !files.contains_key(&path) {
            if outside_scope(kind, raw_locator) {
                output
                    .report
                    .ignored(format!("item fora de escopo ausente: {path}"));
            } else {
                return Err(fail(format!("manifesto aponta item inexistente: {path}")));
            }
        }
    }
    for package in parts::descendants(&manifest, "package") {
        if package.attr("type") != Some("part") {
            output.report.ignored(format!(
                "pacote de tipo {}",
                package.attr("type").unwrap_or("ausente")
            ));
            continue;
        }
        let items = parts::descendants(package, "item");
        let mut manufacturers = BTreeMap::new();
        let mut terminals = Vec::new();
        let mut part_trees = Vec::new();
        let mut macro_ids = Vec::new();
        for item in items {
            let kind = item.attr("type").unwrap_or("");
            let locator = item.attr("locator").unwrap_or("");
            if locator.is_empty() {
                continue;
            }
            let path = archive::path(&format!("items/{kind}/{locator}"))?;
            let Some(data) = files.get(&path) else {
                continue;
            };
            match kind {
                "partxml" => {
                    let root = xml::parse(data).map_err(|e| fail(format!("{path}: {e}")))?;
                    if path.ends_with(".part.xml") {
                        part_trees.push(root);
                    } else if path.ends_with(".manufacturer.xml") {
                        for addr in parts::descendants(&root, "address") {
                            if let Some(short) = addr.attr("P_PART_ADDRESS_SHORTNAME") {
                                manufacturers.insert(
                                    short.to_owned(),
                                    addr.attr("P_PART_ADDRESS_LONGNAME")
                                        .or(addr.attr("P_PART_ADDRESS_NAME1"))
                                        .unwrap_or(short)
                                        .to_owned(),
                                );
                            }
                        }
                    } else if path.ends_with(".connectionpoints.xml") {
                        terminals.push(root);
                    } else {
                        output
                            .report
                            .ignored(format!("partxml não mapeado: {locator}"));
                    }
                }
                "macro" => {
                    if outside_scope(kind, locator)
                        || path.ends_with(".stp")
                        || path.ends_with(".step")
                    {
                        output.report.ignored(format!("macro 3D: {locator}"));
                        continue;
                    }
                    if !path.ends_with(".ema") {
                        output
                            .report
                            .ignored(format!("macro não suportada: {locator}"));
                        continue;
                    }
                    let id = identifier(&path);
                    let converted = if let Some(cached) = converted_macros.get(&path) {
                        cached.clone()
                    } else {
                        let value =
                            pxf::convert(data, &id, &mut output.report, &mut output.unknown_codes)?;
                        converted_macros.insert(path.clone(), value.clone());
                        value
                    };
                    match converted {
                        Some(def) => {
                            macro_ids.push(def.id().as_str().to_owned());
                            if !output.macros.iter().any(|m| m.id() == def.id()) {
                                output.macros.push(def);
                            }
                        }
                        None => output
                            .report
                            .ignored(format!("macro sem conteúdo 2D: {locator}")),
                    }
                }
                _ => output.report.ignored(format!("item {kind}: {locator}")),
            }
        }
        for tree in part_trees {
            for mut part in parts::convert(&tree, &manufacturers, &terminals, &mut output.report)? {
                if !macro_ids.is_empty() {
                    let mut props = part.properties().clone();
                    props
                        .set_custom(
                            "CADCRAFT.MacroIds",
                            PropertyValue::Text(macro_ids.join(";")),
                        )
                        .map_err(|e| fail(e.to_string()))?;
                    part = part.with_properties(props);
                }
                if output.parts.iter().any(|p| p.id() == part.id()) {
                    output
                        .report
                        .info("peça repetida no pacote; primeira definição mantida");
                } else {
                    output.parts.push(part);
                }
            }
        }
    }
    Ok(output)
}
