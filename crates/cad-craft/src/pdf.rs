// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 21: PDF vetorial multipágina a partir da lista de exibição.
//!
//! Uma página PDF por página do projeto (tamanho real, ou o papel escolhido
//! para impressão), Helvetica WinAnsi (fonte padrão, sem embutir), imagens
//! JPEG embutidas como estão e PNG recomprimido, links nas referências
//! cruzadas e marcadores documento → página. Sem GUI.
use crate::display::{DisplayColor, DisplayItem, DisplayList, DisplayText};
use crate::domain::*;
use crate::error::DomainError;
use crate::font;
use pdf_writer::types::{ActionType, AnnotationType, LineCapStyle, LineJoinStyle, PageMode};
use pdf_writer::{Content, Date, Filter, Finish, Name, Pdf, Rect, Ref, Str, TextStr};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

/// Milímetros → pontos PDF.
const PT_PER_MM: f64 = 72.0 / 25.4;
const FONT: Name<'static> = Name(b"F1");

/// Páginas a exportar.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum PdfScope {
    #[default]
    Project,
    Document(DocumentId),
    /// Páginas escolhidas; saem na ordem do projeto.
    Pages(Vec<PageId>),
}

/// Escala no papel de impressão.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PaperFit {
    /// Reduz/amplia para caber no papel, centralizado.
    #[default]
    Fit,
    /// 1:1, centralizado (o que exceder o papel é cortado).
    Actual,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PdfOptions {
    pub scope: PdfScope,
    /// Preto e branco: o conteúdo vetorial sai só em preto e cinza.
    pub monochrome: bool,
    pub bookmarks: bool,
    /// Papel de impressão (mm) e escala; `None` = tamanho real da folha.
    pub paper: Option<(Size2D, PaperFit)>,
}

impl Default for PdfOptions {
    fn default() -> Self {
        Self {
            scope: PdfScope::Project,
            monochrome: false,
            bookmarks: true,
            paper: None,
        }
    }
}

/// O que a exportação fez e o que precisou adaptar.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PdfReport {
    pub pages: usize,
    pub links: usize,
    /// Links cujo destino não está entre as páginas exportadas.
    pub omitted_links: usize,
    /// Caracteres fora da Helvetica WinAnsi, trocados por `?`.
    pub replaced: BTreeSet<char>,
    /// Imagens sem conteúdo ou ilegíveis (desenhadas como caixa).
    pub missing_images: usize,
}

impl PdfReport {
    pub fn summary(&self) -> String {
        let mut text = format!(
            "{} página(s), {} link(s) de referência.",
            self.pages, self.links
        );
        if self.omitted_links > 0 {
            text.push_str(&format!(
                " {} referência(s) para páginas fora do alcance ficaram sem link.",
                self.omitted_links
            ));
        }
        if !self.replaced.is_empty() {
            let chars: String = self.replaced.iter().collect();
            text.push_str(&format!(
                " Caracteres sem suporte na fonte, trocados por '?': {chars}."
            ));
        }
        if self.missing_images > 0 {
            text.push_str(&format!(
                " {} imagem(ns) sem conteúdo desenhada(s) como caixa.",
                self.missing_images
            ));
        }
        text
    }
}

#[derive(Clone, Debug)]
pub struct PdfOutput {
    pub bytes: Vec<u8>,
    pub report: PdfReport,
}

#[derive(Debug)]
pub enum PdfError {
    /// Nenhuma página no alcance.
    Empty,
    Cancelled,
    Domain(DomainError),
}

impl std::fmt::Display for PdfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "nenhuma página no alcance escolhido"),
            Self::Cancelled => write!(f, "exportação cancelada"),
            Self::Domain(e) => write!(f, "{e}"),
        }
    }
}
impl std::error::Error for PdfError {}
impl From<DomainError> for PdfError {
    fn from(e: DomainError) -> Self {
        Self::Domain(e)
    }
}

