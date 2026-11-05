// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! DXF ASCII: leitor e escritor próprios, só das entidades de moldura.
//!
//! Leitura: `LINE`, `LWPOLYLINE`/`POLYLINE` (com arcos por bulge),
//! `CIRCLE`, `ARC`, `TEXT`, `MTEXT` simples, `ATTDEF`/`ATTRIB` e blocos
//! (`INSERT` expandido). Textos `<<código>>` ou `%{código}` e atributos viram
//! textos especiais. Escrita: DXF R12 (AC1009) em mm; a espessura vai no nome
//! da camada (`CAD-CRAFT 0.35mm`) e área útil/zonas num comentário `999`,
//! para que o CAD-CRAFT reimporte a moldura sem perdas de geometria.
use super::build::{Builder, round3};
use super::{
    ImportReport, ImportResult, ImportedFrame, check_size, fail, parse_special_code, qet_variable,
    special_code,
};
use cadcraft_model::domain::{
    FrameElement, FrameTemplate, FrameTextStyle, LocalizedText, Size2D, SpecialField, Text,
    TextAlign, ZoneAxis, ZoneLayout,
};
use std::collections::BTreeMap;
use std::fmt::Write as _;

const DEFAULT_WIDTH: f64 = 0.25;
const MAX_BLOCK_DEPTH: usize = 8;
const MAX_ELEMENTS: usize = 100_000;
const META_PREFIX: &str = "CAD-CRAFT moldura ";
const LAYER_PREFIX: &str = "CAD-CRAFT ";
const WATERMARK_LAYER: &str = "CAD-CRAFT marca d'agua";

#[derive(Debug, Clone)]
pub struct DxfImportOptions {
    pub name: String,
    /// mm por unidade do desenho; `None` = do cabeçalho (`$INSUNITS`) ou mm.
    pub unit_mm: Option<f64>,
    /// Folha imposta; `None` = limites do desenho ou envoltória.
    pub sheet: Option<Size2D>,
}
impl DxfImportOptions {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            unit_mm: None,
            sheet: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct Entity {
    kind: String,
    groups: Vec<(i32, String)>,
    /// Vértices de `POLYLINE` e atributos de `INSERT`.
    children: Vec<Entity>,
}
impl Entity {
    fn get(&self, code: i32) -> Option<&str> {
        self.groups
            .iter()
            .find(|(c, _)| *c == code)
            .map(|(_, v)| v.as_str())
    }
    fn num(&self, code: i32) -> Option<f64> {
        self.get(code)
            .and_then(|v| v.trim().parse::<f64>().ok())
            .filter(|v| v.is_finite())
    }
    fn num_or(&self, code: i32, default: f64) -> f64 {
        self.num(code).unwrap_or(default)
    }
    fn int(&self, code: i32) -> i64 {
        self.get(code)
            .and_then(|v| v.trim().parse::<i64>().ok())
            .unwrap_or(0)
    }
}

struct Block {
    base: (f64, f64),
    entities: Vec<Entity>,
}

/// Texto do arquivo: UTF-8 ou, se inválido, Latin-1 (DXF antigos).
fn decode(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.trim_start_matches('\u{feff}').to_owned(),
        Err(_) => bytes.iter().map(|&b| b as char).collect(),
    }
}

fn pairs(text: &str) -> ImportResult<Vec<(i32, String)>> {
    let mut lines = text.lines();
    let mut out = Vec::new();
    while let Some(code) = lines.next() {
        let code = code.trim();
        if code.is_empty() && out.is_empty() {
            continue;
        }
        let code: i32 = code
            .parse()
            .map_err(|_| fail(format!("DXF inválido: código de grupo '{code}'")))?;
        let value = lines
            .next()
            .ok_or_else(|| fail("DXF truncado (valor ausente)"))?;
        out.push((code, value.trim_end_matches('\r').trim().to_owned()));
        if out.len() > 20_000_000 {
            return Err(fail("DXF grande demais"));
        }
    }
    Ok(out)
}

struct Parsed {
    header: BTreeMap<String, Vec<(i32, String)>>,
    blocks: BTreeMap<String, Block>,
    entities: Vec<Entity>,
    meta: Option<String>,
}

