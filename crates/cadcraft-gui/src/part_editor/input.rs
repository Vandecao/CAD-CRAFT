// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! input support for the part editor; start at the parent module.

use super::*;

pub(in crate::part_editor) fn safe_stem(id: &PartId) -> String {
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
pub(in crate::part_editor) fn read_part(path: &Path) -> GuiResult<Part> {
    if std::fs::metadata(path).map_err(GuiError::from)?.len() > 16 * 1024 * 1024 {
        return Err("arquivo .cadpart acima de 16 MiB".into());
    }
    persistence::read_part_package(&std::fs::read(path).map_err(GuiError::from)?)
        .map_err(GuiError::from)
}
impl PartEditor {
    pub(in crate::part_editor) fn folder(preferences: &Preferences) -> Option<PathBuf> {
        let path = preferences.part_library.trim();
        (!path.is_empty()).then(|| PathBuf::from(path))
    }
    pub(crate) fn reload(&mut self, preferences: &Preferences) -> GuiResult<usize> {
        let Some(dir) = Self::folder(preferences) else {
            return Ok(self.catalog.len());
        };
        if !dir.exists() {
            self.catalog.clear();
            return Ok(0);
        }
        let mut next = BTreeMap::new();
        for entry in std::fs::read_dir(&dir)
            .map_err(GuiError::from)?
            .take(50_000)
        {
            let path = entry.map_err(GuiError::from)?.path();
            if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case(persistence::PART_EXTENSION))
            {
                let part = read_part(&path)
                    .map_err(|error| GuiError::context(path.display().to_string(), error))?;
                next.insert(part.id().clone(), part);
            }
        }
        self.catalog = next;
        Ok(self.catalog.len())
    }
    pub(in crate::part_editor) fn select_part(&mut self, id: PartId) {
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
    pub(in crate::part_editor) fn build_part(&self) -> Result<Part, String> {
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
    pub(in crate::part_editor) fn save_part(
        &mut self,
        preferences: &Preferences,
    ) -> GuiResult<String> {
        let part = self.build_part()?;
        self.store_part(part, preferences)?;
        Ok("Peça salva na biblioteca.".into())
    }
    pub(in crate::part_editor) fn store_part(
        &mut self,
        part: Part,
        preferences: &Preferences,
    ) -> GuiResult<()> {
        if let Some(dir) = Self::folder(preferences) {
            std::fs::create_dir_all(&dir).map_err(GuiError::from)?;
            let path = dir.join(format!(
                "{}.{}",
                safe_stem(part.id()),
                persistence::PART_EXTENSION
            ));
            let bytes = persistence::write_part_package(&part).map_err(GuiError::from)?;
            crate::project_file::write_atomic(&path, &mut bytes.as_slice())
                .map_err(GuiError::from)?;
        }
        let id = part.id().clone();
        self.catalog.insert(id.clone(), part);
        self.selected_part = Some(id);
        Ok(())
    }
}
impl PartEditor {
    pub(in crate::part_editor) fn apply(
        &mut self,
        action: Action,
        session: &mut Session,
        preferences: &Preferences,
    ) -> GuiResult<String> {
        let ok = |text: &str| Ok(text.to_owned());
        match action {
            Action::Save => self.save_part(preferences),
            Action::Reload => self
                .reload(preferences)
                .map(|n| format!("{n} peça(s) carregada(s).")),
            Action::ImportCsv => {
                let bytes = std::fs::read(&self.csv_path).map_err(GuiError::from)?;
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
                    .map_err(GuiError::from)?;
                ok("CSV exportado.")
            }
            Action::ExportBom => {
                let project = session.project().ok_or("nenhum projeto aberto")?;
                let data = cad_craft::interchange::export_bom_csv(&project.bom_rows())?;
                crate::project_file::write_atomic(Path::new(&self.bom_path), &mut data.as_slice())
                    .map_err(GuiError::from)?;
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
                    .map_err(GuiError::from)?;
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
                    .map_err(GuiError::from)?
                    .with_functional_path(original.functional_path().to_vec())
                    .with_location_path(original.location_path().to_vec());
                if let Some(terminal) = original.terminal() {
                    designation = designation.with_terminal(terminal.clone());
                }
                let classification =
                    Text::new(&self.device_classification).map_err(GuiError::from)?;
                session
                    .dispatch_graphic(GraphicEdit::Device(DeviceEdit::Update {
                        device,
                        designation,
                        classification,
                        properties: self.device_properties.clone(),
                    }))
                    .map_err(GuiError::from)?;
                ok("Dispositivo atualizado.")
            }
            Action::Remove(id) => {
                session.remove_part_assignment(id).map_err(GuiError::from)?;
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
                session.update_parts(updates).map_err(GuiError::from)?;
                Ok(format!("{count} peça(s) atualizada(s)."))
            }
            Action::CreateFunction | Action::UpdateFunction => {
                let mut designation =
                    Designation::new(&self.function_name).map_err(GuiError::from)?;
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
                session.edit_function_point(edit).map_err(GuiError::from)?;
                ok("Função atualizada.")
            }
            Action::RemoveFunction => {
                session
                    .edit_function_point(FunctionPointEdit::RemoveFunction {
                        function: self.selected_function.ok_or("escolha a função")?,
                    })
                    .map_err(GuiError::from)?;
                self.selected_function = None;
                ok("Função excluída.")
            }
            Action::CreatePoint | Action::RenamePoint => {
                let designation = DesignationPart::new(&self.point_name).map_err(GuiError::from)?;
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
                session.edit_function_point(edit).map_err(GuiError::from)?;
                ok("Ponto atualizado.")
            }
            Action::RemovePoint => {
                session
                    .edit_function_point(FunctionPointEdit::RemovePoint {
                        point: self.selected_point.ok_or("escolha o ponto")?,
                    })
                    .map_err(GuiError::from)?;
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
                    .map_err(GuiError::from)?;
                ok("Associação ao borne atualizada.")
            }
            Action::Undo => {
                session
                    .dispatch_graphic(GraphicEdit::Undo)
                    .map_err(GuiError::from)?;
                ok("Desfeito.")
            }
            Action::Redo => {
                session
                    .dispatch_graphic(GraphicEdit::Redo)
                    .map_err(GuiError::from)?;
                ok("Refeito.")
            }
            Action::Locate(id) => {
                self.locate = Some(id);
                ok("Navegando para a ocorrência.")
            }
        }
    }
}
