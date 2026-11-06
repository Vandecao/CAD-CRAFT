// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Filesystem services for .cadcraft projects; start with load and save. Core owns the byte format. This module provides atomic replacement using a temporary file beside the destination, cooperative .lock files, crash recovery, recent files and native rfd pickers replaceable by scripts in tests.
use crate::error::{GuiError, GuiResult};
use cad_craft::{
    domain::Project,
    persistence::{self, LoadedProject},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const MAX_RECENT: usize = 10;

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Ensure user-selected paths have the .cadcraft extension.
pub fn with_extension(path: PathBuf) -> PathBuf {
    if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(persistence::EXTENSION))
    {
        path
    } else {
        let mut name = path.into_os_string();
        name.push(".");
        name.push(persistence::EXTENSION);
        PathBuf::from(name)
    }
}

pub fn load(path: &Path) -> GuiResult<LoadedProject> {
    let file = fs::File::open(path).map_err(|e| GuiError::at_path(path, e))?;
    persistence::read_project(io::BufReader::new(file))
        .map_err(|e| GuiError::context(path.display().to_string(), e))
}

/// Serialize and validate in memory before touching the destination; replace the previous file only after syncing the complete temporary file.
pub fn save(project: &Project, path: &Path) -> GuiResult<()> {
    let bytes = persistence::write_project_to_vec(project).map_err(GuiError::from)?;
    write_atomic(path, &mut bytes.as_slice()).map_err(|e| GuiError::at_path(path, e))
}

pub fn write_atomic(path: &Path, source: &mut impl Read) -> io::Result<()> {
    crate::snapshots::write_from(path, source, true)
}

// File locking.

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockInfo {
    pub user: String,
    pub host: String,
    pub pid: u32,
    pub opened_at: u64,
    /// Distinguish sessions within one process, including tests and future windows.
    #[serde(default)]
    pub token: String,
}
impl LockInfo {
    fn current() -> Self {
        let var = |names: &[&str]| {
            names
                .iter()
                .find_map(|n| std::env::var(n).ok())
                .unwrap_or_else(|| "desconhecido".into())
        };
        Self {
            user: var(&["USERNAME", "USER"]),
            host: var(&["COMPUTERNAME", "HOSTNAME"]),
            pid: std::process::id(),
            opened_at: now(),
            token: cad_craft::domain::ProjectId::new().to_string(),
        }
    }
    pub fn describe(&self) -> String {
        format!("{} em {} (processo {})", self.user, self.host, self.pid)
    }
}

pub fn lock_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".lock");
    PathBuf::from(name)
}

#[derive(Debug)]
pub enum LockError {
    /// Another session holds the file; readable owner description.
    Held(String),
    Io(io::Error),
}
impl std::fmt::Display for LockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Held(owner) => write!(f, "arquivo em uso por {owner}"),
            Self::Io(e) => write!(f, "trava indisponível: {e}"),
        }
    }
}

/// Cooperative lock created with create_new. Remove it on drop only if it still belongs to this session and has not been taken over.
#[derive(Debug)]
pub struct FileLock {
    path: PathBuf,
    info: LockInfo,
}
impl FileLock {
    pub fn acquire(project: &Path) -> Result<Self, LockError> {
        let path = lock_path(project);
        let info = LockInfo::current();
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(file) => {
                serde_json::to_writer(&file, &info).map_err(|e| LockError::Io(e.into()))?;
                file.sync_all().map_err(LockError::Io)?;
                Ok(Self { path, info })
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                let owner = fs::read_to_string(&path)
                    .ok()
                    .and_then(|t| serde_json::from_str::<LockInfo>(&t).ok())
                    .map(|i| i.describe())
                    .unwrap_or_else(|| "outra sessão".into());
                Err(LockError::Held(owner))
            }
            Err(e) => Err(LockError::Io(e)),
        }
    }
    /// Explicit user decision to replace an existing lock.
    pub fn take_over(project: &Path) -> io::Result<Self> {
        let path = lock_path(project);
        let info = LockInfo::current();
        let json = serde_json::to_vec(&info).map_err(io::Error::other)?;
        write_atomic(&path, &mut json.as_slice())?;
        Ok(Self { path, info })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
}
impl Drop for FileLock {
    fn drop(&mut self) {
        let ours = fs::read_to_string(&self.path)
            .ok()
            .and_then(|t| serde_json::from_str::<LockInfo>(&t).ok())
            .is_some_and(|i| i == self.info);
        if ours {
            let _ = fs::remove_file(&self.path);
        }
    }
}

