// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! draw support for the part editor; start at the parent module.

use super::*;

impl PartEditor {
    pub fn show(&mut self, session: &mut Session, preferences: &Preferences, ctx: &Context) {
        if !self.open {
            return;
        }
        if self.seen_folder != preferences.part_library {
            self.seen_folder = preferences.part_library.clone();
            if let Err(error) = self.reload(preferences) {
                self.message = error.to_string();
            }
        }
        let mut open = self.open;
        let mut action = None;
        egui::Window::new(crate::strings::PARTS_AND_DEVICES)
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
    pub(in crate::part_editor) fn library_ui(&mut self, ui: &mut Ui, action: &mut Option<Action>) {
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
    pub(in crate::part_editor) fn device_ui(
        &mut self,
        ui: &mut Ui,
        session: &Session,
        action: &mut Option<Action>,
    ) {
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
    pub(in crate::part_editor) fn bom_ui(
        &mut self,
        ui: &mut Ui,
        session: &Session,
        action: &mut Option<Action>,
    ) {
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
}
