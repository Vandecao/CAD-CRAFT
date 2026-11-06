// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Application state and frame orchestration; start with CadApp and its update implementation.
use crate::{
    commands::{self, Command, CommandId},
    dialogs::{Dialog, PendingAction},
    editing::{Edit, GraphicEdit},
    navigation::PageIndex,
    panels::{InsertionState, Inspector, NavigatorState},
    preferences::{PanelId, Preferences},
    state::{Selection, Session},
    theme,
};
use cad_craft::domain::*;
use eframe::egui::{self, Context, Key, Rect, Ui, ViewportCommand};
use std::collections::VecDeque;

mod private_state;
use private_state::*;

mod execute;
mod files;
mod ui;

/// Etapa 12: como tratar a trava ao abrir um `.cadcraft`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LockMode {
    /// Adquire a trava; se outra sessão a mantém, pergunta ao usuário.
    Normal,
    /// Abre sem trava, somente leitura.
    ReadOnly,
    /// Assume a trava existente por decisão explícita do usuário.
    TakeOver,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Level {
    Info,
    Warning,
    Error,
}
pub struct Message {
    pub level: Level,
    pub text: String,
}
#[derive(Default)]
pub struct CadApp {
    pub session: Session,
    pub preferences: Preferences,
    pub last_viewport: Option<Rect>,
    pub(crate) chrome: ChromeState,
    pub(crate) navigation: NavigationState,
    pub(crate) navigator: NavigatorState,
    pub(crate) panels_state: PanelState,
    pub(crate) dialogs_state: DialogState,
    pub(crate) free_content: FreeContentState,
    /// Etapa 12: seletor de arquivos, recuperação, recentes e salvamento automático.
    pub files: crate::project_file::FileServices,
    /// Etapa 14: editor de molduras (fechado quando `frame` é `None`).
    pub frame_editor: crate::frame_editor::FrameEditor,
    /// Etapa 17: editor de símbolos.
    pub symbol_editor: crate::symbol_editor::SymbolEditor,
    pub part_editor: crate::part_editor::PartEditor,
    pub macro_library: crate::macro_library::MacroLibrary,
    /// Etapa 21: exportar PDF e imprimir.
    pub pdf: crate::pdf_export::PdfWindow,
    /// Etapa 24: painel Verificações.
    pub checks: crate::checks_ui::CheckPanel,
    /// Etapa 25: importação de projeto do QElectroTech.
    pub qet: crate::qet_import_ui::QetImportPanel,
    /// Etapa 27: assistente de IA e ajuda (aba ao lado do Centro de inserção).
    pub assistant: crate::assistant::AssistantPanel,
    pub edz: crate::edz_import_ui::EdzImportPanel,
    pub reports: crate::reports_ui::ReportWindow,
}
const RECENT_KEY: &str = "cad-craft.recent.v1";
impl CadApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();
        crate::theme::install_symbol_font(&cc.egui_ctx);
        if let Some(storage) = cc.storage
            && let Some(json) = storage.get_string("cad-craft.visual.v1")
        {
            if let Some(p) = Preferences::from_json(&json) {
                app.preferences = p;
            } else {
                app.log(
                    Level::Warning,
                    "Preferências inválidas ou incompatíveis; padrão restaurado.",
                );
            }
        }
        if let Some(storage) = cc.storage
            && let Some(json) = storage.get_string(RECENT_KEY)
        {
            app.files.recent = serde_json::from_str(&json).unwrap_or_default();
        }
        app.enable_recovery(crate::project_file::Recovery::new(
            crate::project_file::Recovery::default_dir(),
        ));
        app.log(
            Level::Info,
            "CAD-CRAFT pronto. Crie, abra um projeto .cadcraft ou abra a demonstração.",
        );
        app
    }
    /// Ativa a pasta de recuperação e oferece a cópia mais recente, se houver.
    pub fn enable_recovery(&mut self, recovery: crate::project_file::Recovery) {
        let entries = recovery.entries();
        self.files.recovery = Some(recovery);
        if let Some(entry) = entries.into_iter().next() {
            self.log(
                Level::Warning,
                format!("Cópia de recuperação encontrada: {}", entry.name),
            );
            if self.dialogs_state.dialog.is_none() {
                self.dialogs_state.dialog = Some(Dialog::Recover(entry));
            }
        }
    }
    pub fn has_dialog(&self) -> bool {
        self.dialogs_state.dialog.is_some()
    }
    pub(crate) fn log(&mut self, level: Level, text: impl Into<String>) {
        if self.panels_state.messages.len() >= 500 {
            self.panels_state.messages.pop_front();
        }
        self.panels_state.messages.push_back(Message {
            level,
            text: text.into(),
        });
    }
    pub(crate) fn edit(&mut self, edit: Edit) {
        match self.session.apply(edit) {
            Ok(()) => self.log(Level::Info, "Propriedade atualizada pelo Core."),
            Err(e) => self.log(Level::Error, e.to_string()),
        }
    }
    /// Etapa 19: liga a ferramenta de inserção de macro na página ativa.
    pub fn start_macro_insertion(&mut self, definition: MacroDefinition) {
        if self.session.active.is_none() {
            self.log(Level::Warning, "Abra uma página para inserir a macro.");
            return;
        }
        self.session.graphics.cancel();
        self.log(
            Level::Info,
            format!(
                "Inserindo macro \"{}\": clique na folha; Girar/Espelhar ajustam; Esc encerra.",
                definition.name().as_str()
            ),
        );
        self.session.graphics.macro_draft = Some(crate::interaction::MacroDraft::new(definition));
        self.session.graphics.tool = crate::interaction::Tool::InsertMacro;
    }
    pub fn graphic_edit(&mut self, edit: GraphicEdit) {
        match self.session.dispatch_graphic(edit) {
            Ok(true) => self.log(Level::Info, "Operação gráfica registrada no histórico."),
            Ok(false) => {}
            Err(e) => {
                self.session.graphics.cancel();
                self.log(Level::Error, e.to_string());
            }
        }
    }
    pub(crate) fn open_page(&mut self, id: PageId) {
        if let Err(e) = self.session.open_page(id) {
            self.log(Level::Error, e.to_string());
        }
    }
    /// Há diálogo modal aberto (confirmação, edição etc.).
    pub fn dialog_open(&self) -> bool {
        self.dialogs_state.dialog.is_some()
    }
}
impl eframe::App for CadApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.show(ctx);
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(json) = self.preferences.to_json() {
            storage.set_string("cad-craft.visual.v1", json);
        }
        if let Ok(json) = serde_json::to_string(&self.files.recent) {
            storage.set_string(RECENT_KEY, json);
        }
    }
    fn persist_egui_memory(&self) -> bool {
        false
    }
}