/// Session-associated file. Read-only files are unlocked or written by a newer format; saving requires Save As.
#[derive(Debug)]
pub struct ProjectFile {
    pub path: PathBuf,
    pub read_only: bool,
    pub source_format: u32,
    lock: Option<FileLock>,
}
impl ProjectFile {
    pub fn new(path: PathBuf, read_only: bool, source_format: u32, lock: Option<FileLock>) -> Self {
        Self {
            path,
            read_only,
            source_format,
            lock,
        }
    }
    pub fn locked(&self) -> bool {
        self.lock.is_some()
    }
    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.path.display().to_string())
    }
    pub(crate) fn take_lock(&mut self) -> Option<FileLock> {
        self.lock.take()
    }
}

// Recovery.

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryEntry {
    pub project_id: String,
    pub name: String,
    pub origin: Option<PathBuf>,
    pub saved_at: u64,
}

/// Automatic recovery copies of unsaved projects. Normal exit or saving removes the entry; after abrupt termination it remains and is offered at next startup.
#[derive(Clone, Debug)]
pub struct Recovery {
    dir: PathBuf,
}
impl Recovery {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }
    pub fn default_dir() -> PathBuf {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("CAD-CRAFT")
            .join("recovery")
    }
    pub fn dir(&self) -> &Path {
        &self.dir
    }
    fn files(&self, project_id: &str) -> (PathBuf, PathBuf) {
        (
            self.dir
                .join(format!("{project_id}.{}", persistence::EXTENSION)),
            self.dir.join(format!("{project_id}.json")),
        )
    }
    pub fn store(&self, project: &Project, origin: Option<&Path>) -> GuiResult<()> {
        fs::create_dir_all(&self.dir).map_err(GuiError::from)?;
        let id = project.id().to_string();
        let (data, meta) = self.files(&id);
        save(project, &data)?;
        let entry = RecoveryEntry {
            project_id: id,
            name: project.name().as_str().to_owned(),
            origin: origin.map(Path::to_path_buf),
            saved_at: now(),
        };
        let json = serde_json::to_vec_pretty(&entry).map_err(GuiError::from)?;
        write_atomic(&meta, &mut json.as_slice()).map_err(GuiError::from)
    }
    /// Complete entries, newest first; ignore descriptions without data.
    pub fn entries(&self) -> Vec<RecoveryEntry> {
        let Ok(dir) = fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut entries: Vec<RecoveryEntry> = dir
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
            .filter_map(|e| fs::read_to_string(e.path()).ok())
            .filter_map(|t| serde_json::from_str::<RecoveryEntry>(&t).ok())
            .filter(|e| self.files(&e.project_id).0.is_file())
            .collect();
        entries.sort_by(|a, b| b.saved_at.cmp(&a.saved_at).then(a.name.cmp(&b.name)));
        entries
    }
    pub fn load(&self, entry: &RecoveryEntry) -> GuiResult<Project> {
        load(&self.files(&entry.project_id).0).map(|l| l.project)
    }
    pub fn discard(&self, project_id: &str) {
        let (data, meta) = self.files(project_id);
        let _ = fs::remove_file(meta);
        let _ = fs::remove_file(data);
    }
}

// Recent files.

pub fn push_recent(list: &mut Vec<PathBuf>, path: &Path) {
    list.retain(|p| p != path);
    list.insert(0, path.to_path_buf());
    list.truncate(MAX_RECENT);
}

// File picker.

