// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Command specifications for the tools ribbon tab; start with the named constants.
use super::{
    CommandId as C, CommandSpec, Icon as I, PlannedGroupSpec, Requirement as R, RibbonTab as T,
};

pub(super) const GENERATE_REPORT: CommandSpec = (
    C::GenerateReport,
    "Gerar",
    "Gera páginas tabulares de relatório no projeto",
    T::Tools,
    "Relatórios",
    R::Project,
    I::Page,
    "relatório gerar índice dispositivos materiais fios bornes cabos",
);

pub(super) const UPDATE_REPORT: CommandSpec = (
    C::UpdateReport,
    "Atualizar",
    "Ajusta as páginas de relatórios desatualizados",
    T::Tools,
    "Relatórios",
    R::Project,
    I::Page,
    "relatório atualizar páginas",
);

pub(super) const STRUCTURE_NAVIGATOR: CommandSpec = (
    C::StructureNavigator,
    "Estruturas",
    "Exibe classificações documentais derivadas",
    T::Tools,
    "Gerenciamento",
    R::Always,
    I::Panels,
    "função localização",
);

pub(super) const MESSAGES: CommandSpec = (
    C::Messages,
    "Mensagens",
    "Mostra informações, avisos e erros da sessão",
    T::Tools,
    "Verificação",
    R::Always,
    I::Properties,
    "avisos erros",
);

pub(super) const CHECK_PROJECT: CommandSpec = (
    C::CheckProject,
    "Verificar projeto",
    "Executa as regras de consistência e abre o painel Verificações (F8 navega)",
    T::Tools,
    "Verificação",
    R::Project,
    I::Check,
    "verificar consistência erros avisos mensagens",
);

pub(super) const CHECK_PAGE: CommandSpec = (
    C::CheckPage,
    "Verificar página",
    "Executa as regras de consistência só na página ativa",
    T::Tools,
    "Verificação",
    R::ActivePage,
    I::Check,
    "verificar página consistência",
);

pub(super) const VALIDATE: CommandSpec = (
    C::Validate,
    "Integridade do Core",
    "Executa Project::validate; não é ERC nem certificação normativa",
    T::Tools,
    "Verificação",
    R::Project,
    I::Check,
    "verificar validação",
);

pub(super) const SETTINGS: CommandSpec = (
    C::Settings,
    "Configurações",
    "Preferências visuais locais independentes do projeto",
    T::Tools,
    "Aplicativo",
    R::Always,
    I::Settings,
    "configuração",
);

pub(super) const EXIT: CommandSpec = (
    C::Exit,
    "Sair",
    "Fecha a janela com confirmação de alterações não exportadas",
    T::Tools,
    "Aplicativo",
    R::Always,
    I::Close,
    "sair",
);

pub(super) const GROUP_VERIFICACAO: PlannedGroupSpec =
    (T::Tools, "Verificação", I::Check, &["Verificar ERC"]);

pub(super) const GROUP_RELATORIOS: PlannedGroupSpec = (
    T::Tools,
    "Relatórios",
    I::Properties,
    &[
        "Avaliar projeto",
        "Visualizações do modelo",
        "Editar automaticamente",
    ],
);

pub(super) const GROUP_GERENCIAMENTO: PlannedGroupSpec = (
    T::Tools,
    "Gerenciamento",
    I::Settings,
    &[
        "Identificadores de estrutura",
        "Distribuições de propriedades",
        "Layers",
        "Propriedades técnicas",
    ],
);

pub(super) const GROUP_TRADUCAO: PlannedGroupSpec = (
    T::Tools,
    "Tradução",
    I::Text,
    &[
        "Traduzir",
        "Dicionário",
        "Exportar termos faltantes",
        "Entrada de teste",
        "Importar textos",
        "Remover tradução",
    ],
);

pub(super) const GROUP_NORMAS: PlannedGroupSpec = (T::Tools, "Normas", I::Check, &["Substituir"]);

pub(super) const GROUP_SINCRONIZACAO: PlannedGroupSpec = (
    T::Tools,
    "Sincronização",
    I::Connection,
    &["Funções", "Conexões"],
);

pub(super) const GROUP_COMENTARIOS: PlannedGroupSpec = (
    T::Tools,
    "Comentários",
    I::Text,
    &["Navegador de comentários", "Importar comentários"],
);