/// Páginas do alcance, na ordem dos documentos e das páginas.
pub fn pages_in_scope(project: &Project, scope: &PdfScope) -> Result<Vec<PageId>, PdfError> {
    let mut out = vec![];
    for document in project.documents() {
        if let PdfScope::Document(id) = scope
            && document.id() != *id
        {
            continue;
        }
        for page in project.ordered_pages(document.id())? {
            if let PdfScope::Pages(ids) = scope
                && !ids.contains(&page.id())
            {
                continue;
            }
            out.push(page.id());
        }
    }
    if let PdfScope::Pages(ids) = scope
        && let Some(missing) = ids.iter().find(|id| !out.contains(id))
    {
        return Err(DomainError::MissingEntity(EntityRef::Page(*missing)).into());
    }
    Ok(out)
}

/// Afim `[a, b, c, d, e, f]`: `x' = a·x + c·y + e`, `y' = b·x + d·y + f`.
type Affine = [f64; 6];
fn compose(m1: Affine, m2: Affine) -> Affine {
    [
        m1[0] * m2[0] + m1[2] * m2[1],
        m1[1] * m2[0] + m1[3] * m2[1],
        m1[0] * m2[2] + m1[2] * m2[3],
        m1[1] * m2[2] + m1[3] * m2[3],
        m1[0] * m2[4] + m1[2] * m2[5] + m1[4],
        m1[1] * m2[4] + m1[3] * m2[5] + m1[5],
    ]
}
fn f32s(m: Affine) -> [f32; 6] {
    m.map(|v| v as f32)
}

/// Posição de uma folha no papel PDF (pt).
#[derive(Clone, Copy)]
struct Placement {
    media: (f64, f64),
    /// mm da folha → pt.
    matrix: Affine,
}
impl Placement {
    fn new(width: f64, height: f64, paper: Option<(Size2D, PaperFit)>) -> Self {
        let Some((paper, fit)) = paper else {
            return Self {
                media: (width * PT_PER_MM, height * PT_PER_MM),
                matrix: [PT_PER_MM, 0.0, 0.0, PT_PER_MM, 0.0, 0.0],
            };
        };
        // Papel na mesma orientação da folha.
        let (mut pw, mut ph) = (paper.width(), paper.height());
        if (width > height) != (pw > ph) {
            std::mem::swap(&mut pw, &mut ph);
        }
        let scale = match fit {
            PaperFit::Fit => (pw / width).min(ph / height),
            PaperFit::Actual => 1.0,
        };
        let (ox, oy) = ((pw - width * scale) / 2.0, (ph - height * scale) / 2.0);
        let k = scale * PT_PER_MM;
        Self {
            media: (pw * PT_PER_MM, ph * PT_PER_MM),
            matrix: [k, 0.0, 0.0, k, ox * PT_PER_MM, oy * PT_PER_MM],
        }
    }
    fn point(&self, x: f64, y: f64) -> (f32, f32) {
        let m = self.matrix;
        ((m[0] * x + m[4]) as f32, (m[3] * y + m[5]) as f32)
    }
    fn length(&self, mm: f64) -> f32 {
        (mm * self.matrix[0]) as f32
    }
    fn rect(&self, r: Rect2D) -> Rect {
        let (x1, y1) = self.point(r.origin().x(), r.origin().y());
        let (x2, y2) = self.point(r.max().x(), r.max().y());
        Rect::new(x1, y1, x2, y2)
    }
}

