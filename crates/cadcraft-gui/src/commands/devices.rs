// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Command specifications for the devices ribbon tab; start with the named constants.
use super::{
    CommandId as C, CommandSpec, Icon as I, PlannedGroupSpec, Requirement as R, RibbonTab as T,
};

pub(super) const REFERENCE_PROPERTIES: CommandSpec = (
    C::ReferenceProperties,
    "Identificação e referências",
    "Edita identificação, funções, associações e destinos reais",
    T::Devices,
    "Identificação",
    R::Project,
    I::Properties,
    "dispositivo funções referências potencial",
);

pub(super) const LOCATE_MAIN_FUNCTION: CommandSpec = (
    C::LocateMainFunction,
    "Localizar função principal",
    "Abre o destino único ou a lista de ocorrências para escolher",
    T::Devices,
    "Referências",
    R::Project,
    I::Search,
    "bobina principal",
);

pub(super) const LOCATE_AUXILIARY_FUNCTIONS: CommandSpec = (
    C::LocateAuxiliaryFunctions,
    "Localizar auxiliares",
    "Consulta funções auxiliares reais e suas ocorrências",
    T::Devices,
    "Referências",
    R::Project,
    I::Search,
    "contatos auxiliares",
);

pub(super) const NUMBER_DEVICES: CommandSpec = (
    C::NumberDevices,
    "Numerar dispositivos",
    "Prévia e aplicação da regra de numeração de dispositivos",
    T::Devices,
    "Dispositivos",
    R::Project,
    I::Device,
    "numeração renumerar dispositivos",
);

pub(super) const DEVICE_NAVIGATOR: CommandSpec = (
    C::DeviceNavigator,
    "Dispositivos",
    "Exibe dispositivos reais da sessão",
    T::Devices,
    "Dispositivos",
    R::Always,
    I::Device,
    "navegador",
);

pub(super) const TERMINAL_NAVIGATOR: CommandSpec = (
    C::TerminalNavigator,
    "Bornes",
    "Exibe terminais e réguas existentes",
    T::Devices,
    "Bornes",
    R::Always,
    I::Connection,
    "navegador",
);

pub(super) const TERMINAL_STRIP_EDITOR: CommandSpec = (
    C::TerminalStripEditor,
    "Editor de réguas",
    "Cria e edita réguas, bornes e pontes explícitas",
    T::Devices,
    "Bornes",
    R::Project,
    I::Connection,
    "régua terminal ponte",
);

pub(super) const CONNECTOR_NAVIGATOR: CommandSpec = (
    C::ConnectorNavigator,
    "Navegador de conectores",
    "Exibe conectores, contatos e bornes especializados no navegador Bornes",
    T::Devices,
    "Conectores",
    R::Always,
    I::Connection,
    "navegador conector contato",
);

pub(super) const CONNECTOR_EDITOR: CommandSpec = (
    C::ConnectorEditor,
    "Conectores",
    "Cria conectores, numera contatos, associa pontos e registra acoplamentos",
    T::Devices,
    "Conectores",
    R::Project,
    I::Connection,
    "conector contato plugue acoplamento numerar",
);

pub(super) const GROUP_DISPOSITIVOS: PlannedGroupSpec = (
    T::Devices,
    "Dispositivos",
    I::Device,
    &["Lista de materiais", "Numerar dispositivos", "Extras"],
);

pub(super) const GROUP_CLP: PlannedGroupSpec = (
    T::Devices,
    "CLP",
    I::Device,
    &[
        "Navegador CLP",
        "Endereçar",
        "Lista de atribuição",
        "Endereços iniciais",
        "Pontos de conexão",
        "Mover",
    ],
);

pub(super) const GROUP_BORNES: PlannedGroupSpec = (
    T::Devices,
    "Bornes",
    I::Connection,
    &["Numerar bornes", "Régua de bornes", "Extras de bornes"],
);

pub(super) const GROUP_LAYOUT_DO_PAINEL_2D: PlannedGroupSpec = (
    T::Devices,
    "Layout do painel 2D",
    I::Device,
    &["Navegador de layout"],
);

pub(super) const GROUP_OPCOES_DO_PROJETO: PlannedGroupSpec = (
    T::Devices,
    "Opções do projeto",
    I::Settings,
    &["Navegador de opções", "Objetos placeholder", "Gerar seção"],
);