fn parse(bytes: &[u8]) -> ImportResult<Parsed> {
    check_size(bytes)?;
    let text = decode(bytes);
    let pairs = pairs(&text)?;
    if !pairs.iter().any(|(c, v)| *c == 0 && v == "SECTION") {
        return Err(fail("o arquivo não é um DXF ASCII"));
    }
    if !pairs.iter().any(|(c, v)| *c == 0 && v == "EOF") {
        return Err(fail("DXF truncado (sem EOF)"));
    }
    let mut parsed = Parsed {
        header: BTreeMap::new(),
        blocks: BTreeMap::new(),
        entities: Vec::new(),
        meta: None,
    };
    let mut i = 0;
    while i < pairs.len() {
        let (code, value) = &pairs[i];
        if *code == 999 && value.starts_with(META_PREFIX) {
            parsed.meta = Some(value[META_PREFIX.len()..].to_owned());
        }
        if *code == 0 && value == "SECTION" && i + 1 < pairs.len() && pairs[i + 1].0 == 2 {
            let name = pairs[i + 1].1.clone();
            let end = (i + 2..pairs.len())
                .find(|&j| pairs[j].0 == 0 && pairs[j].1 == "ENDSEC")
                .ok_or_else(|| fail(format!("DXF truncado (seção {name} sem ENDSEC)")))?;
            let body = &pairs[i + 2..end];
            match name.as_str() {
                "HEADER" => {
                    let mut current: Option<String> = None;
                    for (c, v) in body {
                        if *c == 9 {
                            current = Some(v.clone());
                        } else if let Some(k) = &current {
                            parsed
                                .header
                                .entry(k.clone())
                                .or_default()
                                .push((*c, v.clone()));
                        }
                    }
                }
                "BLOCKS" => {
                    for block in entities(body)? {
                        if block.kind == "BLOCK" {
                            let name = block.get(2).unwrap_or("").to_owned();
                            parsed.blocks.insert(
                                name,
                                Block {
                                    base: (block.num_or(10, 0.0), block.num_or(20, 0.0)),
                                    entities: block.children,
                                },
                            );
                        }
                    }
                }
                "ENTITIES" => parsed.entities = entities(body)?,
                _ => {}
            }
            i = end;
        }
        i += 1;
    }
    Ok(parsed)
}

/// Agrupa pares em entidades; `POLYLINE`/`INSERT` recebem os filhos até
/// `SEQEND`, `BLOCK` recebe as entidades até `ENDBLK`.
fn entities(body: &[(i32, String)]) -> ImportResult<Vec<Entity>> {
    let mut flat: Vec<Entity> = Vec::new();
    for (c, v) in body {
        if *c == 0 {
            flat.push(Entity {
                kind: v.clone(),
                ..Default::default()
            });
        } else if let Some(e) = flat.last_mut() {
            e.groups.push((*c, v.clone()));
        }
    }
    let mut out: Vec<Entity> = Vec::new();
    let mut block: Option<Entity> = None;
    let mut open: Option<Entity> = None;
    for e in flat {
        let target = |block: &mut Option<Entity>, out: &mut Vec<Entity>, e: Entity| match block {
            Some(b) => b.children.push(e),
            None => out.push(e),
        };
        match e.kind.as_str() {
            "BLOCK" => block = Some(e),
            "ENDBLK" => {
                if let Some(p) = open.take() {
                    target(&mut block, &mut out, p);
                }
                if let Some(b) = block.take() {
                    out.push(b);
                }
            }
            "VERTEX" | "ATTRIB" if open.is_some() => {
                open.as_mut().expect("aberto").children.push(e);
            }
            "SEQEND" => {
                if let Some(p) = open.take() {
                    target(&mut block, &mut out, p);
                }
            }
            _ => {
                if let Some(p) = open.take() {
                    target(&mut block, &mut out, p);
                }
                let has_children = e.kind == "POLYLINE" || (e.kind == "INSERT" && e.int(66) == 1);
                if has_children {
                    open = Some(e);
                } else {
                    target(&mut block, &mut out, e);
                }
            }
        }
    }
    if let Some(p) = open {
        out.push(p);
    }
    Ok(out)
}

