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

/// Formato gravado por esta versão.
pub const FORMAT: u32 = 9;
/// Formato mais antigo capaz de ler o que esta versão grava.
pub const MIN_READER: u32 = 9;
/// Formato mais recente que esta versão sabe ler.
pub const SUPPORTED_FORMAT: u32 = FORMAT;
pub const MIMETYPE: &str = "application/x-cad-craft-project+zip";
pub const EXTENSION: &str = "cadcraft";
/// Limites de segurança do contêiner, verificados antes de ler o projeto.
pub const MAX_ENTRIES: usize = 4096;
pub const MAX_UNPACKED_BYTES: u64 = 1 << 30;
/// Limite de leitura de um snapshot JSON (v0) aberto como arquivo.
pub const MAX_LEGACY_BYTES: u64 = 256 * 1024 * 1024;

const MANIFEST: &str = "manifest.json";
const PROJECT: &str = "project.json";
const MIMETYPE_ENTRY: &str = "mimetype";
const MAX_MANIFEST_BYTES: u64 = 64 * 1024;

/// Campos desconhecidos são tolerados: o manifesto pode crescer sem elevar
/// `min_reader`. O conteúdo do projeto continua estrito.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub format: u32,
    pub min_reader: u32,
    pub writer: String,
}
impl Manifest {
    pub fn current() -> Self {
        Self {
            format: FORMAT,
            min_reader: MIN_READER,
            writer: format!("CAD-CRAFT {}", env!("CARGO_PKG_VERSION")),
        }
    }
}

/// Resultado de uma leitura bem-sucedida.
#[derive(Debug)]
pub struct LoadedProject {
    pub project: Project,
    /// 0 para o snapshot técnico JSON; caso contrário o `format` do manifesto.
    pub source_format: u32,
    /// Verdadeiro quando o arquivo foi gravado por formato mais novo: salvar
    /// por cima poderia descartar dados que esta versão não conhece.
    pub read_only: bool,
    pub manifest: Option<Manifest>,
}
impl LoadedProject {
    /// Conteúdo gravado por formato anterior e convertido na leitura.
    pub fn migrated(&self) -> bool {
        self.source_format < FORMAT
    }
    /// Snapshot técnico JSON (v0): precisa de "Salvar como" para virar `.cadcraft`.
    /// Arquivos `.cadcraft` antigos continuam associados ao seu caminho.
    pub fn is_legacy_snapshot(&self) -> bool {
        self.source_format == 0
    }
}

#[derive(Debug)]
pub enum PersistenceError {
    Io(io::Error),
    NotAProject,
    Container(String),
    UnsafeEntry(String),
    TooLarge(&'static str),
    MissingEntry(&'static str),
    WrongMimetype,
    Manifest(String),
    UnsupportedFormat { min_reader: u32, supported: u32 },
    Json(String),
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
pub type Result<T> = std::result::Result<T, PersistenceError>;

/// Grava o formato atual. O projeto é validado antes de qualquer byte escrito.
pub fn write_project<W: Write + Seek>(project: &Project, writer: W) -> Result<W> {
    project.validate()?;
    let manifest = serde_json::to_vec_pretty(&Manifest::current())
        .map_err(|e| PersistenceError::Json(e.to_string()))?;
    let content = serde_json::to_vec(project).map_err(|e| PersistenceError::Json(e.to_string()))?;
    // Etapa 14: imagens com conteúdo vão para `assets/`; sem conteúdo (lidas
    // de snapshot técnico) não podem ser gravadas.
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
            // PNG/JPEG já são comprimidos.
            zip.start_file(asset_entry(asset), stored)?;
            zip.write_all(bytes)?;
        }
    }
    Ok(zip.finish()?)
}

/// Conveniência para testes e memória.
pub fn write_project_to_vec(project: &Project) -> Result<Vec<u8>> {
    Ok(write_project(project, io::Cursor::new(Vec::new()))?.into_inner())
}

/// Lê v1 (ZIP) ou v0 (snapshot JSON). Nada é extraído para o disco.
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

/// Nome de entrada aceitável: relativo, sem componentes especiais.
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

/// Contêiner ZIP com todas as verificações de segurança (Etapa 12).
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
        // Nomes que só diferem por maiúsculas colidiriam em Windows.
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
    // O tamanho declarado pode mentir; o limite também vale para os bytes reais.
    read_bounded(entry, limit, "entrada acima do limite")
}

