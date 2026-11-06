// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Command specifications for the home ribbon tab; start with the named constants.
use super::{
    CommandId as C, CommandSpec, FreeCommandSpec, Icon as I, PlannedGroupSpec, Requirement as R,
    RibbonTab as T,
};

pub(super) const EXPORT_LIST: CommandSpec = (
    C::ExportList,
    crate::strings::EXPORT_LIST,
    "Exporta relatório tabular em CSV ou XLSX",
    T::Home,
    "Sessão",
    R::Project,
    I::Page,
    "lista csv xlsx exportar",
);

pub(super) const SHOW_PROPERTIES: CommandSpec = (
    C::ShowProperties,
    "Propriedades da seleção",
    "Abre o inspetor da seleção atual",
    T::Home,
    "Edição",
    R::Project,
    I::Properties,
    "coordenadas ângulo",
);

pub(super) const LOCATE_REFERENCE: CommandSpec = (
    C::LocateReference,
    "Localizar referência",
    "Abre outra ocorrência do mesmo alvo elétrico",
    T::Home,
    "Edição",
    R::Related,
    I::Connection,
    "referência navegar",
);

pub(super) const NEW_PROJECT: CommandSpec = (
    C::NewProject,
    "Nova sessão",
    "Cria um projeto em memória",
    T::Home,
    "Sessão",
    R::Always,
    I::Page,
    "novo projeto",
);

pub(super) const DEMO: CommandSpec = (
    C::Demo,
    "Abrir demonstração",
    "Abre o exemplo documental próprio do CAD-CRAFT",
    T::Home,
    "Sessão",
    R::Always,
    I::Folder,
    "demo exemplo",
);

pub(super) const IMPORT_QET: CommandSpec = (
    C::ImportQet,
    crate::strings::IMPORT_QET_PROJECT,
    "Cria um projeto novo a partir de um .qet do QElectroTech, com relatório do que foi convertido",
    T::Home,
    "Sessão",
    R::Always,
    I::Folder,
    "qet qelectrotech importar projeto",
);

pub(super) const OPEN_SNAPSHOT: CommandSpec = (
    C::OpenSnapshot,
    "Importar snapshot técnico",
    "Importa JSON estrutural do Core; não é o formato final .cadcraft",
    T::Home,
    "Sessão",
    R::Always,
    I::Folder,
    "abrir carregar json",
);

pub(super) const EXPORT_SNAPSHOT: CommandSpec = (
    C::ExportSnapshot,
    "Exportar snapshot técnico",
    "Exporta JSON estrutural; não é backup versionado",
    T::Home,
    "Sessão",
    R::Project,
    I::Save,
    "json salvar",
);

pub(super) const SAVE_PROJECT: CommandSpec = (
    C::SaveProject,
    "Salvar projeto",
    "Grava o projeto no arquivo .cadcraft associado; sem arquivo, pergunta o destino (Ctrl+S)",
    T::Home,
    "Sessão",
    R::Project,
    I::Save,
    "salvar gravar cadcraft arquivo",
);

pub(super) const SAVE_PROJECT_AS: CommandSpec = (
    C::SaveProjectAs,
    crate::strings::SAVE_AS,
    "Grava o projeto em um novo arquivo .cadcraft (Ctrl+Shift+S)",
    T::Home,
    "Sessão",
    R::Project,
    I::Save,
    "salvar como copiar arquivo cadcraft",
);

pub(super) const OPEN_PROJECT: CommandSpec = (
    C::OpenProject,
    crate::strings::OPEN_PROJECT,
    "Abre um projeto .cadcraft ou um snapshot técnico JSON (formato v0) (Ctrl+O)",
    T::Home,
    "Sessão",
    R::Always,
    I::Folder,
    "abrir carregar arquivo cadcraft projeto",
);

pub(super) const EXPORT_PDF: CommandSpec = (
    C::ExportPdf,
    crate::strings::EXPORT_PDF,
    "PDF vetorial multipágina com links nas referências cruzadas e marcadores",
    T::Home,
    "Sessão",
    R::Project,
    I::Page,
    "pdf exportar documentação links marcadores",
);

pub(super) const EXPORT_DXF: CommandSpec = (
    C::ExportDxf,
    crate::strings::EXPORT_DXF,
    "DXF R12 em mm, um arquivo por página, camadas por categoria",
    T::Home,
    "Sessão",
    R::Project,
    I::Page,
    "dxf autocad exportar camadas",
);

pub(super) const EXPORT_DWG: CommandSpec = (
    C::ExportDwg,
    crate::strings::EXPORT_DWG,
    "DWG pelo conversor externo configurado (ODA File Converter)",
    T::Home,
    "Sessão",
    R::Project,
    I::Page,
    "dwg autocad oda conversor",
);

pub(super) const PRINT: CommandSpec = (
    C::Print,
    crate::strings::PRINT,
    "Imprime pelo leitor de PDF do sistema (PDF temporário)",
    T::Home,
    "Sessão",
    R::Project,
    I::Page,
    "imprimir impressora papel pdf",
);

pub(super) const NUMBER_PAGES: CommandSpec = (
    C::NumberPages,
    crate::strings::NUMBER_PAGES,
    "Prévia e aplicação da numeração das páginas do documento",
    T::Home,
    "Página",
    R::Document,
    I::Page,
    "numeração folhas documento",
);

