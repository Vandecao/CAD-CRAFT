// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 22: DXF R12 (AC1009) de página a partir da lista de exibição.
//!
//! - Unidades em mm (`$INSUNITS = 4`), texto em ANSI 1252
//!   (`$DWGCODEPAGE`); outros caracteres como `\U+XXXX`.
//! - Uma camada por categoria e espessura (`CAD-CRAFT Fios 0.35mm`), com cor
//!   ACI por categoria; tracejado por tipo de linha.
//! - Geometria explodida, igual ao PDF; preenchimentos em `SOLID`.
//! - Imagens não existem no R12: viram caixa marcada e são relatadas.
//!
//! Sem GUI.
use crate::display::{DisplayColor, DisplayItem, DisplayLayer, DisplayList, DisplayText};
use crate::domain::*;
use crate::font;
use crate::pdf::{PdfError, PdfScope, pages_in_scope};
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
        let _ = writeln!(Line(&mut self.out), "{code:>3}\n{value}");
    }
    fn pair_bytes(&mut self, code: i32, value: &[u8]) {
        let _ = writeln!(Line(&mut self.out), "{code:>3}");
        self.out.extend_from_slice(value);
        self.out.push(b'\n');
    }
    fn point(&mut self, base: i32, x: f64, y: f64) {
        self.pair(base, &num(x));
        self.pair(base + 10, &num(y));
        self.pair(base + 20, "0");
    }
    fn common(&mut self, layer: &str, linetype: Option<&str>, color: Option<u8>) {
        self.pair(8, layer);
        if let Some(lt) = linetype {
            self.pair(6, lt);
        }
        if let Some(c) = color {
            self.pair(62, &c.to_string());
        }
    }
    fn linetype(&mut self, dash: &[f64]) -> Option<String> {
        if dash.is_empty() {
            return None;
        }
        let key: Vec<i64> = dash.iter().map(|d| (d * 1000.0).round() as i64).collect();
        let next = self.dashes.len() + 1;
        Some(
            self.dashes
                .entry(key)
                .or_insert_with(|| format!("CAD-CRAFT-TRACO-{next}"))
                .clone(),
        )
    }

    fn path(
        &mut self,
        points: &[Point2D],
        closed: bool,
        stroke: Option<&crate::display::DisplayStroke>,
        fill: Option<DisplayColor>,
        layer: DisplayLayer,
    ) {
        if let Some(color) = fill
            && points.len() > 2
        {
            let name = layer_name(layer, None);
            self.layers.insert((layer, None));
            // Leque a partir do primeiro ponto (formas convexas da folha).
            for i in 1..points.len() - 1 {
                let (a, b, c) = (points[0], points[i], points[i + 1]);
                self.pair(0, "SOLID");
                self.common(&name, None, entity_color(color));
                self.point(10, a.x(), a.y());
                self.point(11, b.x(), b.y());
                self.point(12, c.x(), c.y());
                self.point(13, c.x(), c.y());
                self.report.entities += 1;
            }
        }
        let Some(stroke) = stroke else {
            return;
        };
        let key = width_key(stroke.width);
        let name = layer_name(layer, Some(stroke.width));
        self.layers.insert((layer, Some(key)));
        let linetype = self.linetype(&stroke.dash);
        let color = entity_color(stroke.color);
        if points.len() == 2 && !closed {
            self.pair(0, "LINE");
            self.common(&name, linetype.as_deref(), color);
            self.point(10, points[0].x(), points[0].y());
            self.point(11, points[1].x(), points[1].y());
        } else {
            self.pair(0, "POLYLINE");
            self.common(&name, linetype.as_deref(), color);
            self.pair(66, "1");
            self.point(10, 0.0, 0.0);
            self.pair(70, if closed { "1" } else { "0" });
            for p in points {
                self.pair(0, "VERTEX");
                self.pair(8, &name);
                self.point(10, p.x(), p.y());
            }
            self.pair(0, "SEQEND");
            self.pair(8, &name);
        }
        self.report.entities += 1;
    }

    fn text(&mut self, text: &DisplayText) {
        let [a, b, c, d, _, _] = text.matrix;
        let (ux, uy) = (a, b);
        let v_len = c.hypot(d);
        let u_len = ux.hypot(uy);
        if v_len <= 0.0 || u_len <= 0.0 {
            return;
        }
        let natural = text.natural_width();
        let stretch = match text.width {
            Some(w) if natural > 0.0 => w / natural,
            _ => 1.0,
        };
        // Altura de maiúscula; fator de largura relativo à altura.
        let height = text.size * font::CAP_HEIGHT * v_len;
        let factor = u_len * stretch / v_len;
        let mirrored = a * d - b * c < 0.0;
        let rotation = uy.atan2(ux).to_degrees();
        let (x, y) = text.map(0.0, text.baseline);
        let bytes = encode_text(&text.text, &mut self.report.escaped);
        let name = layer_name(text.layer, None);
        self.layers.insert((text.layer, None));
        self.pair(0, "TEXT");
        self.common(&name, None, entity_color(text.color));
        self.point(10, x, y);
        self.pair(40, &num(height));
        self.pair_bytes(1, &bytes);
        if rotation.abs() > 1e-9 {
            self.pair(50, &num(rotation));
        }
        if (factor - 1.0).abs() > 1e-6 {
            self.pair(41, &num(factor));
        }
        if mirrored {
            self.pair(71, "2");
        }
        self.report.entities += 1;
    }

    fn image(&mut self, rect: Rect2D) {
        let (o, m) = (rect.origin(), rect.max());
        let pts = [
            (o.x(), o.y()),
            (m.x(), o.y()),
            (m.x(), m.y()),
            (o.x(), m.y()),
        ]
        .map(|(x, y)| Point2D::new(x, y).expect("retângulo válido"));
        let guide = crate::display::DisplayStroke {
            width: 0.18,
            dash: vec![],
            color: DisplayColor::Guide,
        };
        self.path(&pts, true, Some(&guide), None, DisplayLayer::Guides);
        self.path(
            &[pts[0], pts[2]],
            false,
            Some(&guide),
            None,
            DisplayLayer::Guides,
        );
        self.path(
            &[pts[1], pts[3]],
            false,
            Some(&guide),
            None,
            DisplayLayer::Guides,
        );
        self.report.images += 1;
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
