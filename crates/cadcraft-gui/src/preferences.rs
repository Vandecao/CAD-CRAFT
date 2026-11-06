// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Persisted UI preferences; start with Preferences and its defaults.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RibbonTab {
    #[default]
    Home,
    Insert,
    Edit,
    View,
    Devices,
    Connections,
    Cables,
    Tools,
    Preplanning,
    MasterData,
    Integrations,
}
impl RibbonTab {
    pub const ALL: [Self; 11] = [
        Self::Home,
        Self::Insert,
        Self::Edit,
        Self::View,
        Self::Devices,
        Self::Connections,
        Self::Cables,
        Self::Tools,
        Self::Preplanning,
        Self::MasterData,
        Self::Integrations,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Home => "Página Inicial",
            Self::Insert => "Inserir",
            Self::Edit => "Editar",
            Self::View => "Visualização",
            Self::Devices => "Dispositivos",
            Self::Connections => "Conexões",
            Self::Cables => "Cabos",
            Self::Tools => "Ferramentas",
            Self::Preplanning => "Pré-planejamento",
            Self::MasterData => "Dados-mestre",
            Self::Integrations => "Integrações",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PanelId {
    Navigator,
    Insertion,
    Properties,
    Preview,
    Messages,
}
impl PanelId {
    pub const ALL: [Self; 5] = [
        Self::Navigator,
        Self::Insertion,
        Self::Properties,
        Self::Preview,
        Self::Messages,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Navigator => "Páginas",
            Self::Insertion => "Centro de inserção",
            Self::Properties => "Visão geral de propriedades",
            Self::Preview => "Pré-visualização",
            Self::Messages => "Mensagens",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PanelPreference {
    pub visible: bool,
    pub pinned: bool,
    pub collapsed: bool,
    pub width: f32,
}
impl Default for PanelPreference {
    fn default() -> Self {
        Self {
            visible: true,
            pinned: true,
            collapsed: false,
            width: 240.0,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub version: u32,
    pub ribbon_tab: RibbonTab,
    pub ribbon_collapsed: bool,
    pub panels: [PanelPreference; 5],
    pub preview_height: f32,
    pub messages_height: f32,
    pub grid_mm: f64,
    pub grid_visible: bool,
    pub snap_enabled: bool,
    pub junction_outline: bool,
    pub thumbnails_grid: bool,
    /// Initial directory for .cadframe import and export.
    pub frame_library: String,
    /// Symbol-library directory containing .cadsym files.
    #[serde(default)]
    pub symbol_library: String,
    /// Part-library directory containing .cadpart files.
    pub part_library: String,
    /// Macro-library directory containing .cadmacro files.
    pub macro_library: String,
    /// External DWG converter path, normally ODA File Converter.
    pub dwg_converter: String,
    /// QElectroTech .elmt import scale in millimetres per pixel.
    pub elmt_mm_per_px: f64,
    /// Opaque installation identifier required by Sign in with ChatGPT (urn:uuid). It does not identify the user.
    pub assistant_host_id: String,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            version: 1,
            ribbon_tab: RibbonTab::Home,
            ribbon_collapsed: false,
            panels: std::array::from_fn(|i| PanelPreference {
                width: if i == 0 { 280.0 } else { 225.0 },
                ..Default::default()
            }),
            preview_height: 175.0,
            messages_height: 95.0,
            grid_mm: 4.0,
            grid_visible: true,
            snap_enabled: false,
            junction_outline: false,
            thumbnails_grid: true,
            frame_library: String::new(),
            symbol_library: String::new(),
            part_library: String::new(),
            macro_library: String::new(),
            dwg_converter: String::new(),
            elmt_mm_per_px: cad_craft::interchange::elmt::DEFAULT_MM_PER_PX,
            assistant_host_id: String::new(),
        }
    }
}
impl Preferences {
    pub fn panel(&self, id: PanelId) -> &PanelPreference {
        &self.panels[id as usize]
    }
    pub fn panel_mut(&mut self, id: PanelId) -> &mut PanelPreference {
        &mut self.panels[id as usize]
    }
    pub fn sanitize(&mut self) {
        for p in &mut self.panels {
            p.width = finite(p.width, 225.0).clamp(100.0, 1600.0);
        }
        self.preview_height = finite(self.preview_height, 175.0).clamp(70.0, 800.0);
        self.messages_height = finite(self.messages_height, 95.0).clamp(55.0, 600.0);
        if !self.grid_mm.is_finite() || self.grid_mm < 0.1 || self.grid_mm > 1000.0 {
            self.grid_mm = 4.0;
        }
        if !self.elmt_mm_per_px.is_finite()
            || self.elmt_mm_per_px <= 0.0
            || self.elmt_mm_per_px > 100.0
        {
            self.elmt_mm_per_px = cad_craft::interchange::elmt::DEFAULT_MM_PER_PX;
        }
    }
    pub fn from_json(json: &str) -> Option<Self> {
        if json.len() > 64 * 1024 {
            return None;
        }
        let mut p: Self = serde_json::from_str(json).ok()?;
        if p.version != 1 {
            return None;
        }
        p.sanitize();
        Some(p)
    }
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    pub fn restore_layout(&mut self) {
        let defaults = Self::default();
        self.panels = defaults.panels;
        self.preview_height = defaults.preview_height;
        self.messages_height = defaults.messages_height;
        self.ribbon_collapsed = false;
    }
}
fn finite(value: f32, default: f32) -> f32 {
    if value.is_finite() { value } else { default }
}
