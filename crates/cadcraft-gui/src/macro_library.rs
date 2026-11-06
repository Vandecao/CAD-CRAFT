// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Macro library in a configurable .cadmacro directory and its browser. Start with the catalog: category tree, search, thumbnails, capture selection or page, insertion, import, export and deletion.
use crate::error::{GuiError, GuiResult};
use crate::{camera::Camera, preferences::Preferences, state::Session};
use cad_craft::{domain::*, persistence};
use eframe::egui::{self, Context, Ui};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct MacroLibrary {
    pub open: bool,
    catalog: BTreeMap<MacroId, MacroDefinition>,
    seen_folder: String,
    selected: Option<MacroId>,
    search: String,
    id: String,
    name: String,
    category: String,
    description: String,
    path: String,
    confirm_delete: bool,
    pub message: String,
    insert: Option<MacroDefinition>,
}

#[derive(Clone)]
enum Action {
    Reload,
    CreateFromSelection,
    CreateFromPage,
    Insert(MacroId),
    InsertPage(MacroId),
    Delete(MacroId),
    Export(MacroId),
    Import,
}

fn safe_stem(id: &MacroId) -> String {
    id.as_str()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect()
}
pub fn read_macro(path: &Path) -> GuiResult<MacroDefinition> {
    if std::fs::metadata(path).map_err(GuiError::from)?.len() > persistence::MAX_MACRO_FILE_BYTES {
        return Err("arquivo .cadmacro grande demais".into());
    }
    persistence::read_macro_package(&std::fs::read(path).map_err(GuiError::from)?)
        .map_err(GuiError::from)
}

