// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Command specifications for the edit ribbon tab; start with the named constants.
use super::{
    CommandId as C, CommandSpec, Icon as I, PlannedGroupSpec, Requirement as R, RibbonTab as T,
};

pub(super) const SELECT: CommandSpec = (
    C::Select,
    "Selecionar",
    "Clique, Ctrl+clique ou janela; Esc cancela o gesto",
    T::Edit,
    "Gráfico",
    R::ActivePage,
    I::Search,
    "seleção",
);

pub(super) const MOVE: CommandSpec = (
    C::Move,
    "Mover",
    "Arraste uma representação selecionada; snap usa a âncora do objeto arrastado",
    T::Edit,
    "Gráfico",
    R::GraphicSelection,
    I::Tools,
    "mover arrastar",
);

pub(super) const ROTATE90: CommandSpec = (
    C::Rotate90,
    "Girar 90°",
    "Rotação CCW em torno da âncora de cada objeto",
    T::Edit,
    "Gráfico",
    R::GraphicSelection,
    I::Tools,
    "rotacionar",
);

pub(super) const ROTATE180: CommandSpec = (
    C::Rotate180,
    "Girar 180°",
    "Rotação em torno da âncora de cada objeto",
    T::Edit,
    "Gráfico",
    R::GraphicSelection,
    I::Tools,
    "rotacionar",
);

pub(super) const ROTATE270: CommandSpec = (
    C::Rotate270,
    "Girar 270°",
    "Rotação CCW em torno da âncora de cada objeto",
    T::Edit,
    "Gráfico",
    R::GraphicSelection,
    I::Tools,
    "rotacionar",
);

pub(super) const MIRROR_HORIZONTAL: CommandSpec = (
    C::MirrorHorizontal,
    "Espelhar horizontal",
    "Inverte X da página em torno de cada âncora",
    T::Edit,
    "Gráfico",
    R::GraphicSelection,
    I::Tools,
    "espelhar",
);

pub(super) const MIRROR_VERTICAL: CommandSpec = (
    C::MirrorVertical,
    "Espelhar vertical",
    "Inverte Y da página em torno de cada âncora",
    T::Edit,
    "Gráfico",
    R::GraphicSelection,
    I::Tools,
    "espelhar",
);

pub(super) const UNDO: CommandSpec = (
    C::Undo,
    "Desfazer",
    "Desfaz uma operação gráfica confirmada (Ctrl+Z)",
    T::Edit,
    "Histórico",
    R::Undo,
    I::Undo,
    "undo",
);

pub(super) const REDO: CommandSpec = (
    C::Redo,
    "Refazer",
    "Refaz uma transformação desfeita (Ctrl+Y)",
    T::Edit,
    "Histórico",
    R::Redo,
    I::Redo,
    "redo",
);

pub(super) const REPEAT: CommandSpec = (
    C::Repeat,
    "Repetir operação",
    "Repetição de operações é futura",
    T::Edit,
    "Histórico",
    R::Future,
    I::Redo,
    "repetir",
);

pub(super) const RENAME_PAGE: CommandSpec = (
    C::RenamePage,
    crate::strings::RENAME_PAGE,
    "Edita o nome da página por operação validada",
    T::Edit,
    "Página",
    R::SelectedPage,
    I::Text,
    "nome renomear",
);

pub(super) const DELETE_PAGE: CommandSpec = (
    C::DeletePage,
    crate::strings::DELETE_PAGE,
    "Remove somente página sem representações ou referências externas",
    T::Edit,
    "Página",
    R::SelectedPage,
    I::Delete,
    "remover excluir",
);

pub(super) const PAGE_UP: CommandSpec = (
    C::PageUp,
    "Mover página acima",
    "Reordena no documento; não altera identidades",
    T::Edit,
    "Página",
    R::SelectedPage,
    I::ArrowLeft,
    "reordenar",
);

pub(super) const PAGE_DOWN: CommandSpec = (
    C::PageDown,
    "Mover página abaixo",
    "Reordena no documento; não altera identidades",
    T::Edit,
    "Página",
    R::SelectedPage,
    I::ArrowRight,
    "reordenar",
);

pub(super) const GROUP_AREA_DE_TRANSFERENCIA: PlannedGroupSpec = (
    T::Edit,
    "Área de transferência",
    I::Page,
    &["Colar", "Recortar", "Copiar"],
);

pub(super) const GROUP_GRAFICO: PlannedGroupSpec = (
    T::Edit,
    "Gráfico",
    I::Tools,
    &[
        "Esticar",
        "Escalar",
        "Aparar",
        "Arredondar",
        "Chanfrar",
        "Duplicar",
    ],
);

pub(super) const GROUP_AGRUPAR: PlannedGroupSpec = (
    T::Edit,
    "Agrupar",
    I::Panels,
    &["Agrupar", "Desfazer agrupamento"],
);

pub(super) const GROUP_BLOCO: PlannedGroupSpec = (
    T::Edit,
    "Bloco",
    I::Device,
    &["Formar", "Desfazer", "Converter"],
);

pub(super) const GROUP_ORDENAR: PlannedGroupSpec = (
    T::Edit,
    "Ordenar",
    I::Grid,
    &[
        "Alinhar à grade",
        "Alinhar horizontalmente",
        "Distribuir horizontalmente",
        "Primeiro plano",
        "Plano de fundo",
    ],
);

pub(super) const GROUP_PROTECAO: PlannedGroupSpec = (
    T::Edit,
    "Proteção",
    I::Pin,
    &["Configurar", "Bloquear", "Desbloquear"],
);

pub(super) const GROUP_OPCOES: PlannedGroupSpec = (
    T::Edit,
    "Opções",
    I::Settings,
    &[
        "Edição direta",
        "Intervalo",
        "Janela de entrada de dados",
        "Conexão inteligente",
    ],
);

pub(super) const COORDINATE_INPUT: CommandSpec = (
    C::CoordinateInput,
    "Inserção de coordenadas",
    "Mostra campos X/Y (absolutos ou relativos ao último ponto) ao inserir símbolo, conexão ou linha; Enter aplica",
    T::Edit,
    "Opções",
    R::Always,
    I::Grid,
    "coordenadas xy digitar posição relativa absoluta",
);
