// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Command specifications for the cables ribbon tab; start with the named constants.
use super::{CommandId as C, CommandSpec, Icon as I, Requirement as R, RibbonTab as T};

pub(super) const CABLE_EDITOR: CommandSpec = (
    C::CableEditor,
    "Editor de cabos",
    "Cria cabos e gerencia veias e suas atribuições",
    T::Cables,
    "Cabos",
    R::Project,
    I::Connection,
    "veia cabo fiação",
);

pub(super) const TECHNICAL_LISTS: CommandSpec = (
    C::TechnicalLists,
    "Listas técnicas",
    "Consulta registros derivados de bornes, cabos e condutores",
    T::Cables,
    "Listas",
    R::Project,
    I::Panels,
    "lista relatório tabular",
);
