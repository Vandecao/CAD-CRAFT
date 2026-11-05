// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Carimbo do QElectroTech (`.titleblock`, XML).
//!
//! O carimbo é uma grade (`cols`: `Npx` fixas, `tN%` do total, `rN%` do
//! restante; `rows` em px) com células de campo (rótulo, valor com variáveis
//! `%var`/`%{var}`) e de logotipo. Ele é colocado na base de uma folha
//! escolhida pelo usuário, dentro de uma borda com margem.
use super::build::{Builder, round3};
use super::xml::{self, Element};
use super::{ImportResult, ImportedFrame, base64_decode, check_size, fail, qet_variable};
use crate::domain::{
    FrameTextStyle, LocalizedText, Size2D, TextAlign, ZoneAxis, ZoneLayout, inspect_image,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy)]
pub struct TitleblockOptions {
    pub sheet: Size2D,
    /// Margem da borda (mm).
    pub margin: f64,
    /// Escala das medidas em px do QElectroTech (mm por px).
    pub mm_per_px: f64,
}
impl TitleblockOptions {
    pub fn new(sheet: Size2D) -> Self {
        Self {
            sheet,
            margin: 5.0,
            mm_per_px: 0.25,
        }
    }
}

const LINE_WIDTH: f64 = 0.25;
const BORDER_WIDTH: f64 = 0.35;
const DEFAULT_FONT: f64 = 9.0;

