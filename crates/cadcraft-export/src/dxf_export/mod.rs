// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! DXF page export and reports; start with export_dxf_pages or export_dxf_list.

use crate::pdf::{PdfError, PdfScope, pages_in_scope};
use cadcraft_model::domain::*;
use cadcraft_render::display::PageDisplayList;
use cadcraft_render::display::{DisplayColor, DisplayItem, DisplayLayer, DisplayList, DisplayText};
use cadcraft_render::font;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

const LAYER_PREFIX: &str = "CAD-CRAFT ";

/// O que a exportação fez e o que precisou adaptar.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DxfReport {
    pub pages: usize,
    pub entities: usize,
    /// Caracteres fora do ANSI 1252, gravados como `\U+XXXX`.
    pub escaped: BTreeSet<char>,
    /// Imagens desenhadas como caixa (o R12 não tem `IMAGE`).
    pub images: usize,
}

impl DxfReport {
    fn absorb(&mut self, other: DxfReport) {
        self.pages += other.pages;
        self.entities += other.entities;
        self.escaped.extend(other.escaped);
        self.images += other.images;
    }
    pub fn summary(&self) -> String {
        let mut text = format!(
            "{} página(s), {} entidade(s) DXF.",
            self.pages, self.entities
        );
        if !self.escaped.is_empty() {
            let chars: String = self.escaped.iter().collect();
            text.push_str(&format!(
                " Caracteres fora do ANSI 1252 gravados como \\U+XXXX: {chars}."
            ));
        }
        if self.images > 0 {
            text.push_str(&format!(
                " {} imagem(ns) substituída(s) por caixa (DXF R12 não tem imagens).",
                self.images
            ));
        }
        text
    }
}

/// Um arquivo DXF por página.
#[derive(Clone, Debug)]
pub struct DxfFile {
    pub page: PageId,
    pub name: String,
    pub bytes: Vec<u8>,
}

/// Cor ACI da categoria.
fn layer_color(layer: DisplayLayer) -> u8 {
    match layer {
        DisplayLayer::Frame | DisplayLayer::Symbols | DisplayLayer::Texts => 7,
        DisplayLayer::Wires | DisplayLayer::Junctions => 5,
        DisplayLayer::References => 4,
        DisplayLayer::Guides => 8,
    }
}
/// Cor própria da entidade quando difere da camada (marca d'água, guia).
fn entity_color(color: DisplayColor) -> Option<u8> {
    match color {
        DisplayColor::Watermark => Some(9),
        DisplayColor::Guide => Some(8),
        DisplayColor::Ink | DisplayColor::Reference => None,
    }
}

fn num(v: f64) -> String {
    let v = if v.abs() < 5e-7 { 0.0 } else { v };
    let s = format!("{v:.6}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.into() }
}
fn width_key(w: f64) -> String {
    num((w * 1000.0).round() / 1000.0)
}
fn layer_name(layer: DisplayLayer, width: Option<f64>) -> String {
    match width {
        Some(w) => format!("{LAYER_PREFIX}{} {}mm", layer.name(), width_key(w)),
        None => format!("{LAYER_PREFIX}{}", layer.name()),
    }
}

/// Texto em ANSI 1252; fora dele `\U+XXXX`. Se houver `%%` (prefixo de
/// código de controle no DXF), cada `%` vira `%%%`, que é o `%` literal.
fn encode_text(text: &str, escaped: &mut BTreeSet<char>) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len());
    let literal_percent = text.contains("%%");
    for c in text.chars() {
        match font::win_ansi(c) {
            Some(b'%') if literal_percent => out.extend_from_slice(b"%%%"),
            Some(b) => out.push(b),
            None if c.is_control() => out.push(b' '),
            None => {
                escaped.insert(c);
                out.extend_from_slice(format!("\\U+{:04X}", c as u32).as_bytes());
            }
        }
    }
    out
}

