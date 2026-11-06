// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Command catalog and availability rules; start with CommandId and Command.
use crate::{preferences::RibbonTab, state::Session};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CommandId {
    GenerateReport,
    CheckProject,
    CheckPage,
    UpdateReport,
    ExportList,
    ExportPdf,
    ExportDxf,
    ExportDwg,
    Print,
    NumberDevices,
    NumberConnections,
    NumberPages,
    InsertInterruption,
    InsertPotential,
    ReferenceProperties,
    LocateMainFunction,
    LocateAuxiliaryFunctions,
    NewProject,
    Demo,
    OpenSnapshot,
    ImportQet,
    ImportEdz,
    ExportSnapshot,
    SaveProject,
    SaveProjectAs,
    FrameEditor,
    FrameNew,
    FrameDuplicate,
    FrameDelete,
    FrameExport,
    FrameImport,
    FrameImportExternal,
    FrameExportDxf,
    SymbolNew,
    SymbolEditor,
    SymbolExport,
    SymbolImport,
    SymbolImportElmt,
    SymbolImportElmtFolder,
    SymbolReload,
    SymbolUpdateProject,
    PartEditor,
    MacroNavigator,
    MacroInsert,
    MacroCreate,
    MacroPage,
    OpenProject,
    Undo,
    Redo,
    Select,
    Move,
    Rotate90,
    Rotate180,
    Rotate270,
    MirrorHorizontal,
    MirrorVertical,
    ShowProperties,
    PageProperties,
    LocateReference,
    Repeat,
    Settings,
    CloseProject,
    Exit,
    NewDocument,
    NewPage,
    OpenPage,
    RenamePage,
    IdentifyPage,
    DeletePage,
    PageUp,
    PageDown,
    Navigator,
    Properties,
    Preview,
    Insertion,
    SymbolLibrary,
    DrawConnection,
    DemoConnections,
    InsertConnectionAngle,
    InsertJunction,
    DeleteJunction,
    EditConnection,
    AddConnectionVertex,
    RemoveConnectionVertex,
    DeleteConnection,
    HighlightNetwork,
    ToggleJunctionStyle,
    Messages,
    RestoreLayout,
    FindPages,
    SyncSelection,
    FitPage,
    FitContent,
    ZoomIn,
    ZoomOut,
    Zoom100,
    PreviousView,
    ToggleGrid,
    ToggleSnap,
    DeviceNavigator,
    StructureNavigator,
    ConnectionNavigator,
    CableNavigator,
    TerminalNavigator,
    TerminalStripEditor,
    ConnectorEditor,
    ConnectorNavigator,
    CableEditor,
    WiringEditor,
    TechnicalLists,
    Validate,
    InsertFreeText,
    InsertLine,
    InsertRectangle,
    InsertCircle,
    InsertArc,
    InsertEllipse,
    Copy,
    Cut,
    Paste,
    PasteSameDevice,
    Duplicate,
    DeleteFree,
    Planned(RibbonTab, &'static str, &'static str),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Requirement {
    Always,
    Project,
    Document,
    SelectedPage,
    ActivePage,
    Future,
    GraphicSelection,
    Undo,
    Redo,
    Related,
    ConnectionSelection,
    JunctionSelection,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Page,
    Folder,
    Save,
    Undo,
    Redo,
    Search,
    Grid,
    Snap,
    Fit,
    ZoomIn,
    ZoomOut,
    Panels,
    Properties,
    Device,
    Connection,
    Tools,
    Text,
    Insert,
    Delete,
    Settings,
    Home,
    ArrowLeft,
    ArrowRight,
    Close,
    Pin,
    Collapse,
    More,
    Check,
}
#[derive(Clone, Debug)]
pub struct Command {
    pub id: CommandId,
    pub name: &'static str,
    pub description: String,
    pub tab: RibbonTab,
    pub group: &'static str,
    pub shortcut: Option<&'static str>,
    pub requirement: Requirement,
    pub icon: Icon,
    pub keywords: &'static str,
}
impl Command {
    pub fn available(&self, s: &Session) -> bool {
        self.reason(s).is_none()
    }
    pub fn reason(&self, s: &Session) -> Option<&'static str> {
        // Etapa 19: girar/espelhar ajustam a macro ainda em posicionamento.
        if s.graphics.macro_draft.is_some()
            && matches!(
                self.id,
                CommandId::Rotate90
                    | CommandId::Rotate180
                    | CommandId::Rotate270
                    | CommandId::MirrorHorizontal
                    | CommandId::MirrorVertical
            )
        {
            return None;
        }
        match self.requirement {
            Requirement::ConnectionSelection
                if !s.graphics.selection.primary().is_some_and(|id| {
                    s.project()
                        .is_some_and(|p| p.connection_route(id).is_some())
                }) =>
            {
                Some("Selecione uma conexão gráfica")
            }
            Requirement::JunctionSelection
                if !s.graphics.junction_selection.is_some_and(|id| {
                    s.project()
                        .is_some_and(|p| p.connection_junction(id).is_some())
                }) =>
            {
                Some("Selecione uma junção")
            }
            Requirement::Always => None,
            Requirement::Future => Some("Planejado — indisponível nesta etapa"),
            Requirement::GraphicSelection
                if s.graphics.selection.ids().is_empty()
                    || s.graphics.selection.ids().iter().any(|id| {
                        !s.project()
                            .and_then(|p| p.representation(*id))
                            .is_some_and(|r| Some(r.page_id()) == s.active && r.visual().visible)
                    }) =>
            {
                Some("Selecione representações na página ativa")
            }
            Requirement::Undo if !s.history.can_undo() => {
                Some("Nenhuma operação gráfica para desfazer")
            }
            Requirement::Redo if !s.history.can_redo() => {
                Some("Nenhuma operação gráfica para refazer")
            }
            Requirement::Related
                if !s.graphics.selection.primary().is_some_and(|id| {
                    s.project().is_some_and(|p| {
                        p.marker(id).is_some()
                            || s.selected_reference_device().is_some()
                            || p.related_representations(id).is_ok_and(|r| !r.is_empty())
                    })
                }) =>
            {
                Some("Não há outra ocorrência associada")
            }
            Requirement::Project if s.project().is_none() => Some("Abra ou crie uma sessão"),
            Requirement::Document if s.selected_document().is_none() => {
                Some("Crie ou selecione um documento")
            }
            Requirement::SelectedPage if s.selected_page().is_none() => {
                Some("Selecione uma página no navegador")
            }
            Requirement::ActivePage if s.active.is_none() => Some("Abra uma página"),
            _ => None,
        }
    }
    pub fn matches(&self, q: &str) -> bool {
        let text = format!(
            "{} {} {} {} {}",
            self.name,
            self.description,
            self.tab.label(),
            self.group,
            self.keywords
        )
        .to_lowercase();
        q.to_lowercase()
            .split_whitespace()
            .all(|w| text.contains(w))
    }
}
mod cables;
mod catalog;
mod connections;
mod devices;
mod edit;
mod home;
mod insert;
mod integrations;
mod master_data;
mod preplanning;
mod tools;
mod view;