/// Transformação afim 2D: `p' = m · p + t`.
#[derive(Clone, Copy)]
struct Xf {
    m: [[f64; 2]; 2],
    t: (f64, f64),
}
impl Xf {
    fn scale(s: f64) -> Self {
        Self {
            m: [[s, 0.0], [0.0, s]],
            t: (0.0, 0.0),
        }
    }
    fn apply(&self, p: (f64, f64)) -> (f64, f64) {
        (
            self.m[0][0] * p.0 + self.m[0][1] * p.1 + self.t.0,
            self.m[1][0] * p.0 + self.m[1][1] * p.1 + self.t.1,
        )
    }
    fn then(&self, inner: &Xf) -> Xf {
        // self ∘ inner
        let a = self.m;
        let b = inner.m;
        let m = [
            [
                a[0][0] * b[0][0] + a[0][1] * b[1][0],
                a[0][0] * b[0][1] + a[0][1] * b[1][1],
            ],
            [
                a[1][0] * b[0][0] + a[1][1] * b[1][0],
                a[1][0] * b[0][1] + a[1][1] * b[1][1],
            ],
        ];
        Xf {
            m,
            t: self.apply(inner.t),
        }
    }
    fn det(&self) -> f64 {
        self.m[0][0] * self.m[1][1] - self.m[0][1] * self.m[1][0]
    }
    /// Escalas ao longo dos eixos locais.
    fn scales(&self) -> (f64, f64) {
        (
            self.m[0][0].hypot(self.m[1][0]),
            self.m[0][1].hypot(self.m[1][1]),
        )
    }
    fn angle(&self) -> f64 {
        self.m[1][0].atan2(self.m[0][0]).to_degrees()
    }
}

fn width_of(e: &Entity) -> f64 {
    let layer = e.get(8).unwrap_or("");
    if let Some(w) = layer
        .strip_prefix(LAYER_PREFIX)
        .and_then(|r| r.strip_suffix("mm"))
        .and_then(|w| w.trim().parse::<f64>().ok())
    {
        return w;
    }
    match e.num(370) {
        Some(lw) if lw >= 0.0 => lw / 100.0,
        _ => DEFAULT_WIDTH,
    }
}

/// Remove códigos de formatação e traduz escapes de texto DXF.
fn clean_text(raw: &str, mtext: bool) -> String {
    let mut out = String::new();
    let chars: Vec<char> = raw.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && i + 1 < chars.len() {
            let n = chars[i + 1];
            if n == 'U' && i + 6 < chars.len() && chars[i + 2] == '+' {
                let hex: String = chars[i + 3..i + 7].iter().collect();
                if let Some(ch) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                    out.push(ch);
                    i += 7;
                    continue;
                }
            }
            if mtext {
                match n {
                    'P' => {
                        out.push('\n');
                        i += 2;
                        continue;
                    }
                    '~' => {
                        out.push(' ');
                        i += 2;
                        continue;
                    }
                    '\\' | '{' | '}' => {
                        out.push(n);
                        i += 2;
                        continue;
                    }
                    'L' | 'l' | 'O' | 'o' | 'K' | 'k' => {
                        i += 2;
                        continue;
                    }
                    _ if n.is_ascii_alphabetic() => {
                        // \f…; \H…; \C…; etc.
                        while i < chars.len() && chars[i] != ';' {
                            i += 1;
                        }
                        i += 1;
                        continue;
                    }
                    _ => {}
                }
            }
        }
        if mtext && (c == '{' || c == '}') {
            i += 1;
            continue;
        }
        if !mtext && c == '%' && i + 2 < chars.len() && chars[i + 1] == '%' {
            let code = chars[i + 2].to_ascii_lowercase();
            let mapped = match code {
                'c' => Some("Ø"),
                'd' => Some("°"),
                'p' => Some("±"),
                '%' => Some("%"),
                'u' | 'o' => Some(""),
                _ => None,
            };
            if let Some(m) = mapped {
                out.push_str(m);
                i += 3;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

/// `<<código>>`, `%{código}` → campo especial.
fn placeholder(text: &str) -> Option<(SpecialField, Option<u32>)> {
    let t = text.trim();
    if let Some(inner) = t.strip_prefix("<<").and_then(|r| r.strip_suffix(">>")) {
        return Some(parse_special_code(inner).unwrap_or_else(|| {
            (
                SpecialField::PageProperty(
                    Text::new(inner.trim())
                        .ok()
                        .unwrap_or_else(|| Text::new("DXF").expect("fixo")),
                ),
                None,
            )
        }));
    }
    let inner = t.strip_prefix("%{")?.strip_suffix('}')?;
    Some(parse_special_code(inner).unwrap_or_else(|| (qet_variable(inner), None)))
}

fn text_align(h: i64, v: i64) -> TextAlign {
    use TextAlign::*;
    match (h, v) {
        (1, 3) => TopCenter,
        (2, 3) => TopRight,
        (_, 3) => TopLeft,
        (1, 2) | (4, _) => Center,
        (2, 2) => MiddleRight,
        (_, 2) => MiddleLeft,
        (1, _) => BottomCenter,
        (2, _) => BottomRight,
        _ => BottomLeft,
    }
}
fn mtext_align(attachment: i64) -> TextAlign {
    use TextAlign::*;
    match attachment {
        1 => TopLeft,
        2 => TopCenter,
        3 => TopRight,
        4 => MiddleLeft,
        5 => Center,
        6 => MiddleRight,
        8 => BottomCenter,
        9 => BottomRight,
        _ => BottomLeft,
    }
}

struct Ctx<'a> {
    b: Builder,
    blocks: &'a BTreeMap<String, Block>,
}

struct TextSpec<'a> {
    at: (f64, f64),
    raw: &'a str,
    height: f64,
    rot: f64,
    align: TextAlign,
    watermark: bool,
    tag: Option<&'a str>,
}
fn text_element(ctx: &mut Ctx<'_>, xf: &Xf, spec: TextSpec<'_>) {
    let TextSpec {
        at,
        raw,
        height,
        rot,
        align,
        watermark,
        tag,
    } = spec;
    let (sx, sy) = xf.scales();
    let mut style = FrameTextStyle::new(round3(height * sy.max(1e-12)), align);
    style.rotation_degrees = round3(rot + xf.angle());
    style.watermark = watermark;
    let at = xf.apply(at);
    if (sx - sy).abs() > 1e-6 {
        ctx.b
            .report
            .approximated("texto em bloco com escala não uniforme");
    }
    if let Some(tag) = tag {
        let field = parse_special_code(tag).unwrap_or_else(|| {
            (
                SpecialField::PageProperty(
                    Text::new(tag.trim()).unwrap_or_else(|_| Text::new("DXF").expect("fixo")),
                ),
                None,
            )
        });
        ctx.b.special(at, field.0, field.1, style);
        return;
    }
    match placeholder(raw) {
        Some((field, index)) => ctx.b.special(at, field, index, style),
        None => {
            if raw.contains("<<") || raw.contains("%{") {
                ctx.b
                    .report
                    .approximated("texto com campo misturado a texto fixo mantido como texto fixo");
            }
            ctx.b.text(at, LocalizedText::new(raw), style);
        }
    }
}

