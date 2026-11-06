// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Command specifications for the connections ribbon tab; start with the named constants.
use super::{
    CommandId as C, CommandSpec, Icon as I, PlannedGroupSpec, Requirement as R, RibbonTab as T,
};

pub(super) const DEMO_CONNECTIONS: CommandSpec = (
    C::DemoConnections,
    "Exemplo multifilar",
    "Abre exemplo original com portas vinculadas e duas redes que se cruzam",
    T::Connections,
    "Exemplo",
    R::Always,
    I::Page,
    "demonstração multifilar",
);

pub(super) const DRAW_CONNECTION: CommandSpec = (
    C::DrawConnection,
    "Inserir conexão",
    "Clique na porta de origem, defina a rota e clique na porta de destino; Espaço alterna L",
    T::Connections,
    "Desenho",
    R::ActivePage,
    I::Connection,
    "fio conexão ortogonal",
);

pub(super) const INSERT_JUNCTION: CommandSpec = (
    C::InsertJunction,
    "Inserir junção",
    "Clique na interseção de rotas para unir explicitamente as redes",
    T::Connections,
    "Desenho",
    R::ActivePage,
    I::Insert,
    "junção T cruzamento",
);

pub(super) const EDIT_CONNECTION: CommandSpec = (
    C::EditConnection,
    "Editar rota",
    "Arraste um segmento ou vértice; endpoints elétricos permanecem vinculados",
    T::Connections,
    "Geometria",
    R::ConnectionSelection,
    I::Tools,
    "editar rota segmento",
);

pub(super) const ADD_CONNECTION_VERTEX: CommandSpec = (
    C::AddConnectionVertex,
    "Adicionar vértice",
    "Clique no interior de um segmento da conexão",
    T::Connections,
    "Geometria",
    R::ConnectionSelection,
    I::Insert,
    "vértice waypoint",
);

pub(super) const REMOVE_CONNECTION_VERTEX: CommandSpec = (
    C::RemoveConnectionVertex,
    "Remover vértice",
    "Clique num vértice intermediário para reencaminhar a rota",
    T::Connections,
    "Geometria",
    R::ConnectionSelection,
    I::Delete,
    "vértice ponto",
);

pub(super) const DELETE_CONNECTION: CommandSpec = (
    C::DeleteConnection,
    "Excluir conexão",
    "Remove conexão lógica e sua rota em uma transação; recusa dependências",
    T::Connections,
    "Edição",
    R::ConnectionSelection,
    I::Delete,
    "excluir fio",
);

pub(super) const DELETE_JUNCTION: CommandSpec = (
    C::DeleteJunction,
    "Remover junção",
    "Remove apenas a união explícita e suas arestas auxiliares",
    T::Connections,
    "Edição",
    R::JunctionSelection,
    I::Delete,
    "excluir junção",
);

pub(super) const HIGHLIGHT_NETWORK: CommandSpec = (
    C::HighlightNetwork,
    "Destacar rede",
    "Consulta a rede derivada no Core e destaca suas rotas",
    T::Connections,
    "Consulta",
    R::ConnectionSelection,
    I::Search,
    "rede destacar",
);

pub(super) const TOGGLE_JUNCTION_STYLE: CommandSpec = (
    C::ToggleJunctionStyle,
    "Marca de junção",
    "Alterna ponto cheio/anel; cruzamentos sem junção continuam sem marca",
    T::Connections,
    "Consulta",
    R::Always,
    I::Settings,
    "convenção junção visual",
);

pub(super) const NUMBER_CONNECTIONS: CommandSpec = (
    C::NumberConnections,
    "Numerar conexões",
    "Prévia e aplicação da numeração de fios por rede",
    T::Connections,
    "Conexões",
    R::Project,
    I::Connection,
    "numeração fios potenciais",
);

pub(super) const WIRING_EDITOR: CommandSpec = (
    C::WiringEditor,
    "Atribuir fiação",
    "Cadastra condutores e vincula conexões físicas",
    T::Connections,
    "Fiação física",
    R::Project,
    I::Connection,
    "condutor conexão física",
);

pub(super) const CABLE_NAVIGATOR: CommandSpec = (
    C::CableNavigator,
    "Cabos",
    "Exibe cabos reais da sessão",
    T::Connections,
    "Cabos",
    R::Always,
    I::Connection,
    "navegador",
);

pub(super) const CONNECTION_NAVIGATOR: CommandSpec = (
    C::ConnectionNavigator,
    "Conexões",
    "Exibe conexões lógicas existentes; não são fios desenhados",
    T::Connections,
    "Conexões",
    R::Always,
    I::Connection,
    "navegador",
);

pub(super) const GROUP_CABOS: PlannedGroupSpec = (
    T::Connections,
    "Cabos",
    I::Connection,
    &["Numerar cabos", "Editar cabos", "Atribuir tudo novamente"],
);

pub(super) const GROUP_CONEXOES: PlannedGroupSpec = (
    T::Connections,
    "Conexões",
    I::Connection,
    &[
        "Potenciais",
        "Atualizar",
        "Fonte e alvo",
        "Designar",
        "Posicionar pontos de definição",
        "Alinhar e formatar",
    ],
);

pub(super) const GROUP_PONTOS_DE_INTERRUPCAO: PlannedGroupSpec = (
    T::Connections,
    "Pontos de interrupção",
    I::Connection,
    &["Navegador de interrupções"],
);

pub(super) const GROUP_TOPOLOGIA: PlannedGroupSpec = (
    T::Connections,
    "Topologia",
    I::Connection,
    &[
        "Navegador de topologia",
        "Rotear",
        "Capacidade de preenchimento",
        "Gerar",
    ],
);

pub(super) const GROUP_CHICOTE: PlannedGroupSpec = (
    T::Connections,
    "Chicote",
    I::Connection,
    &["Navegador de chicotes"],
);
