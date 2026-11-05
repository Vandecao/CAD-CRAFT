// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 18: biblioteca de peças e navegação de dispositivos em janela própria.
use crate::{editing::GraphicEdit, preferences::Preferences, state::Session};
use cad_craft::{
    domain::*,
    interchange::{export_parts_csv, import_parts_csv},
    persistence,
};
use eframe::egui::{self, Context, Ui};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct PartEditor {
    pub open: bool,
    catalog: BTreeMap<PartId, Part>,
    seen_folder: String,
    selected_part: Option<PartId>,
    selected_device: Option<DeviceId>,
    selected_function: Option<FunctionId>,
    selected_point: Option<ConnectionPointId>,
    device_name: String,
    device_classification: String,
    device_properties: TechnicalProperties,
    device_known_key: usize,
    device_known_value: String,
    device_custom_key: String,
    device_custom_value: String,
    selected_terminal: Option<TerminalId>,
    selected_terminal_level: Option<TerminalLevelId>,
    id: String,
    manufacturer: String,
    code: String,
    description: String,
    classification: String,
    properties: TechnicalProperties,
    translations: BTreeMap<String, String>,
    known_key: usize,
    known_value: String,
    custom_key: String,
    custom_value: String,
    language: String,
    translated_description: String,
    model: Vec<PartFunction>,
    model_kind: usize,
    model_role: FunctionRole,
    model_points: String,
    quantity: u32,
    csv_path: String,
    bom_path: String,
    function_name: String,
    function_kind: usize,
    function_role: FunctionRole,
    point_name: String,
    message: String,
    locate: Option<RepresentationId>,
}

#[derive(Clone, Copy)]
enum Action {
    Save,
    ImportCsv,
    ExportCsv,
    ExportBom,
    Reload,
    UpdateProject,
    Assign,
    Remove(ComponentId),
    UpdateDevice,
    CreateFunction,
    UpdateFunction,
    RemoveFunction,
    CreatePoint,
    RenamePoint,
    RemovePoint,
    AssociateTerminal,
    Undo,
    Redo,
    Locate(RepresentationId),
}

