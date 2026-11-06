// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Native project and reusable library packages; start with read_project,
//! write_project and migrate, then the read_*_package / write_*_package sections.
//!
//! The current project format is a versioned ZIP with mimetype, manifest.json,
//! project.json and optional image bytes under assets/. Legacy technical JSON is
//! read as format 0 and migrated. All decoded projects are validated. This crate
//! uses bounded readers and writers and never extracts archive entries to disk;
//! the application owns file locks, atomic saves and recovery.
#![warn(missing_docs)]

use cadcraft_model::domain::{
    AssetId, FramePackage, FrameTemplate, ImageAsset, MAX_IMAGE_BYTES, MacroDefinition, Part,
    Project, SymbolDefinition,
};
use cadcraft_model::error::DomainError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fmt;
use std::io::{self, Read, Seek, SeekFrom, Write};
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

/// Format version written by this release.
pub const FORMAT: u32 = 9;
/// Oldest reader format that can read files written by this release.
pub const MIN_READER: u32 = 9;
/// Newest format version accepted by this reader.
pub const SUPPORTED_FORMAT: u32 = FORMAT;
/// Content type of the native project ZIP container.
pub const MIMETYPE: &str = "application/x-cad-craft-project+zip";
/// Native project filename extension without a leading dot.
pub const EXTENSION: &str = "cadcraft";
/// Maximum archive entries, checked before reading project content.
pub const MAX_ENTRIES: usize = 4096;
/// Maximum aggregate declared uncompressed archive size.
pub const MAX_UNPACKED_BYTES: u64 = 1 << 30;
/// Maximum size of a legacy JSON snapshot (format 0).
pub const MAX_LEGACY_BYTES: u64 = 256 * 1024 * 1024;

const MANIFEST: &str = "manifest.json";
const PROJECT: &str = "project.json";
const MIMETYPE_ENTRY: &str = "mimetype";
const MAX_MANIFEST_BYTES: u64 = 64 * 1024;

/// Unknown manifest fields are tolerated so the manifest can grow without raising
/// `min_reader`. Project content remains strict.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// Version of the content stored in this container.
    pub format: u32,
    /// Minimum reader version required to interpret the content.
    pub min_reader: u32,
    /// Application name and version that wrote the container.
    pub writer: String,
}
impl Manifest {
    /// Manifest for projects written by this application release.
    pub fn current() -> Self {
        Self {
            format: FORMAT,
            min_reader: MIN_READER,
            writer: format!("CAD-CRAFT {}", env!("CARGO_PKG_VERSION")),
        }
    }
}

/// Validated project and source metadata returned by a successful read.
#[derive(Debug)]
pub struct LoadedProject {
    /// Decoded project with model invariants validated.
    pub project: Project,
    /// Zero for a legacy JSON snapshot; otherwise the manifest's `format`.
    pub source_format: u32,
    /// True for files written in a newer format: overwriting them could
    /// discard data unknown to this reader.
    pub read_only: bool,
    /// Source manifest; absent for legacy JSON snapshots.
    pub manifest: Option<Manifest>,
}
impl LoadedProject {
    /// Whether older content was migrated during reading.
    pub fn migrated(&self) -> bool {
        self.source_format < FORMAT
    }
    /// Whether this is a legacy JSON snapshot requiring Save As to become `.cadcraft`.
    /// Older `.cadcraft` files remain associated with their paths.
    pub fn is_legacy_snapshot(&self) -> bool {
        self.source_format == 0
    }
}