// ------------------------------------------------------------ .cadframe

/// Etapa 14: moldura exportável (`.cadframe`).
pub const FRAME_MIMETYPE: &str = "application/x-cad-craft-frame+zip";
pub const FRAME_EXTENSION: &str = "cadframe";
pub const FRAME_FORMAT: u32 = 1;
const FRAME: &str = "frame.json";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FrameFile {
    frame: FrameTemplate,
    assets: Vec<ImageAsset>,
}

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

/// Lê `.cadframe` com as mesmas defesas do `.cadcraft`; as imagens precisam
/// estar presentes e corresponder aos metadados.
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

/// Migrações encadeadas `vN → vN+1` sobre o JSON do projeto.
/// v0 (snapshot técnico) → v1: o conteúdo do projeto é idêntico; só o
/// contêiner muda. Novas versões acrescentam um passo aqui.
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
            // v1 (Etapa 12) → v2 (Etapa 13): molduras, moldura por página e
            // formato de referência entram com valores padrão; nada a converter.
            1 => 2,
            // v2 (Etapa 13) → v3 (Etapa 14): imagens, elementos de imagem e marca
            // d'água entram com valores padrão; nada a converter.
            2 => 3,
            // v3 (Etapa 14/15) → v4 (Etapa 16): conteúdo livre é opcional
            // nas representações existentes.
            3 => 4,
            // v4 (Etapa 16) → v5 (Etapa 17): estilos, elipse e textos dinâmicos
            // de símbolo são opcionais; nada a converter.
            4 => 5,
            // v5 (Etapa 17) → v6 (Etapa 18): peças e atribuições são opcionais.
            5 => 6,
            // v6 → v7: esquema e rótulos são opcionais nas entidades antigas.
            6 => 7,
            // v7 → v8: definição de relatório opcional em cada página.
            7 => 8,
            // v8 → v9 (Etapa 24): esquema de verificação opcional; acoplamentos
            // sem `owned` recebem as conexões de posse na carga (DEC-11-01).
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

/// Etapa 17: símbolo de biblioteca (`.cadsym`): ZIP com `mimetype`,
/// `manifest.json` e `symbol.json`, lido com as defesas do `.cadcraft`.
pub const SYMBOL_MIMETYPE: &str = "application/x-cad-craft-symbol+zip";
pub const SYMBOL_EXTENSION: &str = "cadsym";
pub const SYMBOL_FORMAT: u32 = 1;
const SYMBOL: &str = "symbol.json";

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

pub const PART_MIMETYPE: &str = "application/x-cad-craft-part+zip";
pub const PART_EXTENSION: &str = "cadpart";
pub const PART_FORMAT: u32 = 1;
const PART: &str = "part.json";

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

/// Etapa 19: macro de biblioteca (`.cadmacro`): ZIP com `mimetype`,
/// `manifest.json` e `macro.json` (símbolos e peças embutidos), lido com as
/// defesas do `.cadcraft` e validado por inserção num projeto de rascunho.
pub const MACRO_MIMETYPE: &str = "application/x-cad-craft-macro+zip";
pub const MACRO_EXTENSION: &str = "cadmacro";
pub const MACRO_FORMAT: u32 = 1;
const MACRO: &str = "macro.json";
/// Tamanho máximo aceito para um `.cadmacro` (compactado).
pub const MAX_MACRO_FILE_BYTES: u64 = 64 * 1024 * 1024;

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