pub trait FilePicker {
    fn open_project(&mut self) -> Option<PathBuf>;
    fn save_project(&mut self, suggested_name: &str) -> Option<PathBuf>;
    /// PNG/JPEG logo for a frame.
    fn pick_image(&mut self) -> Option<PathBuf>;
    /// .cadframe file, starting in the library directory if configured.
    fn open_frame(&mut self, library: Option<&Path>) -> Option<PathBuf>;
    fn save_frame(&mut self, suggested_name: &str, library: Option<&Path>) -> Option<PathBuf>;
    fn open_external_frame(&mut self, library: Option<&Path>) -> Option<PathBuf>;
    fn save_dxf(&mut self, suggested_name: &str, library: Option<&Path>) -> Option<PathBuf>;
    /// .cadsym symbol, starting in the library directory.
    fn open_symbol(&mut self, library: Option<&Path>) -> Option<PathBuf>;
    fn save_symbol(&mut self, suggested_name: &str, library: Option<&Path>) -> Option<PathBuf>;
    /// One or more QElectroTech .elmt elements.
    fn open_elmt(&mut self) -> Vec<PathBuf>;
    /// QElectroTech .qet project.
    fn open_qet(&mut self) -> Option<PathBuf>;
    /// Manufacturer packages; the default preserves existing pickers.
    fn open_edz(&mut self) -> Vec<PathBuf> {
        Vec::new()
    }
    /// Directory for batch .elmt import.
    fn pick_folder(&mut self, title: &str) -> Option<PathBuf>;
    /// Exported PDF destination.
    fn save_pdf(&mut self, suggested_name: &str) -> Option<PathBuf>;
    fn save_list(&mut self, suggested_name: &str, xlsx: bool) -> Option<PathBuf>;
}
fn in_library(dialog: rfd::FileDialog, library: Option<&Path>) -> rfd::FileDialog {
    match library.filter(|p| p.is_dir()) {
        Some(dir) => dialog.set_directory(dir),
        None => dialog,
    }
}

/// Native operating-system dialogs.
pub struct NativePicker;
impl FilePicker for NativePicker {
    fn open_edz(&mut self) -> Vec<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Importar pacotes de fabricantes")
            .add_filter("Pacote EPLAN", &["edz"])
            .pick_files()
            .unwrap_or_default()
    }
    fn open_project(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Abrir projeto CAD-CRAFT")
            .add_filter("Projeto CAD-CRAFT", &[persistence::EXTENSION])
            .add_filter("Snapshot técnico JSON (formato v0)", &["json"])
            .pick_file()
    }
    fn save_project(&mut self, suggested_name: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Salvar projeto CAD-CRAFT")
            .add_filter("Projeto CAD-CRAFT", &[persistence::EXTENSION])
            .set_file_name(format!("{suggested_name}.{}", persistence::EXTENSION))
            .save_file()
    }
    fn pick_image(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Inserir imagem na moldura")
            .add_filter("Imagem PNG ou JPEG", &["png", "jpg", "jpeg"])
            .pick_file()
    }
    fn open_frame(&mut self, library: Option<&Path>) -> Option<PathBuf> {
        in_library(rfd::FileDialog::new(), library)
            .set_title("Importar moldura CAD-CRAFT")
            .add_filter("Moldura CAD-CRAFT", &[persistence::FRAME_EXTENSION])
            .pick_file()
    }
    fn save_frame(&mut self, suggested_name: &str, library: Option<&Path>) -> Option<PathBuf> {
        in_library(rfd::FileDialog::new(), library)
            .set_title("Exportar moldura CAD-CRAFT")
            .add_filter("Moldura CAD-CRAFT", &[persistence::FRAME_EXTENSION])
            .set_file_name(format!("{suggested_name}.{}", persistence::FRAME_EXTENSION))
            .save_file()
    }
    fn open_external_frame(&mut self, library: Option<&Path>) -> Option<PathBuf> {
        in_library(rfd::FileDialog::new(), library)
            .set_title("Importar moldura de outro sistema")
            .add_filter(
                "Molduras EPLAN, QElectroTech e DXF",
                &["fn1", "titleblock", "dxf"],
            )
            .pick_file()
    }
    fn save_dxf(&mut self, suggested_name: &str, library: Option<&Path>) -> Option<PathBuf> {
        in_library(rfd::FileDialog::new(), library)
            .set_title("Exportar moldura DXF")
            .add_filter("Desenho DXF", &["dxf"])
            .set_file_name(format!("{suggested_name}.dxf"))
            .save_file()
    }
    fn open_symbol(&mut self, library: Option<&Path>) -> Option<PathBuf> {
        in_library(rfd::FileDialog::new(), library)
            .set_title(crate::strings::IMPORT_SYMBOL)
            .add_filter("Símbolo CAD-CRAFT", &[persistence::SYMBOL_EXTENSION])
            .pick_file()
    }
    fn save_symbol(&mut self, suggested_name: &str, library: Option<&Path>) -> Option<PathBuf> {
        in_library(rfd::FileDialog::new(), library)
            .set_title(crate::strings::EXPORT_SYMBOL)
            .add_filter("Símbolo CAD-CRAFT", &[persistence::SYMBOL_EXTENSION])
            .set_file_name(format!(
                "{suggested_name}.{}",
                persistence::SYMBOL_EXTENSION
            ))
            .save_file()
    }
    fn open_elmt(&mut self) -> Vec<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Importar elementos do QElectroTech")
            .add_filter("Elemento QElectroTech", &["elmt"])
            .pick_files()
            .unwrap_or_default()
    }
    fn open_qet(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Importar projeto do QElectroTech")
            .add_filter("Projeto QElectroTech", &["qet"])
            .pick_file()
    }
    fn pick_folder(&mut self, title: &str) -> Option<PathBuf> {
        rfd::FileDialog::new().set_title(title).pick_folder()
    }
    fn save_pdf(&mut self, suggested_name: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title(crate::strings::EXPORT_PDF)
            .add_filter("PDF", &["pdf"])
            .set_file_name(format!("{suggested_name}.pdf"))
            .save_file()
    }
    fn save_list(&mut self, suggested_name: &str, xlsx: bool) -> Option<PathBuf> {
        let extension = if xlsx { "xlsx" } else { "csv" };
        rfd::FileDialog::new()
            .set_title(crate::strings::EXPORT_LIST)
            .add_filter("Lista", &[extension])
            .set_file_name(format!("{suggested_name}.{extension}"))
            .save_file()
    }
}

