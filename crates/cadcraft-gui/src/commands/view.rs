// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Command specifications for the view ribbon tab; start with the named constants.
use super::{
    CommandId as C, CommandSpec, Icon as I, PlannedGroupSpec, Requirement as R, RibbonTab as T,
};

pub(super) const PAGE_PROPERTIES: CommandSpec = (
    C::PageProperties,
    "Propriedades da página",
    "Seleciona a página ativa no inspetor",
    T::View,
    "Página",
    R::ActivePage,
    I::Properties,
    "folha",
);

pub(super) const FIT_PAGE: CommandSpec = (
    C::FitPage,
    "Enquadrar página",
    "Ajusta a câmera para a folha inteira",
    T::View,
    "Câmera",
    R::ActivePage,
    I::Fit,
    "zoom página inteira",
);

pub(super) const FIT_CONTENT: CommandSpec = (
    C::FitContent,
    "Enquadrar conteúdo",
    "Ajusta limites documentais, inclusive fora da folha e invisíveis",
    T::View,
    "Câmera",
    R::ActivePage,
    I::Fit,
    "zoom objetos",
);

pub(super) const ZOOM_IN: CommandSpec = (
    C::ZoomIn,
    "Zoom in",
    "Aproxima a câmera",
    T::View,
    "Zoom",
    R::ActivePage,
    I::ZoomIn,
    "ampliar",
);

pub(super) const ZOOM_OUT: CommandSpec = (
    C::ZoomOut,
    "Zoom out",
    "Afasta a câmera sem reduzir a região interativa",
    T::View,
    "Zoom",
    R::ActivePage,
    I::ZoomOut,
    "reduzir",
);

pub(super) const ZOOM100: CommandSpec = (
    C::Zoom100,
    "Zoom 100%",
    "96 pontos lógicos por polegada; não é calibração física do monitor",
    T::View,
    "Zoom",
    R::ActivePage,
    I::Search,
    "escala",
);

pub(super) const PREVIOUS_VIEW: CommandSpec = (
    C::PreviousView,
    "Visualização anterior",
    "Restaura a última câmera guardada",
    T::View,
    "Zoom",
    R::ActivePage,
    I::Undo,
    "voltar zoom",
);

pub(super) const TOGGLE_GRID: CommandSpec = (
    C::ToggleGrid,
    "Mostrar grade",
    "Alterna a grade em coordenadas do documento",
    T::View,
    "Grade",
    R::Always,
    I::Grid,
    "grid",
);

pub(super) const TOGGLE_SNAP: CommandSpec = (
    C::ToggleSnap,
    "Ajustar à grade",
    "Ajusta coordenadas do cursor à grade, independentemente de sua visibilidade",
    T::View,
    "Grade",
    R::Always,
    I::Snap,
    "snap",
);

pub(super) const PREVIEW: CommandSpec = (
    C::Preview,
    "Pré-visualização",
    "Mostra ou oculta miniaturas derivadas das páginas",
    T::View,
    "Painéis",
    R::Always,
    I::Page,
    "miniatura",
);

pub(super) const INSERTION: CommandSpec = (
    C::Insertion,
    "Centro de inserção",
    "Mostra ou oculta categorias e registros disponíveis",
    T::View,
    "Painéis",
    R::Always,
    I::Insert,
    "biblioteca",
);

pub(super) const RESTORE_LAYOUT: CommandSpec = (
    C::RestoreLayout,
    "Restaurar layout",
    "Restaura proporções, visibilidade e divisores padrão",
    T::View,
    "Painéis",
    R::Always,
    I::Panels,
    "restaurar redefinir",
);

pub(super) const GROUP_GERAL: PlannedGroupSpec = (
    T::View,
    "Geral",
    I::Search,
    &[
        "Acompanhamento de rede",
        "Acompanhamento do potencial",
        "Acompanhamento do sinal",
        "Pontos de inserção",
        "Elementos ocultos",
        "Caminhos",
        "Marcadores de revisão",
        "Alvos externos",
        "Textos vazios",
        "Conexões de cabos",
        "Deslocar conteúdo",
        "Designações de pontos de conexão",
    ],
);