fn emit(ctx: &mut Ctx<'_>, list: &[Entity], xf: &Xf, depth: usize) -> ImportResult<()> {
    for e in list {
        if ctx.b.elements.len() > MAX_ELEMENTS {
            return Err(fail("DXF com elementos demais para uma moldura"));
        }
        let w = width_of(e);
        let p = |cx: i32, cy: i32| (e.num_or(cx, 0.0), e.num_or(cy, 0.0));
        match e.kind.as_str() {
            "LINE" => ctx.b.line(xf.apply(p(10, 20)), xf.apply(p(11, 21)), w),
            "CIRCLE" | "ARC" => {
                let (sx, sy) = xf.scales();
                if (sx - sy).abs() > 1e-6 {
                    ctx.b
                        .report
                        .approximated("círculo/arco em bloco com escala não uniforme");
                }
                let c = xf.apply(p(10, 20));
                let r = e.num_or(40, 0.0) * (sx + sy) / 2.0;
                if e.kind == "CIRCLE" {
                    ctx.b.circle(c, r, w);
                } else {
                    let (a0, a1) = (e.num_or(50, 0.0), e.num_or(51, 360.0));
                    let mut sweep = (a1 - a0).rem_euclid(360.0);
                    if sweep < 1e-9 {
                        sweep = 360.0;
                    }
                    // Direção do início após a transformação (espelho inverte o sentido).
                    let dir = |deg: f64| {
                        let (s, c0) = deg.to_radians().sin_cos();
                        let q = xf.apply((c0, s));
                        let o = xf.apply((0.0, 0.0));
                        (q.1 - o.1).atan2(q.0 - o.0).to_degrees()
                    };
                    let start = dir(a0);
                    let sweep = if xf.det() < 0.0 { -sweep } else { sweep };
                    ctx.b.arc(c, r, round3(start), sweep, w);
                }
            }
            "LWPOLYLINE" => {
                let mut v: Vec<((f64, f64), f64)> = Vec::new();
                for (c, val) in &e.groups {
                    let n = val.trim().parse::<f64>().unwrap_or(0.0);
                    match c {
                        10 => v.push(((n, 0.0), 0.0)),
                        20 => {
                            if let Some(last) = v.last_mut() {
                                last.0.1 = n;
                            }
                        }
                        42 => {
                            if let Some(last) = v.last_mut() {
                                last.1 = n;
                            }
                        }
                        _ => {}
                    }
                }
                polyline(ctx, xf, &v, e.int(70) & 1 == 1, w);
            }
            "POLYLINE" => {
                if e.int(70) & (8 | 16 | 64) != 0 {
                    ctx.b.report.ignored("polilinha 3D ou malha");
                    continue;
                }
                let v: Vec<((f64, f64), f64)> = e
                    .children
                    .iter()
                    .filter(|c| c.kind == "VERTEX")
                    .map(|c| ((c.num_or(10, 0.0), c.num_or(20, 0.0)), c.num_or(42, 0.0)))
                    .collect();
                polyline(ctx, xf, &v, e.int(70) & 1 == 1, w);
            }
            "TEXT" | "ATTDEF" | "ATTRIB" => {
                let (h, v) = (e.int(72), e.int(73));
                let at = if (h != 0 || v != 0) && e.num(11).is_some() {
                    p(11, 21)
                } else {
                    p(10, 20)
                };
                let raw = clean_text(e.get(1).unwrap_or(""), false);
                let tag =
                    matches!(e.kind.as_str(), "ATTDEF" | "ATTRIB").then(|| e.get(2).unwrap_or(""));
                let tag = tag.filter(|t| !t.trim().is_empty());
                if e.kind == "ATTDEF" && depth > 0 {
                    continue; // definição de atributo dentro de bloco inserido
                }
                if tag.is_none() && raw.trim().is_empty() {
                    continue;
                }
                let watermark = e.get(8) == Some(WATERMARK_LAYER);
                text_element(
                    ctx,
                    xf,
                    TextSpec {
                        at,
                        raw: &raw,
                        height: e.num_or(40, 2.5),
                        rot: e.num_or(50, 0.0),
                        align: text_align(h, if e.kind == "TEXT" { v } else { e.int(74) }),
                        watermark,
                        tag,
                    },
                );
            }
            "MTEXT" => {
                let mut raw = String::new();
                for (c, val) in &e.groups {
                    if *c == 3 {
                        raw.push_str(val);
                    }
                }
                raw.push_str(e.get(1).unwrap_or(""));
                let raw = clean_text(&raw, true);
                if raw.trim().is_empty() {
                    continue;
                }
                if raw.contains('\n') {
                    ctx.b
                        .report
                        .approximated("texto de várias linhas (MTEXT) mantido em um texto");
                }
                let rot = match e.num(50) {
                    Some(r) => r,
                    None => e.num_or(21, 0.0).atan2(e.num_or(11, 1.0)).to_degrees(),
                };
                let watermark = e.get(8) == Some(WATERMARK_LAYER);
                text_element(
                    ctx,
                    xf,
                    TextSpec {
                        at: p(10, 20),
                        raw: &raw,
                        height: e.num_or(40, 2.5),
                        rot,
                        align: mtext_align(e.int(71)),
                        watermark,
                        tag: None,
                    },
                );
            }
            "INSERT" => {
                if depth >= MAX_BLOCK_DEPTH {
                    ctx.b
                        .report
                        .ignored("bloco aninhado em profundidade excessiva");
                    continue;
                }
                let name = e.get(2).unwrap_or("");
                let Some(block) = ctx.blocks.get(name) else {
                    ctx.b.report.ignored(format!("bloco '{name}' inexistente"));
                    continue;
                };
                let (sx, sy) = (e.num_or(41, 1.0), e.num_or(42, 1.0));
                let rot = e.num_or(50, 0.0).to_radians();
                let (cols, rows) = (e.int(70).max(1), e.int(71).max(1));
                let (dc, dr) = (e.num_or(44, 0.0), e.num_or(45, 0.0));
                let entities = block.entities.clone();
                let base = block.base;
                for ci in 0..cols.min(100) {
                    for ri in 0..rows.min(100) {
                        let (c, s) = (rot.cos(), rot.sin());
                        let ox = ci as f64 * dc;
                        let oy = ri as f64 * dr;
                        let local = Xf {
                            m: [[c * sx, -s * sy], [s * sx, c * sy]],
                            t: (
                                e.num_or(10, 0.0) + c * ox
                                    - s * oy
                                    - (c * sx * base.0 - s * sy * base.1),
                                e.num_or(20, 0.0) + s * ox + c * oy
                                    - (s * sx * base.0 + c * sy * base.1),
                            ),
                        };
                        emit(ctx, &entities, &xf.then(&local), depth + 1)?;
                    }
                }
                let attribs: Vec<Entity> = e
                    .children
                    .iter()
                    .filter(|c| c.kind == "ATTRIB")
                    .cloned()
                    .collect();
                emit(ctx, &attribs, xf, depth + 1)?;
            }
            "POINT" | "VIEWPORT" | "SEQEND" | "ENDBLK" => {}
            other => ctx
                .b
                .report
                .ignored(format!("entidade DXF {other} sem suporte")),
        }
    }
    Ok(())
}

