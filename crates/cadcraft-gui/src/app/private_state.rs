// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Auxiliary application state by subject; start with NavigationState and PanelState.
use super::*;

pub(crate) struct NavigationState {
    pub(crate) index: PageIndex,
    pub(crate) index_revision: u64,
}
impl Default for NavigationState {
    fn default() -> Self {
        Self {
            index: PageIndex::default(),
            index_revision: u64::MAX,
        }
    }
}

#[derive(Default)]
pub(crate) struct PanelState {
    pub(crate) insertion: InsertionState,
    pub(crate) inspector: Inspector,
    pub(crate) reference_inspector: crate::reference_properties::ReferenceInspector,
    pub(crate) connection_scene: crate::connection_editor::ConnectionScene,
    pub(crate) connection_inspector: crate::connection_properties::ConnectionInspector,
    pub(crate) physical_editor: crate::physical_editor::PhysicalEditor,
    pub(crate) messages: VecDeque<Message>,
    pub(crate) message_filter: Option<Level>,
}
pub(crate) struct DialogState {
    pub(crate) backstage: Option<&'static str>,
    pub(crate) backstage_section: &'static str,
    pub(crate) dialog: Option<Dialog>,
    pub(crate) numbering: crate::numbering_ui::NumberingWindow,
    pub(crate) snapshot_text: String,
    pub(crate) snapshot_path: String,
}
impl Default for DialogState {
    fn default() -> Self {
        Self {
            backstage: None,
            backstage_section: "Propriedades",
            dialog: None,
            numbering: crate::numbering_ui::NumberingWindow::default(),
            snapshot_text: String::new(),
            snapshot_path: String::new(),
        }
    }
}

pub(crate) struct FreeContentState {
    pub(crate) free_text: String,
    pub(crate) free_height: f64,
    pub(crate) free_stroke: f64,
    pub(crate) free_align: TextAlignment,
    pub(crate) free_edit: Option<(RepresentationId, SymbolPrimitive, f64, TextAlignment)>,
    pub(crate) free_edit_revision: u64,
}
impl Default for FreeContentState {
    fn default() -> Self {
        Self {
            free_text: "Texto".into(),
            free_height: 4.0,
            free_stroke: 0.25,
            free_align: TextAlignment::Left,
            free_edit: None,
            free_edit_revision: u64::MAX,
        }
    }
}

pub(crate) struct ChromeState {
    pub(crate) commands: Vec<Command>,
    pub(crate) command_query: String,
    pub(crate) cursor_world: Option<Point2D>,
    pub(crate) context_object: Option<RepresentationId>,
    pub(crate) layout_generation: u64,
    pub(crate) fullscreen: bool,
    pub(crate) exit_confirmed: bool,
    pub(crate) theme_ready: bool,
    pub(in crate::app) last_system_theme: Option<egui::Theme>,
    /// Last loaded symbol-library directory; reload when it changes.
    pub(in crate::app) symbol_library_seen: Option<String>,
}
impl Default for ChromeState {
    fn default() -> Self {
        Self {
            commands: commands::registry(),
            command_query: String::new(),
            cursor_world: None,
            context_object: None,
            layout_generation: 0,
            fullscreen: false,
            exit_confirmed: false,
            theme_ready: false,
            last_system_theme: None,
            symbol_library_seen: None,
        }
    }
}