struct Writer {
    out: Vec<u8>,
    report: DxfReport,
    /// Tipos de linha: padrão (mm, em milésimos) → nome.
    dashes: BTreeMap<Vec<i64>, String>,
    layers: BTreeSet<(DisplayLayer, Option<String>)>,
}

impl Writer {
    fn pair(&mut self, code: i32, value: &str) {
        let _ = cadcraft_model::dxf::write_group(&mut Line(&mut self.out), code, value);
    }
    fn pair_bytes(&mut self, code: i32, value: &[u8]) {
        let _ = writeln!(Line(&mut self.out), "{code:>3}");
        self.out.extend_from_slice(value);
        self.out.push(b'\n');
    }
}

/// Adaptador `fmt::Write` sobre o buffer de bytes.
struct Line<'a>(&'a mut Vec<u8>);
impl std::fmt::Write for Line<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

/// DXF de uma página.
pub fn export_dxf_list(list: &DisplayList) -> (Vec<u8>, DxfReport) {
    let mut body = Writer {
        out: vec![],
        report: DxfReport {
            pages: 1,
            ..DxfReport::default()
        },
        dashes: BTreeMap::new(),
        layers: BTreeSet::new(),
    };
    for item in &list.items {
        match item {
            DisplayItem::Path {
                points,
                closed,
                stroke,
                fill,
                layer,
            } => body.path(points, *closed, stroke.as_ref(), *fill, *layer),
            DisplayItem::Text(text) => body.text(text),
            DisplayItem::Image { rect, .. } => body.image(*rect),
            DisplayItem::Link { .. } => {}
        }
    }
    let mut head = Writer {
        out: vec![],
        report: DxfReport::default(),
        dashes: BTreeMap::new(),
        layers: BTreeSet::new(),
    };
    head.pair(999, "CAD-CRAFT - pagina exportada (DXF R12, mm)");
    head.pair(0, "SECTION");
    head.pair(2, "HEADER");
    head.pair(9, "$ACADVER");
    head.pair(1, "AC1009");
    head.pair(9, "$DWGCODEPAGE");
    head.pair(3, "ANSI_1252");
    head.pair(9, "$INSUNITS");
    head.pair(70, "4");
    for (var, x, y) in [
        ("$LIMMIN", 0.0, 0.0),
        ("$LIMMAX", list.width, list.height),
        ("$EXTMIN", 0.0, 0.0),
        ("$EXTMAX", list.width, list.height),
    ] {
        head.pair(9, var);
        head.pair(10, &num(x));
        head.pair(20, &num(y));
    }
    head.pair(0, "ENDSEC");
    head.pair(0, "SECTION");
    head.pair(2, "TABLES");
    // Tipos de linha.
    head.pair(0, "TABLE");
    head.pair(2, "LTYPE");
    head.pair(70, &(body.dashes.len() + 1).to_string());
    head.pair(0, "LTYPE");
    head.pair(2, "CONTINUOUS");
    head.pair(70, "0");
    head.pair(3, "Solid line");
    head.pair(72, "65");
    head.pair(73, "0");
    head.pair(40, "0");
    for (pattern, name) in &body.dashes {
        let lengths: Vec<f64> = pattern.iter().map(|v| *v as f64 / 1000.0).collect();
        head.pair(0, "LTYPE");
        head.pair(2, name);
        head.pair(70, "0");
        head.pair(3, "CAD-CRAFT");
        head.pair(72, "65");
        head.pair(73, &lengths.len().to_string());
        head.pair(40, &num(lengths.iter().sum()));
        for (i, l) in lengths.iter().enumerate() {
            // Traços positivos, vãos negativos.
            head.pair(49, &num(if i % 2 == 0 { *l } else { -l }));
        }
    }
    head.pair(0, "ENDTAB");
    // Camadas.
    head.pair(0, "TABLE");
    head.pair(2, "LAYER");
    head.pair(70, &(body.layers.len() + 1).to_string());
    head.pair(0, "LAYER");
    head.pair(2, "0");
    head.pair(70, "0");
    head.pair(62, "7");
    head.pair(6, "CONTINUOUS");
    for (layer, width) in &body.layers {
        let name = match width {
            Some(w) => format!("{LAYER_PREFIX}{} {w}mm", layer.name()),
            None => format!("{LAYER_PREFIX}{}", layer.name()),
        };
        head.pair(0, "LAYER");
        head.pair(2, &name);
        head.pair(70, "0");
        head.pair(62, &layer_color(*layer).to_string());
        head.pair(6, "CONTINUOUS");
    }
    head.pair(0, "ENDTAB");
    // Estilo de texto padrão.
    head.pair(0, "TABLE");
    head.pair(2, "STYLE");
    head.pair(70, "1");
    head.pair(0, "STYLE");
    head.pair(2, "STANDARD");
    head.pair(70, "0");
    head.pair(40, "0");
    head.pair(41, "1");
    head.pair(50, "0");
    head.pair(71, "0");
    head.pair(42, "2.5");
    head.pair(3, "txt");
    head.pair(4, "");
    head.pair(0, "ENDTAB");
    head.pair(0, "ENDSEC");
    head.pair(0, "SECTION");
    head.pair(2, "ENTITIES");
    let mut bytes = head.out;
    bytes.extend_from_slice(&body.out);
    let mut tail = Writer {
        out: vec![],
        report: DxfReport::default(),
        dashes: BTreeMap::new(),
        layers: BTreeSet::new(),
    };
    tail.pair(0, "ENDSEC");
    tail.pair(0, "EOF");
    bytes.extend_from_slice(&tail.out);
    (bytes, body.report)
}

