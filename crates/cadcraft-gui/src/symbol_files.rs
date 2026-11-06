// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! External .cadsym symbol library, .elmt import and project-symbol updates; start with library loading.
use crate::app::{CadApp, Level};
use crate::commands::CommandId;
use crate::error::{GuiError, GuiResult};
use cad_craft::{domain::*, interchange::elmt, persistence};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const MAX_SYMBOL_FILE: u64 = 16 * 1024 * 1024;
const MAX_LIBRARY_FILES: usize = 50_000;
const MAX_SCAN_DEPTH: usize = 8;

fn collect(dir: &Path, ext: &str, depth: usize, out: &mut Vec<PathBuf>) {
    if depth > MAX_SCAN_DEPTH || out.len() >= MAX_LIBRARY_FILES {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect(&path, ext, depth + 1, out);
        } else if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case(ext))
        {
            out.push(path);
            if out.len() >= MAX_LIBRARY_FILES {
                return;
            }
        }
    }
}

/// Safe filename derived from the symbol name.
fn file_stem(name: &str) -> String {
    let stem: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '-' | '_' | ' ' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim()
        .trim_matches('.')
        .chars()
        .take(80)
        .collect();
    if stem.is_empty() {
        "simbolo".into()
    } else {
        stem
    }
}

pub(crate) fn read_symbol_file(path: &Path) -> GuiResult<SymbolDefinition> {
    let meta = std::fs::metadata(path).map_err(GuiError::from)?;
    if meta.len() > MAX_SYMBOL_FILE {
        return Err("arquivo de símbolo acima de 16 MiB".into());
    }
    let bytes = std::fs::read(path).map_err(GuiError::from)?;
    persistence::read_symbol_package(&bytes).map_err(GuiError::from)
}

impl CadApp {
    pub(crate) fn symbol_library_dir(&self) -> Option<PathBuf> {
        let s = self.preferences.symbol_library.trim();
        (!s.is_empty()).then(|| PathBuf::from(s))
    }

    /// Read the library directory, keeping the highest version for duplicate symbols; return the number loaded.
    pub fn reload_symbol_library(&mut self) -> usize {
        let mut by_id: BTreeMap<SymbolDefinitionId, SymbolDefinition> = BTreeMap::new();
        let mut errors = Vec::new();
        if let Some(dir) = self.symbol_library_dir() {
            if dir.is_dir() {
                let mut files = Vec::new();
                collect(&dir, persistence::SYMBOL_EXTENSION, 0, &mut files);
                for path in files {
                    match read_symbol_file(&path) {
                        Ok(d) => {
                            let keep = by_id
                                .get(d.id())
                                .is_none_or(|old| old.version() < d.version());
                            if keep {
                                by_id.insert(d.id().clone(), d);
                            }
                        }
                        Err(e) => errors.push(format!("{}: {e}", path.display())),
                    }
                }
            } else {
                errors.push(format!(
                    "pasta da biblioteca inexistente: {}",
                    dir.display()
                ));
            }
        }
        // Symbols held only in memory remain available without a directory.
        if self.symbol_library_dir().is_none() {
            for d in std::mem::take(&mut self.session.symbol_library) {
                by_id.entry(d.id().clone()).or_insert(d);
            }
        }
        let count = by_id.len();
        self.session.symbol_library = by_id.into_values().collect();
        self.session.symbol_library_revision = self.session.symbol_library_revision.wrapping_add(1);
        for e in errors.iter().take(5) {
            self.log(Level::Warning, format!("Biblioteca de símbolos: {e}"));
        }
        if errors.len() > 5 {
            self.log(
                Level::Warning,
                format!("Biblioteca de símbolos: mais {} erro(s).", errors.len() - 5),
            );
        }
        count
    }

