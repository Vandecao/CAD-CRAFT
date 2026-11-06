// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Reads bounded EDZ archives and their raw manifests; start at `read`.
use super::*;

pub(super) struct RawEdz {
    pub(super) container: EdzContainer,
    pub(super) files: BTreeMap<String, Vec<u8>>,
    pub(super) manifest: xml::Element,
}

pub(super) fn read(bytes: &[u8], limits: &EdzLimits) -> ImportResult<RawEdz> {
    let (container, files) = archive::open(bytes, limits)?;
    let manifest = xml::parse(
        files
            .get("manifest.xml")
            .ok_or_else(|| fail("EDZ sem manifest.xml"))?,
    )?;
    if manifest.name != "manifest" || manifest.attr("version") != Some("2.0") {
        return Err(fail("manifesto EDZ inválido ou versão não suportada"));
    }
    Ok(RawEdz {
        container,
        files,
        manifest,
    })
}
