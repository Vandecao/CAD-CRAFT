// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! EDZ archive extraction bounded in memory without temporary files; start at `Archive`.
use super::{EdzContainer, EdzLimits};
use crate::{ImportResult, fail};
use std::{
    collections::BTreeMap,
    io::{Cursor, Read},
};

pub(super) fn path(raw: &str) -> ImportResult<String> {
    let path = raw.replace('\\', "/");
    if path.is_empty()
        || path.starts_with('/')
        || path.contains(':')
        || path.chars().any(char::is_control)
        || path.split('/').any(|p| p == ".." || p == ".")
    {
        return Err(fail(format!("caminho inseguro no EDZ: {raw}")));
    }
    Ok(path.trim_end_matches('/').to_lowercase())
}
fn check(size: u64, total: &mut u64, limits: &EdzLimits) -> ImportResult<()> {
    *total = total
        .checked_add(size)
        .ok_or_else(|| fail("tamanho EDZ excedido"))?;
    if size > limits.max_item_bytes || *total > limits.max_total_bytes {
        return Err(fail("item ou tamanho total do EDZ acima do limite"));
    }
    Ok(())
}
fn read(reader: &mut dyn Read, size: u64, limits: &EdzLimits) -> ImportResult<Vec<u8>> {
    let mut bytes = Vec::new();
    reader
        .take(limits.max_item_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|e| fail(format!("item EDZ inválido: {e}")))?;
    if bytes.len() as u64 != size || bytes.len() as u64 > limits.max_item_bytes {
        return Err(fail("tamanho do item EDZ inválido ou acima do limite"));
    }
    Ok(bytes)
}
pub(super) fn open(
    bytes: &[u8],
    limits: &EdzLimits,
) -> ImportResult<(EdzContainer, BTreeMap<String, Vec<u8>>)> {
    if bytes.len() as u64 > limits.max_input_bytes {
        return Err(fail("pacote EDZ acima do limite"));
    }
    let mut files = BTreeMap::new();
    let mut total = 0;
    let container = if bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06") {
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
            .map_err(|e| fail(format!("ZIP inválido ou truncado: {e}")))?;
        if archive.len() > limits.max_entries {
            return Err(fail("itens demais no EDZ"));
        }
        for i in 0..archive.len() {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| fail(format!("ZIP inválido: {e}")))?;
            let raw = entry
                .name()
                .map_err(|e| fail(format!("nome ZIP inválido: {e}")))?;
            if entry.is_dir() && matches!(raw.as_ref(), "" | "." | "./") {
                continue;
            }
            let name = path(&raw)?;
            if entry.is_symlink() {
                return Err(fail("link simbólico recusado no EDZ"));
            }
            check(entry.size(), &mut total, limits)?;
            if entry.is_dir() {
                continue;
            }
            let size = entry.size();
            let content = read(&mut entry, size, limits)?;
            if files.insert(name, content).is_some() {
                return Err(fail("caminho duplicado no EDZ"));
            }
        }
        EdzContainer::Zip
    } else if bytes.starts_with(b"7z\xbc\xaf\x27\x1c") {
        super::sevenz_guard::preflight(bytes, limits)?;
        let mut archive =
            sevenz_rust2::ArchiveReader::new(Cursor::new(bytes), sevenz_rust2::Password::empty())
                .map_err(|e| fail(format!("7z inválido ou truncado: {e}")))?;
        archive.set_thread_count(1);
        super::sevenz_guard::post_archive(archive.archive(), limits)?;
        if archive.archive().files.len() > limits.max_entries {
            return Err(fail("itens demais no EDZ"));
        }
        let mut names = std::collections::BTreeSet::new();
        for entry in &archive.archive().files {
            if entry.is_directory && matches!(entry.name.as_str(), "" | "." | "./") {
                continue;
            }
            let name = path(&entry.name)?;
            if !names.insert(name) {
                return Err(fail("caminho duplicado no EDZ"));
            }
            if entry.is_anti_item || entry.windows_attributes & 0x400 != 0 {
                return Err(fail("link ou item especial recusado no EDZ"));
            }
            check(entry.size, &mut total, limits)?;
        }
        if archive
            .archive()
            .blocks
            .iter()
            .any(|b| b.get_unpack_size() > limits.max_total_bytes)
        {
            return Err(fail("bloco 7z acima do limite"));
        }
        archive
            .for_each_entries(|entry, reader| {
                if !entry.is_directory {
                    let name =
                        path(&entry.name).map_err(|e| sevenz_rust2::Error::Other(e.0.into()))?;
                    let content = read(reader, entry.size, limits)
                        .map_err(|e| sevenz_rust2::Error::Other(e.0.into()))?;
                    files.insert(name, content);
                }
                Ok(true)
            })
            .map_err(|e| fail(format!("7z inválido ou truncado: {e}")))?;
        EdzContainer::SevenZ
    } else {
        return Err(fail("assinatura EDZ desconhecida: esperado ZIP ou 7z"));
    };
    Ok((container, files))
}