type CommandSpec = (
    CommandId,
    &'static str,
    &'static str,
    RibbonTab,
    &'static str,
    Requirement,
    Icon,
    &'static str,
);
type FreeCommandSpec = (
    CommandId,
    &'static str,
    RibbonTab,
    &'static str,
    Requirement,
    Icon,
    Option<&'static str>,
);
type PlannedGroupSpec = (RibbonTab, &'static str, Icon, &'static [&'static str]);

pub fn registry() -> Vec<Command> {
    use CommandId as C;
    use Requirement as R;
    let mut r = Vec::new();
    for (id, name, description, tab, group, requirement, icon, keywords) in
        catalog::OPERATIONAL.iter().copied()
    {
        r.push(Command {
            id,
            name,
            description: description.into(),
            tab,
            group,
            requirement,
            icon,
            keywords,
            shortcut: match id {
                C::OpenPage => Some("Enter no navegador"),
                C::Undo => Some("Ctrl+Z"),
                C::Redo => Some("Ctrl+Y"),
                C::SaveProject => Some("Ctrl+S"),
                C::SaveProjectAs => Some("Ctrl+Shift+S"),
                C::OpenProject => Some("Ctrl+O"),
                _ => None,
            },
        });
    }
    for (id, name, tab, group, requirement, icon, shortcut) in
        catalog::FREE_COMMANDS.iter().copied()
    {
        r.push(Command {
            id,
            name,
            description: format!("{name} — Etapa 16"),
            tab,
            group,
            shortcut,
            requirement,
            icon,
            keywords: "conteúdo livre clipboard",
        });
    }
    let groups = catalog::PLANNED_GROUPS;
    for &(tab, group, icon, names) in groups {
        for &name in names {
            if matches!(
                name,
                "Colar"
                    | "Recortar"
                    | "Copiar"
                    | "Excluir"
                    | "Duplicar"
                    | "Texto"
                    | "Inserir texto"
            ) {
                continue;
            }
            r.push(Command{id:C::Planned(tab,group,name),name,description:format!("{name} — {group}. Recurso planejado, depende da infraestrutura das próximas etapas."),tab,group,shortcut:None,requirement:R::Future,icon,keywords:group});
        }
    }
    r
}