fn polyline(ctx: &mut Ctx<'_>, xf: &Xf, v: &[((f64, f64), f64)], closed: bool, w: f64) {
    if v.len() < 2 {
        ctx.b.report.ignored("polilinha com menos de dois pontos");
        return;
    }
    let pts: Vec<(f64, f64)> = v.iter().map(|(p, _)| xf.apply(*p)).collect();
    let flip = if xf.det() < 0.0 { -1.0 } else { 1.0 };
    // Polilinha fechada de 4 lados retos e alinhados aos eixos = retângulo.
    let mut ring = pts.clone();
    if ring.len() == 5
        && (ring[0].0 - ring[4].0).abs() < 1e-9
        && (ring[0].1 - ring[4].1).abs() < 1e-9
    {
        ring.pop();
    }
    let straight = v.iter().all(|(_, b)| b.abs() < 1e-12);
    if straight && ring.len() == 4 && (closed || ring.len() < pts.len()) {
        let axis = (0..4).all(|i| {
            let (a, b) = (ring[i], ring[(i + 1) % 4]);
            (a.0 - b.0).abs() < 1e-9 || (a.1 - b.1).abs() < 1e-9
        });
        if axis {
            let xs = ring.iter().map(|p| p.0);
            let ys = ring.iter().map(|p| p.1);
            let (x0, x1) = (
                xs.clone().fold(f64::MAX, f64::min),
                xs.fold(f64::MIN, f64::max),
            );
            let (y0, y1) = (
                ys.clone().fold(f64::MAX, f64::min),
                ys.fold(f64::MIN, f64::max),
            );
            ctx.b.rect((x0, y0), (x1, y1), w);
            return;
        }
    }
    let n = pts.len();
    let segments = if closed { n } else { n - 1 };
    for i in 0..segments {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        ctx.b.segment(a, b, v[i].1 * flip, w);
    }
}

