// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Command specifications for the master data ribbon tab; start with the named constants.
use super::{
    CommandId as C, CommandSpec, Icon as I, PlannedGroupSpec, Requirement as R, RibbonTab as T,
};

pub(super) const IMPORT_EDZ: CommandSpec = (
    C::ImportEdz,
    "Importar .edz",
    "Importa peças e macros 2D de pacotes de fabricantes, com prévia e relatório",
    T::MasterData,
    "Importação",
    R::Always,
    I::Folder,
    "eplan edz fabricante importar peças macros",
);

pub(super) const FRAME_EDITOR: CommandSpec = (
    C::FrameEditor,
    "Editar moldura",
    "Abre o editor com a moldura da página ativa ou a padrão do projeto",
    T::MasterData,
    "Molduras",
    R::Project,
    I::Page,
    "moldura carimbo editar formulário",
);

pub(super) const FRAME_NEW: CommandSpec = (
    C::FrameNew,
    "Nova moldura",
    "Cria uma moldura em branco no tamanho da página ativa e abre o editor",
    T::MasterData,
    "Molduras",
    R::Project,
    I::Insert,
    "moldura nova carimbo",
);

pub(super) const FRAME_DUPLICATE: CommandSpec = (
    C::FrameDuplicate,
    "Duplicar moldura",
    "Cria uma cópia independente da moldura aberta no editor",
    T::MasterData,
    "Molduras",
    R::Project,
    I::Page,
    "moldura copiar duplicar",
);

pub(super) const FRAME_DELETE: CommandSpec = (
    C::FrameDelete,
    "Excluir moldura",
    "Exclui a moldura aberta no editor se nenhuma página ou o projeto a usar",
    T::MasterData,
    "Molduras",
    R::Project,
    I::Delete,
    "moldura excluir remover",
);

pub(super) const FRAME_EXPORT: CommandSpec = (
    C::FrameExport,
    "Exportar moldura",
    "Grava a moldura aberta no editor como arquivo .cadframe, com as imagens",
    T::MasterData,
    "Molduras",
    R::Project,
    I::Save,
    "moldura exportar cadframe biblioteca",
);

pub(super) const FRAME_IMPORT: CommandSpec = (
    C::FrameImport,
    "Importar moldura",
    "Importa um arquivo .cadframe para o projeto e abre o editor",
    T::MasterData,
    "Molduras",
    R::Project,
    I::Folder,
    "moldura importar cadframe biblioteca",
);

pub(super) const FRAME_IMPORT_EXTERNAL: CommandSpec = (
    C::FrameImportExternal,
    "Importar de outro sistema…",
    "Importa moldura EPLAN (.fn1), QElectroTech (.titleblock) ou DXF com prévia e relatório",
    T::MasterData,
    "Molduras",
    R::Project,
    I::Folder,
    "moldura importar eplan fn1 qelectrotech titleblock dxf",
);

pub(super) const FRAME_EXPORT_DXF: CommandSpec = (
    C::FrameExportDxf,
    crate::strings::EXPORT_DXF,
    "Exporta a moldura aberta no editor para DXF ASCII em milímetros",
    T::MasterData,
    "Molduras",
    R::Project,
    I::Save,
    "moldura exportar dxf",
);

pub(super) const SYMBOL_NEW: CommandSpec = (
    C::SymbolNew,
    crate::strings::NEW_SYMBOL,
    "Abre o editor de símbolos com um símbolo em branco",
    T::MasterData,
    "Símbolos",
    R::Project,
    I::Connection,
    "símbolo novo editor criar",
);

pub(super) const SYMBOL_EDITOR: CommandSpec = (
    C::SymbolEditor,
    "Editar símbolo",
    "Edita o símbolo selecionado na página ou no Centro de inserção (nova versão)",
    T::MasterData,
    "Símbolos",
    R::Project,
    I::Connection,
    "símbolo editar editor versão",
);

pub(super) const SYMBOL_EXPORT: CommandSpec = (
    C::SymbolExport,
    crate::strings::EXPORT_SYMBOL,
    "Grava o símbolo aberto no editor ou selecionado como arquivo .cadsym",
    T::MasterData,
    "Símbolos",
    R::Project,
    I::Save,
    "símbolo exportar cadsym biblioteca",
);