    /// Save to the library directory, or retain in memory without one, then reload.
    pub(crate) fn add_to_symbol_library(
        &mut self,
        definition: SymbolDefinition,
    ) -> GuiResult<Option<PathBuf>> {
        let Some(dir) = self.symbol_library_dir() else {
            self.session
                .symbol_library
                .retain(|d| d.id() != definition.id());
            self.session.symbol_library.push(definition);
            self.session.symbol_library_revision =
                self.session.symbol_library_revision.wrapping_add(1);
            return Ok(None);
        };
        std::fs::create_dir_all(&dir).map_err(GuiError::from)?;
        let bytes = persistence::write_symbol_package(&definition).map_err(GuiError::from)?;
        let stem = file_stem(definition.name());
        let mut path = dir.join(format!("{stem}.{}", persistence::SYMBOL_EXTENSION));
        // Append the ID when another symbol has the same filename.
        if path.exists() && read_symbol_file(&path).is_ok_and(|d| d.id() != definition.id()) {
            path = dir.join(format!(
                "{stem}-{}.{}",
                file_stem(definition.id().as_str()),
                persistence::SYMBOL_EXTENSION
            ));
        }
        crate::project_file::write_atomic(&path, &mut bytes.as_slice()).map_err(GuiError::from)?;
        Ok(Some(path))
    }

    /// Current export symbol: open editor, selected occurrence or insertion-centre selection.
    pub(crate) fn current_symbol(&self) -> Option<SymbolDefinition> {
        if let Some(d) = self.symbol_editor.definition() {
            return Some(d);
        }
        if let Some(id) = self
            .session
            .graphics
            .selection
            .primary()
            .and_then(|r| self.session.project()?.representation(r)?.symbol().cloned())
        {
            return self.session.find_symbol(&id);
        }
        self.selected_library_symbol()
            .and_then(|id| self.session.find_symbol(&id))
    }

    pub(crate) fn symbol_command(&mut self, id: CommandId) {
        use CommandId as C;
        match id {
            C::SymbolNew => self.open_symbol_editor_new(),
            C::SymbolEditor => match self.current_symbol() {
                Some(d) => self.open_symbol_editor(d),
                None => self.log(
                    Level::Warning,
                    "Selecione um símbolo na página ou no Centro de inserção.",
                ),
            },
            C::SymbolExport => {
                let Some(d) = self.current_symbol() else {
                    self.log(Level::Warning, "Nenhum símbolo para exportar.");
                    return;
                };
                let dir = self.symbol_library_dir();
                let Some(path) = self
                    .files
                    .picker
                    .save_symbol(&file_stem(d.name()), dir.as_deref())
                else {
                    return;
                };
                let path = if path.extension().is_none() {
                    path.with_extension(persistence::SYMBOL_EXTENSION)
                } else {
                    path
                };
                let result = persistence::write_symbol_package(&d)
                    .map_err(GuiError::from)
                    .and_then(|bytes| {
                        crate::project_file::write_atomic(&path, &mut bytes.as_slice())
                            .map_err(GuiError::from)
                    });
                match result {
                    Ok(()) => {
                        self.log(
                            Level::Info,
                            format!("Símbolo exportado em {}.", path.display()),
                        );
                        if dir.is_some_and(|d| path.starts_with(d)) {
                            self.reload_symbol_library();
                        }
                    }
                    Err(e) => self.log(Level::Error, format!("Exportação recusada: {e}")),
                }
            }
            C::SymbolImport => {
                let dir = self.symbol_library_dir();
                let Some(path) = self.files.picker.open_symbol(dir.as_deref()) else {
                    return;
                };
                match read_symbol_file(&path).and_then(|d| {
                    let name = d.name().to_owned();
                    self.add_to_symbol_library(d).map(|p| (name, p))
                }) {
                    Ok((name, _)) => {
                        self.reload_symbol_library();
                        self.log(
                            Level::Info,
                            format!("Símbolo \u{201c}{name}\u{201d} na biblioteca."),
                        );
                    }
                    Err(e) => self.log(Level::Error, format!("Importação recusada: {e}")),
                }
            }
            C::SymbolImportElmt => {
                let files = self.files.picker.open_elmt();
                if !files.is_empty() {
                    self.import_elmt_files(files);
                }
            }
            C::SymbolImportElmtFolder => {
                if let Some(dir) = self
                    .files
                    .picker
                    .pick_folder("Pasta com elementos do QElectroTech")
                {
                    let mut files = Vec::new();
                    collect(&dir, "elmt", 0, &mut files);
                    if files.is_empty() {
                        self.log(Level::Warning, "Nenhum arquivo .elmt na pasta.");
                    } else {
                        self.import_elmt_files(files);
                    }
                }
            }
            C::SymbolReload => {
                let n = self.reload_symbol_library();
                self.log(
                    Level::Info,
                    format!("Biblioteca de símbolos: {n} símbolo(s)."),
                );
            }
            C::SymbolUpdateProject => self.update_project_symbols(),
            _ => {}
        }
    }

