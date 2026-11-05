// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 21: lista de exibição de uma página, em milímetros (origem inferior
//! esquerda, Y para cima). Reproduz o que a folha mostra na tela, sem os
//! auxílios de edição (grade, âncoras, portas, seleção). É a fonte do PDF e
//! será a do DXF (Etapa 22). Não depende de GUI.
use crate::font;
use cadcraft_model::domain::*;
use cadcraft_model::error::Result;

/// Espessura (mm) do traço de símbolos sem espessura própria.
pub const SYMBOL_STROKE: f64 = 0.25;
/// Espessura (mm) dos fios.
pub const ROUTE_STROKE: f64 = 0.35;
/// Raio (mm) do ponto de junção.
pub const JUNCTION_RADIUS: f64 = 0.6;
/// Altura (mm) dos rótulos de zona e de textos auxiliares da folha.
pub const LABEL_HEIGHT: f64 = 2.5;
/// Altura da linha de texto em relação ao corpo (caixa de alinhamento).
pub const LINE_HEIGHT: f64 = 1.2;
/// Linha de base acima do pé da caixa, em relação ao corpo.
pub const BASELINE: f64 = 0.25;
/// Erro máximo (mm) da tesselagem de curvas.
const CURVE_TOLERANCE: f64 = 0.01;

/// Cor lógica; quem escreve escolhe o valor (colorido ou preto e branco).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayColor {
    /// Desenho principal.
    Ink,
    /// Referências cruzadas, espelho e interrupções.
    Reference,
    /// Marca d'água da moldura.
    Watermark,
    /// Guias claras (margem sem moldura, caixa sem geometria, imagem ausente).
    Guide,
}

/// Categoria do item (camada no DXF; o PDF a ignora).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DisplayLayer {
    Frame,
    Symbols,
    Wires,
    Texts,
    References,
    Junctions,
    Guides,
}