/// Failure reading, validating, migrating or writing native packages.
#[derive(Debug)]
pub enum PersistenceError {
    /// Underlying reader or writer failed.
    Io(io::Error),
    /// Input does not have a recognized native package or snapshot signature.
    NotAProject,
    /// ZIP container is malformed or package content is incomplete.
    Container(String),
    /// Archive entry has an unsafe path, duplicate name or unsupported encoding.
    UnsafeEntry(String),
    /// A container or content resource limit was exceeded.
    TooLarge(&'static str),
    /// A required package entry is missing.
    MissingEntry(&'static str),
    /// Package content type does not match the requested reader.
    WrongMimetype,
    /// Manifest JSON or version relationships are invalid.
    Manifest(String),
    /// Package requires a newer reader than this application provides.
    UnsupportedFormat {
        /// Minimum reader version required by the file.
        min_reader: u32,
        /// Highest format version supported by this reader.
        supported: u32,
    },
    /// Content JSON could not be decoded or migrated.
    Json(String),
    /// Decoded or supplied content violates model invariants.
    Domain(DomainError),
}
impl fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "falha de leitura/gravação: {e}"),
            Self::NotAProject => f.write_str("o arquivo não é um projeto CAD-CRAFT"),
            Self::Container(e) => write!(f, "contêiner .cadcraft inválido: {e}"),
            Self::UnsafeEntry(e) => write!(f, "entrada insegura recusada: {e}"),
            Self::TooLarge(what) => write!(f, "limite de segurança excedido: {what}"),
            Self::MissingEntry(name) => write!(f, "entrada obrigatória ausente: {name}"),
            Self::WrongMimetype => f.write_str("tipo de conteúdo .cadcraft não reconhecido"),
            Self::Manifest(e) => write!(f, "manifesto inválido: {e}"),
            Self::UnsupportedFormat {
                min_reader,
                supported,
            } => write!(
                f,
                "arquivo exige leitor do formato {min_reader}; esta versão lê até o formato {supported}"
            ),
            Self::Json(e) => write!(f, "conteúdo do projeto inválido: {e}"),
            Self::Domain(e) => write!(f, "projeto inconsistente: {e}"),
        }
    }
}
impl std::error::Error for PersistenceError {}
impl From<io::Error> for PersistenceError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<zip::result::ZipError> for PersistenceError {
    fn from(e: zip::result::ZipError) -> Self {
        match e {
            zip::result::ZipError::Io(e) => Self::Io(e),
            other => Self::Container(other.to_string()),
        }
    }
}
impl From<DomainError> for PersistenceError {
    fn from(e: DomainError) -> Self {
        Self::Domain(e)
    }
}
/// Result of reading or writing a native project or library package.
pub type Result<T> = std::result::Result<T, PersistenceError>;

/// Writes the current format, validating the project before writing any bytes.
pub fn write_project<W: Write + Seek>(project: &Project, writer: W) -> Result<W> {
    project.validate()?;
    let manifest = serde_json::to_vec_pretty(&Manifest::current())
        .map_err(|e| PersistenceError::Json(e.to_string()))?;
    let content = serde_json::to_vec(project).map_err(|e| PersistenceError::Json(e.to_string()))?;
    // Images with content are stored under `assets/`; metadata-only images
    // from legacy snapshots cannot be saved.
    let assets: Vec<&ImageAsset> = project.assets().collect();
    if assets.iter().any(|a| a.bytes().is_none()) {
        return Err(PersistenceError::Container(
            "imagem sem conteúdo; reinsira a imagem antes de salvar".into(),
        ));
    }
    write_zip(writer, MIMETYPE, &manifest, PROJECT, &content, &assets)
}

fn asset_entry(asset: &ImageAsset) -> String {
    format!("assets/{}.{}", asset.id(), asset.format().extension())
}

fn write_zip<W: Write + Seek>(
    writer: W,
    mimetype: &str,
    manifest: &[u8],
    content_name: &str,
    content: &[u8],
    assets: &[&ImageAsset],
) -> Result<W> {
    let mut zip = ZipWriter::new(writer);
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let deflated = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .large_file(content.len() as u64 >= u32::MAX as u64);
    zip.start_file(MIMETYPE_ENTRY, stored)?;
    zip.write_all(mimetype.as_bytes())?;
    zip.start_file(MANIFEST, deflated)?;
    zip.write_all(manifest)?;
    zip.start_file(content_name, deflated)?;
    zip.write_all(content)?;
    for asset in assets {
        if let Some(bytes) = asset.bytes() {
            // PNG and JPEG content is already compressed.
            zip.start_file(asset_entry(asset), stored)?;
            zip.write_all(bytes)?;
        }
    }
    Ok(zip.finish()?)
}

/// Serializes a project to an in-memory archive.
pub fn write_project_to_vec(project: &Project) -> Result<Vec<u8>> {
    Ok(write_project(project, io::Cursor::new(Vec::new()))?.into_inner())
}

/// Reads a ZIP project or legacy JSON snapshot without extracting files to disk.
pub fn read_project<R: Read + Seek>(mut reader: R) -> Result<LoadedProject> {
    let mut prefix = [0u8; 256];
    let read = read_prefix(&mut reader, &mut prefix)?;
    reader.seek(SeekFrom::Start(0))?;
    if read >= 4 && prefix[..4] == *b"PK\x03\x04" {
        return read_container(reader);
    }
    let text = prefix[..read]
        .strip_prefix(&[0xEF, 0xBB, 0xBF])
        .unwrap_or(&prefix[..read]);
    let first = text.iter().copied().find(|b| !b.is_ascii_whitespace());
    if first == Some(b'{') {
        return read_legacy(reader);
    }
    Err(PersistenceError::NotAProject)
}

