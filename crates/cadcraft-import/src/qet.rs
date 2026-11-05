// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 25: projeto do QElectroTech (`.qet`, XML) → projeto CAD-CRAFT.
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
use super::elmt::{self, ElmtOptions};
use super::titleblock::{self, TitleblockOptions};
use super::xml::{self, Element};
use super::{ImportReport, ImportResult, check_size, fail};
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
fn domain(e: impl std::fmt::Display) -> super::ImportError {
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

impl Importer<'_> {
    fn collection(&mut self, e: &Element, path: &str) {
        for child in &e.children {
            let name = child.attr("name").unwrap_or("").trim();
            let key = format!("{path}/{name}");
            match child.name.as_str() {
                "category" => self.collection(child, &key),
                "element" => {
                    let Some(def) = child.child("definition") else {
                        self.report.ignored("elemento embutido sem definição");
                        continue;
                    };
                    let options = ElmtOptions {
                        mm_per_px: self.s,
                        category: "QElectroTech (importado)".into(),
                        id_hint: safe_id(&key.replace("embed://", "")),
                    };
                    match elmt::import_definition(def, &options) {
                        Ok(imported) => {
                            for (note, count) in imported.report.notes {
                                // As contagens por símbolo só poluem o relatório.
                                if note.kind == super::NoteKind::Info {
                                    continue;
                                }
                                for _ in 0..count {
                                    self.report.note(note.kind, note.message.clone());
                                }
                            }
                            let kind = def
                                .child("kindInformations")
                                .into_iter()
                                .flat_map(|k| k.children_named("kindInformation"))
                                .find(|k| k.attr("name") == Some("type"))
                                .map(|k| k.text.trim().to_owned())
                                .unwrap_or_default();
                            let terminals = def
                                .child("description")
                                .into_iter()
                                .flat_map(|d| d.children_named("terminal"))
                                .filter_map(|t| {
                                    Some((trim_uuid(t.attr("uuid")?), (num(t, "x")?, num(t, "y")?)))
                                })
                                .collect();
                            self.definitions.insert(
                                key,
                                Definition {
                                    symbol: imported.definition,
                                    link: def.attr("link_type").unwrap_or("simple").to_owned(),
                                    kind,
                                    terminals,
                                },
                            );
                        }
                        Err(e) => self
                            .report
                            .ignored(format!("elemento embutido {name}: {e}")),
                    }
                }
                _ => {}
            }
        }
    }

    fn sheet_of(&self, d: &Element, defaults: Option<&Element>) -> Sheet {
        let pick = |k: &str, fallback: f64| {
            num(d, k)
                .or_else(|| defaults.and_then(|b| num(b, k)))
                .unwrap_or(fallback)
        };
        let flag = |k: &str| {
            d.attr(k)
                .or_else(|| defaults.and_then(|b| b.attr(k)))
                .is_none_or(|v| v != "false")
        };
        let titleblock_px = d
            .attr("titleblocktemplate")
            .and_then(|n| self.templates.get(n))
            .and_then(|t| t.child("grid"))
            .and_then(|g| g.attr("rows"))
            .map(|rows| {
                rows.split(';')
                    .filter_map(|r| {
                        let digits: String = r
                            .chars()
                            .filter(|c| c.is_ascii_digit() || *c == '.')
                            .collect();
                        digits.parse::<f64>().ok()
                    })
                    .sum::<f64>()
            })
            .filter(|h| *h > 0.0)
            .unwrap_or(DEFAULT_TITLEBLOCK_PX);
        Sheet {
            cols: pick("cols", 17.0).clamp(1.0, 200.0) as usize,
            rows: pick("rows", 8.0).clamp(1.0, 200.0) as usize,
            colsize: pick("colsize", 60.0).clamp(5.0, 2000.0),
            rowsize: pick("rowsize", 80.0).clamp(5.0, 2000.0),
            show_cols: flag("displaycols"),
            show_rows: flag("displayrows"),
            titleblock_px,
        }
    }

    /// Moldura com a grade do QElectroTech (zonas) e o carimbo embutido.
    fn frame_for(&mut self, sheet: Sheet, template: Option<&str>) -> ImportResult<FramePackage> {
        let s = self.s;
        let mut assets = vec![];
        let (w, h) = (sheet.width_px() * s, sheet.height_px() * s);
        let size = Size2D::new(w, h).map_err(domain)?;
        let mut elements = vec![];
        let tpl = template.and_then(|n| self.templates.get(n).copied());
        match tpl.map(|t| {
            titleblock::import_template(
                t,
                TitleblockOptions {
                    sheet: size,
                    margin: MARGIN_PX * s,
                    mm_per_px: s,
                },
            )
        }) {
            Some(Ok(imported)) => {
                elements.extend(imported.package.frame.elements().iter().cloned());
                assets = imported.package.assets;
            }
            Some(Err(e)) => self.report.approximated(format!("carimbo embutido: {e}")),
            None => {
                if let Some(name) = template.filter(|n| !n.is_empty()) {
                    self.report
                        .approximated(format!("carimbo '{name}' não embutido: carimbo simples"));
                }
                let tb = sheet.titleblock_px * s;
                let m = MARGIN_PX * s;
                let rect = Rect2D::new(
                    Point2D::new(m, m).map_err(domain)?,
                    Size2D::new(w - 2.0 * m, tb).map_err(domain)?,
                )
                .map_err(domain)?;
                elements.push(FrameElement::Rectangle { rect, width: 0.35 });
                let style = FrameTextStyle::new(3.0, TextAlign::MiddleLeft);
                for (i, field) in [SpecialField::ProjectName, SpecialField::PageName]
                    .into_iter()
                    .enumerate()
                {
                    elements.push(FrameElement::Special {
                        origin: Point2D::new(m + 3.0, m + tb * (0.75 - 0.5 * i as f64))
                            .map_err(domain)?,
                        field,
                        index: None,
                        style,
                    });
                }
                elements.push(FrameElement::Special {
                    origin: Point2D::new(w - m - 30.0, m + tb / 2.0).map_err(domain)?,
                    field: SpecialField::PageNumber,
                    index: None,
                    style,
                });
            }
        }
        // Borda do desenho e área útil (abaixo dos cabeçalhos).
        let m = MARGIN_PX * s;
        let diagram_h = (HEADER_PX + sheet.rows as f64 * sheet.rowsize) * s;
        let diagram = Rect2D::new(
            Point2D::new(m, h - m - diagram_h).map_err(domain)?,
            Size2D::new(w - 2.0 * m, diagram_h).map_err(domain)?,
        )
        .map_err(domain)?;
        elements.push(FrameElement::Rectangle {
            rect: diagram,
            width: 0.35,
        });
        let usable = Rect2D::new(
            Point2D::new((MARGIN_PX + HEADER_PX) * s, h - m - diagram_h).map_err(domain)?,
            Size2D::new(
                sheet.cols as f64 * sheet.colsize * s,
                sheet.rows as f64 * sheet.rowsize * s,
            )
            .map_err(domain)?,
        )
        .map_err(domain)?;
        elements.push(FrameElement::Rectangle {
            rect: usable,
            width: 0.18,
        });
        let mut columns = ZoneAxis::uniform(
            sheet.cols,
            sheet.colsize * s,
            ZoneLabelStyle::Numbers { start: 1 },
        );
        columns.labels_start = sheet.show_cols;
        let mut rows = ZoneAxis::uniform(
            sheet.rows,
            sheet.rowsize * s,
            ZoneLabelStyle::Letters { start: 0 },
        );
        rows.labels_start = sheet.show_rows;
        let name = format!(
            "QElectroTech {}×{} ({}×{} px){}",
            sheet.cols,
            sheet.rows,
            sheet.colsize,
            sheet.rowsize,
            template.map(|t| format!(" {t}")).unwrap_or_default()
        );
        let frame = FrameTemplate::new(name, size, usable, ZoneLayout { columns, rows }, elements)
            .map_err(domain)?;
        Ok(FramePackage { frame, assets })
    }

    fn port_for(&self, inst: &Instance, terminal: &str) -> Option<String> {
        let def = self.definitions.get(&inst.def)?;
        let (x, y) = inst
            .terminals
            .iter()
            .find(|(id, _, _)| id == terminal)
            .map(|(_, x, y)| (*x, *y))
            .or_else(|| def.terminals.get(terminal).copied())?;
        let (lx, ly) = (x * self.s, -y * self.s);
        def.symbol
            .ports()
            .iter()
            .find(|p| {
                (p.position().x() - lx).abs() < PORT_TOLERANCE
                    && (p.position().y() - ly).abs() < PORT_TOLERANCE
            })
            .or_else(|| {
                let index = inst
                    .terminals
                    .iter()
                    .position(|(id, _, _)| id == terminal)?;
                def.symbol.ports().get(index)
            })
            .map(|p| p.id().to_owned())
    }
}