fn unit_mm(code: i64) -> Option<f64> {
    Some(match code {
        1 => 25.4,
        2 => 304.8,
        4 => 1.0,
        5 => 10.0,
        6 => 1000.0,
        8 => 25.4e-6,
        9 => 25.4e-3,
        10 => 914.4,
        13 => 1e-3,
        14 => 100.0,
        _ => return None,
    })
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Meta {
    name: String,
    description: String,
    usable: [f64; 4],
    zones: ZoneLayout,
}

pub fn import(bytes: &[u8], options: &DxfImportOptions) -> ImportResult<ImportedFrame> {
    let parsed = parse(bytes)?;
    let header_num = |var: &str, code: i32| -> Option<f64> {
        parsed
            .header
            .get(var)?
            .iter()
            .find(|(c, _)| *c == code)?
            .1
            .trim()
            .parse::<f64>()
            .ok()
    };
    let mut report = ImportReport::default();
    let unit = match options.unit_mm {
        Some(u) => u,
        None => match header_num("$INSUNITS", 70).map(|c| c as i64) {
            Some(code) => match unit_mm(code) {
                Some(u) => u,
                None => {
                    report.info("unidade do desenho não informada: usado milímetro");
                    1.0
                }
            },
            None => {
                report.info("unidade do desenho não informada: usado milímetro");
                1.0
            }
        },
    };
    if !(unit.is_finite() && unit > 0.0) {
        return Err(fail("unidade inválida"));
    }
    let mut ctx = Ctx {
        b: Builder {
            report,
            ..Default::default()
        },
        blocks: &parsed.blocks,
    };
    emit(&mut ctx, &parsed.entities, &Xf::scale(unit), 0)?;
    let meta: Option<Meta> = parsed
        .meta
        .as_deref()
        .and_then(|m| serde_json::from_str(m).ok());
    // Folha: imposta, limites do desenho ($LIMMIN/$LIMMAX) ou envoltória.
    let limits = match (
        header_num("$LIMMIN", 10),
        header_num("$LIMMIN", 20),
        header_num("$LIMMAX", 10),
        header_num("$LIMMAX", 20),
    ) {
        (Some(x0), Some(y0), Some(x1), Some(y1)) if x1 > x0 && y1 > y0 && meta.is_some() => {
            Some((x0 * unit, y0 * unit, x1 * unit, y1 * unit))
        }
        _ => None,
    };
    let mut b = ctx.b;
    let (sheet, to_origin) = match (options.sheet, limits) {
        (Some(s), _) => (Some((s.width(), s.height())), false),
        (None, Some((x0, y0, x1, y1))) => {
            if x0.abs() > 1e-9 || y0.abs() > 1e-9 {
                b.elements = b
                    .elements
                    .iter()
                    .map(|e| e.translated(-x0, -y0))
                    .collect::<cadcraft_model::error::Result<_>>()
                    .map_err(|e| fail(e.to_string()))?;
            }
            (Some((x1 - x0, y1 - y0)), false)
        }
        (None, None) => (None, true),
    };
    let (name, description, usable, zones) = match meta {
        Some(m) => (
            m.name,
            m.description,
            Some((m.usable[0], m.usable[1], m.usable[2], m.usable[3])),
            m.zones,
        ),
        None => (
            options.name.clone(),
            String::new(),
            None,
            ZoneLayout {
                columns: ZoneAxis::none(),
                rows: ZoneAxis::none(),
            },
        ),
    };
    b.finish(&name, &description, sheet, usable, zones, to_origin)
}

fn num(v: f64) -> String {
    let s = format!("{:.6}", round6(v));
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s.is_empty() || s == "-0" {
        "0".to_owned()
    } else {
        s.to_owned()
    }
}
fn round6(v: f64) -> f64 {
    (v * 1e6).round() / 1e6
}
/// Texto DXF R12: não-ASCII como `\U+XXXX`.
fn escape(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        match c {
            '\n' | '\r' => out.push(' '),
            c if c.is_ascii() && !c.is_ascii_control() => out.push(c),
            c if (c as u32) <= 0xFFFF => {
                let _ = write!(out, "\\U+{:04X}", c as u32);
            }
            _ => out.push('?'),
        }
    }
    out
}
fn align_codes(a: TextAlign) -> (i64, i64) {
    use TextAlign::*;
    match a {
        BottomLeft => (0, 1),
        BottomCenter => (1, 1),
        BottomRight => (2, 1),
        MiddleLeft => (0, 2),
        Center => (1, 2),
        MiddleRight => (2, 2),
        TopLeft => (0, 3),
        TopCenter => (1, 3),
        TopRight => (2, 3),
    }
}