/// Reads a project from memory with the same checks as [`read_project`].
pub fn read_project_from_slice(bytes: &[u8]) -> Result<LoadedProject> {
    read_project(io::Cursor::new(bytes))
}

fn read_prefix(reader: &mut impl Read, buffer: &mut [u8]) -> io::Result<usize> {
    let mut total = 0;
    while total < buffer.len() {
        match reader.read(&mut buffer[total..])? {
            0 => break,
            n => total += n,
        }
    }
    Ok(total)
}

fn read_bounded(reader: impl Read, limit: u64, what: &'static str) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(PersistenceError::TooLarge(what));
    }
    Ok(bytes)
}

fn read_legacy(reader: impl Read) -> Result<LoadedProject> {
    let bytes = read_bounded(reader, MAX_LEGACY_BYTES, "snapshot JSON acima de 256 MiB")?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|e| PersistenceError::Json(e.to_string()))?
        .trim_start_matches('\u{feff}');
    let value: Value =
        serde_json::from_str(text).map_err(|e| PersistenceError::Json(e.to_string()))?;
    let project = decode_project(migrate(value, 0)?)?;
    Ok(LoadedProject {
        project,
        source_format: 0,
        read_only: false,
        manifest: None,
    })
}

/// Accepts relative entry names without special path components.
fn check_entry_name(name: &str) -> Result<()> {
    let unsafe_name = name.is_empty()
        || name.starts_with('/')
        || name.contains('\\')
        || name.contains(':')
        || name.contains('\0')
        || name.split('/').any(|part| part == ".." || part == ".");
    if unsafe_name {
        return Err(PersistenceError::UnsafeEntry(name.to_owned()));
    }
    Ok(())
}

fn read_container<R: Read + Seek>(reader: R) -> Result<LoadedProject> {
    let mut archive = open_checked(reader)?;
    let manifest = read_manifest(&mut archive, MIMETYPE, SUPPORTED_FORMAT)?;
    let content = read_entry(&mut archive, PROJECT, MAX_UNPACKED_BYTES)?;
    let value: Value =
        serde_json::from_slice(&content).map_err(|e| PersistenceError::Json(e.to_string()))?;
    let mut project = decode_project(migrate(value, manifest.format.min(SUPPORTED_FORMAT))?)?;
    let assets: Vec<(AssetId, String)> =
        project.assets().map(|a| (a.id(), asset_entry(a))).collect();
    for (id, name) in assets {
        let bytes = read_entry(&mut archive, &name, MAX_IMAGE_BYTES as u64)?;
        project.attach_asset_bytes(id, bytes)?;
    }
    Ok(LoadedProject {
        project,
        source_format: manifest.format,
        read_only: manifest.format > SUPPORTED_FORMAT,
        manifest: Some(manifest),
    })
}

/// Opens a ZIP archive after validating its entries and resource limits.
fn open_checked<R: Read + Seek>(reader: R) -> Result<ZipArchive<R>> {
    let mut archive = ZipArchive::new(reader)?;
    if archive.has_duplicate_names() {
        return Err(PersistenceError::UnsafeEntry(
            "nome de entrada repetido".into(),
        ));
    }
    if archive.len() > MAX_ENTRIES {
        return Err(PersistenceError::TooLarge("mais de 4.096 entradas"));
    }
    let mut names = BTreeSet::new();
    let mut total: u64 = 0;
    for index in 0..archive.len() {
        let entry = archive.by_index_raw(index)?;
        let name = entry.name()?.into_owned();
        check_entry_name(&name)?;
        // Names differing only in case would collide on Windows.
        if !names.insert(name.to_lowercase()) {
            return Err(PersistenceError::UnsafeEntry(name));
        }
        if entry.encrypted() {
            return Err(PersistenceError::UnsafeEntry(format!(
                "{name} (criptografada)"
            )));
        }
        if !matches!(
            entry.compression(),
            CompressionMethod::Stored | CompressionMethod::Deflated
        ) {
            return Err(PersistenceError::UnsafeEntry(format!(
                "{name} (compressão não suportada)"
            )));
        }
        total = total.saturating_add(entry.size());
        if total > MAX_UNPACKED_BYTES {
            return Err(PersistenceError::TooLarge(
                "conteúdo descompactado acima de 1 GiB",
            ));
        }
    }
    archive.validate_header_consistency()?;
    Ok(archive)
}

