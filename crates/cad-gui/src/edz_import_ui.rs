// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 26: análise EDZ em lote, prévia e confirmação na biblioteca.
use crate::app::CadApp;
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
    fn rename(&self, id: &str, name: &str) -> Result<Self, String> {
        match self {
            Self::Macro(m) => m
                .as_ref()
                .clone()
                .with_id(id)
                .and_then(|m| m.with_name(name))
                .map(|m| Self::Macro(Box::new(m)))
                .map_err(|e| e.to_string()),
            Self::Part(p) => {
                let mut next = Part::new(
                    id,
                    p.manufacturer().as_str(),
                    p.code().as_str(),
                    p.description().as_str(),
                    p.classification().as_str(),
                )
                .and_then(|p2| p2.with_functions(p.functions().to_vec()))
                .map_err(|e| e.to_string())?
                .with_properties(p.properties().clone());
                for (lang, text) in p.translations() {
                    next = next
                        .with_translation(lang.as_str(), text.as_str())
                        .map_err(|e| e.to_string())?;
                }
                Ok(Self::Part(Box::new(next)))
            }
        }
    }
    fn bytes(&self) -> Result<Vec<u8>, String> {
        match self {
            Self::Part(p) => persistence::write_part_package(p),
            Self::Macro(m) => persistence::write_macro_package(m),
        }
        .map_err(|e| e.to_string())
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
    Saved(
        Vec<EdzPackage>,
        Result<(), String>,
        Result<Catalogs, String>,
    ),
    Prepared(Vec<EdzPackage>, Result<(), String>),
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
fn read_package(path: &Path) -> Result<EdzImport, String> {
    if !is_edz(path) {
        return Err("Escolha um pacote .edz.".into());
    }
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > MAX_FILE_BYTES {
        return Err("Pacote acima de 128 MiB.".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err("Pacote acima de 128 MiB.".into());
    }
    edz::import(&bytes).map_err(|e| e.to_string())
}

/// Nenhum nome vindo do arquivo é usado diretamente como caminho.
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
fn inventory(dir: &Path, extension: &str) -> Result<Vec<Existing>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut found = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
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
        if std::fs::metadata(&path).map_err(|e| e.to_string())?.len() > max_bytes {
            return Err(format!(
                "Arquivo de biblioteca grande demais: {}",
                path.display()
            ));
        }
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        let data = if extension == persistence::PART_EXTENSION {
            Data::Part(Box::new(
                persistence::read_part_package(&bytes).map_err(|e| e.to_string())?,
            ))
        } else {
            Data::Macro(Box::new(
                persistence::read_macro_package(&bytes).map_err(|e| e.to_string())?,
            ))
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

fn load_catalogs(part_folder: &str, macro_folder: &str) -> Result<Catalogs, String> {
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

impl CadApp {
    pub(crate) fn start_edz_analysis(&mut self) {
        let mut paths = self.edz.paths.clone();
        paths.sort();
        paths.dedup();
        if paths.is_empty() || paths.len() > MAX_BATCH {
            self.edz.status = "Escolha entre 1 e 1.000 pacotes.".into();
            return;
        }
        let (sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        self.edz.total = paths.len();
        self.edz.completed = 0;
        self.edz.packages.clear();
        self.edz.prepared = false;
        self.edz.confirmed = false;
        self.edz.status = "Analisando…".into();
        self.edz.job = Some(Job { receiver, cancel });
        std::thread::spawn(move || {
            for path in paths {
                if flag.load(Ordering::Relaxed) {
                    break;
                }
                let preview = read_package(&path);
                if flag.load(Ordering::Relaxed) {
                    break;
                }
                if sender
                    .send(Message::Package(Box::new(EdzPackage {
                        path,
                        preview,
                        outcome: String::new(),
                        artifacts: Vec::new(),
                    })))
                    .is_err()
                {
                    return;
                }
            }
            let _ = sender.send(Message::Finished);
        });
    }
    pub(crate) fn poll_edz(&mut self, ctx: &Context) {
        let Some(job) = &self.edz.job else {
            return;
        };
        let mut done = false;
        loop {
            match job.receiver.try_recv() {
                Ok(Message::Package(p)) => self.edz.packages.push(*p),
                Ok(Message::Finished) => {
                    done = true;
                    break;
                }
                Ok(Message::Progress(n, total)) => {
                    self.edz.completed = n;
                    self.edz.total = total;
                }
                Ok(Message::Prepared(packages, result)) => {
                    let cancelled = job.cancel.load(Ordering::Relaxed);
                    self.edz.packages = packages;
                    self.edz.job = None;
                    self.edz.prepared = result.is_ok() && !cancelled;
                    self.edz.confirmed = cancelled;
                    self.edz.status = if cancelled {
                        "Cancelado; nenhuma biblioteca foi alterada.".into()
                    } else {
                        match result {
                            Ok(()) => {
                                format!("Análise concluída: {} pacote(s).", self.edz.packages.len())
                            }
                            Err(e) => format!("Erro: {e}"),
                        }
                    };
                    return;
                }
                Ok(Message::Saved(packages, result, catalogs)) => {
                    self.edz.packages = packages;
                    self.edz.job = None;
                    self.edz.confirmed = true;
                    self.edz.status = match result {
                        Ok(()) => "Importação concluída.".into(),
                        Err(e) => format!("Erro: {e}"),
                    };
                    match catalogs {
                        Ok((parts, macros, part_folder, macro_folder)) => {
                            if let Some(parts) = parts
                                && self.preferences.part_library == part_folder
                            {
                                self.part_editor.install_catalog(parts, &part_folder);
                            }
                            if let Some(macros) = macros
                                && self.preferences.macro_library == macro_folder
                            {
                                self.macro_library.install_catalog(macros, &macro_folder);
                            }
                        }
                        Err(e) => self.edz.status.push_str(&format!(" Biblioteca: {e}")),
                    }
                    return;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.edz.status = "Erro: análise interrompida.".into();
                    self.edz.job = None;
                    return;
                }
                Err(mpsc::TryRecvError::Empty) => break,
            }
        }
        if done {
            let cancelled = job.cancel.load(Ordering::Relaxed);
            self.edz.job = None;
            self.edz.status = if cancelled {
                "Cancelado; nenhuma biblioteca foi alterada.".into()
            } else {
                format!("Análise concluída: {} pacote(s).", self.edz.packages.len())
            };
            if cancelled {
                self.edz.confirmed = true;
            } else {
                self.start_edz_preparation(ctx);
            }
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(40));
        }
    }
    #[cfg(test)]
    fn prepare_edz(&mut self) -> Result<(), String> {
        prepare_packages(
            &mut self.edz.packages,
            &self.preferences.part_library,
            &self.preferences.macro_library,
            &AtomicBool::new(false),
        )?;
        self.edz.prepared = true;
        Ok(())
    }
    fn start_edz_preparation(&mut self, ctx: &Context) {
        let mut packages = std::mem::take(&mut self.edz.packages);
        let part_folder = self.preferences.part_library.clone();
        let macro_folder = self.preferences.macro_library.clone();
        let (sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        self.edz.status = "Verificando conflitos nas bibliotecas…".into();
        self.edz.job = Some(Job { receiver, cancel });
        std::thread::spawn(move || {
            let result = prepare_packages(&mut packages, &part_folder, &macro_folder, &flag);
            let _ = sender.send(Message::Prepared(packages, result));
        });
        ctx.request_repaint();
    }
    fn start_edz_persistence(&mut self) {
        let mut packages = std::mem::take(&mut self.edz.packages);
        let part_folder = self.preferences.part_library.clone();
        let macro_folder = self.preferences.macro_library.clone();
        let (sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        self.edz.total = packages.iter().map(|p| p.artifacts.len()).sum::<usize>() * 2;
        self.edz.completed = 0;
        self.edz.status = "Preparando e gravando bibliotecas…".into();
        self.edz.job = Some(Job { receiver, cancel });
        std::thread::spawn(move || {
            let result = save_packages(
                &mut packages,
                &part_folder,
                &macro_folder,
                &flag,
                Some(&sender),
            );
            if let Err(e) = &result {
                for package in &mut packages {
                    if package.outcome.is_empty() {
                        package.outcome =
                            format!("Erro: {e}; nenhum arquivo deste pacote foi gravado.");
                    }
                }
            }
            let catalogs = load_catalogs(&part_folder, &macro_folder);
            let _ = sender.send(Message::Saved(packages, result, catalogs));
        });
    }
    #[cfg(test)]
    fn persist_edz(&mut self) -> Result<(), String> {
        save_packages(
            &mut self.edz.packages,
            &self.preferences.part_library,
            &self.preferences.macro_library,
            &AtomicBool::new(false),
            None,
        )?;
        self.part_editor.reload(&self.preferences)?;
        self.macro_library.reload(&self.preferences)?;
        Ok(())
    }
    pub(crate) fn show_edz(&mut self, ctx: &Context) {
        if !self.edz.open {
            return;
        }
        let mut open = true;
        egui::Window::new("Importar .edz")
            .open(&mut open)
            .default_size([760., 620.])
            .show(ctx, |ui| {
                let busy = self.edz.busy();
                ui.add_enabled_ui(!busy, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Escolher arquivos .edz…").clicked() {
                            let paths = self.files.picker.open_edz();
                            if !paths.is_empty() {
                                self.edz.paths = paths;
                                self.edz.prepared = false;
                                self.edz.packages.clear();
                            }
                        }
                        if ui.button("Escolher pasta…").clicked()
                            && let Some(dir) = self
                                .files
                                .picker
                                .pick_folder("Importar pasta de pacotes .edz")
                        {
                            match std::fs::read_dir(dir) {
                                Ok(entries) => {
                                    self.edz.paths = entries
                                        .filter_map(Result::ok)
                                        .map(|e| e.path())
                                        .filter(|p| is_edz(p))
                                        .take(MAX_BATCH + 1)
                                        .collect();
                                    self.edz.prepared = false;
                                    self.edz.packages.clear();
                                }
                                Err(e) => self.edz.status = format!("Erro: {e}"),
                            }
                        }
                        if ui
                            .add_enabled(
                                !self.edz.paths.is_empty(),
                                egui::Button::new("Analisar pacotes"),
                            )
                            .clicked()
                        {
                            self.start_edz_analysis();
                        }
                    });
                });
                ui.label(format!(
                    "{} arquivo(s) selecionado(s)",
                    self.edz.paths.len()
                ));
                if busy {
                    let completed = self.edz.completed.max(self.edz.packages.len());
                    ui.add(
                        egui::ProgressBar::new(completed as f32 / self.edz.total.max(1) as f32)
                            .text(format!("{} / {}", completed, self.edz.total)),
                    );
                    if ui.button("Cancelar operação").clicked() {
                        self.edz.cancel();
                    }
                }
                ui.label(&self.edz.status);
                let unresolved = self
                    .edz
                    .packages
                    .iter()
                    .flat_map(|p| &p.artifacts)
                    .any(|a| a.conflict && a.choice.is_none());
                if ui
                    .add_enabled(
                        self.edz.prepared && !busy && !self.edz.confirmed && !unresolved,
                        egui::Button::new("Confirmar importação nas bibliotecas"),
                    )
                    .clicked()
                {
                    self.start_edz_persistence();
                }
                egui::ScrollArea::vertical()
                    .id_salt("edz_packages")
                    .show(ui, |ui| {
                        for (index, package) in self.edz.packages.iter_mut().enumerate() {
                            ui.push_id(index, |ui| {
                                ui.separator();
                                ui.strong(
                                    package
                                        .path
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy(),
                                );
                                if !package.outcome.is_empty() {
                                    ui.label(&package.outcome);
                                }
                                match &package.preview {
                                    Err(e) => {
                                        ui.label(format!("Erro: {e}"));
                                    }
                                    Ok(p) => {
                                        let symbols: usize = p
                                            .macros
                                            .iter()
                                            .map(|m| m.content().definitions().len())
                                            .sum();
                                        ui.label(format!(
                                            "{:?} · {} peça(s), {} macro(s), {} símbolo(s)",
                                            p.container,
                                            p.parts.len(),
                                            p.macros.len(),
                                            symbols
                                        ));
                                        for part in &p.parts {
                                            ui.label(format!(
                                                "Peça: {} · {}",
                                                part.code().as_str(),
                                                part.manufacturer().as_str()
                                            ));
                                        }
                                        for m in &p.macros {
                                            ui.label(format!("Macro: {}", m.name().as_str()));
                                        }
                                        for (n, a) in package
                                            .artifacts
                                            .iter_mut()
                                            .enumerate()
                                            .filter(|(_, a)| a.conflict)
                                        {
                                            ui.push_id(n, |ui| {
                                                ui.label(format!(
                                                    "Conflito: {} ({})",
                                                    a.data.name(),
                                                    a.data.id()
                                                ));
                                                ui.add_enabled_ui(!self.edz.confirmed, |ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.radio_value(
                                                            &mut a.choice,
                                                            Some(ConflictChoice::Replace),
                                                            "Substituir",
                                                        );
                                                        ui.radio_value(
                                                            &mut a.choice,
                                                            Some(ConflictChoice::KeepBoth),
                                                            "Manter as duas",
                                                        );
                                                        ui.radio_value(
                                                            &mut a.choice,
                                                            Some(ConflictChoice::Skip),
                                                            "Pular",
                                                        );
                                                    });
                                                });
                                            });
                                        }
                                        for line in p.report.lines() {
                                            ui.label(line);
                                        }
                                        if !p.unknown_codes.is_empty() {
                                            ui.label(format!(
                                                "Códigos desconhecidos: {}",
                                                p.unknown_codes
                                                    .iter()
                                                    .cloned()
                                                    .collect::<Vec<_>>()
                                                    .join(", ")
                                            ));
                                        }
                                    }
                                }
                            });
                        }
                    });
            });
        self.edz.open = open;
    }
}