/// Valores de cor (cinza ou RGB) da cor lógica.
enum Paint {
    Gray(f32),
    Rgb(f32, f32, f32),
}
fn paint(color: DisplayColor, monochrome: bool) -> Paint {
    match (color, monochrome) {
        (DisplayColor::Ink, _) | (DisplayColor::Reference, true) => Paint::Gray(0.0),
        (DisplayColor::Reference, false) => Paint::Rgb(0.0, 0.36, 0.72),
        (DisplayColor::Watermark, _) => Paint::Gray(0.8),
        (DisplayColor::Guide, _) => Paint::Gray(0.75),
    }
}
fn set_stroke(content: &mut Content, color: DisplayColor, monochrome: bool) {
    match paint(color, monochrome) {
        Paint::Gray(g) => content.set_stroke_gray(g),
        Paint::Rgb(r, g, b) => content.set_stroke_rgb(r, g, b),
    };
}
fn set_fill(content: &mut Content, color: DisplayColor, monochrome: bool) {
    match paint(color, monochrome) {
        Paint::Gray(g) => content.set_fill_gray(g),
        Paint::Rgb(r, g, b) => content.set_fill_rgb(r, g, b),
    };
}

fn deflate(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(bytes).expect("compressão em memória");
    encoder.finish().expect("compressão em memória")
}

/// Componentes de cor do JPEG (marcador SOF); `None` se ilegível.
fn jpeg_components(bytes: &[u8]) -> Option<u8> {
    let mut i = 2;
    while i + 9 < bytes.len() {
        if bytes[i] != 0xFF {
            return None;
        }
        let marker = bytes[i + 1];
        let length = usize::from(u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]));
        if matches!(marker, 0xC0..=0xCF) && !matches!(marker, 0xC4 | 0xC8 | 0xCC) {
            return bytes.get(i + 9).copied();
        }
        i += 2 + length;
    }
    None
}

/// Referências de objeto em sequência.
struct Refs(i32);
impl Refs {
    fn next(&mut self) -> Ref {
        self.0 += 1;
        Ref::new(self.0)
    }
}

/// Imagem já escrita no PDF (nome do recurso e referência).
type ImageEntry = Option<(String, Ref)>;

struct Writer<'a> {
    project: &'a Project,
    options: &'a PdfOptions,
    pdf: Pdf,
    refs: Refs,
    images: BTreeMap<AssetId, ImageEntry>,
    report: PdfReport,
}