fn read_manifest<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    expected_mimetype: &str,
    supported: u32,
) -> Result<Manifest> {
    let mimetype = read_entry(archive, MIMETYPE_ENTRY, 256)?;
    if mimetype != expected_mimetype.as_bytes() {
        return Err(PersistenceError::WrongMimetype);
    }
    let manifest: Manifest =
        serde_json::from_slice(&read_entry(archive, MANIFEST, MAX_MANIFEST_BYTES)?)
            .map_err(|e| PersistenceError::Manifest(e.to_string()))?;
    if manifest.format == 0 || manifest.min_reader == 0 || manifest.min_reader > manifest.format {
        return Err(PersistenceError::Manifest(
            "format/min_reader incoerentes".into(),
        ));
    }
    if manifest.min_reader > supported {
        return Err(PersistenceError::UnsupportedFormat {
            min_reader: manifest.min_reader,
            supported,
        });
    }
    Ok(manifest)
}

fn read_entry<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    name: &str,
    limit: u64,
) -> Result<Vec<u8>> {
    let missing = || match name {
        MIMETYPE_ENTRY => PersistenceError::MissingEntry(MIMETYPE_ENTRY),
        MANIFEST => PersistenceError::MissingEntry(MANIFEST),
        PROJECT => PersistenceError::MissingEntry(PROJECT),
        FRAME => PersistenceError::MissingEntry(FRAME),
        other => PersistenceError::Container(format!("entrada obrigatória ausente: {other}")),
    };
    let index = archive.index_for_name(name).ok_or_else(missing)?;
    let entry = archive.by_index(index)?;
    if entry.size() > limit {
        return Err(PersistenceError::TooLarge("entrada acima do limite"));
    }
    // Declared sizes may be false; also bound the bytes actually read.
    read_bounded(entry, limit, "entrada acima do limite")
}

// ------------------------------------------------------------ .cadframe

/// Content type of an exportable frame package (`.cadframe`).
pub const FRAME_MIMETYPE: &str = "application/x-cad-craft-frame+zip";
/// Frame package filename extension without a leading dot.
pub const FRAME_EXTENSION: &str = "cadframe";
/// Current frame package format version.
pub const FRAME_FORMAT: u32 = 1;
const FRAME: &str = "frame.json";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FrameFile {
    frame: FrameTemplate,
    assets: Vec<ImageAsset>,
}

/// Serializes a validated frame and its image bytes into a frame package.
pub fn write_frame_package(package: &FramePackage) -> Result<Vec<u8>> {
    package.frame.validate()?;
    if package.assets.iter().any(|a| a.bytes().is_none()) {
        return Err(PersistenceError::Container("imagem sem conteúdo".into()));
    }
    let manifest = serde_json::to_vec_pretty(&Manifest {
        format: FRAME_FORMAT,
        min_reader: FRAME_FORMAT,
        writer: format!("CAD-CRAFT {}", env!("CARGO_PKG_VERSION")),
    })
    .map_err(|e| PersistenceError::Json(e.to_string()))?;
    let file = FrameFile {
        frame: package.frame.clone(),
        assets: package.assets.clone(),
    };
    let content = serde_json::to_vec(&file).map_err(|e| PersistenceError::Json(e.to_string()))?;
    let assets: Vec<&ImageAsset> = package.assets.iter().collect();
    Ok(write_zip(
        io::Cursor::new(Vec::new()),
        FRAME_MIMETYPE,
        &manifest,
        FRAME,
        &content,
        &assets,
    )?
    .into_inner())
}

/// Reads `.cadframe` with the same safeguards as `.cadcraft`; images must
/// be present and match their metadata.
pub fn read_frame_package(bytes: &[u8]) -> Result<FramePackage> {
    if !bytes.starts_with(b"PK\x03\x04") {
        return Err(PersistenceError::NotAProject);
    }
    let mut archive = open_checked(io::Cursor::new(bytes))?;
    read_manifest(&mut archive, FRAME_MIMETYPE, FRAME_FORMAT)?;
    let content = read_entry(&mut archive, FRAME, MAX_UNPACKED_BYTES)?;
    let file: FrameFile =
        serde_json::from_slice(&content).map_err(|e| PersistenceError::Json(e.to_string()))?;
    let mut assets = Vec::with_capacity(file.assets.len());
    for mut asset in file.assets {
        let bytes = read_entry(&mut archive, &asset_entry(&asset), MAX_IMAGE_BYTES as u64)?;
        asset.attach(bytes)?;
        assets.push(asset);
    }
    let known: BTreeSet<AssetId> = assets.iter().map(ImageAsset::id).collect();
    if known.len() != assets.len() || file.frame.assets().any(|a| !known.contains(&a)) {
        return Err(PersistenceError::Json(
            "imagens da moldura ausentes ou repetidas".into(),
        ));
    }
    Ok(FramePackage {
        frame: file.frame,
        assets,
    })
}