/// Importa um projeto `.qet`.
pub fn import(bytes: &[u8], options: &QetOptions) -> ImportResult<QetImport> {
    check_size(bytes)?;
    let s = options.mm_per_px;
    if !(s.is_finite() && s > 0.0) {
        return Err(fail("escala inválida"));
    }
    let root = xml::parse(bytes)?;
    if root.name != "project" {
        return Err(fail("o arquivo não é um projeto do QElectroTech"));
    }
    if count_nodes(&root) > MAX_QET_NODES {
        return Err(fail("projeto grande demais"));
    }
    let title = root
        .attr("title")
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .unwrap_or("Projeto QElectroTech");
    let mut importer = Importer {
        s,
        project: Project::new(title).map_err(domain)?,
        report: ImportReport::default(),
        stats: QetStats::default(),
        definitions: BTreeMap::new(),
        templates: BTreeMap::new(),
    };
    importer.report.info(format!(
        "QElectroTech {} · escala {s} mm/px · símbolos CC-BY 3.0 da coleção QElectroTech, quando houver",
        root.attr("version").unwrap_or("?")
    ));
    if let Some(templates) = root.child("titleblocktemplates") {
        for t in templates.children_named("titleblocktemplate") {
            if let Some(name) = t.attr("name") {
                importer.templates.insert(name.to_owned(), t);
            }
        }
    }
    if let Some(collection) = root.child("collection") {
        importer.collection(collection, "embed:/");
    }
    importer.stats.symbols = importer.definitions.len();
    let doc = importer
        .project
        .insert_document(Document::new(importer.project.id(), "Esquema").map_err(domain)?)
        .map_err(domain)?;
    let defaults = root.child("newdiagrams").and_then(|n| n.child("border"));
    let mut diagrams: Vec<&Element> = root.children_named("diagram").collect();
    diagrams.sort_by_key(|d| num(d, "order").map_or(usize::MAX, |o| o as usize));
    if diagrams.is_empty() {
        return Err(fail("projeto sem folhas"));
    }

    // Folhas e molduras.
    let mut pages = vec![];
    let mut frames: Vec<(Sheet, Option<String>, FrameTemplateId)> = vec![];
    let mut instances: Vec<Instance> = vec![];
    let mut wires: Vec<Wire> = vec![];
    let mut texts: Vec<(usize, f64, f64, String, f64)> = vec![];
    for (index, d) in diagrams.iter().enumerate() {
        let sheet = importer.sheet_of(d, defaults);
        let template = d.attr("titleblocktemplate").map(str::to_owned);
        let size = Size2D::new(sheet.width_px() * s, sheet.height_px() * s).map_err(domain)?;
        let name = d
            .attr("title")
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map_or_else(|| format!("Folha {}", index + 1), str::to_owned);
        let page = importer
            .project
            .insert_page(
                Page::new(
                    importer.project.id(),
                    doc,
                    name,
                    (index + 1).to_string(),
                    PageKind::MultiLine,
                    size,
                )
                .map_err(domain)?,
            )
            .map_err(domain)?;
        let frame = match frames
            .iter()
            .find(|(f, t, _)| *f == sheet && *t == template)
        {
            Some((_, _, id)) => Some(*id),
            None => match importer.frame_for(sheet, template.as_deref()) {
                Ok(package) => {
                    let (edits, id) = importer
                        .project
                        .frame_import_edits(&package)
                        .map_err(domain)?;
                    for edit in edits {
                        importer.project.edit_frame(edit).map_err(domain)?;
                    }
                    frames.push((sheet, template.clone(), id));
                    Some(id)
                }
                Err(e) => {
                    importer
                        .report
                        .approximated(format!("folha {}: moldura: {e}", index + 1));
                    None
                }
            },
        };
        if let Some(frame) = frame {
            importer
                .project
                .edit_frame(FrameEdit::SetPageFrame {
                    page,
                    frame: Some(frame),
                })
                .map_err(domain)?;
        }
        for (attr, key) in [
            ("author", "Autor"),
            ("date", "Data"),
            ("indexrev", "Revisão"),
        ] {
            if let Some(value) = d.attr(attr).map(str::trim).filter(|v| !v.is_empty()) {
                importer
                    .project
                    .edit_frame(FrameEdit::SetPageMetadata {
                        page,
                        key: Text::new(key).map_err(domain)?,
                        value: Some(value.to_owned()),
                    })
                    .map_err(domain)?;
            }
        }
        pages.push((page, sheet));
        for e in d
            .child("elements")
            .into_iter()
            .flat_map(|es| es.children_named("element"))
        {
            let terminals = e
                .child("terminals")
                .into_iter()
                .flat_map(|t| t.children_named("terminal"))
                .filter_map(|t| Some((t.attr("id")?.to_owned(), num(t, "x")?, num(t, "y")?)))
                .collect();
            let links = e
                .child("links_uuids")
                .into_iter()
                .flat_map(|l| l.children_named("link_uuid"))
                .filter_map(|l| l.attr("uuid").map(trim_uuid))
                .collect();
            // Textos dinâmicos da ocorrência (soltos ou em grupos).
            let dynamic: Vec<&Element> = e
                .child("dynamic_texts")
                .into_iter()
                .flat_map(|d| d.children_named("dynamic_elmt_text"))
                .chain(
                    e.child("texts_groups")
                        .into_iter()
                        .flat_map(|g| g.children_named("texts_group"))
                        .flat_map(|g| g.children_named("dynamic_elmt_text")),
                )
                .collect();
            let mut label_text = None;
            let mut user_texts = vec![];
            for t in dynamic {
                let (Some(x), Some(y)) = (num(t, "x"), num(t, "y")) else {
                    continue;
                };
                let height = elmt::font_height(t, s);
                let info_name = t.child("info_name").map(|i| i.text.trim().to_owned());
                match (t.attr("text_from").unwrap_or(""), info_name.as_deref()) {
                    ("ElementInfo", Some("label")) => {
                        label_text = Point2D::new(x * s, -y * s).ok().map(|origin| {
                            SymbolPrimitive::DynamicText {
                                origin,
                                field: SymbolTextField::Designation,
                                height,
                                align: elmt::align_of(t),
                                rotation_degrees: -num(t, "rotation").unwrap_or(0.0),
                            }
                        });
                    }
                    ("UserText", _) => {
                        let text = t
                            .child("text")
                            .map(|x| plain_text(&x.text))
                            .unwrap_or_default();
                        if !text.is_empty() {
                            user_texts.push((x * s, -y * s, text, height));
                        }
                    }
                    _ => importer
                        .report
                        .ignored("texto dinâmico da ocorrência sem equivalente"),
                }
            }
            instances.push(Instance {
                page: index,
                uuid: trim_uuid(e.attr("uuid").unwrap_or("")),
                def: e.attr("type").unwrap_or("").to_owned(),
                x: num(e, "x").unwrap_or(0.0),
                y: num(e, "y").unwrap_or(0.0),
                orientation: num(e, "orientation").unwrap_or(0.0) as u32,
                label: info(e, "label"),
                prefix: e.attr("prefix").unwrap_or("").to_owned(),
                links,
                terminals,
                label_text,
                user_texts,
            });
        }
        for c in d
            .child("conductors")
            .into_iter()
            .flat_map(|cs| cs.children_named("conductor"))
        {
            let (Some(t1), Some(t2)) = (c.attr("terminal1"), c.attr("terminal2")) else {
                importer.report.ignored("condutor sem terminais");
                continue;
            };
            let (e1, e2) = (
                c.attr("element1").map(trim_uuid),
                c.attr("element2").map(trim_uuid),
            );
            let key = |t: &str, e: &Option<String>| {
                if e.is_some() {
                    trim_uuid(t)
                } else {
                    t.to_owned()
                }
            };
            wires.push(Wire {
                page: index,
                t1: key(t1, &e1),
                t2: key(t2, &e2),
                e1,
                e2,
                num: c.attr("num").unwrap_or("").trim().to_owned(),
                segments: c
                    .children_named("segment")
                    .filter_map(|g| Some((g.attr("orientation")? == "vertical", num(g, "length")?)))
                    .collect(),
            });
        }
        for input in d
            .child("inputs")
            .into_iter()
            .flat_map(|i| i.children_named("input"))
        {
            let text = plain_text(input.attr("text").unwrap_or(""));
            if let (Some(x), Some(y)) = (num(input, "x"), num(input, "y"))
                && !text.is_empty()
            {
                texts.push((index, x, y, text, num(input, "size").unwrap_or(9.0)));
            }
        }
        for (tag, what) in [("images", "imagem"), ("shapes", "forma")] {
            let n = d.child(tag).map_or(0, |c| c.children.len());
            for _ in 0..n {
                importer
                    .report
                    .ignored(format!("{what} na folha (sem suporte nesta etapa)"));
            }
        }
    }
    importer.stats.pages = pages.len();
    let world = |page: usize, x: f64, y: f64| -> ImportResult<Point2D> {
        let h = pages[page].1.height_px();
        Point2D::new(x * s, (h - y) * s).map_err(domain)
    };

    // QET 0.7+ guarda o texto do rótulo na ocorrência. Sem designação na
    // definição, ela ganha o texto dinâmico na posição da 1ª ocorrência.
    for inst in &instances {
        let Some(label) = &inst.label_text else {
            continue;
        };
        let Some(def) = importer.definitions.get_mut(&inst.def) else {
            continue;
        };
        let has = def.symbol.primitives().iter().any(|p| {
            matches!(
                p,
                SymbolPrimitive::DynamicText {
                    field: SymbolTextField::Designation,
                    ..
                }
            )
        });
        if has {
            continue;
        }
        let mut primitives = def.symbol.primitives().to_vec();
        primitives.push(label.clone());
        let rebuilt = SymbolDefinition::new(
            def.symbol.id().clone(),
            def.symbol.version(),
            def.symbol.name(),
            def.symbol.category(),
            primitives,
            def.symbol.ports().to_vec(),
        )
        .and_then(|d| d.with_styles(def.symbol.styles().clone()));
        match rebuilt {
            Ok(symbol) => def.symbol = symbol,
            Err(e) => importer
                .report
                .ignored(format!("rótulo da ocorrência: {e}")),
        }
    }

    // Ocorrências: mestres primeiro (decisão 3), depois escravos e simples.
    let by_uuid: BTreeMap<String, usize> = instances
        .iter()
        .enumerate()
        .filter(|(_, i)| !i.uuid.is_empty())
        .map(|(n, i)| (i.uuid.clone(), n))
        .collect();
    let link_of = |i: &Instance| {
        importer
            .definitions
            .get(&i.def)
            .map_or("simple".to_owned(), |d| d.link.clone())
    };
    let mut order: Vec<usize> = (0..instances.len()).collect();
    order.sort_by_key(|n| match link_of(&instances[*n]).as_str() {
        "master" => 0,
        "slave" => 1,
        _ => 2,
    });
    let mut placed: BTreeMap<usize, RepresentationId> = BTreeMap::new();
    let mut device_of: BTreeMap<usize, DeviceId> = BTreeMap::new();
    let mut reports: Vec<usize> = vec![];
    for n in order {
        let inst = &instances[n];
        let link = link_of(inst);
        if link.ends_with("_report") {
            reports.push(n);
            continue;
        }
        let Some(def) = importer.definitions.get(&inst.def) else {
            importer.report.ignored(format!(
                "elemento '{}' ausente da coleção do projeto",
                inst.def
            ));
            continue;
        };
        // Símbolo sem terminais (desenho, plano de montagem): ocorrência sem
        // dispositivo.
        if def.symbol.ports().is_empty() {
            let symbol = def.symbol.clone();
            let placed_graphic = Transform2D::new(
                world(inst.page, inst.x, inst.y)?,
                rotation(inst.orientation),
                false,
                false,
            )
            .and_then(|t| {
                importer.project.add_symbol_definition(symbol.clone())?;
                Representation::new_symbol(
                    importer.project.id(),
                    pages[inst.page].0,
                    &symbol,
                    None,
                    t,
                )
            })
            .and_then(|r| importer.project.insert_representation(r));
            match placed_graphic {
                Ok(_) => {
                    importer.stats.occurrences += 1;
                    importer.stats.drawings += 1;
                }
                Err(e) => importer
                    .report
                    .ignored(format!("elemento '{}': {e}", inst.def)),
            }
            continue;
        }
        let master = if link == "slave" {
            inst.links
                .iter()
                .filter_map(|u| by_uuid.get(u))
                .find_map(|m| device_of.get(m).copied())
        } else {
            None
        };
        // Rótulo repetido fora de mestre/escravo (bornes, componente mostrado
        // em várias folhas): mesma identidade no QElectroTech → mesmo
        // dispositivo.
        let repeated = if master.is_none() && link != "master" {
            device_part(&inst.label)
                .and_then(|d| Designation::new(d).ok())
                .and_then(|c| {
                    importer
                        .project
                        .devices()
                        .find(|x| x.designation() == &c)
                        .map(Device::id)
                })
        } else {
            None
        };
        if repeated.is_some() {
            importer
                .report
                .approximated("rótulo repetido: ocorrência agrupada no dispositivo existente");
        }
        let device = match master.or(repeated) {
            Some(id) => PlacementDevice::Existing(id),
            None => {
                let wanted = device_part(&inst.label).filter(|d| {
                    let candidate = Designation::new(d.as_str());
                    candidate
                        .is_ok_and(|c| importer.project.devices().all(|x| x.designation() != &c))
                });
                let designation = match wanted {
                    Some(d) => Designation::new(d).map_err(domain)?,
                    None => {
                        let prefix = if inst.prefix.trim().is_empty() {
                            prefix_of(&inst.label)
                        } else {
                            prefix_of(&inst.prefix)
                        };
                        if !inst.label.trim().is_empty() {
                            importer.report.approximated(
                                "rótulo inválido ou repetido de mestre: próxima designação livre",
                            );
                        }
                        importer
                            .project
                            .next_device_designation(&prefix)
                            .map_err(domain)?
                    }
                };
                PlacementDevice::New {
                    designation,
                    classification: def.symbol.name().to_owned(),
                }
            }
        };
        let kind = match (link.as_str(), def.kind.as_str()) {
            ("master", "coil") => FunctionKind::Coil,
            ("slave", "power") => FunctionKind::MainContact,
            ("slave", _) => FunctionKind::AuxiliaryContact,
            ("master", k) | (_, k) => FunctionKind::Other(
                Text::new(if k.is_empty() { "componente" } else { k })
                    .unwrap_or_else(|_| Text::new("componente").expect("constante")),
            ),
        };
        let transform = Transform2D::new(
            world(inst.page, inst.x, inst.y)?,
            rotation(inst.orientation),
            false,
            false,
        )
        .map_err(domain)?;
        let result = importer.project.place_symbol(SymbolPlacement {
            page: pages[inst.page].0,
            definition: def.symbol.clone(),
            transform,
            device,
            function_kind: kind,
        });
        match result {
            Ok(delta) => {
                placed.insert(n, delta.representation_id());
                device_of.insert(n, delta.device_id());
                if delta.created_device() {
                    importer.stats.devices += 1;
                }
                if link == "master" {
                    let _ = importer.project.edit_device(DeviceEdit::SetMainFunction {
                        device: delta.device_id(),
                        function: Some(delta.function_id()),
                    });
                }
                importer.stats.occurrences += 1;
            }
            Err(e) => importer
                .report
                .ignored(format!("elemento '{}': {e}", inst.def)),
        }
    }

    // Terminais por folha: id → (instância, id do terminal).
    let mut terminal_index: BTreeMap<(usize, String), usize> = BTreeMap::new();
    for (n, inst) in instances.iter().enumerate() {
        for (id, _, _) in &inst.terminals {
            terminal_index.insert((inst.page, id.clone()), n);
        }
    }
    let port_ref = |importer: &Importer, n: usize, terminal: &str| -> Option<PortRef> {
        let rep = *placed.get(&n)?;
        let port = importer.port_for(&instances[n], terminal)?;
        PortRef::new(rep, port).ok()
    };

    // Relatórios de folha: ponto do outro lado do condutor.
    let report_set: BTreeSet<usize> = reports.iter().copied().collect();
    let mut report_point: BTreeMap<usize, ConnectionPointId> = BTreeMap::new();
    let mut report_lines: Vec<(usize, Vec<Point2D>)> = vec![];
    let terminal_world = |importer: &Importer, n: usize, terminal: &str| -> Option<Point2D> {
        let inst = &instances[n];
        let (x, y) = inst
            .terminals
            .iter()
            .find(|(id, _, _)| id == terminal)
            .map(|(_, x, y)| (*x, *y))
            .or_else(|| {
                importer
                    .definitions
                    .get(&inst.def)?
                    .terminals
                    .get(terminal)
                    .copied()
            })?;
        let t = Transform2D::new(
            world(inst.page, inst.x, inst.y).ok()?,
            rotation(inst.orientation),
            false,
            false,
        )
        .ok()?;
        t.apply(Point2D::new(x * s, -y * s).ok()?).ok()
    };
    for wire in &wires {
        let end_of = |e: &Option<String>, t: &String| match e {
            Some(uuid) => by_uuid.get(uuid).copied(),
            None => terminal_index.get(&(wire.page, t.clone())).copied(),
        };
        let ends = [end_of(&wire.e1, &wire.t1), end_of(&wire.e2, &wire.t2)];
        let (Some(a), Some(b)) = (ends[0], ends[1]) else {
            importer.report.ignored("condutor com terminal inexistente");
            continue;
        };
        if report_set.contains(&a) || report_set.contains(&b) {
            let (report, other, terminal) = if report_set.contains(&a) {
                (a, b, &wire.t2)
            } else {
                (b, a, &wire.t1)
            };
            if let Some(port) = port_ref(&importer, other, terminal)
                && let Some(point) = importer.project.port_binding(&port)
            {
                report_point.insert(report, point);
            }
            // O traço do condutor fica como desenho, sem efeito elétrico.
            let report_terminal = if report == a { &wire.t1 } else { &wire.t2 };
            let other_pos = port_ref(&importer, other, terminal)
                .and_then(|p| importer.project.port_position(&p).ok())
                .or_else(|| terminal_world(&importer, other, terminal));
            if let (Some(rp), Some(op)) = (
                terminal_world(&importer, report, report_terminal),
                other_pos,
            ) {
                let (start, end) = if report == a { (rp, op) } else { (op, rp) };
                let path = qet_path(start, end, &wire.segments, s);
                if path.len() >= 2 {
                    report_lines.push((wire.page, path));
                }
            }
            continue;
        }
        let (Some(source), Some(target)) = (
            port_ref(&importer, a, &wire.t1),
            port_ref(&importer, b, &wire.t2),
        ) else {
            importer
                .report
                .ignored("condutor ligado a elemento não importado ou terminal sem porta");
            continue;
        };
        let (Ok(start), Ok(end)) = (
            importer.project.port_position(&source),
            importer.project.port_position(&target),
        ) else {
            importer.report.ignored("condutor sem posição de porta");
            continue;
        };
        let points = qet_path(start, end, &wire.segments, s);
        let result = importer
            .project
            .edit_connection_drawing(ConnectionDrawingEdit::CreateRouteExact {
                source: source.clone(),
                target: target.clone(),
                points: points.clone(),
            })
            .or_else(|_| {
                // Caminho original inválido: rota simples em L.
                let mut simple = vec![start];
                if start.x() != end.x()
                    && start.y() != end.y()
                    && let Ok(corner) = Point2D::new(end.x(), start.y())
                {
                    simple.push(corner);
                }
                simple.push(end);
                let simple = simplify(simple);
                importer
                    .project
                    .edit_connection_drawing(ConnectionDrawingEdit::CreateRouteExact {
                        source: source.clone(),
                        target: target.clone(),
                        points: simple,
                    })
                    .inspect(|_| {
                        importer
                            .report
                            .approximated("caminho do condutor refeito em L");
                    })
            });
        match result {
            Ok(_) => {
                importer.stats.conductors += 1;
                let connection = importer
                    .project
                    .connection_routes()
                    .find(|r| r.source() == &source && r.target() == &target)
                    .map(ConnectionRoute::connection_id);
                if let (Some(connection), false) = (connection, wire.num.is_empty()) {
                    match Text::new(wire.num.clone()) {
                        Ok(number) => {
                            let _ = importer.project.set_connection_number(
                                connection,
                                Some(number),
                                true,
                            );
                        }
                        Err(_) => importer.report.ignored("número de fio inválido"),
                    }
                }
            }
            Err(e) => importer.report.ignored(format!("condutor: {e}")),
        }
    }

    // Marcadores de interrupção e grupos.
    let mut marker_of: BTreeMap<usize, RepresentationId> = BTreeMap::new();
    for n in &reports {
        let inst = &instances[*n];
        let Some(def) = importer.definitions.get(&inst.def) else {
            importer.report.ignored("relatório de folha sem definição");
            continue;
        };
        let Some(first) = def.symbol.ports().first() else {
            importer.report.ignored("relatório de folha sem terminal");
            continue;
        };
        // O marcador exige a porta única "p" na origem: o desenho é deslocado
        // e a posição da ocorrência compensa o deslocamento.
        let offset = first.position();
        let port = SymbolPort::new(
            "p",
            Point2D::new(0.0, 0.0).map_err(domain)?,
            first.orientation_degrees(),
            "P",
            None,
        )
        .map_err(domain)?;
        // Sem dispositivo, a designação mostraria só "-K?": o nome do
        // marcador já identifica a interrupção. Os estilos seguem os índices.
        let mut primitives = vec![];
        let mut styles = BTreeMap::new();
        for (i, p) in def.symbol.primitives().iter().enumerate() {
            if matches!(
                p,
                SymbolPrimitive::DynamicText {
                    field: SymbolTextField::Designation,
                    ..
                }
            ) {
                continue;
            }
            if let Some(style) = def.symbol.styles().get(&i) {
                styles.insert(primitives.len(), *style);
            }
            primitives.push(p.translated(-offset.x(), -offset.y()).map_err(domain)?);
        }
        let marker_def = SymbolDefinition::new(
            SymbolDefinitionId::new(format!("{}.relatorio", def.symbol.id().as_str()))
                .map_err(domain)?,
            1,
            def.symbol.name(),
            "QElectroTech (relatório de folha)",
            primitives,
            vec![port],
        )
        .and_then(|d| d.with_styles(styles))
        .map_err(domain)?;
        let point = report_point.get(n).copied();
        let rotate = Transform2D::new(
            world(inst.page, inst.x, inst.y)?,
            rotation(inst.orientation),
            false,
            false,
        )
        .map_err(domain)?;
        let transform = Transform2D::new(
            rotate.apply(offset).map_err(domain)?,
            rotation(inst.orientation),
            false,
            false,
        )
        .map_err(domain)?;
        let rep = Representation::new_symbol(
            importer.project.id(),
            pages[inst.page].0,
            &marker_def,
            point.map(RepresentationTarget::ConnectionPoint),
            transform,
        )
        .map_err(domain)?;
        let id = rep.id();
        let name = if inst.label.trim().is_empty() {
            format!("R{}", marker_of.len() + 1)
        } else {
            inst.label.trim().to_owned()
        };
        let insert = importer.project.edit_marker(MarkerEdit::Insert {
            marker: ElectricalMarker::new(
                id,
                Text::new(name).map_err(domain)?,
                MarkerKind::Interruption,
                point,
            ),
            representation: Box::new(rep),
            definition: Box::new(marker_def),
        });
        match insert {
            Ok(_) => {
                marker_of.insert(*n, id);
            }
            Err(e) => importer.report.ignored(format!("relatório de folha: {e}")),
        }
    }
    let mut grouped = BTreeSet::new();
    for (n, rep) in &marker_of {
        if grouped.contains(n) {
            continue;
        }
        let partners: Vec<usize> = instances[*n]
            .links
            .iter()
            .filter_map(|u| by_uuid.get(u).copied())
            .filter(|p| marker_of.contains_key(p) && !grouped.contains(p))
            .collect();
        if partners.is_empty() {
            importer.report.approximated("relatório de folha sem par");
            continue;
        }
        let mut members = vec![*rep];
        members.extend(partners.iter().map(|p| marker_of[p]));
        grouped.insert(*n);
        grouped.extend(partners);
        match importer.project.edit_marker(MarkerEdit::SetGroup {
            id: ReferenceGroupId::new(),
            mode: ReferenceMode::Star,
            members,
            continuity: false,
        }) {
            Ok(_) => importer.stats.sheet_reports += 1,
            Err(e) => importer
                .report
                .ignored(format!("grupo de relatório de folha: {e}")),
        }
    }

    // Traços dos condutores dos relatórios de folha (só desenho, um traço
    // por segmento: o conteúdo livre não tem polilinha).
    for (page, path) in report_lines {
        let mut failed = None;
        for w in path.windows(2) {
            let (a, b) = (w[0], w[1]);
            let rep = Point2D::new(b.x() - a.x(), b.y() - a.y())
                .and_then(|end| {
                    PageGraphic::new(
                        SymbolPrimitive::Line {
                            start: Point2D::new(0.0, 0.0)?,
                            end,
                        },
                        0.25,
                        TextAlignment::Left,
                    )
                })
                .and_then(|g| {
                    Representation::new_graphic(
                        importer.project.id(),
                        pages[page].0,
                        g,
                        Transform2D::new(a, 0.0, false, false)?,
                    )
                })
                .and_then(|r| importer.project.insert_representation(r));
            if let Err(e) = rep {
                failed = Some(e.to_string());
            }
        }
        if let Some(e) = failed {
            importer
                .report
                .ignored(format!("traço de condutor de relatório de folha: {e}"));
        }
    }

    // Textos livres: os da folha e os de usuário das ocorrências (posição
    // local levada à folha pela transformação da ocorrência).
    let mut free: Vec<(usize, Point2D, String, f64)> = vec![];
    for (page, x, y, text, size) in texts {
        if let Ok(origin) = Point2D::new(x * s, (pages[page].1.height_px() - y) * s) {
            free.push((page, origin, text, size * s * 0.8));
        }
    }
    for inst in &instances {
        if inst.user_texts.is_empty() {
            continue;
        }
        let Ok(t) = world(inst.page, inst.x, inst.y).and_then(|p| {
            Transform2D::new(p, rotation(inst.orientation), false, false).map_err(domain)
        }) else {
            continue;
        };
        for (lx, ly, text, height) in &inst.user_texts {
            if let Ok(origin) = Point2D::new(*lx, *ly).and_then(|p| t.apply(p)) {
                free.push((inst.page, origin, text.clone(), *height));
            }
        }
    }
    for (page, origin, text, height) in free {
        let graphic = PageGraphic::new(
            SymbolPrimitive::Text {
                origin: Point2D::new(0.0, 0.0).expect("origem"),
                text: text.chars().take(400).collect(),
                height: height.clamp(1.0, 50.0),
            },
            0.25,
            TextAlignment::Left,
        );
        let rep = graphic.and_then(|g| {
            Representation::new_graphic(
                importer.project.id(),
                pages[page].0,
                g,
                Transform2D::new(origin, 0.0, false, false)?,
            )
        });
        match rep.and_then(|r| importer.project.insert_representation(r)) {
            Ok(_) => importer.stats.texts += 1,
            Err(e) => importer.report.ignored(format!("texto livre: {e}")),
        }
    }
    importer.project.validate().map_err(domain)?;
    let st = &importer.stats;
    importer.report.info(format!(
        "{} folha(s), {} símbolo(s), {} ocorrência(s), {} dispositivo(s), {} fio(s), {} relatório(s) de folha, {} texto(s), {} desenho(s) sem terminal",
        st.pages, st.symbols, st.occurrences, st.devices, st.conductors, st.sheet_reports, st.texts, st.drawings
    ));
    Ok(QetImport {
        project: importer.project,
        report: importer.report,
        stats: importer.stats,
    })
}