pub(super) const SYMBOL_IMPORT: CommandSpec = (
    C::SymbolImport,
    crate::strings::IMPORT_SYMBOL,
    "Copia um arquivo .cadsym para a biblioteca de símbolos",
    T::MasterData,
    "Símbolos",
    R::Always,
    I::Folder,
    "símbolo importar cadsym biblioteca",
);

pub(super) const SYMBOL_IMPORT_ELMT: CommandSpec = (
    C::SymbolImportElmt,
    "Importar .elmt…",
    "Converte elementos do QElectroTech em símbolos da biblioteca (0,5 mm por px)",
    T::MasterData,
    "Símbolos",
    R::Always,
    I::Folder,
    "símbolo importar elmt qelectrotech elemento",
);

pub(super) const SYMBOL_IMPORT_ELMT_FOLDER: CommandSpec = (
    C::SymbolImportElmtFolder,
    "Importar pasta .elmt…",
    "Converte todos os elementos .elmt de uma pasta (e subpastas) para a biblioteca",
    T::MasterData,
    "Símbolos",
    R::Always,
    I::Folder,
    "símbolo importar pasta elmt qelectrotech lote",
);

pub(super) const SYMBOL_RELOAD: CommandSpec = (
    C::SymbolReload,
    "Recarregar biblioteca",
    "Lê novamente os arquivos .cadsym da pasta da biblioteca",
    T::MasterData,
    "Símbolos",
    R::Always,
    I::Folder,
    "símbolo biblioteca recarregar pasta",
);

pub(super) const SYMBOL_UPDATE_PROJECT: CommandSpec = (
    C::SymbolUpdateProject,
    "Atualizar símbolos do projeto",
    "Troca as ocorrências pela versão mais nova da biblioteca quando as portas são compatíveis",
    T::MasterData,
    "Símbolos",
    R::Project,
    I::Connection,
    "símbolo atualizar versão biblioteca projeto",
);

pub(super) const PART_EDITOR: CommandSpec = (
    C::PartEditor,
    crate::strings::PARTS_AND_DEVICES,
    "Abre a biblioteca de peças, atribuições, funções, pontos e lista de peças",
    T::MasterData,
    "Peças",
    R::Project,
    I::Device,
    "peça dispositivo função ponto bom cadpart csv",
);

pub(super) const MACRO_CREATE: CommandSpec = (
    C::MacroCreate,
    "Criar macro da seleção",
    "Guarda a seleção (símbolos, fios, junções, textos) como macro na biblioteca",
    T::MasterData,
    "Macros",
    R::GraphicSelection,
    I::Insert,
    "macro criar seleção",
);

pub(super) const GROUP_PECAS: PlannedGroupSpec = (
    T::MasterData,
    "Peças",
    I::Device,
    &["Gerenciamento", "Sincronizar", "Atualizar", "Completar"],
);

pub(super) const GROUP_MACROS: PlannedGroupSpec =
    (T::MasterData, "Macros", I::Insert, &["Grupo de macros"]);

pub(super) const GROUP_SINCRONIZACAO: PlannedGroupSpec = (
    T::MasterData,
    "Sincronização",
    I::Tools,
    &[
        "Sincronizar projeto",
        "Atualizar projeto",
        "Armazenar documentos",
    ],
);

pub(super) const GROUP_FORMULARIOS: PlannedGroupSpec = (
    T::MasterData,
    "Formulários",
    I::Properties,
    &["Editar formulário"],
);

pub(super) const GROUP_SIMBOLOS: PlannedGroupSpec = (T::MasterData, "Símbolos", I::Connection, &[]);

pub(super) const GROUP_PONTOS_DE_CONEXAO: PlannedGroupSpec = (
    T::MasterData,
    "Pontos de conexão",
    I::Connection,
    &["Gerenciar pontos"],
);

pub(super) const GROUP_GERENCIAMENTO_DE_TAGS: PlannedGroupSpec = (
    T::MasterData,
    "Gerenciamento de tags",
    I::Text,
    &["Gerenciar tags"],
);