/// Applies chained `vN → vN+1` migrations to project JSON.
/// v0 (legacy snapshot) → v1 preserves project content; only the
/// container changes. Add a transition here for each new version.
fn migrate(value: Value, from: u32) -> Result<Value> {
    let mut version = from;
    while version < FORMAT {
        version = match version {
            0 => {
                if !value.is_object() {
                    return Err(PersistenceError::Json(
                        "snapshot v0 deve ser um objeto JSON".into(),
                    ));
                }
                1
            }
            // v1 → v2: frames, page frame assignments and reference formatting
            // use default values; no content conversion is needed.
            1 => 2,
            // v2 → v3: images, image elements and watermarks
            // use default values; no content conversion is needed.
            2 => 3,
            // v3 → v4: free drawing content is optional
            // in existing representations.
            3 => 4,
            // v4 → v5: styles, ellipses and dynamic symbol texts
            // are optional; no content conversion is needed.
            4 => 5,
            // v5 → v6: parts and assignments are optional.
            5 => 6,
            // v6 → v7: numbering schemes and labels are optional in existing entities.
            6 => 7,
            // v7 → v8: report definitions are optional on each page.
            7 => 8,
            // v8 → v9: checking schemes are optional; couplings
            // without `owned` acquire their owned connections during loading.
            8 => 9,
            _ => unreachable!("cada formato anterior possui migração"),
        };
    }
    Ok(value)
}

fn decode_project(value: Value) -> Result<Project> {
    let project: Project =
        serde_json::from_value(value).map_err(|e| PersistenceError::Json(e.to_string()))?;
    project.validate()?;
    Ok(project)
}

// ------------------------------------------------------------ .cadsym

/// Content type of a library symbol package (`.cadsym`): ZIP with `mimetype`,
/// `manifest.json` and `symbol.json`, read with the `.cadcraft` safeguards.
pub const SYMBOL_MIMETYPE: &str = "application/x-cad-craft-symbol+zip";
/// Symbol package filename extension without a leading dot.
pub const SYMBOL_EXTENSION: &str = "cadsym";
/// Current symbol package format version.
pub const SYMBOL_FORMAT: u32 = 1;
const SYMBOL: &str = "symbol.json";

/// Validates and serializes a symbol definition into a library package.
pub fn write_symbol_package(definition: &SymbolDefinition) -> Result<Vec<u8>> {
    definition.validate()?;
    let manifest = serde_json::to_vec_pretty(&Manifest {
        format: SYMBOL_FORMAT,
        min_reader: SYMBOL_FORMAT,
        writer: format!("CAD-CRAFT {}", env!("CARGO_PKG_VERSION")),
    })
    .map_err(|e| PersistenceError::Json(e.to_string()))?;
    let content =
        serde_json::to_vec_pretty(definition).map_err(|e| PersistenceError::Json(e.to_string()))?;
    Ok(write_zip(
        io::Cursor::new(Vec::new()),
        SYMBOL_MIMETYPE,
        &manifest,
        SYMBOL,
        &content,
        &[],
    )?
    .into_inner())
}

/// Reads a checked symbol package and validates its definition.
pub fn read_symbol_package(bytes: &[u8]) -> Result<SymbolDefinition> {
    if !bytes.starts_with(b"PK\x03\x04") {
        return Err(PersistenceError::NotAProject);
    }
    let mut archive = open_checked(io::Cursor::new(bytes))?;
    read_manifest(&mut archive, SYMBOL_MIMETYPE, SYMBOL_FORMAT)?;
    let content = read_entry(&mut archive, SYMBOL, MAX_UNPACKED_BYTES)?;
    let definition: SymbolDefinition =
        serde_json::from_slice(&content).map_err(|e| PersistenceError::Json(e.to_string()))?;
    definition.validate()?;
    Ok(definition)
}

// ------------------------------------------------------------ .cadpart

