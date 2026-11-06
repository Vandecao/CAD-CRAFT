// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! QElectroTech project import. Start at `import`, then follow read, convert and report.
//!
//! Implementação própria a partir do formato dos arquivos (o código-fonte do
//! QElectroTech, GPL, foi consultado só para entender o formato; nada foi
//! copiado nem traduzido). Coordenadas QET: px, origem no canto superior
//! esquerdo da cena, Y para baixo; a folha tem margem de 5 px, cabeçalhos de
//! 20 px, a grade (colunas × linhas) e o carimbo embaixo. CAD-CRAFT: mm, Y
//! para cima, escala do `.elmt` (Etapa 17).
//!
//! - Elementos embutidos (`<collection>`) → símbolos, pelo importador `.elmt`.
//! - `<element>` → ocorrência; mestre e escravos (referências cruzadas) → um
//!   dispositivo com função principal e auxiliares (decisão 3).
//! - `<conductor>` → conexão lógica + rota; `num` → número do fio travado
//!   (decisão 4).
//! - Relatórios de folha (`next_report`/`previous_report`) → par de
//!   interrupções agrupadas.
//! - `<inputs>` → textos livres. Imagens, formas e o que não tiver
//!   equivalente vão para o relatório.
use crate::elmt::{self, ElmtOptions};
use crate::titleblock::{self, TitleblockOptions};
use crate::xml::{self, Element};
use crate::{ImportReport, ImportResult, check_size, fail};
use cadcraft_model::domain::*;
use std::collections::{BTreeMap, BTreeSet};

/// Limite de nós XML (defesa contra arquivo malicioso).
pub const MAX_QET_NODES: usize = 500_000;
const MARGIN_PX: f64 = 5.0;
const HEADER_PX: f64 = 20.0;
/// Altura do carimbo quando o modelo não está embutido no projeto.
const DEFAULT_TITLEBLOCK_PX: f64 = 50.0;
/// Tolerância (mm) para casar o terminal da ocorrência com a porta.
const PORT_TOLERANCE: f64 = 1e-6;

#[derive(Debug, Clone)]
pub struct QetOptions {
    pub mm_per_px: f64,
}
impl Default for QetOptions {
    fn default() -> Self {
        Self {
            mm_per_px: elmt::DEFAULT_MM_PER_PX,
        }
    }
}

/// Contagens do que foi convertido.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QetStats {
    pub pages: usize,
    pub symbols: usize,
    pub occurrences: usize,
    pub devices: usize,
    pub conductors: usize,
    pub sheet_reports: usize,
    pub texts: usize,
    /// Ocorrências sem terminais (desenhos), sem dispositivo.
    pub drawings: usize,
}

#[derive(Debug, Clone)]
pub struct QetImport {
    pub project: Project,
    pub report: ImportReport,
    pub stats: QetStats,
}

struct Definition {
    symbol: SymbolDefinition,
    link: String,
    kind: String,
    /// UUID do terminal (QElectroTech 0.9+) → posição local em px.
    terminals: BTreeMap<String, (f64, f64)>,
}

/// Geometria da folha em px.
#[derive(Clone, Copy, PartialEq)]
struct Sheet {
    cols: usize,
    rows: usize,
    colsize: f64,
    rowsize: f64,
    show_cols: bool,
    show_rows: bool,
    titleblock_px: f64,
}
impl Sheet {
    fn width_px(&self) -> f64 {
        2.0 * MARGIN_PX + HEADER_PX + self.cols as f64 * self.colsize
    }
    fn height_px(&self) -> f64 {
        2.0 * MARGIN_PX + HEADER_PX + self.rows as f64 * self.rowsize + self.titleblock_px
    }
}

struct Instance {
    page: usize,
    uuid: String,
    def: String,
    x: f64,
    y: f64,
    orientation: u32,
    label: String,
    prefix: String,
    links: Vec<String>,
    terminals: Vec<(String, f64, f64)>,
    /// Texto do rótulo guardado na ocorrência (QET 0.7+), em mm locais.
    label_text: Option<SymbolPrimitive>,
    /// Textos de usuário da ocorrência: posição local (mm), texto, altura.
    user_texts: Vec<(f64, f64, String, f64)>,
}

struct Wire {
    page: usize,
    /// UUIDs dos elementos (QElectroTech 0.9+); sem eles, `t1`/`t2` são
    /// IDs de terminal da folha.
    e1: Option<String>,
    e2: Option<String>,
    t1: String,
    t2: String,
    num: String,
    segments: Vec<(bool, f64)>,
}

fn num(e: &Element, k: &str) -> Option<f64> {
    e.attr(k)?
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
}
fn count_nodes(e: &Element) -> usize {
    1 + e.children.iter().map(count_nodes).sum::<usize>()
}
fn domain(e: impl std::fmt::Display) -> crate::ImportError {
    fail(e.to_string())
}
fn trim_uuid(s: &str) -> String {
    s.trim().trim_matches(|c| c == '{' || c == '}').to_owned()
}
fn safe_id(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
/// Texto sem marcação HTML (textos ricos do QElectroTech).
fn plain_text(text: &str) -> String {
    if !text.contains('<') {
        return text.trim().to_owned();
    }
    let body = text.split_once("<body").map_or(text, |(_, b)| b);
    let mut out = String::new();
    let mut tag = false;
    for c in body.chars() {
        match c {
            '<' => tag = true,
            '>' => {
                tag = false;
                out.push(' ');
            }
            c if !tag => out.push(c),
            _ => {}
        }
    }
    let out = out
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&amp;", "&");
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}
fn info(e: &Element, name: &str) -> String {
    e.child("elementInformations")
        .into_iter()
        .flat_map(|i| i.children_named("elementInformation"))
        .find(|i| i.attr("name") == Some(name))
        .map(|i| i.text.trim().to_owned())
        .unwrap_or_default()
}

/// Rotação CAD-CRAFT (anti-horária) da orientação QET (passos de 90° no
/// sentido horário da tela).
fn rotation(orientation: u32) -> f64 {
    f64::from((4 - orientation % 4) % 4) * 90.0
}

/// Designação a partir do rótulo QET: `-K1`, `K1` ou `=A+B-K1` (só a parte
/// do dispositivo); `None` quando não houver nome utilizável.
fn device_part(label: &str) -> Option<String> {
    let label = label.trim();
    let part = label.rsplit_once('-').map_or(label, |(_, d)| d).trim();
    DesignationPart::new(part).ok().map(|_| part.to_owned())
}
fn prefix_of(label: &str) -> String {
    let p: String = device_part(label)
        .unwrap_or_default()
        .chars()
        .take_while(|c| c.is_alphabetic())
        .collect();
    if p.is_empty() { "D".into() } else { p }
}

struct Importer<'a> {
    s: f64,
    project: Project,
    report: ImportReport,
    stats: QetStats,
    definitions: BTreeMap<String, Definition>,
    templates: BTreeMap<String, &'a Element>,
}

mod convert;
mod read;
mod report;

/// Imports a QElectroTech project through the read, convert and report phases.
pub fn import(bytes: &[u8], options: &QetOptions) -> ImportResult<QetImport> {
    let root = read::read(bytes, options)?;
    convert::convert(&root, options)
}
