// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Command specifications for the preplanning ribbon tab; start with the named constants.
use super::{Icon as I, PlannedGroupSpec, RibbonTab as T};

pub(super) const GROUP_GERAL: PlannedGroupSpec = (
    T::Preplanning,
    "Geral",
    I::Panels,
    &["Navegador", "Segmentos modelo", "Importar"],
);

pub(super) const GROUP_EDITAR: PlannedGroupSpec = (
    T::Preplanning,
    "Editar",
    I::Settings,
    &["Configurar segmentos", "Numerar pontos"],
);

pub(super) const GROUP_TUBULACAO: PlannedGroupSpec = (
    T::Preplanning,
    "Tubulação",
    I::Connection,
    &["Navegador de tubulação", "Editar forma"],
);

pub(super) const GROUP_DIAGRAMA_P_I: PlannedGroupSpec = (
    T::Preplanning,
    "Diagrama P&I",
    I::Device,
    &["Reservatório", "Ponto de conexão", "Ponto de definição"],
);

pub(super) const GROUP_MACRO_DE_PRE_PLANEJAMENTO: PlannedGroupSpec = (
    T::Preplanning,
    "Macro de pré-planejamento",
    I::Insert,
    &["Inserir", "Criar"],
);