/// Content type of a library part package (`.cadpart`).
pub const PART_MIMETYPE: &str = "application/x-cad-craft-part+zip";
/// Part package filename extension without a leading dot.
pub const PART_EXTENSION: &str = "cadpart";
/// Current part package format version.
pub const PART_FORMAT: u32 = 1;
const PART: &str = "part.json";

/// Validates and serializes a part into a library package.
pub fn write_part_package(part: &Part) -> Result<Vec<u8>> {
    part.validate()?;
    let manifest = serde_json::to_vec_pretty(&Manifest {
        format: PART_FORMAT,
        min_reader: PART_FORMAT,
        writer: format!("CAD-CRAFT {}", env!("CARGO_PKG_VERSION")),
    })
    .map_err(|e| PersistenceError::Json(e.to_string()))?;
    let content =
        serde_json::to_vec_pretty(part).map_err(|e| PersistenceError::Json(e.to_string()))?;
    Ok(write_zip(
        io::Cursor::new(Vec::new()),
        PART_MIMETYPE,
        &manifest,
        PART,
        &content,
        &[],
    )?
    .into_inner())
}

/// Reads a part from a checked library package.
pub fn read_part_package(bytes: &[u8]) -> Result<Part> {
    if !bytes.starts_with(b"PK\x03\x04") {
        return Err(PersistenceError::NotAProject);
    }
    let mut archive = open_checked(io::Cursor::new(bytes))?;
    read_manifest(&mut archive, PART_MIMETYPE, PART_FORMAT)?;
    let content = read_entry(&mut archive, PART, MAX_UNPACKED_BYTES)?;
    serde_json::from_slice(&content).map_err(|e| PersistenceError::Json(e.to_string()))
}

// ------------------------------------------------------------ .cadmacro

/// Content type of a library macro package (`.cadmacro`): ZIP with `mimetype`,
/// `manifest.json` and `macro.json` (embedded symbols and parts), read with
/// the `.cadcraft` safeguards and validated by insertion into a scratch project.
pub const MACRO_MIMETYPE: &str = "application/x-cad-craft-macro+zip";
/// Macro package filename extension without a leading dot.
pub const MACRO_EXTENSION: &str = "cadmacro";
/// Current macro package format version.
pub const MACRO_FORMAT: u32 = 1;
const MACRO: &str = "macro.json";
/// Maximum accepted compressed `.cadmacro` size.
pub const MAX_MACRO_FILE_BYTES: u64 = 64 * 1024 * 1024;

/// Validates and serializes a macro with its embedded symbols and parts.
pub fn write_macro_package(definition: &MacroDefinition) -> Result<Vec<u8>> {
    definition.validate()?;
    let manifest = serde_json::to_vec_pretty(&Manifest {
        format: MACRO_FORMAT,
        min_reader: MACRO_FORMAT,
        writer: format!("CAD-CRAFT {}", env!("CARGO_PKG_VERSION")),
    })
    .map_err(|e| PersistenceError::Json(e.to_string()))?;
    let content =
        serde_json::to_vec(definition).map_err(|e| PersistenceError::Json(e.to_string()))?;
    Ok(write_zip(
        io::Cursor::new(Vec::new()),
        MACRO_MIMETYPE,
        &manifest,
        MACRO,
        &content,
        &[],
    )?
    .into_inner())
}

/// Reads a size-bounded macro package and validates its definition.
pub fn read_macro_package(bytes: &[u8]) -> Result<MacroDefinition> {
    if bytes.len() as u64 > MAX_MACRO_FILE_BYTES {
        return Err(PersistenceError::Container("macro grande demais".into()));
    }
    if !bytes.starts_with(b"PK\x03\x04") {
        return Err(PersistenceError::NotAProject);
    }
    let mut archive = open_checked(io::Cursor::new(bytes))?;
    read_manifest(&mut archive, MACRO_MIMETYPE, MACRO_FORMAT)?;
    let content = read_entry(&mut archive, MACRO, MAX_UNPACKED_BYTES)?;
    let definition: MacroDefinition =
        serde_json::from_slice(&content).map_err(|e| PersistenceError::Json(e.to_string()))?;
    definition.validate()?;
    Ok(definition)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_names_are_checked_component_by_component() {
        for ok in ["project.json", "assets/a.png", "a..b/c"] {
            assert!(check_entry_name(ok).is_ok(), "{ok}");
        }
        for bad in [
            "", "/etc/x", "../x", "a/../x", "a\\x", "C:x", "./x", "a/./b", "x\0",
        ] {
            assert!(check_entry_name(bad).is_err(), "{bad:?}");
        }
    }
}