    /// Convert .elmt files into the library; default scale is 0.5 mm per pixel.
    pub(crate) fn import_elmt_files(&mut self, files: Vec<PathBuf>) {
        let (mut ok, mut failed, mut notes) = (0usize, Vec::new(), 0usize);
        for path in &files {
            let stem = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "elemento".into());
            let mut options = elmt::ElmtOptions::new(stem);
            options.mm_per_px = self.preferences.elmt_mm_per_px;
            if let Some(category) = path
                .parent()
                .and_then(|p| p.file_name())
                .map(|s| s.to_string_lossy().into_owned())
            {
                options.category = category;
            }
            let result = std::fs::metadata(path)
                .map_err(GuiError::from)
                .and_then(|m| {
                    if m.len() > MAX_SYMBOL_FILE {
                        Err("arquivo acima de 16 MiB".into())
                    } else {
                        std::fs::read(path).map_err(GuiError::from)
                    }
                })
                .and_then(|bytes| elmt::import(&bytes, &options).map_err(GuiError::from));
            match result.and_then(|s| {
                notes += s.report.notes.iter().map(|(_, n)| n).sum::<usize>();
                self.add_to_symbol_library(s.definition)
            }) {
                Ok(_) => ok += 1,
                Err(e) => failed.push(format!("{}: {e}", path.display())),
            }
        }
        self.reload_symbol_library();
        self.log(
            if failed.is_empty() {
                Level::Info
            } else {
                Level::Warning
            },
            format!(
                "Importação .elmt: {ok} símbolo(s) na biblioteca, {} recusado(s), {notes} aviso(s) de conversão.",
                failed.len()
            ),
        );
        for e in failed.iter().take(5) {
            self.log(Level::Warning, e.clone());
        }
    }

    /// Replace project definitions with the latest library versions having compatible ports in one undo step; list rejected updates.
    pub(crate) fn update_project_symbols(&mut self) {
        let Some(project) = self.session.project() else {
            return;
        };
        let mut compatible = Vec::new();
        let mut refused = Vec::new();
        for current in project.symbol_definitions() {
            let Some(newer) = self
                .session
                .symbol_library
                .iter()
                .find(|d| d.id() == current.id() && d.version() > current.version())
            else {
                continue;
            };
            let problems = project.symbol_update_problems(newer);
            if problems.is_empty() {
                compatible.push(newer.clone());
            } else {
                refused.push(format!(
                    "{} v{}: {}",
                    newer.name(),
                    newer.version(),
                    problems.join("; ")
                ));
            }
        }
        let names: Vec<String> = compatible
            .iter()
            .map(|d| format!("{} v{}", d.name(), d.version()))
            .collect();
        if !compatible.is_empty() {
            match self.session.update_symbols(compatible) {
                Ok(_) => self.log(
                    Level::Info,
                    format!("Símbolos atualizados: {}.", names.join(", ")),
                ),
                Err(e) => self.log(Level::Error, format!("Atualização recusada: {e}")),
            }
        }
        for r in &refused {
            self.log(Level::Warning, format!("Não atualizado: {r}"));
        }
        if names.is_empty() && refused.is_empty() {
            self.log(
                Level::Info,
                "Os símbolos do projeto já estão na versão da biblioteca.",
            );
        }
    }
}