impl MacroLibrary {
    pub(crate) fn install_catalog(&mut self, macros: Vec<MacroDefinition>, folder: &str) {
        self.seen_folder = folder.to_owned();
        self.catalog = macros
            .into_iter()
            .map(|definition| (definition.id().clone(), definition))
            .collect();
        if self
            .selected
            .as_ref()
            .is_some_and(|id| !self.catalog.contains_key(id))
        {
            self.selected = None;
        }
    }
    /// Macro chosen for insertion; the application activates the tool.
    pub fn take_insert(&mut self) -> Option<MacroDefinition> {
        self.insert.take()
    }
    pub fn selected(&self) -> Option<&MacroDefinition> {
        self.selected.as_ref().and_then(|id| self.catalog.get(id))
    }
    pub fn catalog(&self) -> impl Iterator<Item = &MacroDefinition> {
        self.catalog.values()
    }
    fn folder(preferences: &Preferences) -> Option<PathBuf> {
        let path = preferences.macro_library.trim();
        (!path.is_empty()).then(|| PathBuf::from(path))
    }
    pub fn reload(&mut self, preferences: &Preferences) -> GuiResult<usize> {
        let Some(dir) = Self::folder(preferences) else {
            return Ok(self.catalog.len());
        };
        if !dir.exists() {
            self.catalog.clear();
            return Ok(0);
        }
        let mut next = BTreeMap::new();
        let mut refused = Vec::new();
        for entry in std::fs::read_dir(&dir)
            .map_err(GuiError::from)?
            .take(50_000)
        {
            let path = entry.map_err(GuiError::from)?.path();
            if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case(persistence::MACRO_EXTENSION))
            {
                match read_macro(&path) {
                    Ok(definition) => {
                        next.insert(definition.id().clone(), definition);
                    }
                    Err(error) => refused.push(format!("{}: {error}", path.display())),
                }
            }
        }
        self.catalog = next;
        if self
            .selected
            .as_ref()
            .is_some_and(|id| !self.catalog.contains_key(id))
        {
            self.selected = None;
        }
        if refused.is_empty() {
            Ok(self.catalog.len())
        } else {
            Err(format!(
                "{} macro(s) carregada(s); recusadas: {}",
                self.catalog.len(),
                refused.join("; ")
            )
            .into())
        }
    }
    /// Save in the configured directory and session catalog; the same ID replaces the macro as a new version.
    pub fn store(
        &mut self,
        mut definition: MacroDefinition,
        preferences: &Preferences,
    ) -> GuiResult<String> {
        if let Some(existing) = self.catalog.get(definition.id()) {
            definition = definition
                .with_version(existing.version().saturating_add(1))
                .map_err(GuiError::from)?;
        }
        let saved = if let Some(dir) = Self::folder(preferences) {
            std::fs::create_dir_all(&dir).map_err(GuiError::from)?;
            let path = dir.join(format!(
                "{}.{}",
                safe_stem(definition.id()),
                persistence::MACRO_EXTENSION
            ));
            let bytes = persistence::write_macro_package(&definition).map_err(GuiError::from)?;
            crate::project_file::write_atomic(&path, &mut bytes.as_slice())
                .map_err(GuiError::from)?;
            format!("gravada em {}", path.display())
        } else {
            "mantida só nesta sessão (configure a pasta de macros em Configurações)".into()
        };
        let message = format!(
            "Macro \"{}\" versão {} {saved}.",
            definition.name().as_str(),
            definition.version()
        );
        let id = definition.id().clone();
        self.catalog.insert(id.clone(), definition);
        self.selected = Some(id);
        Ok(message)
    }
    fn delete(&mut self, id: &MacroId, preferences: &Preferences) -> GuiResult<String> {
        if !self.confirm_delete {
            return Err("marque \"Confirmar exclusão\" antes de excluir".into());
        }
        if let Some(dir) = Self::folder(preferences) {
            let path = dir.join(format!(
                "{}.{}",
                safe_stem(id),
                persistence::MACRO_EXTENSION
            ));
            if path.exists() {
                std::fs::remove_file(&path).map_err(GuiError::from)?;
            }
        }
        self.catalog.remove(id);
        self.selected = None;
        self.confirm_delete = false;
        Ok(format!("Macro {} excluída da biblioteca.", id.as_str()))
    }

    pub fn show(&mut self, session: &mut Session, preferences: &Preferences, ctx: &Context) {
        if !self.open {
            return;
        }
        if self.seen_folder != preferences.macro_library {
            self.seen_folder = preferences.macro_library.clone();
            if let Err(error) = self.reload(preferences) {
                self.message = error.to_string();
            }
        }
        let mut open = self.open;
        let mut action = None;
        egui::Window::new("Macros")
            .open(&mut open)
            .default_size([720.0, 600.0])
            .resizable(true)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if !self.message.is_empty() {
                        ui.label(&self.message);
                    }
                    self.navigator_ui(ui, &mut action);
                    ui.separator();
                    self.create_ui(ui, &mut action);
                });
            });
        self.open = open;
        if let Some(action) = action {
            self.message = match self.apply(action, session, preferences) {
                Ok(message) => message,
                Err(error) => format!("Operação recusada: {error}"),
            };
        }
    }

    fn navigator_ui(&mut self, ui: &mut Ui, action: &mut Option<Action>) {
        ui.heading(crate::strings::MACRO_NAVIGATOR);
        ui.horizontal(|ui| {
            ui.label("Buscar");
            ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("nome, ID ou categoria"));
            if ui.button("Recarregar macros").clicked() {
                *action = Some(Action::Reload);
            }
        });
        let needle = self.search.trim().to_lowercase();
        let mut groups: BTreeMap<String, Vec<&MacroDefinition>> = BTreeMap::new();
        for definition in self.catalog.values() {
            let haystack = format!(
                "{} {} {} {}",
                definition.id().as_str(),
                definition.name().as_str(),
                definition.category(),
                definition.description()
            )
            .to_lowercase();
            if needle.is_empty() || haystack.contains(&needle) {
                let category = if definition.category().is_empty() {
                    "Sem categoria".to_owned()
                } else {
                    definition.category().to_owned()
                };
                groups.entry(category).or_default().push(definition);
            }
        }
        if groups.is_empty() {
            ui.label("Nenhuma macro na biblioteca.");
        }
        let mut selected = None;
        for (category, items) in &groups {
            egui::CollapsingHeader::new(category)
                .default_open(true)
                .show(ui, |ui| {
                    for definition in items {
                        let label = format!(
                            "{} · {} · v{}{}",
                            definition.name().as_str(),
                            definition.id().as_str(),
                            definition.version(),
                            if definition.is_page() {
                                " · página"
                            } else {
                                ""
                            }
                        );
                        if ui
                            .selectable_label(
                                self.selected.as_ref() == Some(definition.id()),
                                label,
                            )
                            .clicked()
                        {
                            selected = Some(definition.id().clone());
                        }
                    }
                });
        }
        if let Some(id) = selected {
            self.selected = Some(id);
            self.confirm_delete = false;
        }
        let Some(definition) = self.selected().cloned() else {
            return;
        };
        ui.separator();
        ui.label(format!(
            "{} — {} símbolo(s)/gráfico(s), {} fio(s), {} dispositivo(s)",
            definition.name().as_str(),
            definition.content().representations().len(),
            definition.content().routes().len(),
            definition.content().devices().len()
        ));
        if !definition.description().is_empty() {
            ui.label(definition.description());
        }
        paint_thumbnail(ui, &definition);
        ui.horizontal(|ui| {
            if definition.is_page() {
                if ui.button("Inserir como página nova").clicked() {
                    *action = Some(Action::InsertPage(definition.id().clone()));
                }
            } else if ui.button("Inserir na página").clicked() {
                *action = Some(Action::Insert(definition.id().clone()));
            }
            ui.checkbox(&mut self.confirm_delete, "Confirmar exclusão");
            if ui.button("Excluir da biblioteca").clicked() {
                *action = Some(Action::Delete(definition.id().clone()));
            }
        });
        ui.horizontal(|ui| {
            ui.label("Arquivo");
            ui.add(
                egui::TextEdit::singleline(&mut self.path)
                    .hint_text("C:/pasta/macro.cadmacro")
                    .desired_width(300.0),
            );
            if ui.button("Exportar macro").clicked() {
                *action = Some(Action::Export(definition.id().clone()));
            }
        });
    }

    fn create_ui(&mut self, ui: &mut Ui, action: &mut Option<Action>) {
        ui.heading("Nova macro");
        for (label, value) in [
            ("ID da macro", &mut self.id),
            ("Nome da macro", &mut self.name),
            ("Categoria da macro", &mut self.category),
            ("Descrição da macro", &mut self.description),
        ] {
            ui.horizontal(|ui| {
                ui.label(label);
                ui.add(egui::TextEdit::singleline(value).desired_width(320.0));
            });
        }
        ui.horizontal(|ui| {
            if ui.button("Criar da seleção").clicked() {
                *action = Some(Action::CreateFromSelection);
            }
            if ui.button("Criar da página ativa").clicked() {
                *action = Some(Action::CreateFromPage);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Importar de");
            ui.add(
                egui::TextEdit::singleline(&mut self.path)
                    .hint_text("C:/pasta/macro.cadmacro")
                    .desired_width(300.0),
            );
            if ui.button("Importar macro").clicked() {
                *action = Some(Action::Import);
            }
        });
    }

    fn apply(
        &mut self,
        action: Action,
        session: &mut Session,
        preferences: &Preferences,
    ) -> GuiResult<String> {
        match action {
            Action::Reload => self
                .reload(preferences)
                .map(|n| format!("{n} macro(s) na biblioteca.")),
            Action::CreateFromSelection | Action::CreateFromPage => {
                let name = if self.name.trim().is_empty() {
                    self.id.trim()
                } else {
                    self.name.trim()
                };
                let (definition, report) = if matches!(action, Action::CreateFromPage) {
                    session.macro_from_page(self.id.trim(), name, &self.category, &self.description)
                } else {
                    session.macro_from_selection(
                        self.id.trim(),
                        name,
                        &self.category,
                        &self.description,
                        preferences.grid_mm,
                    )
                }
                .map_err(GuiError::from)?;
                let message = self.store(definition, preferences)?;
                let summary = report.summary();
                Ok(if summary.is_empty() {
                    message
                } else {
                    format!("{message} {summary}")
                })
            }
            Action::Insert(id) => {
                let definition = self
                    .catalog
                    .get(&id)
                    .cloned()
                    .ok_or(crate::strings::MISSING_MACRO)?;
                self.insert = Some(definition);
                Ok("Clique na folha para inserir; Girar/Espelhar ajustam; Esc encerra.".into())
            }
            Action::InsertPage(id) => {
                let definition = self
                    .catalog
                    .get(&id)
                    .cloned()
                    .ok_or(crate::strings::MISSING_MACRO)?;
                let (_, report) = session
                    .insert_page_macro(&definition)
                    .map_err(GuiError::from)?;
                Ok(format!(
                    "Página criada pela macro \"{}\". {}",
                    definition.name().as_str(),
                    report.summary()
                )
                .trim()
                .to_owned())
            }
            Action::Delete(id) => self.delete(&id, preferences),
            Action::Export(id) => {
                let definition = self.catalog.get(&id).ok_or(crate::strings::MISSING_MACRO)?;
                let path = PathBuf::from(self.path.trim());
                if path.as_os_str().is_empty() {
                    return Err("informe o arquivo de destino".into());
                }
                let bytes = persistence::write_macro_package(definition).map_err(GuiError::from)?;
                crate::project_file::write_atomic(&path, &mut bytes.as_slice())
                    .map_err(GuiError::from)?;
                Ok(format!("Macro exportada para {}.", path.display()))
            }
            Action::Import => {
                let path = PathBuf::from(self.path.trim());
                if path.as_os_str().is_empty() {
                    return Err("informe o arquivo .cadmacro".into());
                }
                let definition = read_macro(&path)?;
                self.store(definition, preferences)
            }
        }
    }
}

/// Vector thumbnail fitted to the rectangle.
fn paint_thumbnail(ui: &mut Ui, definition: &MacroDefinition) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(260.0, 160.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2.0, ui.visuals().extreme_bg_color);
    let Ok(bounds) = definition.content().bounds() else {
        return;
    };
    let Some(mut camera) = Camera::new([0.0, 0.0], 1.0) else {
        return;
    };
    camera.fit(bounds, rect);
    let Ok(identity) = Transform2D::new(definition.base(), 0.0, false, false) else {
        return;
    };
    crate::editor_view::paint_macro_preview(
        &painter,
        definition,
        identity,
        camera,
        rect,
        egui::Stroke::new(1.0_f32, ui.visuals().text_color()),
    );
}