impl DisplayLayer {
    pub const ALL: [Self; 7] = [
        Self::Frame,
        Self::Symbols,
        Self::Wires,
        Self::Texts,
        Self::References,
        Self::Junctions,
        Self::Guides,
    ];
    /// Nome da camada (sem acentos, aceito por qualquer leitor DXF).
    pub fn name(self) -> &'static str {
        match self {
            Self::Frame => "Moldura",
            Self::Symbols => "Simbolos",
            Self::Wires => "Fios",
            Self::Texts => "Textos",
            Self::References => "Referencias",
            Self::Junctions => "Juncoes",
            Self::Guides => "Guias",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DisplayStroke {
    pub width: f64,
    /// Traço/vão em mm; vazio = contínuo.
    pub dash: Vec<f64>,
    pub color: DisplayColor,
}

/// Texto numa caixa local em mm. `matrix` leva a caixa para a página:
/// `x' = a·x + c·y + e`, `y' = b·x + d·y + f` (`[a, b, c, d, e, f]`).
/// O corpo da fonte é `size`; a linha de base fica em `y = baseline`.
/// Com `width`, o texto é esticado/comprimido até essa largura.
#[derive(Clone, Debug, PartialEq)]
pub struct DisplayText {
    pub text: String,
    pub matrix: [f64; 6],
    pub size: f64,
    pub baseline: f64,
    pub width: Option<f64>,
    pub color: DisplayColor,
    pub layer: DisplayLayer,
}

impl DisplayText {
    /// Largura natural (Helvetica) na unidade da caixa.
    pub fn natural_width(&self) -> f64 {
        font::text_width(&self.text, self.size)
    }
    /// Ponto da página correspondente ao ponto `(x, y)` da caixa.
    pub fn map(&self, x: f64, y: f64) -> (f64, f64) {
        let [a, b, c, d, e, f] = self.matrix;
        (a * x + c * y + e, b * x + d * y + f)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DisplayItem {
    Path {
        points: Vec<Point2D>,
        closed: bool,
        stroke: Option<DisplayStroke>,
        fill: Option<DisplayColor>,
        layer: DisplayLayer,
    },
    Text(DisplayText),
    Image {
        asset: AssetId,
        rect: Rect2D,
        layer: DisplayLayer,
    },
    /// Área clicável que leva à ocorrência `target`.
    Link {
        rect: Rect2D,
        target: RepresentationId,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct DisplayList {
    pub page: PageId,
    pub width: f64,
    pub height: f64,
    pub items: Vec<DisplayItem>,
}

impl DisplayList {
    pub fn texts(&self) -> impl Iterator<Item = &DisplayText> {
        self.items.iter().filter_map(|i| match i {
            DisplayItem::Text(t) => Some(t),
            _ => None,
        })
    }
    pub fn links(&self) -> impl Iterator<Item = (Rect2D, RepresentationId)> + '_ {
        self.items.iter().filter_map(|i| match i {
            DisplayItem::Link { rect, target } => Some((*rect, *target)),
            _ => None,
        })
    }
}

/// Posições (mm) dos rótulos de zona: no centro da faixa de até 5 mm entre a
/// área útil e a borda da folha, em cada lado habilitado.
pub fn frame_zone_labels(frame: &FrameTemplate) -> Vec<(String, Point2D)> {
    let u = frame.usable();
    let (min, max) = (u.origin(), u.max());
    let sheet = frame.sheet();
    let band = |space: f64| space.clamp(0.0, 5.0) / 2.0;
    let pt = |x, y| Point2D::new(x, y).ok();
    let mut out = vec![];
    let columns = &frame.zones().columns;
    for (i, (a, b)) in columns.bounds().into_iter().enumerate() {
        let x = min.x() + (a + b) / 2.0;
        let label = columns.style.label(i);
        if columns.labels_start
            && let Some(p) = pt(x, max.y() + band(sheet.height() - max.y()))
        {
            out.push((label.clone(), p));
        }
        if columns.labels_end
            && let Some(p) = pt(x, min.y() - band(min.y()))
        {
            out.push((label, p));
        }
    }
    let rows = &frame.zones().rows;
    for (i, (a, b)) in rows.bounds().into_iter().enumerate() {
        let y = max.y() - (a + b) / 2.0;
        let label = rows.style.label(i);
        if rows.labels_start
            && let Some(p) = pt(min.x() - band(min.x()), y)
        {
            out.push((label.clone(), p));
        }
        if rows.labels_end
            && let Some(p) = pt(max.x() + band(sheet.width() - max.x()), y)
        {
            out.push((label, p));
        }
    }
    out
}

/// Pontos de um arco (graus, anti-horário) com erro máximo de tesselagem.
fn arc_points(center: Point2D, radius: f64, start: f64, sweep: f64) -> Vec<Point2D> {
    let step = if radius > CURVE_TOLERANCE {
        (2.0 * (1.0 - CURVE_TOLERANCE / radius).acos()).to_degrees()
    } else {
        45.0
    };
    let n = ((sweep.abs() / step.max(1e-6)).ceil() as usize).clamp(8, 720);
    (0..=n)
        .filter_map(|i| {
            let a = (start + sweep * i as f64 / n as f64).to_radians();
            center.translated(radius * a.cos(), radius * a.sin()).ok()
        })
        .collect()
}

fn ellipse_points(center: Point2D, rx: f64, ry: f64, rotation: f64) -> Vec<Point2D> {
    let mut points = arc_points(
        Point2D::new(0.0, 0.0).expect("origem"),
        rx.max(ry),
        0.0,
        360.0,
    );
    points.pop();
    let (s, c) = rotation.to_radians().sin_cos();
    let n = points.len();
    (0..n)
        .filter_map(|i| {
            let a = std::f64::consts::TAU * i as f64 / n as f64;
            let (x, y) = (rx * a.cos(), ry * a.sin());
            center.translated(x * c - y * s, x * s + y * c).ok()
        })
        .collect()
}

fn rect_corners(r: Rect2D) -> Vec<Point2D> {
    let (a, b) = (r.origin(), r.max());
    [
        (a.x(), a.y()),
        (b.x(), a.y()),
        (b.x(), b.y()),
        (a.x(), b.y()),
    ]
    .into_iter()
    .filter_map(|(x, y)| Point2D::new(x, y).ok())
    .collect()
}

/// Círculo/arco da primitiva levado ao mundo (espelhos invertem o sentido).
fn world_curve(primitive: &SymbolPrimitive, t: Transform2D) -> Option<(Point2D, f64, f64, f64)> {
    let (center, radius, mut start, mut sweep) = match primitive {
        SymbolPrimitive::Circle { center, radius } => (*center, *radius, 0.0, 360.0),
        SymbolPrimitive::Arc {
            center,
            radius,
            start_degrees,
            sweep_degrees,
        } => (*center, *radius, *start_degrees, *sweep_degrees),
        _ => return None,
    };
    start = start.rem_euclid(360.0);
    if t.mirror_x() {
        start = 180.0 - start;
        sweep = -sweep;
    }
    if t.mirror_y() {
        start = -start;
        sweep = -sweep;
    }
    Some((
        t.apply(center).ok()?,
        radius,
        (start + t.rotation_degrees()).rem_euclid(360.0),
        sweep,
    ))
}

struct Builder<'a> {
    project: &'a Project,
    items: Vec<DisplayItem>,
    /// Camada dos itens acrescentados a seguir.
    layer: DisplayLayer,
}

impl Builder<'_> {
    fn report(&mut self, page: &Page, definition: &ReportDefinition) -> Result<()> {
        let mut table = self
            .project
            .report_table(definition.kind, &definition.filter)?;
        if definition.sort == ReportSort::Reverse {
            table.rows.reverse();
        }
        if !definition.columns.is_empty() {
            let selected: Vec<_> = table
                .columns
                .iter()
                .enumerate()
                .filter(|(_, c)| definition.columns.iter().any(|name| name == c.title))
                .map(|(i, _)| i)
                .collect();
            if !selected.is_empty() {
                table.columns = selected.iter().map(|i| table.columns[*i].clone()).collect();
                for row in &mut table.rows {
                    row.cells = selected.iter().map(|i| row.cells[*i].clone()).collect();
                }
            }
        }
        let area = self
            .project
            .page_frame(page.id())
            .map(FrameTemplate::usable)
            .unwrap_or_else(|| {
                Rect2D::new(
                    Point2D::new(10.0, 10.0).expect("finite"),
                    Size2D::new(page.size().width() - 20.0, page.size().height() - 20.0)
                        .expect("finite"),
                )
                .expect("finite")
            });
        let per_page = self
            .project
            .report_rows_per_page(page.size(), page.frame())?;
        let total_pages = table.rows.len().div_ceil(per_page).max(1);
        let page_index = definition.index as usize;
        let rows = table
            .rows
            .iter()
            .skip(page_index.saturating_mul(per_page))
            .take(per_page);
        let width = area.size().width();
        let sum: f64 = table.columns.iter().map(|c| c.width_mm).sum();
        if sum <= 0.0 {
            return Ok(());
        }
        let sizes: Vec<f64> = table
            .columns
            .iter()
            .map(|c| c.width_mm / sum * width)
            .collect();
        let top = area.max().y();
        let left = area.origin().x();
        self.layer = DisplayLayer::Texts;
        let mut x = left;
        for (column, w) in table.columns.iter().zip(&sizes) {
            let title = truncate_report_text(column.title, *w - 2.0, 2.5);
            self.text(
                Point2D::new(x + 1.0, top - 1.0)?,
                &title,
                FrameTextStyle::new(2.5, TextAlign::TopLeft),
                DisplayColor::Ink,
            );
            x += w;
        }
        self.report_grid(left, top, &sizes, 8.0);
        let mut y = top - 8.0;
        for row in rows {
            let mut x = left;
            for (cell, w) in row.cells.iter().zip(&sizes) {
                let value = truncate_report_text(&cell.text(), *w - 2.0, 2.5);
                self.text(
                    Point2D::new(x + 1.0, y - 1.0)?,
                    &value,
                    FrameTextStyle::new(2.5, TextAlign::TopLeft),
                    DisplayColor::Ink,
                );
                if let Some(target) = cell.target
                    && let Ok(rect) = Rect2D::new(Point2D::new(x, y - 6.0)?, Size2D::new(*w, 6.0)?)
                {
                    self.items.push(DisplayItem::Link { rect, target });
                }
                x += w;
            }
            self.report_grid(left, y, &sizes, 6.0);
            y -= 6.0;
        }
        let footer = format!("página {} de {} do relatório", page_index + 1, total_pages);
        self.text(
            Point2D::new(left, area.origin().y() + 4.0)?,
            &footer,
            FrameTextStyle::new(2.5, TextAlign::BottomLeft),
            DisplayColor::Ink,
        );
        if self.project.report_stale(definition.group)? {
            self.text(
                Point2D::new(left, area.origin().y() + 8.0)?,
                "Relatório desatualizado",
                FrameTextStyle::new(2.5, TextAlign::BottomLeft),
                DisplayColor::Reference,
            );
        }
        Ok(())
    }

    fn report_grid(&mut self, left: f64, top: f64, widths: &[f64], height: f64) {
        let mut x = left;
        self.stroke(
            vec![
                Point2D::new(x, top).expect("finite"),
                Point2D::new(x, top - height).expect("finite"),
            ],
            false,
            0.15,
            DisplayColor::Ink,
        );
        for width in widths {
            x += width;
            self.stroke(
                vec![
                    Point2D::new(x, top).expect("finite"),
                    Point2D::new(x, top - height).expect("finite"),
                ],
                false,
                0.15,
                DisplayColor::Ink,
            );
        }
        self.stroke(
            vec![
                Point2D::new(left, top).expect("finite"),
                Point2D::new(x, top).expect("finite"),
            ],
            false,
            0.15,
            DisplayColor::Ink,
        );
        self.stroke(
            vec![
                Point2D::new(left, top - height).expect("finite"),
                Point2D::new(x, top - height).expect("finite"),
            ],
            false,
            0.15,
            DisplayColor::Ink,
        );
    }
    fn path(
        &mut self,
        points: Vec<Point2D>,
        closed: bool,
        stroke: Option<DisplayStroke>,
        fill: Option<DisplayColor>,
    ) {
        if points.len() < 2 || (stroke.is_none() && fill.is_none()) {
            return;
        }
        let layer = self.layer;
        self.items.push(DisplayItem::Path {
            points,
            closed,
            stroke,
            fill,
            layer,
        });
    }
    fn stroke(&mut self, points: Vec<Point2D>, closed: bool, width: f64, color: DisplayColor) {
        self.path(
            points,
            closed,
            Some(DisplayStroke {
                width,
                dash: vec![],
                color,
            }),
            None,
        );
    }

    /// Texto de moldura/folha: corpo `style.height`, alinhamento de 9 pontos
    /// numa caixa `largura × 1,2·corpo`, rotação anti-horária na origem.
    fn text(&mut self, origin: Point2D, text: &str, style: FrameTextStyle, color: DisplayColor) {
        if text.trim().is_empty() || style.height <= 0.0 {
            return;
        }
        let size = style.height;
        let (w, h) = (font::text_width(text, size), size * LINE_HEIGHT);
        let (fx, fy) = style.align.fractions();
        let (s, c) = style.rotation_degrees.to_radians().sin_cos();
        let (ox, oy) = (-fx * w, -fy * h);
        self.items.push(DisplayItem::Text(DisplayText {
            text: text.to_owned(),
            matrix: [
                c,
                s,
                -s,
                c,
                origin.x() + c * ox - s * oy,
                origin.y() + s * ox + c * oy,
            ],
            size,
            baseline: size * BASELINE,
            width: None,
            color,
            layer: self.layer,
        }));
    }

    /// Texto de símbolo: caixa nominal `[0, largura] × [0, altura]` levada à
    /// página por `map` (afim); as maiúsculas ocupam a altura, como na tela.
    fn mapped_text(
        &mut self,
        text: &str,
        (height, width): (f64, f64),
        map: &dyn Fn(f64, f64) -> Option<Point2D>,
        color: DisplayColor,
    ) {
        if text.trim().is_empty() || height <= 0.0 || width <= 0.0 {
            return;
        }
        let (Some(p0), Some(px), Some(py)) = (map(0.0, 0.0), map(1.0, 0.0), map(0.0, 1.0)) else {
            return;
        };
        self.items.push(DisplayItem::Text(DisplayText {
            text: text.to_owned(),
            matrix: [
                px.x() - p0.x(),
                px.y() - p0.y(),
                py.x() - p0.x(),
                py.y() - p0.y(),
                p0.x(),
                p0.y(),
            ],
            size: height / font::CAP_HEIGHT,
            baseline: 0.0,
            width: Some(width),
            color,
            layer: self.layer,
        }));
    }

    fn frame(&mut self, page: PageId, frame: &FrameTemplate) {
        self.layer = DisplayLayer::Frame;
        let watermark = |e: &&FrameElement| {
            matches!(
                e,
                FrameElement::Text { style, .. } | FrameElement::Special { style, .. } if style.watermark
            )
        };
        let (marks, rest): (Vec<_>, Vec<_>) = frame.elements().iter().partition(watermark);
        for element in marks.into_iter().chain(rest) {
            let ink = DisplayColor::Ink;
            match element {
                FrameElement::Line { start, end, width } => {
                    self.stroke(vec![*start, *end], false, *width, ink);
                }
                FrameElement::Rectangle { rect, width } => {
                    self.stroke(rect_corners(*rect), true, *width, ink);
                }
                FrameElement::Circle {
                    center,
                    radius,
                    width,
                } => {
                    let mut points = arc_points(*center, *radius, 0.0, 360.0);
                    points.pop();
                    self.stroke(points, true, *width, ink);
                }
                FrameElement::Arc {
                    center,
                    radius,
                    start_degrees,
                    sweep_degrees,
                    width,
                } => self.stroke(
                    arc_points(*center, *radius, *start_degrees, *sweep_degrees),
                    false,
                    *width,
                    ink,
                ),
                FrameElement::Text {
                    origin,
                    text,
                    style,
                } => {
                    let color = if style.watermark {
                        DisplayColor::Watermark
                    } else {
                        ink
                    };
                    self.text(*origin, text.get(None), *style, color);
                }
                FrameElement::Special {
                    origin,
                    field,
                    index,
                    style,
                } => {
                    let color = if style.watermark {
                        DisplayColor::Watermark
                    } else {
                        ink
                    };
                    let value = self.project.resolve_special(page, field, *index);
                    self.text(*origin, &value, *style, color);
                }
                FrameElement::Image { rect, asset } => self.items.push(DisplayItem::Image {
                    asset: *asset,
                    rect: *rect,
                    layer: DisplayLayer::Frame,
                }),
            }
        }
        for (label, origin) in frame_zone_labels(frame) {
            self.text(
                origin,
                &label,
                FrameTextStyle::new(LABEL_HEIGHT, TextAlign::Center),
                DisplayColor::Ink,
            );
        }
    }

    /// Margem e cabeçalho provisórios de página sem moldura (como na tela).
    fn provisional_frame(&mut self, page: &Page) {
        self.layer = DisplayLayer::Guides;
        let (w, h) = (page.size().width(), page.size().height());
        let margin = 10.0_f64.min(w * 0.1).min(h * 0.1);
        if let (Ok(origin), Ok(size)) = (
            Point2D::new(margin, margin),
            Size2D::new(w - 2.0 * margin, h - 2.0 * margin),
        ) && let Ok(inner) = Rect2D::new(origin, size)
        {
            self.stroke(rect_corners(inner), true, 0.18, DisplayColor::Guide);
        }
        self.layer = DisplayLayer::Frame;
        if let Ok(origin) = Point2D::new(3.0, h - 3.0) {
            let header = format!(
                "{} · {}  |  {} × {} mm",
                page.number().as_str(),
                page.name().as_str(),
                w,
                h
            );
            self.text(
                origin,
                &header,
                FrameTextStyle::new(3.0, TextAlign::TopLeft),
                DisplayColor::Ink,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn primitive(
        &mut self,
        primitive: &SymbolPrimitive,
        style: PrimitiveStyle,
        t: Transform2D,
        base_width: f64,
        color: DisplayColor,
        text: &dyn Fn(&SymbolTextField) -> String,
    ) {
        use SymbolPrimitive as P;
        let chars = |s: &str| s.chars().count() as f64;
        let (points, closed) = match primitive {
            P::Text {
                origin,
                text,
                height,
            } => {
                let origin = *origin;
                let map = move |x: f64, y: f64| {
                    origin.translated(x, y).ok().and_then(|p| t.apply(p).ok())
                };
                self.mapped_text(text, (*height, height * 0.6 * chars(text)), &map, color);
                return;
            }
            P::DynamicText {
                origin,
                field,
                height,
                align,
                rotation_degrees,
            } => {
                let value = text(field);
                let width = height * 0.6 * chars(&value);
                let (fx, fy) = align.fractions();
                let (s, c) = rotation_degrees.to_radians().sin_cos();
                let (origin, height) = (*origin, *height);
                let map = move |x: f64, y: f64| {
                    let (lx, ly) = (x - fx * width, y - fy * height);
                    origin
                        .translated(lx * c - ly * s, lx * s + ly * c)
                        .ok()
                        .and_then(|p| t.apply(p).ok())
                };
                self.mapped_text(&value, (height, width), &map, color);
                return;
            }
            P::Anchor { .. } => return,
            P::Circle { .. } | P::Arc { .. } => {
                let Some((center, radius, start, sweep)) = world_curve(primitive, t) else {
                    return;
                };
                let mut points = arc_points(center, radius, start, sweep);
                let closed = matches!(primitive, P::Circle { .. });
                if closed {
                    points.pop();
                }
                (points, closed)
            }
            other => {
                let (local, closed) = match other {
                    P::Line { start, end } => (vec![*start, *end], false),
                    P::Polyline { points, closed } => (points.clone(), *closed),
                    P::Rectangle { rect } => (rect_corners(*rect), true),
                    P::Ellipse {
                        center,
                        radius_x,
                        radius_y,
                        rotation_degrees,
                    } => (
                        ellipse_points(*center, *radius_x, *radius_y, *rotation_degrees),
                        true,
                    ),
                    _ => return,
                };
                let Some(world) = local
                    .iter()
                    .map(|p| t.apply(*p).ok())
                    .collect::<Option<Vec<_>>>()
                else {
                    return;
                };
                (world, closed)
            }
        };
        let width = style.width_mm.map_or(base_width, |w| w.max(base_width));
        let fill = (style.fill && closed && points.len() > 2).then_some(color);
        self.path(
            points,
            closed,
            Some(DisplayStroke {
                width,
                dash: style.dash.pattern_mm().to_vec(),
                color,
            }),
            fill,
        );
    }

    fn representations(&mut self, page: PageId) {
        let project = self.project;
        let mut reps = project.representations_on_page(page).unwrap_or_default();
        reps.sort_by_key(|r| {
            (
                r.visual().layer,
                u8::from(project.connection_route(r.id()).is_none()),
                r.id(),
            )
        });
        for rep in reps {
            if !rep.visual().visible {
                continue;
            }
            let t = rep.transform();
            if let Some(route) = project.connection_route(rep.id()) {
                self.layer = DisplayLayer::Texts;
                let points = route.points().to_vec();
                if let Some(number) = project
                    .connection(route.connection_id())
                    .and_then(Connection::number)
                    && let Some((origin, align)) =
                        wire_number_placement(&points, route.number_label_position())
                {
                    self.text(
                        origin,
                        number.as_str(),
                        FrameTextStyle::new(LABEL_HEIGHT, align),
                        DisplayColor::Ink,
                    );
                }
                self.layer = DisplayLayer::Wires;
                self.stroke(points, false, ROUTE_STROKE, DisplayColor::Ink);
                continue;
            }
            if let Some(graphic) = rep.graphic() {
                self.layer = DisplayLayer::Texts;
                if let Ok(primitive) = graphic.drawable() {
                    self.primitive(
                        &primitive,
                        graphic.style(),
                        t,
                        graphic.stroke_mm(),
                        DisplayColor::Ink,
                        &|field| field.placeholder(),
                    );
                }
            } else if let Some(d) = rep.symbol().and_then(|id| project.symbol_definition(id)) {
                self.layer = DisplayLayer::Symbols;
                let rid = rep.id();
                let text = |field: &SymbolTextField| {
                    project
                        .symbol_text(rid, field)
                        .unwrap_or_else(|| field.placeholder())
                };
                for (i, primitive) in d.primitives().iter().enumerate() {
                    self.primitive(
                        primitive,
                        d.style(i),
                        t,
                        SYMBOL_STROKE,
                        DisplayColor::Ink,
                        &text,
                    );
                }
            } else if let Some(local) = rep.local_bounds() {
                self.layer = DisplayLayer::Guides;
                let corners: Option<Vec<_>> = rect_corners(local)
                    .into_iter()
                    .map(|p| t.apply(p).ok())
                    .collect();
                if let Some(corners) = corners {
                    self.stroke(corners, true, 0.18, DisplayColor::Guide);
                }
            }
            self.layer = DisplayLayer::Texts;
            if let Some(label) = &rep.visual().label
                && let Ok(origin) = t.translation().translated(1.5, -1.5)
            {
                self.text(
                    origin,
                    label.as_str(),
                    FrameTextStyle::new(LABEL_HEIGHT, TextAlign::TopLeft),
                    DisplayColor::Ink,
                );
            }
        }
    }

    fn references(&mut self, page: PageId) {
        self.layer = DisplayLayer::References;
        let style = FrameTextStyle::new(SHEET_REFERENCE_HEIGHT, TextAlign::TopLeft);
        let line = SHEET_REFERENCE_HEIGHT * LINE_HEIGHT;
        for reference in self.project.sheet_references(page) {
            let text = reference.text();
            self.text(reference.origin, &text, style, DisplayColor::Reference);
            let mut x = 0.0;
            for segment in &reference.segments {
                let w = font::text_width(&segment.text, SHEET_REFERENCE_HEIGHT);
                if let Some(target) = segment.target
                    && w > 0.0
                    && let (Ok(origin), Ok(size)) =
                        (reference.origin.translated(x, -line), Size2D::new(w, line))
                    && let Ok(rect) = Rect2D::new(origin, size)
                {
                    self.items.push(DisplayItem::Link { rect, target });
                }
                x += w;
            }
        }
    }

    fn junctions(&mut self, page: PageId) {
        self.layer = DisplayLayer::Junctions;
        let project = self.project;
        for j in project
            .connection_junctions()
            .filter(|j| j.page_id() == page)
        {
            let mut points = arc_points(j.position(), JUNCTION_RADIUS, 0.0, 360.0);
            points.pop();
            self.path(points, true, None, Some(DisplayColor::Ink));
        }
    }
}

fn truncate_report_text(text: &str, width: f64, size: f64) -> String {
    if width <= 0.0 {
        return String::new();
    }
    if font::text_width(text, size) <= width {
        return text.to_owned();
    }
    if font::text_width("…", size) > width {
        return String::new();
    }
    let mut result = String::new();
    for c in text.chars() {
        let next = format!("{result}{c}…");
        if font::text_width(&next, size) > width {
            break;
        }
        result.push(c);
    }
    result.push('…');
    result
}

/// Afastamento (mm) do número do fio em relação ao traço.
pub const WIRE_NUMBER_GAP: f64 = 0.8;

/// Onde escrever o número do fio: a posição guardada na rota (texto à direita
/// dela) ou, sem ela, o meio do trecho mais longo — acima de trecho
/// horizontal, à direita de trecho vertical, para não ficar sobre o traço.
pub fn wire_number_placement(
    points: &[Point2D],
    explicit: Option<Point2D>,
) -> Option<(Point2D, TextAlign)> {
    if let Some(at) = explicit {
        return Some((at, TextAlign::MiddleLeft));
    }
    let pair = points.windows(2).max_by(|a, b| {
        let len = |p: &[Point2D]| (p[1].x() - p[0].x()).hypot(p[1].y() - p[0].y());
        len(a).total_cmp(&len(b))
    })?;
    let (dx, dy) = (pair[1].x() - pair[0].x(), pair[1].y() - pair[0].y());
    let (mx, my) = (
        (pair[0].x() + pair[1].x()) / 2.0,
        (pair[0].y() + pair[1].y()) / 2.0,
    );
    if dx.abs() >= dy.abs() {
        Point2D::new(mx, my + WIRE_NUMBER_GAP)
            .ok()
            .map(|p| (p, TextAlign::BottomCenter))
    } else {
        Point2D::new(mx + WIRE_NUMBER_GAP, my)
            .ok()
            .map(|p| (p, TextAlign::MiddleLeft))
    }
}
/// Display list of a page. An extension trait because methods of [`Project`]
/// can only be declared in the model crate; the method name is unchanged.
pub trait PageDisplayList {
    /// Lista de exibição da página: moldura, conteúdo, referências e junções,
    /// na ordem de desenho da tela.
    fn page_display_list(&self, page: PageId) -> Result<DisplayList>;
}
impl PageDisplayList for Project {
    /// Lista de exibição da página: moldura, conteúdo, referências e junções,
    /// na ordem de desenho da tela.
    fn page_display_list(&self, page: PageId) -> Result<DisplayList> {
        let p = self
            .page(page)
            .ok_or(cadcraft_model::error::DomainError::MissingEntity(
                EntityRef::Page(page),
            ))?;
        let mut builder = Builder {
            project: self,
            items: vec![],
            layer: DisplayLayer::Frame,
        };
        match self.page_frame(page) {
            Some(frame) => builder.frame(page, frame),
            None => builder.provisional_frame(p),
        }
        if let Some(definition) = p.report() {
            builder.report(p, definition)?;
        }
        builder.representations(page);
        builder.references(page);
        builder.junctions(page);
        Ok(DisplayList {
            page,
            width: p.size().width(),
            height: p.size().height(),
            items: builder.items,
        })
    }
}
