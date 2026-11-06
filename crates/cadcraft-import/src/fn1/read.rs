// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Reads raw EPLAN frame XML before conversion; start at `read`.
use super::*;

pub(super) fn read(bytes: &[u8]) -> ImportResult<Element> {
    check_size(bytes)?;
    xml::parse(bytes)
}
