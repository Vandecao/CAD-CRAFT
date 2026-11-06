// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Command specifications for the insert ribbon tab; start with the named constants.
use super::{
    CommandId as C, CommandSpec, FreeCommandSpec, Icon as I, PlannedGroupSpec, Requirement as R,
    RibbonTab as T,
};

pub(super) const INSERT_INTERRUPTION: CommandSpec = (
    C::InsertInterruption,
    "Ponto de interrupção",
    "Insere marcador de continuidade com associação explícita",
    T::Insert,
    "Referências",
    R::ActivePage,
    I::Insert,
    "interrupção estrela cadeia",
);

pub(super) const INSERT_POTENTIAL: CommandSpec = (
    C::InsertPotential,
    "Identificação de potencial",
    "Insere identificação e permite associar um ponto elétrico existente",
    T::Insert,
    "Referências",
    R::ActivePage,
    I::Connection,
    "potencial rede",
);

pub(super) const INSERT_CONNECTION_ANGLE: CommandSpec = (
    C::InsertConnectionAngle,
    "Inserir ângulo",
    "Desenhe uma conexão em L; Espaço alterna a direção inicial",
    T::Insert,
    "Símbolos de conexão",
    R::ActivePage,
    I::Connection,
    "ângulo L conexão",
);

pub(super) const SYMBOL_LIBRARY: CommandSpec = (
    C::SymbolLibrary,
    "Biblioteca de símbolos",
    "Escolha um símbolo no centro de inserção; clique na página e Esc para encerrar",
    T::Insert,
    "Símbolos",
    R::Always,
    I::Connection,
    "símbolos biblioteca inserir galeria",
);

pub(super) const MACRO_NAVIGATOR: CommandSpec = (
    C::MacroNavigator,
    "Navegador de macros",
    "Abre a biblioteca de macros: categorias, busca, miniatura, importar e exportar",
    T::Insert,
    "Macros",
    R::Project,
    I::Insert,
    "macro biblioteca navegador cadmacro",
);

pub(super) const MACRO_INSERT: CommandSpec = (
    C::MacroInsert,
    "Inserir macro",
    "Insere a macro escolhida no navegador; clique na folha, Girar/Espelhar ajustam",
    T::Insert,
    "Macros",
    R::ActivePage,
    I::Insert,
    "macro inserir posicionar",
);

pub(super) const FREE_INSERT_FREE_TEXT: FreeCommandSpec = (
    C::InsertFreeText,
    "Texto",
    T::Insert,
    "Gráfico",
    R::ActivePage,
    I::Text,
    None,
);

pub(super) const FREE_INSERT_LINE: FreeCommandSpec = (
    C::InsertLine,
    "Linha",
    T::Insert,
    "Gráfico",
    R::ActivePage,
    I::Tools,
    None,
);

pub(super) const FREE_INSERT_RECTANGLE: FreeCommandSpec = (
    C::InsertRectangle,
    "Retângulo",
    T::Insert,
    "Gráfico",
    R::ActivePage,
    I::Tools,
    None,
);

pub(super) const FREE_INSERT_CIRCLE: FreeCommandSpec = (
    C::InsertCircle,
    "Círculo",
    T::Insert,
    "Gráfico",
    R::ActivePage,
    I::Tools,
    None,
);

pub(super) const FREE_INSERT_ARC: FreeCommandSpec = (
    C::InsertArc,
    "Arco",
    T::Insert,
    "Gráfico",
    R::ActivePage,
    I::Tools,
    None,
);

pub(super) const FREE_INSERT_ELLIPSE: FreeCommandSpec = (
    C::InsertEllipse,
    "Elipse",
    T::Insert,
    "Gráfico",
    R::ActivePage,
    I::Tools,
    None,
);

pub(super) const GROUP_SIMBOLOS: PlannedGroupSpec = (
    T::Insert,
    "Símbolos",
    I::Connection,
    &["Variantes gráficas"],
);

pub(super) const GROUP_DISPOSITIVOS: PlannedGroupSpec = (
    T::Insert,
    "Dispositivos",
    I::Device,
    &[
        "Caixa de dispositivos",
        "Ponto de conexão do dispositivo",
        "Definição da peça",
        "Caixa de CLP",
        "Pontos de conexão de CLP",
        "Caixa de estrutura",
        "Funções conectadas",
    ],
);

pub(super) const GROUP_CABOS_CONEXOES: PlannedGroupSpec = (
    T::Insert,
    "Cabos / Conexões",
    I::Connection,
    &["Cabo", "Blindagem", "Conexão"],
);

pub(super) const GROUP_BORNES: PlannedGroupSpec = (
    T::Insert,
    "Bornes",
    I::Connection,
    &["Borne", "Borne distribuído", "Definição da régua de bornes"],
);

pub(super) const GROUP_CONECTORES: PlannedGroupSpec = (
    T::Insert,
    "Conectores",
    I::Connection,
    &["Contato do conector", "Definição do conector"],
);

pub(super) const GROUP_GRAFICO: PlannedGroupSpec = (
    T::Insert,
    "Gráfico",
    I::Text,
    &["Texto", "Galeria de gráficos"],
);

pub(super) const GROUP_DIMENSIONAMENTO: PlannedGroupSpec = (
    T::Insert,
    "Dimensionamento",
    I::Tools,
    &["Dimensionamento linear"],
);

pub(super) const GROUP_EXTERNO: PlannedGroupSpec =
    (T::Insert, "Externo", I::Folder, &["Arquivo externo"]);

pub(super) const GROUP_TOPOLOGIA: PlannedGroupSpec = (
    T::Insert,
    "Topologia",
    I::Connection,
    &["Inserir topologia"],
);

pub(super) const GROUP_VISUALIZACAO: PlannedGroupSpec = (
    T::Insert,
    "Visualização",
    I::Search,
    &["Visualização do modelo"],
);

pub(super) const GROUP_LAYOUT_DO_PAINEL_2D: PlannedGroupSpec = (
    T::Insert,
    "Layout do painel 2D",
    I::Device,
    &["Inserir layout"],
);