/// Predefined answers consumed in order, for tests and automation.
#[derive(Default)]
pub struct ScriptedPicker {
    pub open: VecDeque<PathBuf>,
    pub save: VecDeque<PathBuf>,
    pub images: VecDeque<PathBuf>,
    pub frames_open: VecDeque<PathBuf>,
    pub frames_save: VecDeque<PathBuf>,
    pub external_open: VecDeque<PathBuf>,
    pub dxf_save: VecDeque<PathBuf>,
    pub symbols_open: VecDeque<PathBuf>,
    pub symbols_save: VecDeque<PathBuf>,
    pub elmt_open: VecDeque<Vec<PathBuf>>,
    pub qet_open: VecDeque<PathBuf>,
    pub edz_open: VecDeque<Vec<PathBuf>>,
    pub folders: VecDeque<PathBuf>,
    pub pdf_save: VecDeque<PathBuf>,
    pub list_save: VecDeque<PathBuf>,
}
impl FilePicker for ScriptedPicker {
    fn open_edz(&mut self) -> Vec<PathBuf> {
        self.edz_open.pop_front().unwrap_or_default()
    }
    fn open_project(&mut self) -> Option<PathBuf> {
        self.open.pop_front()
    }
    fn save_project(&mut self, _: &str) -> Option<PathBuf> {
        self.save.pop_front()
    }
    fn pick_image(&mut self) -> Option<PathBuf> {
        self.images.pop_front()
    }
    fn open_frame(&mut self, _: Option<&Path>) -> Option<PathBuf> {
        self.frames_open.pop_front()
    }
    fn save_frame(&mut self, _: &str, _: Option<&Path>) -> Option<PathBuf> {
        self.frames_save.pop_front()
    }
    fn open_external_frame(&mut self, _: Option<&Path>) -> Option<PathBuf> {
        self.external_open.pop_front()
    }
    fn save_dxf(&mut self, _: &str, _: Option<&Path>) -> Option<PathBuf> {
        self.dxf_save.pop_front()
    }
    fn open_symbol(&mut self, _: Option<&Path>) -> Option<PathBuf> {
        self.symbols_open.pop_front()
    }
    fn save_symbol(&mut self, _: &str, _: Option<&Path>) -> Option<PathBuf> {
        self.symbols_save.pop_front()
    }
    fn open_elmt(&mut self) -> Vec<PathBuf> {
        self.elmt_open.pop_front().unwrap_or_default()
    }
    fn open_qet(&mut self) -> Option<PathBuf> {
        self.qet_open.pop_front()
    }
    fn pick_folder(&mut self, _: &str) -> Option<PathBuf> {
        self.folders.pop_front()
    }
    fn save_pdf(&mut self, _: &str) -> Option<PathBuf> {
        self.pdf_save.pop_front()
    }
    fn save_list(&mut self, _: &str, _: bool) -> Option<PathBuf> {
        self.list_save.pop_front()
    }
}