/// Texto preferido entre as traduções (pt-BR, pt, en, primeira) e as demais.
fn translations(e: Option<&Element>) -> LocalizedText {
    let mut entries: Vec<(String, String)> = e
        .map(|e| {
            e.children_named("translation")
                .filter(|t| !t.text.trim().is_empty())
                .map(|t| {
                    (
                        t.attr("lang").unwrap_or("").replace('_', "-"),
                        t.text.trim().to_owned(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let i = ["pt-BR", "pt", "en"]
        .iter()
        .find_map(|l| entries.iter().position(|(k, _)| k == l))
        .unwrap_or(0);
    if entries.is_empty() {
        return LocalizedText::new("");
    }
    let (lang, default) = entries.remove(i);
    let mut text = LocalizedText::new(default);
    for (k, v) in entries {
        if k != lang && !k.is_empty() {
            text.translations.insert(k, v);
        }
    }
    text
}

enum Token {
    Literal(String),
    Variable(String),
}
fn tokens(value: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut literal = String::new();
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' {
            literal.push(c);
            continue;
        }
        let name: String = if chars.peek() == Some(&'{') {
            chars.next();
            let mut n = String::new();
            for c in chars.by_ref() {
                if c == '}' {
                    break;
                }
                n.push(c);
            }
            n
        } else {
            let mut n = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    n.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            n
        };
        if name.is_empty() {
            literal.push('%');
            continue;
        }
        if !literal.trim().is_empty() {
            out.push(Token::Literal(std::mem::take(&mut literal)));
        }
        literal.clear();
        out.push(Token::Variable(name));
    }
    if !literal.trim().is_empty() {
        out.push(Token::Literal(literal));
    }
    out
}

/// Larguras das colunas (mm) a partir da especificação da grade.
fn column_widths(spec: &str, total: f64, s: f64) -> ImportResult<Vec<f64>> {
    enum Col {
        Fixed(f64),
        Total(f64),
        Rest(f64),
    }
    let mut cols = Vec::new();
    for part in spec.split(';').map(str::trim).filter(|p| !p.is_empty()) {
        let bad = || fail(format!("coluna inválida no carimbo: '{part}'"));
        let pct = |p: &str| -> ImportResult<f64> {
            p.trim_end_matches('%').parse::<f64>().map_err(|_| bad())
        };
        cols.push(if let Some(p) = part.strip_prefix('t') {
            Col::Total(pct(p)? / 100.0)
        } else if let Some(p) = part.strip_prefix('r') {
            Col::Rest(pct(p)? / 100.0)
        } else {
            Col::Fixed(
                part.trim_end_matches("px")
                    .parse::<f64>()
                    .map_err(|_| bad())?
                    * s,
            )
        });
    }
    let used: f64 = cols
        .iter()
        .map(|c| match c {
            Col::Fixed(w) => *w,
            Col::Total(p) => p * total,
            Col::Rest(_) => 0.0,
        })
        .sum();
    let rest = (total - used).max(0.0);
    Ok(cols
        .iter()
        .map(|c| match c {
            Col::Fixed(w) => *w,
            Col::Total(p) => p * total,
            Col::Rest(p) => p * rest,
        })
        .collect())
}

pub fn import(bytes: &[u8], options: TitleblockOptions) -> ImportResult<ImportedFrame> {
    check_size(bytes)?;
    import_template(&xml::parse(bytes)?, options)
}

/// Etapa 25: modelo já lido (carimbo embutido num projeto `.qet`).
pub(crate) fn import_template(
    root: &xml::Element,
    options: TitleblockOptions,
) -> ImportResult<ImportedFrame> {
    if root.name != "titleblocktemplate" {
        return Err(fail("o arquivo não é um carimbo do QElectroTech"));
    }
    let grid = root
        .child("grid")
        .ok_or_else(|| fail("carimbo sem grade"))?;
    let (sw, sh, m, s) = (
        options.sheet.width(),
        options.sheet.height(),
        options.margin.max(0.0),
        options.mm_per_px,
    );
    if !(s.is_finite() && s > 0.0) || sw <= 2.0 * m || sh <= 2.0 * m {
        return Err(fail("folha, margem ou escala inválidas"));
    }
    let width = sw - 2.0 * m;
    let mut b = Builder::default();
    let mut cols = column_widths(grid.attr("cols").unwrap_or(""), width, s)?;
    let total: f64 = cols.iter().sum();
    if total > width + 1e-6 {
        b.report
            .approximated("colunas do carimbo reduzidas para caber na folha");
        for c in &mut cols {
            *c *= width / total;
        }
    }
    let rows: Vec<f64> = grid
        .attr("rows")
        .unwrap_or("")
        .split(';')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|p| {
            p.trim_end_matches("px")
                .parse::<f64>()
                .map(|v| v * s)
                .map_err(|_| fail(format!("linha inválida no carimbo: '{p}'")))
        })
        .collect::<ImportResult<_>>()?;
    if cols.is_empty() || rows.is_empty() {
        return Err(fail("carimbo sem colunas ou linhas"));
    }
    let height: f64 = rows.iter().sum();
    if height > sh - 2.0 * m {
        return Err(fail("carimbo mais alto que a folha"));
    }
    let x_at = |i: usize| m + cols[..i.min(cols.len())].iter().sum::<f64>();
    let top = m + height;
    let y_at = |i: usize| top - rows[..i.min(rows.len())].iter().sum::<f64>();

    let mut logos: BTreeMap<String, (String, String)> = BTreeMap::new();
    if let Some(l) = root.child("logos") {
        for logo in l.children_named("logo") {
            logos.insert(
                logo.attr("name").unwrap_or("").to_owned(),
                (
                    logo.attr("type").unwrap_or("").to_ascii_lowercase(),
                    logo.text.clone(),
                ),
            );
        }
    }

    b.rect((m, m), (sw - m, sh - m), BORDER_WIDTH);
    let mut edges: BTreeSet<(i64, i64, i64, i64)> = BTreeSet::new();
    let key = |v: f64| (v * 1000.0).round() as i64;
    for cell in &grid.children {
        let attr = |k: &str| cell.attr(k).and_then(|v| v.parse::<usize>().ok());
        let (Some(row), Some(col)) = (attr("row"), attr("col")) else {
            continue;
        };
        if row >= rows.len() || col >= cols.len() {
            b.report.ignored("célula do carimbo fora da grade");
            continue;
        }
        let row_end = (row + attr("rowspan").unwrap_or(0) + 1).min(rows.len());
        let col_end = (col + attr("colspan").unwrap_or(0) + 1).min(cols.len());
        let (x0, x1) = (x_at(col), x_at(col_end));
        let (y1, y0) = (y_at(row), y_at(row_end));
        for (a, z) in [
            ((x0, y0), (x1, y0)),
            ((x1, y0), (x1, y1)),
            ((x0, y1), (x1, y1)),
            ((x0, y0), (x0, y1)),
        ] {
            edges.insert((key(a.0), key(a.1), key(z.0), key(z.1)));
        }
        match cell.name.as_str() {
            "field" => field(&mut b, cell, (x0, y0, x1, y1), s),
            "logo" => {
                let resource = cell.attr("resource").unwrap_or("");
                match logos.get(resource) {
                    Some((kind, data)) if matches!(kind.as_str(), "png" | "jpg" | "jpeg") => {
                        match base64_decode(data) {
                            Some(bytes) => logo(&mut b, bytes, (x0, y0, x1, y1), resource),
                            None => b
                                .report
                                .ignored(format!("logotipo '{resource}' com base64 inválido")),
                        }
                    }
                    Some((kind, _)) => b.report.ignored(format!(
                        "logotipo '{resource}' em formato {kind} (sem suporte)"
                    )),
                    None => b
                        .report
                        .ignored(format!("célula de logotipo sem imagem ('{resource}')")),
                }
            }
            _ => {}
        }
    }
    for (x0, y0, x1, y1) in edges {
        let f = |v: i64| v as f64 / 1000.0;
        b.line((f(x0), f(y0)), (f(x1), f(y1)), LINE_WIDTH);
    }
    b.report.info(format!(
        "carimbo de {} × {} células, {:.1} × {:.1} mm ({} mm por px)",
        cols.len(),
        rows.len(),
        cols.iter().sum::<f64>(),
        height,
        s
    ));
    let name = root.attr("name").unwrap_or("Carimbo QElectroTech");
    let description = root
        .child("information")
        .map(|i| i.text.trim().to_owned())
        .unwrap_or_default();
    let usable = (m, round3(top), width, round3(sh - m - top));
    b.finish(
        name,
        &description,
        Some((sw, sh)),
        Some(usable),
        ZoneLayout {
            columns: ZoneAxis::none(),
            rows: ZoneAxis::none(),
        },
        false,
    )
}

fn field(b: &mut Builder, cell: &Element, (x0, y0, x1, y1): (f64, f64, f64, f64), s: f64) {
    let font = cell
        .attr("fontsize")
        .and_then(|v| v.parse::<f64>().ok())
        .filter(|v| *v > 0.0)
        .unwrap_or(DEFAULT_FONT);
    let h = (font * s).min((y1 - y0) * 0.8).max(0.5);
    let mut parts: Vec<Token> = Vec::new();
    if cell.attr("displaylabel") == Some("true") {
        let label = translations(cell.child("label"));
        if !label.default.is_empty() {
            parts.push(Token::Literal(format!("{}:", label.default)));
        }
    }
    let value = translations(cell.child("value"));
    parts.extend(tokens(&value.default));
    if parts.is_empty() {
        return;
    }
    if parts.len() > 1 {
        b.report
            .approximated("célula com rótulo/variáveis dividida em textos separados");
    }
    // Largura estimada (0,6 × altura por caractere; campo especial ≈ 8).
    let widths: Vec<f64> = parts
        .iter()
        .map(|p| match p {
            Token::Literal(t) => t.chars().count() as f64 * 0.6 * h,
            Token::Variable(_) => 8.0 * 0.6 * h,
        } + 0.6 * h)
        .collect();
    let total: f64 = widths.iter().sum();
    let pad = 0.8;
    let mut x = match cell.attr("align") {
        Some("center") => (x0 + x1) / 2.0 - total / 2.0,
        Some("right") => x1 - pad - total,
        _ => x0 + pad,
    }
    .max(x0 + pad / 2.0);
    let (y, align) = match cell.attr("valign") {
        Some("top") => (y1 - pad, TextAlign::TopLeft),
        Some("bottom") => (y0 + pad, TextAlign::BottomLeft),
        _ => ((y0 + y1) / 2.0, TextAlign::MiddleLeft),
    };
    let style = FrameTextStyle::new(round3(h), align);
    // Valor só com texto fixo (sem rótulo): mantém as traduções.
    let single = parts.len() == 1;
    for (part, w) in parts.into_iter().zip(widths) {
        let at = (round3(x.min(x1 - pad)), round3(y));
        match part {
            Token::Literal(t) => {
                let text = if single && t == value.default {
                    value.clone()
                } else {
                    LocalizedText::new(t.trim())
                };
                b.text(at, text, style);
            }
            Token::Variable(v) => b.special(at, qet_variable(&v), None, style),
        }
        x += w;
    }
}

fn logo(b: &mut Builder, bytes: Vec<u8>, (x0, y0, x1, y1): (f64, f64, f64, f64), name: &str) {
    let Ok((_, wpx, hpx)) = inspect_image(&bytes) else {
        b.report.ignored(format!("logotipo '{name}' inválido"));
        return;
    };
    let (cw, ch) = ((x1 - x0) - 1.0, (y1 - y0) - 1.0);
    if cw <= 0.0 || ch <= 0.0 {
        b.report
            .ignored(format!("célula pequena demais para o logotipo '{name}'"));
        return;
    }
    let aspect = wpx as f64 / hpx as f64;
    let (w, h) = if cw / ch > aspect {
        (ch * aspect, ch)
    } else {
        (cw, cw / aspect)
    };
    let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    b.image(
        (round3(cx - w / 2.0), round3(cy - h / 2.0)),
        (round3(cx + w / 2.0), round3(cy + h / 2.0)),
        bytes,
        name,
    );
}