/// Tira pontos repetidos e pontos no meio de um trecho reto (inclusive volta
/// sobre o mesmo eixo), mantendo o caminho ortogonal.
fn simplify(points: Vec<Point2D>) -> Vec<Point2D> {
    let mut out: Vec<Point2D> = Vec::with_capacity(points.len());
    for p in points {
        if out.last() == Some(&p) {
            continue;
        }
        if out.len() >= 2 {
            let (a, b) = (out[out.len() - 2], out[out.len() - 1]);
            if (a.x() == b.x() && b.x() == p.x()) || (a.y() == b.y() && b.y() == p.y()) {
                out.pop();
                if out.last() == Some(&p) {
                    continue;
                }
            }
        }
        out.push(p);
    }
    out
}

/// Percurso ortogonal do condutor QET: segmentos a partir de `start`
/// (vertical positivo = para baixo na tela), fechado até `end`.
fn qet_path(start: Point2D, end: Point2D, segments: &[(bool, f64)], s: f64) -> Vec<Point2D> {
    let mut points = vec![start];
    let mut last = start;
    for (vertical, length) in segments {
        let next = if *vertical {
            Point2D::new(last.x(), last.y() - length * s)
        } else {
            Point2D::new(last.x() + length * s, last.y())
        };
        if let Ok(next) = next
            && next != last
        {
            points.push(next);
            last = next;
        }
    }
    if last != end {
        if last.x() != end.x()
            && last.y() != end.y()
            && let Ok(corner) = Point2D::new(end.x(), last.y())
        {
            points.push(corner);
        }
        points.push(end);
    }
    simplify(points)
}
