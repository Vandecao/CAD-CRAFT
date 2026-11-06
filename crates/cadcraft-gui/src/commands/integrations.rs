// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Command specifications for the integrations ribbon tab; start with the named constants.
use super::{Icon as I, PlannedGroupSpec, RibbonTab as T};

pub(super) const GROUP_INTEGRACOES_CAD_CRAFT: PlannedGroupSpec = (
    T::Integrations,
    "Integrações CAD-CRAFT",
    I::Connection,
    &[
        "Catálogos externos",
        "Colaboração",
        "Serviços de engenharia",
    ],
);