impl Writer<'_> {
    /// Escreve a imagem uma vez; `None` quando não há conteúdo legível.
    fn image(&mut self, asset: AssetId) -> ImageEntry {
        if let Some(entry) = self.images.get(&asset) {
            return entry.clone();
        }
        let entry = self.write_image(asset);
        self.images.insert(asset, entry.clone());
        entry
    }
    fn write_image(&mut self, asset: AssetId) -> ImageEntry {
        let image = self.project.asset(asset)?;
        let bytes = image.bytes()?;
        let name = format!("Im{}", self.images.len() + 1);
        let id = self.refs.next();
        match image.format() {
            ImageFormat::Jpeg => {
                let components = jpeg_components(bytes)?;
                let mut x = self.pdf.image_xobject(id, bytes);
                x.filter(Filter::DctDecode);
                x.width(image.width_px() as i32);
                x.height(image.height_px() as i32);
                match components {
                    1 => x.color_space().device_gray(),
                    4 => x.color_space().device_cmyk(),
                    _ => x.color_space().device_rgb(),
                };
                x.bits_per_component(8);
                x.finish();
            }
            ImageFormat::Png => {
                let decoded =
                    image::load_from_memory_with_format(bytes, image::ImageFormat::Png).ok()?;
                let rgba = decoded.to_rgba8();
                let (w, h) = (rgba.width() as i32, rgba.height() as i32);
                let rgb: Vec<u8> = rgba
                    .pixels()
                    .flat_map(|p| [p.0[0], p.0[1], p.0[2]])
                    .collect();
                let alpha: Vec<u8> = rgba.pixels().map(|p| p.0[3]).collect();
                let mask = alpha.iter().any(|a| *a != 255).then(|| {
                    let mask_id = self.refs.next();
                    let data = deflate(&alpha);
                    let mut m = self.pdf.image_xobject(mask_id, &data);
                    m.filter(Filter::FlateDecode);
                    m.width(w);
                    m.height(h);
                    m.color_space().device_gray();
                    m.bits_per_component(8);
                    m.finish();
                    mask_id
                });
                let data = deflate(&rgb);
                let mut x = self.pdf.image_xobject(id, &data);
                x.filter(Filter::FlateDecode);
                x.width(w);
                x.height(h);
                x.color_space().device_rgb();
                x.bits_per_component(8);
                if let Some(mask) = mask {
                    x.s_mask(mask);
                }
                x.finish();
            }
        }
        Some((name, id))
    }

    fn text(&mut self, content: &mut Content, text: &DisplayText, place: &Placement) {
        let (bytes, missing) = font::encode(&text.text);
        self.report.replaced.extend(missing);
        let natural = text.natural_width();
        let stretch = match text.width {
            Some(w) if natural > 0.0 => w / natural,
            _ => 1.0,
        };
        // Espaço do texto (corpo 1) → caixa → folha (mm) → papel (pt).
        let to_box = [text.size * stretch, 0.0, 0.0, text.size, 0.0, text.baseline];
        let matrix = compose(place.matrix, compose(text.matrix, to_box));
        set_fill(content, text.color, self.options.monochrome);
        content.begin_text();
        content.set_font(FONT, 1.0);
        content.set_text_matrix(f32s(matrix));
        content.show(Str(&bytes));
        content.end_text();
    }

    /// Conteúdo da página; devolve as imagens usadas (nome, referência).
    fn content(&mut self, list: &DisplayList, place: &Placement) -> (Vec<u8>, Vec<(String, Ref)>) {
        let mono = self.options.monochrome;
        let mut content = Content::new();
        content.set_line_cap(LineCapStyle::RoundCap);
        content.set_line_join(LineJoinStyle::RoundJoin);
        let mut used = vec![];
        for item in &list.items {
            match item {
                DisplayItem::Path {
                    points,
                    closed,
                    stroke,
                    fill,
                    ..
                } => {
                    let (x, y) = place.point(points[0].x(), points[0].y());
                    content.move_to(x, y);
                    for p in &points[1..] {
                        let (x, y) = place.point(p.x(), p.y());
                        content.line_to(x, y);
                    }
                    if *closed {
                        content.close_path();
                    }
                    if let Some(color) = fill {
                        set_fill(&mut content, *color, mono);
                    }
                    if let Some(s) = stroke {
                        set_stroke(&mut content, s.color, mono);
                        content.set_line_width(place.length(s.width).max(0.1));
                        content.set_dash_pattern(s.dash.iter().map(|d| place.length(*d)), 0.0);
                    }
                    match (fill.is_some(), stroke.is_some()) {
                        (true, true) => content.fill_nonzero_and_stroke(),
                        (true, false) => content.fill_nonzero(),
                        _ => content.stroke(),
                    };
                }
                DisplayItem::Text(text) => self.text(&mut content, text, place),
                DisplayItem::Image { asset, rect, .. } => match self.image(*asset) {
                    Some((name, id)) => {
                        let r = place.rect(*rect);
                        content.save_state();
                        content.transform([r.x2 - r.x1, 0.0, 0.0, r.y2 - r.y1, r.x1, r.y1]);
                        content.x_object(Name(name.as_bytes()));
                        content.restore_state();
                        if !used.iter().any(|(n, _)| n == &name) {
                            used.push((name, id));
                        }
                    }
                    None => {
                        self.report.missing_images += 1;
                        let r = place.rect(*rect);
                        set_stroke(&mut content, DisplayColor::Guide, mono);
                        content.set_line_width(0.5);
                        content.set_dash_pattern([], 0.0);
                        content.rect(r.x1, r.y1, r.x2 - r.x1, r.y2 - r.y1);
                        content.move_to(r.x1, r.y1);
                        content.line_to(r.x2, r.y2);
                        content.move_to(r.x2, r.y1);
                        content.line_to(r.x1, r.y2);
                        content.stroke();
                    }
                },
                DisplayItem::Link { .. } => {}
            }
        }
        (deflate(&content.finish()), used)
    }
}