/// Parte segura de nome de arquivo (sem caracteres reservados do Windows).
fn safe(part: &str) -> String {
    let cleaned: String = part
        .chars()
        .map(|c| {
            if c.is_control() || r#"<>:"/\|?*"#.contains(c) {
                '_'
            } else {
                c
            }
        })
        .collect();
    let cleaned = cleaned.trim().trim_end_matches(['.', ' ']).to_owned();
    if cleaned.is_empty() {
        "_".into()
    } else {
        cleaned
    }
}

/// Nomes `<projeto>_<documento>_<página>.dxf`, seguros e únicos
/// (repetidos ganham `_2`, `_3`…; comparação sem diferenciar maiúsculas).
pub fn dxf_file_names(project: &Project, pages: &[PageId]) -> Vec<(PageId, String)> {
    let mut used = BTreeSet::new();
    let mut out = vec![];
    for page in pages {
        let Some(p) = project.page(*page) else {
            continue;
        };
        let document = project
            .document(p.document_id())
            .map(|d| d.name().as_str().to_owned())
            .unwrap_or_default();
        let base = format!(
            "{}_{}_{}",
            safe(project.name().as_str()),
            safe(&document),
            safe(p.number().as_str())
        );
        let mut name = format!("{base}.dxf");
        let mut n = 2;
        while !used.insert(name.to_lowercase()) {
            name = format!("{base}_{n}.dxf");
            n += 1;
        }
        out.push((*page, name));
    }
    out
}

/// Exporta o alcance: um DXF por página. `progress(feitas, total)` devolve
/// `false` para cancelar.
pub fn export_dxf_pages(
    project: &Project,
    scope: &PdfScope,
    progress: &mut dyn FnMut(usize, usize) -> bool,
) -> Result<(Vec<DxfFile>, DxfReport), PdfError> {
    let pages = pages_in_scope(project, scope)?;
    if pages.is_empty() {
        return Err(PdfError::Empty);
    }
    let names = dxf_file_names(project, &pages);
    let total = names.len();
    let mut files = Vec::with_capacity(total);
    let mut report = DxfReport::default();
    for (done, (page, name)) in names.into_iter().enumerate() {
        if !progress(done, total) {
            return Err(PdfError::Cancelled);
        }
        let list = project.page_display_list(page)?;
        let (bytes, page_report) = export_dxf_list(&list);
        report.absorb(page_report);
        files.push(DxfFile { page, name, bytes });
    }
    progress(total, total);
    Ok((files, report))
}

mod entity;
