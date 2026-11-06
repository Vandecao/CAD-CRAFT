// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Batch EDZ analysis, preview and library confirmation; start with the workflow module.
use crate::{
    app::CadApp,
    error::{GuiError, GuiResult},
};
use cad_craft::{
    domain::{MacroDefinition, Part, PropertyValue},
    interchange::{
        NoteKind,
        edz::{self, EdzImport},
    },
    persistence,
};
use eframe::egui::{self, Context};
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};

const MAX_FILE_BYTES: u64 = 128 * 1024 * 1024;
const MAX_BATCH: usize = 1000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConflictChoice {
    Replace,
    KeepBoth,
    Skip,
}

pub struct EdzPackage {
    pub path: PathBuf,
    pub preview: Result<EdzImport, String>,
    pub outcome: String,
    artifacts: Vec<Artifact>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use cad_craft::{
        domain::*,
        interchange::{ImportReport, edz::EdzContainer},
    };
    fn part(id: &str, name: &str) -> Part {
        Part::new(id, "Own", name, "Synthetic", "Relay").unwrap()
    }
    fn app(dir: &Path, parts: Vec<Part>) -> CadApp {
        let mut app = CadApp::default();
        app.preferences.part_library = dir.join("parts").display().to_string();
        app.preferences.macro_library = dir.join("macros").display().to_string();
        app.edz.packages.push(EdzPackage {
            path: "own.edz".into(),
            preview: Ok(EdzImport {
                container: EdzContainer::Zip,
                parts,
                macros: Vec::new(),
                report: ImportReport::default(),
                unknown_codes: Default::default(),
            }),
            outcome: String::new(),
            artifacts: Vec::new(),
        });
        app.prepare_edz().unwrap();
        app
    }
    #[test]
    fn serialization_failure_does_not_write_earlier_valid_part() {
        let dir = tempfile::tempdir().unwrap();
        let mut properties = TechnicalProperties::default();
        properties
            .set_custom(
                "invalid-library-reference",
                PropertyValue::Reference(EntityRef::Device(DeviceId::new())),
            )
            .unwrap();
        let invalid = part("bad", "Bad").with_properties(properties);
        let mut app = app(dir.path(), vec![part("good", "Good"), invalid]);
        assert!(app.persist_edz().is_err());
        assert!(!dir.path().join("parts").exists());
    }
    #[test]
    fn cancelling_before_serialization_never_writes() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = app(dir.path(), vec![part("own", "Own")]);
        let result = save_packages(
            &mut app.edz.packages,
            &app.preferences.part_library,
            &app.preferences.macro_library,
            &AtomicBool::new(true),
            None,
        );
        assert!(result.unwrap_err().contains("Cancelado"));
        assert!(!dir.path().join("parts").exists());
    }
    #[test]
    fn sanitized_and_case_collisions_require_choice_and_keep_both_is_unique() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = app(dir.path(), vec![part("A:B", "One"), part("A_B", "Two")]);
        assert!(!app.edz.packages[0].artifacts[0].conflict);
        assert!(app.edz.packages[0].artifacts[1].conflict);
        assert!(app.persist_edz().is_err());
        app.edz.packages[0].artifacts[1].choice = Some(ConflictChoice::KeepBoth);
        app.persist_edz().unwrap();
        assert_eq!(app.part_editor.catalog().count(), 2);
        let kept = app
            .part_editor
            .catalog()
            .find(|p| p.id().as_str() == "A_B-2")
            .expect("segunda identidade da biblioteca");
        assert_eq!(
            kept.code().as_str(),
            "Two",
            "manter as duas preserva o código de fabricante"
        );
        assert_eq!(kept.manufacturer().as_str(), "Own");
        assert_eq!(kept.description().as_str(), "Synthetic");
        assert_eq!(
            std::fs::read_dir(dir.path().join("parts")).unwrap().count(),
            2
        );
    }
    #[test]
    fn replacement_uses_actual_existing_path_and_identity() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("parts");
        std::fs::create_dir(&folder).unwrap();
        let path = folder.join("arbitrary-user-name.cadpart");
        let bytes = persistence::write_part_package(&part("existing", "Same")).unwrap();
        std::fs::write(&path, bytes).unwrap();
        let mut app = app(dir.path(), vec![part("incoming", "Same")]);
        assert!(app.edz.packages[0].artifacts[0].conflict);
        app.edz.packages[0].artifacts[0].choice = Some(ConflictChoice::Replace);
        app.persist_edz().unwrap();
        assert_eq!(std::fs::read_dir(folder).unwrap().count(), 1);
        assert_eq!(
            app.part_editor.catalog().next().unwrap().id().as_str(),
            "existing"
        );
        assert!(path.exists());
    }
}