/// Exporta a moldura em DXF R12 (mm). O relatório lista o que não cabe no
/// DXF (imagens; traduções ficam só no texto padrão).
pub fn export(frame: &FrameTemplate) -> (Vec<u8>, ImportReport) {
    let mut report = ImportReport::default();
    let mut out = String::new();
    let mut pair = |code: i32, value: &str| {
        let _ = writeln!(out, "{code:>3}\n{value}");
    };
    let meta = Meta {
        name: frame.name().as_str().to_owned(),
        description: frame.description().to_owned(),
        usable: [
            frame.usable().origin().x(),
            frame.usable().origin().y(),
            frame.usable().size().width(),
            frame.usable().size().height(),
        ],
        zones: frame.zones().clone(),
    };
    pair(
        999,
        &format!(
            "{META_PREFIX}{}",
            escape(&serde_json::to_string(&meta).expect("metadados serializáveis"))
        ),
    );
    pair(0, "SECTION");
    pair(2, "HEADER");
    pair(9, "$ACADVER");
    pair(1, "AC1009");
    pair(9, "$INSUNITS");
    pair(70, "4");
    for (var, x, y) in [
        ("$LIMMIN", 0.0, 0.0),
        ("$LIMMAX", frame.sheet().width(), frame.sheet().height()),
        ("$EXTMIN", 0.0, 0.0),
        ("$EXTMAX", frame.sheet().width(), frame.sheet().height()),
    ] {
        pair(9, var);
        pair(10, &num(x));
        pair(20, &num(y));
    }
    pair(0, "ENDSEC");
    pair(0, "SECTION");
    pair(2, "ENTITIES");
    let layer = |w: f64| format!("{LAYER_PREFIX}{}mm", num(w));
    for e in frame.elements() {
        match e {
            FrameElement::Line { start, end, width } => {
                pair(0, "LINE");
                pair(8, &layer(*width));
                for (c, v) in [
                    (10, start.x()),
                    (20, start.y()),
                    (30, 0.0),
                    (11, end.x()),
                    (21, end.y()),
                    (31, 0.0),
                ] {
                    pair(c, &num(v));
                }
            }
            FrameElement::Rectangle { rect, width } => {
                let (o, m) = (rect.origin(), rect.max());
                pair(0, "POLYLINE");
                pair(8, &layer(*width));
                pair(66, "1");
                pair(10, "0");
                pair(20, "0");
                pair(30, "0");
                pair(70, "1");
                for (x, y) in [
                    (o.x(), o.y()),
                    (m.x(), o.y()),
                    (m.x(), m.y()),
                    (o.x(), m.y()),
                ] {
                    pair(0, "VERTEX");
                    pair(8, &layer(*width));
                    pair(10, &num(x));
                    pair(20, &num(y));
                    pair(30, "0");
                }
                pair(0, "SEQEND");
                pair(8, &layer(*width));
            }
            FrameElement::Circle {
                center,
                radius,
                width,
            } => {
                pair(0, "CIRCLE");
                pair(8, &layer(*width));
                pair(10, &num(center.x()));
                pair(20, &num(center.y()));
                pair(30, "0");
                pair(40, &num(*radius));
            }
            FrameElement::Arc {
                center,
                radius,
                start_degrees,
                sweep_degrees,
                width,
            } => {
                let (a0, a1) = if *sweep_degrees >= 0.0 {
                    (*start_degrees, start_degrees + sweep_degrees)
                } else {
                    (start_degrees + sweep_degrees, *start_degrees)
                };
                pair(0, "ARC");
                pair(8, &layer(*width));
                pair(10, &num(center.x()));
                pair(20, &num(center.y()));
                pair(30, "0");
                pair(40, &num(*radius));
                pair(50, &num(a0.rem_euclid(360.0)));
                pair(
                    51,
                    &num(if (a1 - a0 - 360.0).abs() < 1e-9 {
                        a0.rem_euclid(360.0) + 360.0
                    } else {
                        a1.rem_euclid(360.0)
                    }),
                );
            }
            FrameElement::Text {
                origin,
                text,
                style,
            } => {
                if !text.translations.is_empty() {
                    report.info("traduções de textos não vão para o DXF (só o texto padrão)");
                }
                write_text(&mut pair, origin.x(), origin.y(), &text.default, style);
            }
            FrameElement::Special {
                origin,
                field,
                index,
                style,
            } => {
                write_text(
                    &mut pair,
                    origin.x(),
                    origin.y(),
                    &format!("<<{}>>", special_code(field, *index)),
                    style,
                );
            }
            FrameElement::Image { .. } => report.ignored("imagem (o DXF R12 não guarda imagens)"),
        }
    }
    pair(0, "ENDSEC");
    pair(0, "EOF");
    (out.into_bytes(), report)
}

fn write_text(
    pair: &mut impl FnMut(i32, &str),
    x: f64,
    y: f64,
    text: &str,
    style: &FrameTextStyle,
) {
    let (h, v) = align_codes(style.align);
    pair(0, "TEXT");
    pair(
        8,
        if style.watermark {
            WATERMARK_LAYER
        } else {
            "CAD-CRAFT textos"
        },
    );
    pair(10, &num(x));
    pair(20, &num(y));
    pair(30, "0");
    pair(40, &num(style.height));
    pair(1, &escape(text));
    if style.rotation_degrees != 0.0 {
        pair(50, &num(style.rotation_degrees));
    }
    pair(72, &h.to_string());
    pair(11, &num(x));
    pair(21, &num(y));
    pair(31, "0");
    pair(73, &v.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_escapes_and_mtext_codes_are_cleaned() {
        assert_eq!(clean_text("%%c10 %%d %%p1", false), "Ø10 ° ±1");
        assert_eq!(clean_text("\\U+00E7\\U+00E3o", false), "ção");
        assert_eq!(
            clean_text("{\\fArial|b0;Linha}\\PDois", true),
            "Linha\nDois"
        );
        assert_eq!(escape("Ação"), "A\\U+00E7\\U+00E3o");
    }
}
