// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Command dispatch; start with CadApp::execute.
use super::*;

impl CadApp {
    pub fn execute(&mut self, id: CommandId, ctx: &Context) {
        if self.assistant.highlight == Some(id) {
            self.assistant.highlight = None;
        }
        if let Some(reason) = self
            .chrome
            .commands
            .iter()
            .find(|c| c.id == id)
            .and_then(|c| c.reason(&self.session))
        {
            self.log(Level::Warning, reason);
            return;
        }
        use CommandId as C;
        match id {
            C::GenerateReport => self.reports.open(crate::reports_ui::ReportMode::Generate),
            C::UpdateReport => self.reports.open(crate::reports_ui::ReportMode::Update),
            C::ExportList => self.reports.open(crate::reports_ui::ReportMode::Export),
            C::CheckProject => self.start_check(false),
            C::CheckPage => self.start_check(true),
            C::NumberDevices | C::NumberConnections | C::NumberPages => {
                self.dialogs_state.numbering.open(id)
            }
            C::ExportPdf => self.pdf.open(crate::pdf_export::PdfMode::Export),
            C::Print => self.pdf.open(crate::pdf_export::PdfMode::Print),
            C::ExportDxf => self.pdf.open(crate::pdf_export::PdfMode::Dxf),
            C::ExportDwg => self.pdf.open(crate::pdf_export::PdfMode::Dwg),
            C::InsertFreeText
            | C::InsertLine
            | C::InsertRectangle
            | C::InsertCircle
            | C::InsertArc
            | C::InsertEllipse => {
                self.session.graphics.cancel();
                self.preferences.panel_mut(PanelId::Insertion).visible = true;
                self.preferences.panel_mut(PanelId::Insertion).collapsed = false;
                self.session.graphics.tool = match id {
                    C::InsertFreeText => crate::interaction::Tool::InsertText,
                    C::InsertLine => crate::interaction::Tool::InsertGraphic(GraphicKind::Line),
                    C::InsertRectangle => {
                        crate::interaction::Tool::InsertGraphic(GraphicKind::Rectangle)
                    }
                    C::InsertCircle => crate::interaction::Tool::InsertGraphic(GraphicKind::Circle),
                    C::InsertEllipse => {
                        crate::interaction::Tool::InsertGraphic(GraphicKind::Ellipse)
                    }
                    _ => crate::interaction::Tool::InsertGraphic(GraphicKind::Arc),
                };
            }
            C::Copy | C::Cut => {
                if let Err(e) = self.session.capture_selection(id == C::Cut) {
                    self.log(Level::Error, e.to_string());
                } else {
                    // System clipboard text ensures Windows emits Paste for Ctrl+V; the actual content stays in the session.
                    let n = self.session.clipboard.as_ref().map_or(0, |c| c.len());
                    ctx.copy_text(format!("CAD-CRAFT: {n} item(ns) na área de transferência"));
                    self.log(
                        Level::Info,
                        if id == C::Cut {
                            "Seleção recortada (Ctrl+Z desfaz); escolha Colar e clique no destino."
                        } else {
                            "Seleção copiada; escolha Colar e clique no destino."
                        },
                    );
                }
            }
            C::Paste | C::PasteSameDevice => {
                if let Some(clip) = &mut self.session.clipboard {
                    clip.keep_device = id == C::PasteSameDevice;
                    self.session.graphics.cancel();
                    self.session.graphics.tool = crate::interaction::Tool::Paste;
                    self.log(
                        Level::Info,
                        "Clique no destino para colar; repita para várias cópias; Esc encerra.",
                    );
                } else {
                    self.log(Level::Warning, "Área de transferência vazia.");
                }
            }
            C::Duplicate => {
                if let Err(e) = self.session.duplicate_selection() {
                    self.log(Level::Error, e.to_string());
                }
            }
            C::DeleteFree => {
                if let Err(e) = self.session.delete_graphic_selection() {
                    self.log(Level::Error, e.to_string());
                }
            }
            C::InsertInterruption | C::InsertPotential => {
                self.session.graphics.cancel();
                self.session.graphics.tool = if id == C::InsertPotential {
                    crate::interaction::Tool::InsertPotential
                } else {
                    crate::interaction::Tool::InsertInterruption
                };
            }
            C::ReferenceProperties => self.open_reference_properties(),
            C::LocateMainFunction | C::LocateAuxiliaryFunctions => {
                let destinations = self
                    .session
                    .selected_reference_device()
                    .and_then(|d| self.session.project()?.device_references(d).ok())
                    .unwrap_or_default();
                let wanted = if id == C::LocateMainFunction {
                    FunctionRole::Main
                } else {
                    FunctionRole::Auxiliary
                };
                let ids: Vec<_> = destinations
                    .iter()
                    .filter(|r| r.role == wanted)
                    .map(|r| r.representation)
                    .collect();
                if ids.len() == 1 {
                    self.navigate_to_reference(ids[0]);
                } else {
                    self.open_reference_properties();
                }
            }
            C::DemoConnections => self.request(PendingAction::DemoConnections, ctx),
            C::TerminalStripEditor
            | C::ConnectorEditor
            | C::CableEditor
            | C::WiringEditor
            | C::TechnicalLists => {
                use crate::physical_editor::PhysicalTab;
                let tab = match id {
                    C::TerminalStripEditor => PhysicalTab::Strips,
                    C::ConnectorEditor => PhysicalTab::Connectors,
                    C::CableEditor => PhysicalTab::Cables,
                    C::WiringEditor => PhysicalTab::Wiring,
                    _ => PhysicalTab::Lists,
                };
                self.panels_state.physical_editor.open(tab);
                if let crate::state::Selection::Representation(rep) = self.session.selection
                    && let Some(route) =
                        self.session.project().and_then(|p| p.connection_route(rep))
                {
                    self.panels_state
                        .physical_editor
                        .focus_connection(route.connection_id());
                }
            }
            C::DrawConnection | C::InsertConnectionAngle => {
                if let Err(e) = self.session.begin_connection() {
                    self.log(Level::Error, e.to_string());
                }
                if id == C::InsertConnectionAngle
                    && let Some(d) = &mut self.session.graphics.connection
                {
                    d.horizontal_first = false;
                }
            }
            C::InsertJunction
            | C::EditConnection
            | C::AddConnectionVertex
            | C::RemoveConnectionVertex => {
                self.session.graphics.cancel();
                self.session.graphics.tool = match id {
                    C::InsertJunction => crate::interaction::Tool::InsertJunction,
                    C::AddConnectionVertex => crate::interaction::Tool::AddVertex,
                    C::RemoveConnectionVertex => crate::interaction::Tool::RemoveVertex,
                    _ => crate::interaction::Tool::EditRoute,
                };
            }
            C::DeleteConnection => {
                if let Some(id) = self.session.graphics.selection.primary() {
                    self.graphic_edit(GraphicEdit::Drawing(ConnectionDrawingEdit::DeleteRoute {
                        id,
                    }));
                }
            }
            C::DeleteJunction => {
                if let Some(id) = self.session.graphics.junction_selection {
                    self.graphic_edit(GraphicEdit::Drawing(
                        ConnectionDrawingEdit::DeleteJunction { id },
                    ));
                    self.session.graphics.junction_selection = None;
                }
            }
            C::HighlightNetwork => self.session.highlight_network(),
            C::ToggleJunctionStyle => {
                self.preferences.junction_outline = !self.preferences.junction_outline
            }
            C::SymbolLibrary => {
                let panel = self.preferences.panel_mut(PanelId::Insertion);
                panel.visible = true;
                panel.collapsed = false;
                self.panels_state.insertion.show_symbols();
            }
            C::Select => self.session.graphics.cancel(),
            C::Move => {
                self.session.graphics.cancel();
                self.session.graphics.tool = crate::interaction::Tool::Move;
            }
            C::Rotate90 | C::Rotate180 | C::Rotate270 | C::MirrorHorizontal | C::MirrorVertical
                if self.session.graphics.macro_draft.is_some() =>
            {
                // Rotate or mirror the macro while positioning it.
                let draft = self
                    .session
                    .graphics
                    .macro_draft
                    .as_mut()
                    .expect("conferido");
                match id {
                    C::Rotate90 => draft.rotation = (draft.rotation + 90.0) % 360.0,
                    C::Rotate180 => draft.rotation = (draft.rotation + 180.0) % 360.0,
                    C::Rotate270 => draft.rotation = (draft.rotation + 270.0) % 360.0,
                    C::MirrorHorizontal => draft.mirror = !draft.mirror,
                    _ => {
                        draft.mirror = !draft.mirror;
                        draft.rotation = (draft.rotation + 180.0) % 360.0;
                    }
                }
            }
            C::Rotate90 => self.graphic_edit(GraphicEdit::Rotate(90.0)),
            C::Rotate180 => self.graphic_edit(GraphicEdit::Rotate(180.0)),
            C::Rotate270 => self.graphic_edit(GraphicEdit::Rotate(270.0)),
            C::MirrorHorizontal => self.graphic_edit(GraphicEdit::MirrorHorizontal),
            C::MirrorVertical => self.graphic_edit(GraphicEdit::MirrorVertical),
            C::Undo => self.graphic_edit(GraphicEdit::Undo),
            C::Redo => self.graphic_edit(GraphicEdit::Redo),
            C::ShowProperties | C::PageProperties => {
                if id == C::PageProperties
                    && let Some(page) = self.session.active
                {
                    self.session.select(Selection::Page(page));
                }
                let panel = self.preferences.panel_mut(PanelId::Properties);
                panel.visible = true;
                panel.collapsed = false;
            }
            C::LocateReference => {
                let destinations = self.session.reference_destinations();
                if destinations.len() == 1 {
                    self.navigate_to_reference(destinations[0]);
                } else {
                    self.open_reference_properties();
                }
            }
            C::NewProject => self.dialogs_state.dialog = Some(Dialog::NewProject(String::new())),
            C::Demo => self.request(PendingAction::Demo, ctx),
            C::OpenSnapshot => self.dialogs_state.backstage = Some(crate::strings::IMPORT),
            C::ImportQet => self.dialogs_state.backstage = Some(crate::strings::IMPORT),
            C::ImportEdz => self.edz.open = true,
            C::OpenProject => self.open_project_dialog(ctx),
            C::FrameEditor
            | C::FrameNew
            | C::FrameDuplicate
            | C::FrameDelete
            | C::FrameExport
            | C::FrameImport
            | C::FrameImportExternal
            | C::FrameExportDxf => self.frame_command(id),
            C::SymbolNew
            | C::SymbolEditor
            | C::SymbolExport
            | C::SymbolImport
            | C::SymbolImportElmt
            | C::SymbolImportElmtFolder
            | C::SymbolReload
            | C::SymbolUpdateProject => self.symbol_command(id),
            C::PartEditor => self.part_editor.open = true,
            C::MacroNavigator | C::MacroCreate | C::MacroPage => {
                self.macro_library.open = true;
                self.macro_library.message = match id {
                    C::MacroCreate => {
                        "Preencha ID e nome em \"Nova macro\" e clique em Criar da seleção.".into()
                    }
                    C::MacroPage => "Para guardar a página ativa: preencha ID e nome e clique em \
                                     Criar da página ativa. Para inserir: escolha uma macro de \
                                     página e clique em Inserir como página nova."
                        .into(),
                    _ => std::mem::take(&mut self.macro_library.message),
                };
            }
            C::MacroInsert => match self.macro_library.selected().cloned() {
                Some(definition) if definition.is_page() => {
                    match self.session.insert_page_macro(&definition) {
                        Ok((_, report)) => self.log(
                            Level::Info,
                            format!("Página criada pela macro. {}", report.summary()),
                        ),
                        Err(e) => self.log(Level::Error, e.to_string()),
                    }
                }
                Some(definition) => self.start_macro_insertion(definition),
                None => {
                    self.macro_library.open = true;
                    self.log(Level::Warning, "Escolha uma macro no navegador de macros.");
                }
            },
            C::SaveProject => {
                self.save_project();
            }
            C::SaveProjectAs => {
                self.save_project_as();
            }
            C::ExportSnapshot => {
                self.dialogs_state.backstage = Some(crate::strings::EXPORT);
                match self.session.snapshot() {
                    Ok(json) => self.dialogs_state.snapshot_text = json,
                    Err(e) => self.log(Level::Error, e),
                }
            }
            C::CloseProject => self.request(PendingAction::Close, ctx),
            C::Exit => self.request(PendingAction::Exit, ctx),
            C::Settings => self.dialogs_state.backstage = Some(crate::strings::SETTINGS),
            C::NewDocument => self.dialogs_state.dialog = Some(Dialog::NewDocument(String::new())),
            C::NewPage => {
                if let Some(doc) = self.session.selected_document() {
                    self.dialogs_state.dialog = Some(Dialog::new_page(doc));
                }
            }
            C::OpenPage => {
                if let Some(id) = self.session.selected_page() {
                    self.open_page(id);
                }
            }
            C::RenamePage | C::IdentifyPage => {
                if let Some(page_id) = self.session.selected_page()
                    && let Some(page) = self.session.project().and_then(|p| p.page(page_id))
                {
                    let number = id == C::IdentifyPage;
                    self.dialogs_state.dialog = Some(Dialog::Rename {
                        id: page_id,
                        number,
                        value: if number { page.number() } else { page.name() }
                            .as_str()
                            .into(),
                    });
                }
            }
            C::DeletePage => {
                if let Some(id) = self.session.selected_page() {
                    self.dialogs_state.dialog = Some(Dialog::DeletePage(id));
                }
            }
            C::PageUp | C::PageDown => {
                if let Some(page) = self.session.selected_page()
                    && let Err(e) = self
                        .session
                        .move_page(page, if id == C::PageUp { -1 } else { 1 })
                {
                    self.log(Level::Error, e.to_string());
                }
            }
            C::Navigator => self.toggle_panel(PanelId::Navigator),
            C::Properties => self.toggle_panel(PanelId::Properties),
            C::Preview => self.toggle_panel(PanelId::Preview),
            C::Insertion => self.toggle_panel(PanelId::Insertion),
            C::Messages => self.toggle_panel(PanelId::Messages),
            C::FindPages => {
                self.preferences.panel_mut(PanelId::Navigator).visible = true;
                self.preferences.panel_mut(PanelId::Navigator).collapsed = false;
                self.navigator.tab = crate::panels::NavigatorTab::Pages;
                self.navigator.focus_filter = true;
            }
            C::SyncSelection => {
                if let Some(page) = self.session.active {
                    self.session.select(Selection::Page(page));
                    self.navigator.reveal_page(page);
                    let panel = self.preferences.panel_mut(PanelId::Navigator);
                    panel.visible = true;
                    panel.collapsed = false;
                }
            }
            C::RestoreLayout => {
                self.preferences.restore_layout();
                self.chrome.layout_generation += 1;
            }
            C::ToggleGrid => self.preferences.grid_visible = !self.preferences.grid_visible,
            C::ToggleSnap => self.preferences.snap_enabled = !self.preferences.snap_enabled,
            C::FitPage | C::FitContent | C::ZoomIn | C::ZoomOut | C::Zoom100 | C::PreviousView => {
                self.camera_command(id)
            }
            C::DeviceNavigator
            | C::StructureNavigator
            | C::ConnectionNavigator
            | C::CableNavigator
            | C::ConnectorNavigator
            | C::TerminalNavigator => {
                use crate::panels::NavigatorTab as N;
                self.navigator.tab = match id {
                    C::DeviceNavigator => N::Devices,
                    C::StructureNavigator => N::Structures,
                    C::ConnectionNavigator => N::Connections,
                    C::CableNavigator => N::Cables,
                    _ => N::Terminals,
                };
                let p = self.preferences.panel_mut(PanelId::Navigator);
                p.visible = true;
                p.collapsed = false;
            }
            C::Validate => {
                if let Some(p) = self.session.project() {
                    match p.validate() {
                        Ok(()) => self.log(
                            Level::Info,
                            "Integridade estrutural do Core válida. Esta operação não é ERC.",
                        ),
                        Err(e) => self.log(Level::Error, e.to_string()),
                    }
                }
            }
            C::Repeat | C::Planned(..) => self.log(
                Level::Warning,
                "Comando planejado, indisponível nesta etapa.",
            ),
        }
    }
}