fn kind(index: usize) -> FunctionKind {
    match index {
        0 => FunctionKind::Coil,
        1 => FunctionKind::MainContact,
        2 => FunctionKind::AuxiliaryContact,
        _ => FunctionKind::Other(Text::new("Outra").expect("constant")),
    }
}
fn kind_index(kind: &FunctionKind) -> usize {
    match kind {
        FunctionKind::Coil => 0,
        FunctionKind::MainContact => 1,
        FunctionKind::AuxiliaryContact => 2,
        FunctionKind::Other(_) => 3,
    }
}
fn kind_label(index: usize) -> &'static str {
    ["Bobina", "Contato principal", "Contato auxiliar", "Outra"][index.min(3)]
}
/// Tipo de função legível (peças de fabricante trazem tipos próprios).
fn kind_text(kind: &FunctionKind) -> String {
    match kind {
        FunctionKind::Coil => "Bobina".into(),
        FunctionKind::MainContact => "Contato principal".into(),
        FunctionKind::AuxiliaryContact => "Contato auxiliar".into(),
        FunctionKind::Other(t) => t.as_str().to_owned(),
    }
}
fn safe_stem(id: &PartId) -> String {
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
fn read_part(path: &Path) -> Result<Part, String> {
    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > 16 * 1024 * 1024 {
        return Err("arquivo .cadpart acima de 16 MiB".into());
    }
    persistence::read_part_package(&std::fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
impl PartEditor {
    pub(crate) fn install_catalog(&mut self, parts: Vec<Part>, folder: &str) {
        self.seen_folder = folder.to_owned();
        self.catalog = parts
            .into_iter()
            .map(|part| (part.id().clone(), part))
            .collect();
        if self
            .selected_part
            .as_ref()
            .is_some_and(|id| !self.catalog.contains_key(id))
        {
            self.selected_part = None;
        }
    }
    pub fn catalog(&self) -> impl Iterator<Item = &Part> {
        self.catalog.values()
    }
    pub fn take_locate(&mut self) -> Option<RepresentationId> {
        self.locate.take()
    }
    fn folder(preferences: &Preferences) -> Option<PathBuf> {
        let path = preferences.part_library.trim();
        (!path.is_empty()).then(|| PathBuf::from(path))
    }
    pub(crate) fn reload(&mut self, preferences: &Preferences) -> Result<usize, String> {
        let Some(dir) = Self::folder(preferences) else {
            return Ok(self.catalog.len());
        };
        if !dir.exists() {
            self.catalog.clear();
            return Ok(0);
        }
        let mut next = BTreeMap::new();
        for entry in std::fs::read_dir(&dir)
            .map_err(|e| e.to_string())?
            .take(50_000)
        {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case(persistence::PART_EXTENSION))
            {
                let part =
                    read_part(&path).map_err(|error| format!("{}: {error}", path.display()))?;
                next.insert(part.id().clone(), part);
            }
        }
        self.catalog = next;
        Ok(self.catalog.len())
    }
    fn select_part(&mut self, id: PartId) {
        if let Some(part) = self.catalog.get(&id) {
            self.id = part.id().as_str().into();
            self.manufacturer = part.manufacturer().as_str().into();
            self.code = part.code().as_str().into();
            self.description = part.description().as_str().into();
            self.classification = part.classification().as_str().into();
            self.properties = part.properties().clone();
            self.translations = part
                .translations()
                .iter()
                .map(|(k, v)| (k.as_str().to_owned(), v.as_str().to_owned()))
                .collect();
            self.model = part.functions().to_vec();
            self.selected_part = Some(id);
        }
    }
    fn build_part(&self) -> Result<Part, String> {
        let mut part = Part::new(
            &self.id,
            &self.manufacturer,
            &self.code,
            &self.description,
            &self.classification,
        )
        .and_then(|p| p.with_functions(self.model.clone()))
        .map_err(|e| e.to_string())?
        .with_properties(self.properties.clone());
        for (language, description) in &self.translations {
            part = part
                .with_translation(language, description)
                .map_err(|e| e.to_string())?;
        }
        Ok(part)
    }
    fn save_part(&mut self, preferences: &Preferences) -> Result<String, String> {
        let part = self.build_part()?;
        self.store_part(part, preferences)?;
        Ok("Peça salva na biblioteca.".into())
    }
    fn store_part(&mut self, part: Part, preferences: &Preferences) -> Result<(), String> {
        if let Some(dir) = Self::folder(preferences) {
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            let path = dir.join(format!(
                "{}.{}",
                safe_stem(part.id()),
                persistence::PART_EXTENSION
            ));
            let bytes = persistence::write_part_package(&part).map_err(|e| e.to_string())?;
            crate::project_file::write_atomic(&path, &mut bytes.as_slice())
                .map_err(|e| e.to_string())?;
        }
        let id = part.id().clone();
        self.catalog.insert(id.clone(), part);
        self.selected_part = Some(id);
        Ok(())
    }
    pub fn show(&mut self, session: &mut Session, preferences: &Preferences, ctx: &Context) {
        if !self.open {
            return;
        }
        if self.seen_folder != preferences.part_library {
            self.seen_folder = preferences.part_library.clone();
            if let Err(error) = self.reload(preferences) {
                self.message = error;
            }
        }
        let mut open = self.open;
        let mut action = None;
        egui::Window::new("Peças e dispositivos")
            .open(&mut open)
            .default_size([780.0, 640.0])
            .resizable(true)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if !self.message.is_empty() {
                        ui.label(&self.message);
                    }
                    self.library_ui(ui, &mut action);
                    ui.separator();
                    self.device_ui(ui, session, &mut action);
                    ui.separator();
                    self.bom_ui(ui, session, &mut action);
                    ui.horizontal(|ui| {
                        if ui.button("Desfazer edição de peça").clicked() {
                            action = Some(Action::Undo);
                        }
                        if ui.button("Refazer edição de peça").clicked() {
                            action = Some(Action::Redo);
                        }
                    });
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
    fn library_ui(&mut self, ui: &mut Ui, action: &mut Option<Action>) {
        ui.heading("Base de peças");
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("part_library_selection")
                .selected_text(
                    self.selected_part
                        .as_ref()
                        .map_or("Nova peça", PartId::as_str),
                )
                .show_ui(ui, |ui| {
                    let mut selected = None;
                    for part in self.catalog.values() {
                        if ui
                            .selectable_label(
                                self.selected_part.as_ref() == Some(part.id()),
                                format!("{} · {}", part.id().as_str(), part.description().as_str()),
                            )
                            .clicked()
                        {
                            selected = Some(part.id().clone());
                        }
                    }
                    if let Some(id) = selected {
                        self.select_part(id);
                    }
                });
            if ui.button("Nova peça").clicked() {
                self.selected_part = None;
                self.id.clear();
                self.manufacturer.clear();
                self.code.clear();
                self.description.clear();
                self.classification.clear();
                self.model.clear();
                self.properties = TechnicalProperties::default();
                self.translations.clear();
            }
            if ui.button("Recarregar").clicked() {
                *action = Some(Action::Reload);
            }
        });
        for (label, value) in [
            ("ID", &mut self.id),
            ("Fabricante", &mut self.manufacturer),
            ("Código", &mut self.code),
            ("Descrição", &mut self.description),
            ("Classificação", &mut self.classification),
        ] {
            ui.horizontal(|ui| {
                ui.label(label);
                ui.text_edit_singleline(value);
            });
        }
        ui.collapsing("Dados técnicos e traduções", |ui| {
            const KEYS: [KnownProperty; 6] = [
                KnownProperty::RatedVoltage,
                KnownProperty::RatedCurrent,
                KnownProperty::RatedPower,
                KnownProperty::Frequency,
                KnownProperty::Resistance,
                KnownProperty::CrossSection,
            ];
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("part_known_property")
                    .selected_text(format!("{:?}", KEYS[self.known_key.min(5)]))
                    .show_ui(ui, |ui| {
                        for (i, key) in KEYS.iter().enumerate() {
                            ui.selectable_value(&mut self.known_key, i, format!("{key:?}"));
                        }
                    });
                ui.add(egui::TextEdit::singleline(&mut self.known_value).hint_text("Valor SI"));
                if ui.button("Gravar dado técnico").clicked() {
                    self.message = match self
                        .known_value
                        .parse::<f64>()
                        .map_err(|e| e.to_string())
                        .and_then(|value| {
                            self.properties
                                .set_known(KEYS[self.known_key.min(5)], value)
                                .map_err(|e| e.to_string())
                        }) {
                        Ok(()) => "Dado técnico gravado no rascunho.".into(),
                        Err(e) => e,
                    };
                }
            });
            for (key, value) in self.properties.known() {
                ui.label(format!("{key:?}: {}", value.get()));
            }
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.custom_key).hint_text("Propriedade"));
                ui.add(egui::TextEdit::singleline(&mut self.custom_value).hint_text("Texto"));
                if ui.button("Gravar propriedade").clicked() {
                    self.message = match self.properties.set_custom(
                        &self.custom_key,
                        PropertyValue::Text(self.custom_value.clone()),
                    ) {
                        Ok(()) => "Propriedade gravada no rascunho.".into(),
                        Err(e) => e.to_string(),
                    };
                }
            });
            for (key, value) in self.properties.custom() {
                ui.label(format!("{}: {value:?}", key.as_str()));
            }
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.language).hint_text("Idioma, ex.: en-US"),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.translated_description)
                        .hint_text("Descrição traduzida"),
                );
                if ui.button("Adicionar tradução").clicked() {
                    self.translations
                        .insert(self.language.clone(), self.translated_description.clone());
                }
            });
            for (lang, value) in &self.translations {
                ui.label(format!("{lang}: {value}"));
            }
        });
        ui.label("Modelo de funções e pontos");
        for (index, function) in self.model.iter().enumerate() {
            ui.label(format!(
                "{}: {} ({:?}) · {}",
                index + 1,
                kind_text(function.kind()),
                function.role(),
                function
                    .points()
                    .iter()
                    .map(Text::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("part_function_kind")
                .selected_text(kind_label(self.model_kind))
                .show_ui(ui, |ui| {
                    for i in 0..4 {
                        ui.selectable_value(&mut self.model_kind, i, kind_label(i));
                    }
                });
            egui::ComboBox::from_id_salt("part_function_role")
                .selected_text(format!("{:?}", self.model_role))
                .show_ui(ui, |ui| {
                    for (role, label) in [
                        (FunctionRole::Main, "Principal"),
                        (FunctionRole::Auxiliary, "Auxiliar"),
                    ] {
                        ui.selectable_value(&mut self.model_role, role, label);
                    }
                });
            ui.add(egui::TextEdit::singleline(&mut self.model_points).hint_text("A1,A2"));
            if ui.button("Adicionar função ao modelo").clicked() {
                let labels: Vec<_> = self.model_points.split(',').map(str::trim).collect();
                match PartFunction::new(kind(self.model_kind), self.model_role, labels) {
                    Ok(value) => {
                        self.model.push(value);
                        self.model_points.clear();
                    }
                    Err(error) => self.message = error.to_string(),
                }
            }
            if ui.button("Remover última").clicked() {
                self.model.pop();
            }
        });
        if ui.button("Salvar peça").clicked() {
            *action = Some(Action::Save);
        }
        ui.horizontal(|ui| {
            ui.label("CSV");
            ui.add(
                egui::TextEdit::singleline(&mut self.csv_path)
                    .desired_width(340.0)
                    .hint_text("C:/Pecas/pecas.csv"),
            );
            if ui.button("Importar CSV").clicked() {
                *action = Some(Action::ImportCsv);
            }
            if ui.button("Exportar CSV").clicked() {
                *action = Some(Action::ExportCsv);
            }
        });
    }
    fn device_ui(&mut self, ui: &mut Ui, session: &Session, action: &mut Option<Action>) {
        ui.heading("Editor de dispositivo");
        let Some(project) = session.project() else {
            ui.label("Abra um projeto.");
            return;
        };
        for candidate in project.devices() {
            if ui
                .selectable_label(
                    self.selected_device == Some(candidate.id()),
                    format!("Dispositivo {}", candidate.designation()),
                )
                .clicked()
            {
                self.selected_device = Some(candidate.id());
                self.device_name = candidate.designation().device().as_str().into();
                self.device_classification = candidate.classification().as_str().into();
                self.device_properties = candidate.properties().clone();
            }
        }
        egui::ComboBox::from_id_salt("part_device")
            .selected_text(
                self.selected_device
                    .and_then(|id| project.device(id))
                    .map_or_else(
                        || "Escolha o dispositivo".into(),
                        |d| d.designation().to_string(),
                    ),
            )
            .show_ui(ui, |ui| {
                for device in project.devices() {
                    ui.selectable_value(
                        &mut self.selected_device,
                        Some(device.id()),
                        device.designation().to_string(),
                    );
                }
            });
        let Some(device) = self.selected_device.and_then(|id| project.device(id)) else {
            return;
        };
        ui.label(format!(
            "{} · {}",
            device.designation(),
            device.classification().as_str()
        ));
        ui.horizontal(|ui| {
            ui.label("Identificação");
            ui.text_edit_singleline(&mut self.device_name);
            ui.label("Classificação");
            ui.text_edit_singleline(&mut self.device_classification);
            if ui.button("Aplicar dispositivo").clicked() {
                *action = Some(Action::UpdateDevice);
            }
        });
        ui.collapsing("Dados técnicos do dispositivo", |ui| {
            const KEYS: [KnownProperty; 6] = [
                KnownProperty::RatedVoltage,
                KnownProperty::RatedCurrent,
                KnownProperty::RatedPower,
                KnownProperty::Frequency,
                KnownProperty::Resistance,
                KnownProperty::CrossSection,
            ];
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("device_known_property")
                    .selected_text(format!("{:?}", KEYS[self.device_known_key.min(5)]))
                    .show_ui(ui, |ui| {
                        for (index, key) in KEYS.iter().enumerate() {
                            ui.selectable_value(
                                &mut self.device_known_key,
                                index,
                                format!("{key:?}"),
                            );
                        }
                    });
                ui.add(
                    egui::TextEdit::singleline(&mut self.device_known_value).hint_text("Valor SI"),
                );
                if ui.button("Gravar dado do dispositivo").clicked() {
                    self.message = match self
                        .device_known_value
                        .parse::<f64>()
                        .map_err(|e| e.to_string())
                        .and_then(|value| {
                            self.device_properties
                                .set_known(KEYS[self.device_known_key.min(5)], value)
                                .map_err(|e| e.to_string())
                        }) {
                        Ok(()) => "Dado gravado no rascunho; clique Aplicar dispositivo.".into(),
                        Err(error) => error,
                    };
                }
            });
            for (key, value) in self.device_properties.known() {
                ui.label(format!("{key:?}: {}", value.get()));
            }
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.device_custom_key)
                        .hint_text("Propriedade"),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.device_custom_value).hint_text("Texto"),
                );
                if ui.button("Gravar propriedade do dispositivo").clicked() {
                    self.message = match self.device_properties.set_custom(
                        &self.device_custom_key,
                        PropertyValue::Text(self.device_custom_value.clone()),
                    ) {
                        Ok(()) => {
                            "Propriedade gravada no rascunho; clique Aplicar dispositivo.".into()
                        }
                        Err(error) => error.to_string(),
                    };
                }
            });
            for (key, value) in self.device_properties.custom() {
                ui.label(format!("{}: {value:?}", key.as_str()));
            }
        });
        ui.horizontal(|ui| {
            ui.label("Peça");
            egui::ComboBox::from_id_salt("part_assign_selection")
                .selected_text(
                    self.selected_part
                        .as_ref()
                        .map_or("Escolha", PartId::as_str),
                )
                .show_ui(ui, |ui| {
                    for part in self.catalog.values() {
                        ui.selectable_value(
                            &mut self.selected_part,
                            Some(part.id().clone()),
                            format!("{} · {}", part.id().as_str(), part.description().as_str()),
                        );
                    }
                });
            ui.add(
                egui::DragValue::new(&mut self.quantity)
                    .range(1..=100_000)
                    .prefix("Qtd "),
            );
            if ui.button("Atribuir peça").clicked() {
                *action = Some(Action::Assign);
            }
            if ui.button("Atualizar peças do projeto").clicked() {
                *action = Some(Action::UpdateProject);
            }
        });
        for part in self.catalog.values() {
            if ui
                .selectable_label(
                    self.selected_part.as_ref() == Some(part.id()),
                    format!("Peça {}", part.id().as_str()),
                )
                .clicked()
            {
                self.selected_part = Some(part.id().clone());
            }
        }
        for component in project
            .components()
            .filter(|c| c.device_id() == Some(device.id()))
        {
            if let Some(part) = component.part_id() {
                ui.horizontal(|ui| {
                    ui.label(format!("{} × {}", component.quantity(), part.as_str()));
                    if ui.button("Remover atribuição").clicked() {
                        *action = Some(Action::Remove(component.id()));
                    }
                });
            }
        }
        ui.separator();
        ui.heading("Funções e pontos");
        for function in project.functions_for_device(device.id()) {
            let refs = project
                .function_references(function.id())
                .unwrap_or_default();
            let status = refs.first().map_or("não representada".into(), |r| {
                format!("representada em {}", r.reference)
            });
            if ui
                .selectable_label(
                    self.selected_function == Some(function.id()),
                    format!(
                        "{} · {} · {}",
                        kind_text(function.kind()),
                        function.designation(),
                        status
                    ),
                )
                .clicked()
            {
                self.selected_function = Some(function.id());
                self.function_name = function.designation().device().as_str().into();
                self.function_kind = kind_index(function.kind());
                self.function_role = function.role();
                self.selected_point = None;
            }
            if let Some(reference) = refs.first()
                && ui
                    .button(format!("Localizar função {}", function.id()))
                    .clicked()
            {
                *action = Some(Action::Locate(reference.representation));
            }
        }
        ui.horizontal(|ui| {
            ui.label("Identificação");
            ui.text_edit_singleline(&mut self.function_name);
            egui::ComboBox::from_id_salt("function_kind")
                .selected_text(kind_label(self.function_kind))
                .show_ui(ui, |ui| {
                    for i in 0..4 {
                        ui.selectable_value(&mut self.function_kind, i, kind_label(i));
                    }
                });
            egui::ComboBox::from_id_salt("function_role")
                .selected_text(format!("{:?}", self.function_role))
                .show_ui(ui, |ui| {
                    for (role, label) in [
                        (FunctionRole::Main, "Principal"),
                        (FunctionRole::Auxiliary, "Auxiliar"),
                    ] {
                        ui.selectable_value(&mut self.function_role, role, label);
                    }
                });
        });
        ui.horizontal(|ui| {
            if ui.button("Criar função").clicked() {
                *action = Some(Action::CreateFunction);
            }
            if ui.button("Aplicar função").clicked() {
                *action = Some(Action::UpdateFunction);
            }
            if ui.button("Excluir função").clicked() {
                *action = Some(Action::RemoveFunction);
            }
        });
        if let Some(function) = self.selected_function {
            for point in project
                .connection_points()
                .filter(|p| p.function_id() == function)
            {
                if ui
                    .selectable_label(
                        self.selected_point == Some(point.id()),
                        point.designation().as_str(),
                    )
                    .clicked()
                {
                    self.selected_point = Some(point.id());
                    self.point_name = point.designation().as_str().into();
                    self.selected_terminal = point.terminal_id();
                    self.selected_terminal_level = point.terminal_level();
                }
            }
            ui.horizontal(|ui| {
                ui.label("Ponto");
                ui.text_edit_singleline(&mut self.point_name);
                if ui.button("Criar ponto").clicked() {
                    *action = Some(Action::CreatePoint);
                }
                if ui.button("Renomear ponto").clicked() {
                    *action = Some(Action::RenamePoint);
                }
                if ui.button("Excluir ponto").clicked() {
                    *action = Some(Action::RemovePoint);
                }
            });
            if let Some(point) = self.selected_point {
                let terminal_label = self
                    .selected_terminal
                    .map_or("Sem borne".to_owned(), |id| id.to_string());
                egui::ComboBox::from_id_salt("point_terminal")
                    .selected_text(terminal_label)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_terminal, None, "Sem borne");
                        for terminal in project.terminals() {
                            ui.selectable_value(
                                &mut self.selected_terminal,
                                Some(terminal.id()),
                                terminal.id().to_string(),
                            );
                        }
                    });
                if ui.button("Associar ponto ao borne").clicked() {
                    *action = Some(Action::AssociateTerminal);
                }
                if let Some(current) = project
                    .connection_point(point)
                    .and_then(ConnectionPoint::terminal_id)
                {
                    ui.small(format!("Borne atual: {current}"));
                }
                if let Some(terminal) = self.selected_terminal.and_then(|id| project.terminal(id))
                    && !terminal.levels().is_empty()
                {
                    egui::ComboBox::from_id_salt("point_terminal_level")
                        .selected_text(
                            self.selected_terminal_level
                                .and_then(|id| terminal.level(id))
                                .map_or("Escolha o nível", |level| level.designation().as_str()),
                        )
                        .show_ui(ui, |ui| {
                            for level in terminal.levels() {
                                ui.selectable_value(
                                    &mut self.selected_terminal_level,
                                    Some(level.id()),
                                    level.designation().as_str(),
                                );
                            }
                        });
                }
            }
        }
    }
    fn bom_ui(&mut self, ui: &mut Ui, session: &Session, action: &mut Option<Action>) {
        ui.heading("Lista de peças");
        if let Some(project) = session.project() {
            for row in project.bom_rows() {
                ui.label(format!(
                    "{} · {} · {} · {} × {}",
                    row.device_designation,
                    row.manufacturer,
                    row.code,
                    row.description,
                    row.quantity
                ));
            }
        }
        ui.horizontal(|ui| {
            ui.label("Arquivo CSV");
            ui.add(
                egui::TextEdit::singleline(&mut self.bom_path)
                    .desired_width(340.0)
                    .hint_text("C:/Pecas/lista.csv"),
            );
            if ui.button("Exportar lista CSV").clicked() {
                *action = Some(Action::ExportBom);
            }
        });
    }
    fn apply(
        &mut self,
        action: Action,
        session: &mut Session,
        preferences: &Preferences,
    ) -> Result<String, String> {
        let ok = |text: &str| Ok(text.to_owned());
        match action {
            Action::Save => self.save_part(preferences),
            Action::Reload => self
                .reload(preferences)
                .map(|n| format!("{n} peça(s) carregada(s).")),
            Action::ImportCsv => {
                let bytes = std::fs::read(&self.csv_path).map_err(|e| e.to_string())?;
                let report = import_parts_csv(&bytes)?;
                for part in report.parts {
                    self.store_part(part, preferences)?;
                }
                Ok(format!(
                    "{} peça(s) na biblioteca; {} linha(s) recusada(s): {}",
                    self.catalog.len(),
                    report.rejected.len(),
                    report
                        .rejected
                        .iter()
                        .take(5)
                        .map(|r| format!("{}: {}", r.line, r.reason))
                        .collect::<Vec<_>>()
                        .join("; ")
                ))
            }
            Action::ExportCsv => {
                let data = export_parts_csv(&self.catalog.values().cloned().collect::<Vec<_>>())?;
                crate::project_file::write_atomic(Path::new(&self.csv_path), &mut data.as_slice())
                    .map_err(|e| e.to_string())?;
                ok("CSV exportado.")
            }
            Action::ExportBom => {
                let project = session.project().ok_or("nenhum projeto aberto")?;
                let data = cad_craft::interchange::export_bom_csv(&project.bom_rows())?;
                crate::project_file::write_atomic(Path::new(&self.bom_path), &mut data.as_slice())
                    .map_err(|e| e.to_string())?;
                ok("Lista de peças exportada.")
            }
            Action::Assign => {
                let device = self.selected_device.ok_or("escolha o dispositivo")?;
                let part = self
                    .selected_part
                    .as_ref()
                    .and_then(|id| self.catalog.get(id))
                    .cloned()
                    .ok_or("escolha a peça")?;
                session
                    .assign_part(device, part, self.quantity.max(1))
                    .map_err(|e| e.to_string())?;
                ok("Peça atribuída ao dispositivo.")
            }
            Action::UpdateDevice => {
                let device = self.selected_device.ok_or("escolha o dispositivo")?;
                let original = session
                    .project()
                    .and_then(|project| project.device(device))
                    .ok_or("dispositivo não encontrado")?
                    .designation();
                let mut designation = Designation::new(&self.device_name)
                    .map_err(|e| e.to_string())?
                    .with_functional_path(original.functional_path().to_vec())
                    .with_location_path(original.location_path().to_vec());
                if let Some(terminal) = original.terminal() {
                    designation = designation.with_terminal(terminal.clone());
                }
                let classification =
                    Text::new(&self.device_classification).map_err(|e| e.to_string())?;
                session
                    .dispatch_graphic(GraphicEdit::Device(DeviceEdit::Update {
                        device,
                        designation,
                        classification,
                        properties: self.device_properties.clone(),
                    }))
                    .map_err(|e| e.to_string())?;
                ok("Dispositivo atualizado.")
            }
            Action::Remove(id) => {
                session
                    .remove_part_assignment(id)
                    .map_err(|e| e.to_string())?;
                ok("Atribuição removida.")
            }
            Action::UpdateProject => {
                let ids: Vec<_> = session
                    .project()
                    .ok_or("nenhum projeto aberto")?
                    .parts()
                    .map(|p| p.id().clone())
                    .collect();
                let updates: Vec<_> = ids
                    .into_iter()
                    .filter_map(|id| self.catalog.get(&id).cloned())
                    .filter(|part| session.project().and_then(|p| p.part(part.id())) != Some(part))
                    .collect();
                let count = updates.len();
                session.update_parts(updates).map_err(|e| e.to_string())?;
                Ok(format!("{count} peça(s) atualizada(s)."))
            }
            Action::CreateFunction | Action::UpdateFunction => {
                let mut designation =
                    Designation::new(&self.function_name).map_err(|e| e.to_string())?;
                if matches!(action, Action::UpdateFunction) {
                    let original = session
                        .project()
                        .and_then(|project| {
                            self.selected_function.and_then(|id| project.function(id))
                        })
                        .ok_or("função não encontrada")?
                        .designation();
                    designation = designation
                        .with_functional_path(original.functional_path().to_vec())
                        .with_location_path(original.location_path().to_vec());
                    if let Some(terminal) = original.terminal() {
                        designation = designation.with_terminal(terminal.clone());
                    }
                }
                let edit = if matches!(action, Action::CreateFunction) {
                    FunctionPointEdit::CreateFunction {
                        device: self.selected_device.ok_or("escolha o dispositivo")?,
                        designation,
                        kind: kind(self.function_kind),
                        role: self.function_role,
                    }
                } else {
                    FunctionPointEdit::UpdateFunction {
                        function: self.selected_function.ok_or("escolha a função")?,
                        designation,
                        kind: kind(self.function_kind),
                        role: self.function_role,
                    }
                };
                session
                    .edit_function_point(edit)
                    .map_err(|e| e.to_string())?;
                ok("Função atualizada.")
            }
            Action::RemoveFunction => {
                session
                    .edit_function_point(FunctionPointEdit::RemoveFunction {
                        function: self.selected_function.ok_or("escolha a função")?,
                    })
                    .map_err(|e| e.to_string())?;
                self.selected_function = None;
                ok("Função excluída.")
            }
            Action::CreatePoint | Action::RenamePoint => {
                let designation =
                    DesignationPart::new(&self.point_name).map_err(|e| e.to_string())?;
                let edit = if matches!(action, Action::CreatePoint) {
                    FunctionPointEdit::CreatePoint {
                        function: self.selected_function.ok_or("escolha a função")?,
                        designation,
                    }
                } else {
                    FunctionPointEdit::RenamePoint {
                        point: self.selected_point.ok_or("escolha o ponto")?,
                        designation,
                    }
                };
                session
                    .edit_function_point(edit)
                    .map_err(|e| e.to_string())?;
                ok("Ponto atualizado.")
            }
            Action::RemovePoint => {
                session
                    .edit_function_point(FunctionPointEdit::RemovePoint {
                        point: self.selected_point.ok_or("escolha o ponto")?,
                    })
                    .map_err(|e| e.to_string())?;
                self.selected_point = None;
                ok("Ponto excluído.")
            }
            Action::AssociateTerminal => {
                let level = self
                    .selected_terminal
                    .and_then(|id| session.project().and_then(|project| project.terminal(id)))
                    .and_then(|terminal| {
                        (!terminal.levels().is_empty())
                            .then_some(self.selected_terminal_level)
                            .flatten()
                    });
                session
                    .dispatch_physical(PhysicalEdit::SetPointTerminal {
                        point: self.selected_point.ok_or("escolha o ponto")?,
                        terminal: self.selected_terminal,
                        level,
                    })
                    .map_err(|e| e.to_string())?;
                ok("Associação ao borne atualizada.")
            }
            Action::Undo => {
                session
                    .dispatch_graphic(GraphicEdit::Undo)
                    .map_err(|e| e.to_string())?;
                ok("Desfeito.")
            }
            Action::Redo => {
                session
                    .dispatch_graphic(GraphicEdit::Redo)
                    .map_err(|e| e.to_string())?;
                ok("Refeito.")
            }
            Action::Locate(id) => {
                self.locate = Some(id);
                ok("Navegando para a ocorrência.")
            }
        }
    }
}
