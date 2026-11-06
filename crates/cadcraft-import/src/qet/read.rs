// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Reads and validates raw QElectroTech XML; start at `read`.
use super::*;

pub(super) fn read(bytes: &[u8], options: &QetOptions) -> ImportResult<Element> {
    check_size(bytes)?;
    let s = options.mm_per_px;
    if !(s.is_finite() && s > 0.0) {
        return Err(fail("escala inválida"));
    }
    let root = xml::parse(bytes)?;
    if root.name != "project" {
        return Err(fail("o arquivo não é um projeto do QElectroTech"));
    }
    if count_nodes(&root) > MAX_QET_NODES {
        return Err(fail("projeto grande demais"));
    }
    Ok(root)
}