#[derive(Clone)]
enum Data {
    Part(Box<Part>),
    Macro(Box<MacroDefinition>),
}
impl Data {
    fn id(&self) -> &str {
        match self {
            Self::Part(p) => p.id().as_str(),
            Self::Macro(m) => m.id().as_str(),
        }
    }
    fn name(&self) -> &str {
        match self {
            Self::Part(p) => p.code().as_str(),
            Self::Macro(m) => m.name().as_str(),
        }
    }
    fn extension(&self) -> &'static str {
        match self {
            Self::Part(_) => persistence::PART_EXTENSION,
            Self::Macro(_) => persistence::MACRO_EXTENSION,
        }
    }
    fn folder<'a>(&self, part_folder: &'a str, macro_folder: &'a str) -> &'a str {
        match self {
            Self::Part(_) => part_folder,
            Self::Macro(_) => macro_folder,
        }
    }
    fn rename(&self, id: &str, name: &str) -> GuiResult<Self> {
        match self {
            Self::Macro(m) => m
                .as_ref()
                .clone()
                .with_id(id)
                .and_then(|m| m.with_name(name))
                .map(|m| Self::Macro(Box::new(m)))
                .map_err(GuiError::from),
            Self::Part(p) => {
                let mut next = Part::new(
                    id,
                    p.manufacturer().as_str(),
                    p.code().as_str(),
                    p.description().as_str(),
                    p.classification().as_str(),
                )
                .and_then(|p2| p2.with_functions(p.functions().to_vec()))?
                .with_properties(p.properties().clone());
                for (lang, text) in p.translations() {
                    next = next.with_translation(lang.as_str(), text.as_str())?;
                }
                Ok(Self::Part(Box::new(next)))
            }
        }
    }
    fn bytes(&self) -> GuiResult<Vec<u8>> {
        match self {
            Self::Part(p) => persistence::write_part_package(p),
            Self::Macro(m) => persistence::write_macro_package(m),
        }
        .map_err(GuiError::from)
    }
}

struct Artifact {
    data: Data,
    conflict: bool,
    choice: Option<ConflictChoice>,
}
type Catalogs = (
    Option<Vec<Part>>,
    Option<Vec<MacroDefinition>>,
    String,
    String,
);
enum Message {
    Package(Box<EdzPackage>),
    Finished,
    Progress(usize, usize),
    Saved(Vec<EdzPackage>, GuiResult<()>, GuiResult<Catalogs>),
    Prepared(Vec<EdzPackage>, GuiResult<()>),
}
struct Job {
    receiver: mpsc::Receiver<Message>,
    cancel: Arc<AtomicBool>,
}

#[derive(Default)]
pub struct EdzImportPanel {
    pub open: bool,
    pub paths: Vec<PathBuf>,
    pub packages: Vec<EdzPackage>,
    pub status: String,
    total: usize,
    completed: usize,
    job: Option<Job>,
    prepared: bool,
    confirmed: bool,
}
impl EdzImportPanel {
    pub fn busy(&self) -> bool {
        self.job.is_some()
    }
    pub fn cancel(&mut self) {
        if let Some(job) = &self.job {
            job.cancel.store(true, Ordering::Relaxed);
        }
    }
}

fn is_edz(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("edz"))
}
fn read_package(path: &Path) -> GuiResult<EdzImport> {
    if !is_edz(path) {
        return Err("Escolha um pacote .edz.".into());
    }
    let file = std::fs::File::open(path)?;
    if file.metadata()?.len() > MAX_FILE_BYTES {
        return Err("Pacote acima de 128 MiB.".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err("Pacote acima de 128 MiB.".into());
    }
    edz::import(&bytes).map_err(GuiError::from)
}

/// Never use a name from the imported file directly as a path.
fn stem(id: &str) -> String {
    let s: String = id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if s.is_empty() { "edz".into() } else { s }
}

struct Existing {
    id: String,
    name: String,
    path: PathBuf,
    data: Data,
}
fn same_path(a: &Path, b: &Path) -> bool {
    if cfg!(windows) {
        a.to_string_lossy()
            .eq_ignore_ascii_case(&b.to_string_lossy())
    } else {
        a == b
    }
}
fn inventory(dir: &Path, extension: &str) -> GuiResult<Vec<Existing>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut found = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if !path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case(extension))
        {
            continue;
        }
        if found.len() >= 50_000 {
            return Err("Biblioteca acima do limite de 50.000 arquivos.".into());
        }
        let max_bytes = if extension == persistence::MACRO_EXTENSION {
            persistence::MAX_MACRO_FILE_BYTES
        } else {
            16 * 1024 * 1024
        };
        if std::fs::metadata(&path)?.len() > max_bytes {
            return Err(format!("Arquivo de biblioteca grande demais: {}", path.display()).into());
        }
        let bytes = std::fs::read(&path)?;
        let data = if extension == persistence::PART_EXTENSION {
            Data::Part(Box::new(persistence::read_part_package(&bytes)?))
        } else {
            Data::Macro(Box::new(persistence::read_macro_package(&bytes)?))
        };
        found.push(Existing {
            id: data.id().into(),
            name: data.name().into(),
            path,
            data,
        });
    }
    Ok(found)
}

fn load_catalogs(part_folder: &str, macro_folder: &str) -> GuiResult<Catalogs> {
    let parts = if part_folder.trim().is_empty() {
        None
    } else {
        Some(
            inventory(Path::new(part_folder), persistence::PART_EXTENSION)?
                .into_iter()
                .filter_map(|e| match e.data {
                    Data::Part(p) => Some(*p),
                    _ => None,
                })
                .collect(),
        )
    };
    let macros = if macro_folder.trim().is_empty() {
        None
    } else {
        Some(
            inventory(Path::new(macro_folder), persistence::MACRO_EXTENSION)?
                .into_iter()
                .filter_map(|e| match e.data {
                    Data::Macro(m) => Some(*m),
                    _ => None,
                })
                .collect(),
        )
    };
    Ok((
        parts,
        macros,
        part_folder.to_owned(),
        macro_folder.to_owned(),
    ))
}

mod batch;
mod workflow;

use batch::{prepare_packages, save_packages};