/// Application file services. Recovery is disabled by default for tests; CadApp::new enables the user's directory in the desktop application.
pub struct FileServices {
    pub picker: Box<dyn FilePicker>,
    pub recovery: Option<Recovery>,
    pub recent: Vec<PathBuf>,
    pub autosave_interval: std::time::Duration,
    pub last_autosave: Option<std::time::Instant>,
}
impl Default for FileServices {
    fn default() -> Self {
        Self {
            picker: Box::new(NativePicker),
            recovery: None,
            recent: Vec::new(),
            autosave_interval: std::time::Duration::from_secs(120),
            last_autosave: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Broken;
    impl Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("falha injetada"))
        }
    }

    #[test]
    fn extension_is_added_only_when_missing() {
        assert_eq!(with_extension("a/b".into()), PathBuf::from("a/b.cadcraft"));
        assert_eq!(
            with_extension("a/b.CADCRAFT".into()),
            PathBuf::from("a/b.CADCRAFT")
        );
        assert_eq!(
            with_extension("a/b.json".into()),
            PathBuf::from("a/b.json.cadcraft")
        );
    }

    #[test]
    fn failed_save_preserves_previous_project_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("projeto.cadcraft");
        let project = crate::demo::project().unwrap();
        save(&project, &path).unwrap();
        let before = fs::read(&path).unwrap();
        assert!(write_atomic(&path, &mut Broken).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
        assert_eq!(load(&path).unwrap().project, project);
    }

    #[test]
    fn lock_is_exclusive_released_on_drop_and_can_be_taken_over() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p.cadcraft");
        let first = FileLock::acquire(&path).unwrap();
        assert!(lock_path(&path).is_file());
        match FileLock::acquire(&path) {
            Err(LockError::Held(owner)) => assert!(owner.contains("processo")),
            other => panic!("{other:?}"),
        }
        let second = FileLock::take_over(&path).unwrap();
        drop(first); // does not remove the lock taken over by the second session
        assert!(lock_path(&path).is_file());
        drop(second);
        assert!(!lock_path(&path).exists());
        FileLock::acquire(&path).unwrap();
    }

    #[test]
    fn recovery_entries_round_trip_and_discard() {
        let dir = tempfile::tempdir().unwrap();
        let recovery = Recovery::new(dir.path().join("rec"));
        assert!(recovery.entries().is_empty());
        let project = crate::demo::project().unwrap();
        recovery
            .store(&project, Some(Path::new("C:/x/p.cadcraft")))
            .unwrap();
        let entries = recovery.entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].project_id, project.id().to_string());
        assert_eq!(
            entries[0].origin.as_deref(),
            Some(Path::new("C:/x/p.cadcraft"))
        );
        assert_eq!(recovery.load(&entries[0]).unwrap(), project);
        recovery.discard(&entries[0].project_id);
        assert!(recovery.entries().is_empty());
        assert_eq!(fs::read_dir(recovery.dir()).unwrap().count(), 0);
    }

    #[test]
    fn recent_list_is_deduplicated_and_bounded() {
        let mut list = Vec::new();
        for i in 0..12 {
            push_recent(&mut list, Path::new(&format!("p{i}.cadcraft")));
        }
        push_recent(&mut list, Path::new("p5.cadcraft"));
        assert_eq!(list.len(), MAX_RECENT);
        assert_eq!(list[0], PathBuf::from("p5.cadcraft"));
        assert_eq!(
            list.iter()
                .filter(|p| p.as_path() == Path::new("p5.cadcraft"))
                .count(),
            1
        );
    }
}