fn save_packages(
    packages: &mut [EdzPackage],
    part_folder: &str,
    macro_folder: &str,
    cancel: &AtomicBool,
    sender: Option<&mpsc::Sender<Message>>,
) -> Result<(), String> {
    // Planejar e serializar TODO o lote antes de escrever qualquer arquivo.
    let mut inventories = BTreeMap::new();
    for package in packages.iter() {
        for artifact in &package.artifacts {
            let folder = artifact.data.folder(part_folder, macro_folder).trim();
            if folder.is_empty() {
                return Err(
                    "Configure as pastas das bibliotecas de peças e macros antes de confirmar."
                        .into(),
                );
            }
            if artifact.conflict && artifact.choice.is_none() {
                return Err("Resolva cada conflito: substituir, manter as duas ou pular.".into());
            }
            let key = (PathBuf::from(folder), artifact.data.extension());
            if let std::collections::btree_map::Entry::Vacant(entry) =
                inventories.entry(key.clone())
            {
                entry.insert(inventory(&key.0, key.1)?);
            }
        }
    }
    let mut writes: BTreeMap<PathBuf, (usize, Vec<u8>)> = BTreeMap::new();
    let mut counts = vec![0; packages.len()];
    let mut macro_ids = BTreeMap::new();
    let total = packages.iter().map(|p| p.artifacts.len()).sum::<usize>() * 2;
    let mut completed = 0;
    // Macros first so kept/renamed IDs propagate to imported parts.
    for macro_pass in [true, false] {
        for (index, package) in packages.iter().enumerate() {
            for artifact in &package.artifacts {
                if matches!(artifact.data, Data::Macro(_)) != macro_pass {
                    continue;
                }
                if cancel.load(Ordering::Relaxed) {
                    return Err(
                        "Cancelado antes da gravação; nenhuma biblioteca foi alterada.".into(),
                    );
                }
                if artifact.choice == Some(ConflictChoice::Skip) {
                    if macro_pass {
                        let key = (
                            PathBuf::from(artifact.data.folder(part_folder, macro_folder).trim()),
                            artifact.data.extension(),
                        );
                        if let Some(existing) = inventories
                            .get(&key)
                            .and_then(|list| list.iter().find(|e| e.id == artifact.data.id()))
                        {
                            macro_ids.insert(
                                (index, artifact.data.id().to_owned()),
                                existing.id.clone(),
                            );
                        }
                    }
                    continue;
                }
                let mut data = artifact.data.clone();
                let folder = PathBuf::from(data.folder(part_folder, macro_folder).trim());
                let ext = data.extension();
                let list = inventories
                    .get_mut(&(folder.clone(), ext))
                    .expect("inventoried");
                let candidate = folder.join(format!("{}.{}", stem(data.id()), ext));
                let matches: Vec<usize> = list
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| {
                        e.id == data.id()
                            || e.name.eq_ignore_ascii_case(data.name())
                            || same_path(&e.path, &candidate)
                    })
                    .map(|(i, _)| i)
                    .collect();
                let mut path = candidate;
                if !matches.is_empty() || path.exists() || writes.contains_key(&path) {
                    match artifact.choice {
                        Some(ConflictChoice::Replace) => {
                            if matches.len() > 1 {
                                return Err(format!(
                                    "Conflito ambíguo de ID/nome/caminho para {}; escolha manter as duas ou pular.",
                                    data.name()
                                ));
                            }
                            if let Some(&i) = matches.first() {
                                path = list[i].path.clone();
                                data = data.rename(&list[i].id, data.name())?;
                                list.remove(i);
                            }
                        }
                        Some(ConflictChoice::KeepBoth) => {
                            for n in 2..=50_001 {
                                let suffix = format!("-{n}");
                                let mut id = data.id().to_owned();
                                while id.len() + suffix.len() > 160 {
                                    id.pop();
                                }
                                id.push_str(&suffix);
                                let name = format!("{} ({n})", data.name());
                                let next = folder.join(format!("{}.{}", stem(&id), ext));
                                if !next.exists()
                                    && !writes.contains_key(&next)
                                    && !list.iter().any(|e| {
                                        e.id == id
                                            || e.name.eq_ignore_ascii_case(&name)
                                            || same_path(&e.path, &next)
                                    })
                                {
                                    data = data.rename(&id, &name)?;
                                    path = next;
                                    break;
                                }
                            }
                            if path.exists() || writes.contains_key(&path) {
                                return Err("Não foi possível gerar um nome livre.".into());
                            }
                        }
                        _ => {
                            return Err(format!(
                                "Novo conflito em {}; analise novamente.",
                                data.name()
                            ));
                        }
                    }
                }
                if macro_pass {
                    macro_ids.insert((index, artifact.data.id().to_owned()), data.id().to_owned());
                }
                if let Data::Part(part) = &mut data {
                    let mut properties = part.properties().clone();
                    let ids = properties
                        .custom()
                        .iter()
                        .find(|(k, _)| k.as_str() == "CADCRAFT.MacroIds")
                        .and_then(|(_, v)| {
                            if let PropertyValue::Text(s) = v {
                                Some(s.clone())
                            } else {
                                None
                            }
                        });
                    if let Some(ids) = ids {
                        let mapped = ids
                            .split(';')
                            .filter_map(|id| macro_ids.get(&(index, id.to_owned())).cloned())
                            .collect::<Vec<_>>()
                            .join(";");
                        properties
                            .set_custom("CADCRAFT.MacroIds", PropertyValue::Text(mapped))
                            .map_err(|e| e.to_string())?;
                        **part = part.as_ref().clone().with_properties(properties);
                    }
                }
                let bytes = data.bytes()?;
                list.push(Existing {
                    id: data.id().into(),
                    name: data.name().into(),
                    path: path.clone(),
                    data,
                });
                if let Some((previous, _)) = writes.insert(path, (index, bytes)) {
                    counts[previous] -= 1;
                }
                counts[index] += 1;
                completed += 1;
                if let Some(sender) = sender {
                    let _ = sender.send(Message::Progress(completed, total));
                }
            }
        }
    }
    let mut saved = vec![Vec::new(); packages.len()];
    let mut error = None;
    for (path, (index, bytes)) in writes {
        if cancel.load(Ordering::Relaxed) {
            error = Some("Cancelado durante a gravação".to_owned());
            break;
        }
        let result = std::fs::create_dir_all(path.parent().expect("library folder"))
            .and_then(|()| crate::project_file::write_atomic(&path, &mut bytes.as_slice()));
        if let Err(e) = result {
            error = Some(format!("{}: {e}", path.display()));
            break;
        }
        saved[index].push(path.display().to_string());
        completed += 1;
        if let Some(sender) = sender {
            let _ = sender.send(Message::Progress(completed, total));
        }
    }
    for (index, (package, count)) in packages.iter_mut().zip(counts).enumerate() {
        if let Some(e) = &error {
            package.outcome = format!(
                "Erro: {e}; {} arquivo(s) gravado(s) deste pacote: {}",
                saved[index].len(),
                saved[index].join("; ")
            );
            continue;
        }
        package.outcome = match &package.preview {
            Err(e) => format!("Erro: {e}"),
            Ok(_) if count == 0 => "Ignorado".into(),
            Ok(p)
                if p.report.count(NoteKind::Ignored) + p.report.count(NoteKind::Approximated)
                    > 0
                    || count < package.artifacts.len() =>
            {
                "Convertido parcialmente".into()
            }
            Ok(_) => "Importado".into(),
        };
    }

    if let Some(error) = error {
        Err(error)
    } else {
        Ok(())
    }
}

