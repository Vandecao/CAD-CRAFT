// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! QElectroTech project import. Start at `import`, then follow read, convert and report.
//!
//! Independent implementation based on file structure. QElectroTech's GPL
//! source was consulted only to understand the format; no code was copied
//! or translated. QET uses pixels with the scene origin at the upper
//! left and Y downward. Sheets have a 5 px margin, 20 px headers,
//! a column/row grid and a bottom titleblock. CAD-CRAFT uses mm with Y
//! upward and the `.elmt` scale.
//!
//! - Embedded `<collection>` elements → symbols through the `.elmt` importer.
//! - `<element>` → occurrence; masters and slaves with cross-references share
//!   a device with main and auxiliary functions.
//! - `<conductor>` → logical connection and route; `num` → locked wire number.
//!   Wire numbers retain their explicit source assignment.
//! - Sheet reports (`next_report`/`previous_report`) → paired,
//!   grouped interruptions.
//! - `<inputs>` → free texts. Images, shapes and anything without an
//!   equivalent are noted in the report.
use crate::elmt::{self, ElmtOptions};
use crate::titleblock::{self, TitleblockOptions};
use crate::xml::{self, Element};
use crate::{ImportReport, ImportResult, check_size, fail};
use cadcraft_model::domain::*;
use std::collections::{BTreeMap, BTreeSet};

/// Maximum XML node count to bound malicious input.
pub const MAX_QET_NODES: usize = 500_000;
const MARGIN_PX: f64 = 5.0;
const HEADER_PX: f64 = 20.0;
/// Titleblock height when its template is not embedded in the project.
const DEFAULT_TITLEBLOCK_PX: f64 = 50.0;
/// Tolerance in mm for matching occurrence terminals to ports.
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

/// Counts of converted content.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QetStats {
    pub pages: usize,
    pub symbols: usize,
    pub occurrences: usize,
    pub devices: usize,
    pub conductors: usize,
    pub sheet_reports: usize,
    pub texts: usize,
    /// Occurrences without terminals (drawings) and without a device.
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
    /// Terminal UUID (QElectroTech 0.9+) → local pixel position.
    terminals: BTreeMap<String, (f64, f64)>,
}

/// Sheet geometry in pixels.
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
    /// Label text stored on the occurrence (QET 0.7+), in local mm.
    label_text: Option<SymbolPrimitive>,
    /// Occurrence user texts: local position in mm, text and height.
    user_texts: Vec<(f64, f64, String, f64)>,
}

struct Wire {
    page: usize,
    /// Element UUIDs (QElectroTech 0.9+); without them, `t1`/`t2` are
    /// sheet terminal IDs.
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
/// Plain text without QElectroTech rich-text HTML markup.
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

/// CAD-CRAFT counterclockwise rotation from QET orientation, which uses
/// clockwise quarter-turns on screen.
fn rotation(orientation: u32) -> f64 {
    f64::from((4 - orientation % 4) % 4) * 90.0
}

/// Designation from a QET label: `-K1`, `K1` or `=A+B-K1` (device part only);
/// `None` when no usable name exists.
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