/// Data atual (UTC) para os metadados.
fn today() -> Date {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    // Dias desde 1970-01-01 → data civil (algoritmo de H. Hinnant).
    let z = (secs / 86_400) as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    Date::new(year.clamp(0, 9999) as u16)
        .month(month as u8)
        .day(day as u8)
}

/// Exporta o alcance para PDF. `progress(feitas, total)` é chamado a cada
/// página; devolver `false` cancela.
pub fn export_pdf(
    project: &Project,
    options: &PdfOptions,
    progress: &mut dyn FnMut(usize, usize) -> bool,
) -> Result<PdfOutput, PdfError> {
    let pages = pages_in_scope(project, &options.scope)?;
    if pages.is_empty() {
        return Err(PdfError::Empty);
    }
    let mut w = Writer {
        project,
        options,
        pdf: Pdf::new(),
        refs: Refs(0),
        images: BTreeMap::new(),
        report: PdfReport::default(),
    };
    let catalog = w.refs.next();
    let tree = w.refs.next();
    let font_id = w.refs.next();
    let info = w.refs.next();
    // Referência e posição de cada página exportada (destinos dos links).
    let mut index: BTreeMap<PageId, (Ref, Placement)> = BTreeMap::new();
    for page in &pages {
        let p = project
            .page(*page)
            .ok_or(DomainError::MissingEntity(EntityRef::Page(*page)))?;
        let place = Placement::new(p.size().width(), p.size().height(), options.paper);
        index.insert(*page, (w.refs.next(), place));
    }
    let total = pages.len();
    for (done, page) in pages.iter().enumerate() {
        if !progress(done, total) {
            return Err(PdfError::Cancelled);
        }
        let list = project.page_display_list(*page)?;
        let (page_ref, place) = index[page];
        let (data, images) = w.content(&list, &place);
        let content_id = w.refs.next();
        w.pdf.stream(content_id, &data).filter(Filter::FlateDecode);
        // Links: destino na página exportada, na posição da ocorrência.
        let mut annotations = vec![];
        for (rect, target) in list.links() {
            let destination = project.representation(target).and_then(|r| {
                let (dest_ref, dest_place) = index.get(&r.page_id())?;
                let at = r.transform().translation();
                Some((*dest_ref, dest_place.point(at.x() - 20.0, at.y() + 20.0)))
            });
            let Some((dest_ref, (left, top))) = destination else {
                w.report.omitted_links += 1;
                continue;
            };
            let id = w.refs.next();
            let mut a = w.pdf.annotation(id);
            a.subtype(AnnotationType::Link);
            a.rect(place.rect(rect));
            a.border(0.0, 0.0, 0.0, None);
            a.action()
                .action_type(ActionType::GoTo)
                .destination()
                .page(dest_ref)
                .xyz(left.max(0.0), top, None);
            a.finish();
            annotations.push(id);
            w.report.links += 1;
        }
        let mut pdf_page = w.pdf.page(page_ref);
        pdf_page.parent(tree);
        pdf_page.media_box(Rect::new(
            0.0,
            0.0,
            place.media.0 as f32,
            place.media.1 as f32,
        ));
        pdf_page.contents(content_id);
        if !annotations.is_empty() {
            pdf_page.annotations(annotations);
        }
        let mut resources = pdf_page.resources();
        resources.fonts().pair(FONT, font_id);
        if !images.is_empty() {
            let mut x = resources.x_objects();
            for (name, id) in &images {
                x.pair(Name(name.as_bytes()), *id);
            }
        }
        resources.finish();
        pdf_page.finish();
        w.report.pages += 1;
    }
    progress(total, total);
    w.pdf
        .pages(tree)
        .kids(pages.iter().map(|p| index[p].0))
        .count(total as i32);
    w.pdf
        .type1_font(font_id)
        .base_font(Name(b"Helvetica"))
        .encoding_predefined(Name(b"WinAnsiEncoding"));
    let outline = if options.bookmarks {
        write_outline(&mut w, &pages, &index)?
    } else {
        None
    };
    let mut cat = w.pdf.catalog(catalog);
    cat.pages(tree);
    if let Some(root) = outline {
        cat.outlines(root);
        cat.page_mode(PageMode::UseOutlines);
    }
    cat.finish();
    let title = project.name().as_str().to_owned();
    let author = project
        .metadata()
        .iter()
        .find(|(k, _)| {
            let k = k.as_str().to_lowercase();
            k.contains("autor") || k.contains("author")
        })
        .map(|(_, v)| v.clone());
    let mut doc = w.pdf.document_info(info);
    doc.title(TextStr(&title));
    if let Some(author) = &author {
        doc.author(TextStr(author));
    }
    doc.creator(TextStr("CAD-CRAFT"));
    doc.producer(TextStr("CAD-CRAFT (pdf-writer)"));
    doc.creation_date(today());
    doc.finish();
    let report = w.report;
    Ok(PdfOutput {
        bytes: w.pdf.finish(),
        report,
    })
}