fn prepare_packages(
    packages: &mut [EdzPackage],
    part_folder: &str,
    macro_folder: &str,
    cancel: &AtomicBool,
) -> Result<(), String> {
    type Seen = BTreeMap<&'static str, Vec<(String, String, PathBuf)>>;
    let mut seen = Seen::new();
    for (ext, folder) in [
        (persistence::PART_EXTENSION, part_folder),
        (persistence::MACRO_EXTENSION, macro_folder),
    ] {
        if cancel.load(Ordering::Relaxed) {
            return Err("Cancelado antes da gravação.".into());
        }
        let list = if folder.trim().is_empty() {
            Vec::new()
        } else {
            inventory(Path::new(folder), ext)?
        };
        seen.insert(
            ext,
            list.into_iter().map(|e| (e.id, e.name, e.path)).collect(),
        );
    }
    for package in packages.iter_mut() {
        if cancel.load(Ordering::Relaxed) {
            return Err("Cancelado antes da gravação.".into());
        }
        if let Ok(preview) = &package.preview {
            for data in preview
                .macros
                .iter()
                .cloned()
                .map(|m| Data::Macro(Box::new(m)))
                .chain(
                    preview
                        .parts
                        .iter()
                        .cloned()
                        .map(|p| Data::Part(Box::new(p))),
                )
            {
                let list = seen.get_mut(data.extension()).expect("both types");
                let candidate = PathBuf::from(match &data {
                    Data::Part(_) => part_folder,
                    Data::Macro(_) => macro_folder,
                })
                .join(format!("{}.{}", stem(data.id()), data.extension()));
                let conflict = list.iter().any(|(id, name, path)| {
                    id == data.id()
                        || name.eq_ignore_ascii_case(data.name())
                        || same_path(path, &candidate)
                }) || candidate.exists();
                list.push((data.id().into(), data.name().into(), candidate));
                package.artifacts.push(Artifact {
                    data,
                    conflict,
                    choice: None,
                });
            }
        }
    }

    Ok(())
}