pub(super) const MACRO_PAGE: CommandSpec = (
    C::MacroPage,
    "Macro de página",
    "Guarda a página ativa como macro de página ou insere uma como página nova",
    T::Home,
    "Página",
    R::ActivePage,
    I::Page,
    "macro página",
);

pub(super) const NEW_DOCUMENT: CommandSpec = (
    C::NewDocument,
    crate::strings::NEW_DOCUMENT,
    "Cria um documento usando o Core",
    T::Home,
    "Página",
    R::Project,
    I::Folder,
    "documento",
);

pub(super) const NEW_PAGE: CommandSpec = (
    C::NewPage,
    crate::strings::NEW_PAGE,
    "Cria uma folha no documento selecionado",
    T::Home,
    "Página",
    R::Document,
    I::Page,
    "folha criar",
);

pub(super) const NAVIGATOR: CommandSpec = (
    C::Navigator,
    "Navegador",
    "Mostra ou oculta o navegador esquerdo",
    T::Home,
    "Página",
    R::Always,
    I::Panels,
    "páginas painel",
);

pub(super) const IDENTIFY_PAGE: CommandSpec = (
    C::IdentifyPage,
    "Identificação",
    "Edita a identificação da página; preserva PageId",
    T::Home,
    "Página",
    R::SelectedPage,
    I::Text,
    "numerar número",
);

pub(super) const OPEN_PAGE: CommandSpec = (
    C::OpenPage,
    "Abrir página",
    "Abre a página selecionada em uma aba",
    T::Home,
    "Visualização gráfica",
    R::SelectedPage,
    I::Page,
    "abrir documento",
);

pub(super) const SYNC_SELECTION: CommandSpec = (
    C::SyncSelection,
    "Sincronizar seleção",
    "Seleciona no navegador a página aberta",
    T::Home,
    crate::strings::BROWSE,
    R::ActivePage,
    I::Check,
    "seleção",
);

pub(super) const PROPERTIES: CommandSpec = (
    C::Properties,
    "Visão geral de propriedades",
    "Mostra ou oculta o inspetor contextual",
    T::Home,
    "Editar",
    R::Always,
    I::Properties,
    "inspetor propriedades",
);

pub(super) const FIND_PAGES: CommandSpec = (
    C::FindPages,
    "Procurar páginas",
    "Abre e foca o filtro de páginas/documentos",
    T::Home,
    crate::strings::BROWSE,
    R::Always,
    I::Search,
    "buscar filtro pesquisa",
);

pub(super) const CLOSE_PROJECT: CommandSpec = (
    C::CloseProject,
    "Fechar sessão",
    "Fecha o projeto em memória com confirmação de descarte",
    T::Home,
    "Sessão",
    R::Project,
    I::Close,
    "fechar",
);

pub(super) const FREE_COPY: FreeCommandSpec = (
    C::Copy,
    "Copiar",
    T::Home,
    "Área de transferência",
    R::GraphicSelection,
    I::Page,
    Some("Ctrl+C"),
);

pub(super) const FREE_CUT: FreeCommandSpec = (
    C::Cut,
    "Recortar",
    T::Home,
    "Área de transferência",
    R::GraphicSelection,
    I::Page,
    Some("Ctrl+X"),
);

pub(super) const FREE_PASTE: FreeCommandSpec = (
    C::Paste,
    "Colar",
    T::Home,
    "Área de transferência",
    R::ActivePage,
    I::Page,
    Some("Ctrl+V"),
);

pub(super) const FREE_PASTE_SAME_DEVICE: FreeCommandSpec = (
    C::PasteSameDevice,
    "Colar no mesmo dispositivo",
    T::Home,
    "Área de transferência",
    R::ActivePage,
    I::Page,
    None,
);

pub(super) const FREE_DUPLICATE: FreeCommandSpec = (
    C::Duplicate,
    "Duplicar",
    T::Home,
    "Área de transferência",
    R::GraphicSelection,
    I::Page,
    Some("Ctrl+D"),
);

pub(super) const FREE_DELETE_FREE: FreeCommandSpec = (
    C::DeleteFree,
    crate::strings::DELETE,
    T::Home,
    "Área de transferência",
    R::GraphicSelection,
    I::Delete,
    Some("Delete"),
);

pub(super) const GROUP_AREA_DE_TRANSFERENCIA: PlannedGroupSpec = (
    T::Home,
    "Área de transferência",
    I::Page,
    &[
        "Colar",
        "Recortar",
        "Copiar",
        crate::strings::DELETE,
        "Copiar formato",
        "Atribuir formato",
    ],
);

pub(super) const GROUP_ESPACO_DO_LAYOUT_3D: PlannedGroupSpec = (
    T::Home,
    "Espaço do layout 3D",
    I::Device,
    &["Navegador 3D", "Novo 3D", "Medir"],
);

pub(super) const GROUP_TEXTO: PlannedGroupSpec = (
    T::Home,
    "Texto",
    I::Text,
    &["Inserir texto", "Texto de função do caminho", "Mover texto"],
);

pub(super) const GROUP_EDITAR: PlannedGroupSpec = (T::Home, "Editar", I::Properties, &["Tabular"]);