/// Marcadores: um item por documento com as suas páginas exportadas.
fn write_outline(
    w: &mut Writer<'_>,
    pages: &[PageId],
    index: &BTreeMap<PageId, (Ref, Placement)>,
) -> Result<Option<Ref>, PdfError> {
    let project = w.project;
    let mut groups: Vec<(String, Vec<(String, Ref)>)> = vec![];
    for document in project.documents() {
        let entries: Vec<_> = project
            .ordered_pages(document.id())?
            .into_iter()
            .filter(|p| pages.contains(&p.id()))
            .map(|p| {
                (
                    format!("{} — {}", p.number().as_str(), p.name().as_str()),
                    index[&p.id()].0,
                )
            })
            .collect();
        if !entries.is_empty() {
            groups.push((document.name().as_str().to_owned(), entries));
        }
    }
    if groups.is_empty() {
        return Ok(None);
    }
    let root = w.refs.next();
    let doc_refs: Vec<Ref> = groups.iter().map(|_| w.refs.next()).collect();
    let page_refs: Vec<Vec<Ref>> = groups
        .iter()
        .map(|(_, e)| e.iter().map(|_| w.refs.next()).collect())
        .collect();
    let visible: usize = groups.iter().map(|(_, e)| 1 + e.len()).sum();
    w.pdf
        .outline(root)
        .first(doc_refs[0])
        .last(*doc_refs.last().expect("grupos"))
        .count(visible as i32);
    for (i, (name, entries)) in groups.iter().enumerate() {
        let mut item = w.pdf.outline_item(doc_refs[i]);
        item.title(TextStr(name));
        item.parent(root);
        if i > 0 {
            item.prev(doc_refs[i - 1]);
        }
        if i + 1 < doc_refs.len() {
            item.next(doc_refs[i + 1]);
        }
        item.first(page_refs[i][0]);
        item.last(*page_refs[i].last().expect("páginas"));
        item.count(entries.len() as i32);
        item.dest().page(entries[0].1).fit();
        item.finish();
        for (j, (title, page_ref)) in entries.iter().enumerate() {
            let mut child = w.pdf.outline_item(page_refs[i][j]);
            child.title(TextStr(title));
            child.parent(doc_refs[i]);
            if j > 0 {
                child.prev(page_refs[i][j - 1]);
            }
            if j + 1 < entries.len() {
                child.next(page_refs[i][j + 1]);
            }
            child.dest().page(*page_ref).fit();
            child.finish();
        }
    }
    Ok(Some(root))
}
