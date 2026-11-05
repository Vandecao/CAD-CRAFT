// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 14: editor de molduras (equivalente a Dados-mestre → Moldura do
//! EPLAN). Ocupa a área de documentos enquanto aberto. Cada edição confirmada
//! gera uma nova versão da moldura (`FrameEdit::UpsertFrame`) no histórico
//! comum; imagens novas entram na mesma transação (`FrameEdit::Batch`).
use crate::{
    app::{CadApp, Level},
    camera::{Camera, grid_points, snap},
    editing::GraphicEdit,
    frame_view, theme,
};
use cad_craft::domain::*;
use eframe::egui::{self, Key, PointerButton, Rect, Sense, Stroke, Ui};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FrameTool {
    #[default]
    Select,
    Line,
    Rectangle,
    Circle,
    Arc,
    Text,
    Special,
    Image,
}
impl FrameTool {
    pub const ALL: [Self; 8] = [
        Self::Select,
        Self::Line,
        Self::Rectangle,
        Self::Circle,
        Self::Arc,
        Self::Text,
        Self::Special,
        Self::Image,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Select => "Selecionar",
            Self::Line => "Linha",
            Self::Rectangle => "Retângulo",
            Self::Circle => "Círculo",
            Self::Arc => "Arco",
            Self::Text => "Texto",
            Self::Special => "Texto especial",
            Self::Image => "Imagem",
        }
    }
    /// Cliques necessários para criar o elemento.
    fn clicks(self) -> usize {
        match self {
            Self::Select => 0,
            Self::Text | Self::Special | Self::Image => 1,
            Self::Line | Self::Rectangle | Self::Circle => 2,
            Self::Arc => 3,
        }
    }
    fn hint(self) -> &'static str {
        match self {
            Self::Select => {
                "Clique seleciona (Ctrl acumula); arraste move; Delete exclui; Ctrl+D duplica."
            }
            Self::Line => "Clique no início e no fim da linha.",
            Self::Rectangle => "Clique em dois cantos opostos.",
            Self::Circle => "Clique no centro e depois num ponto do contorno.",
            Self::Arc => "Clique no centro, no início e no fim (sentido anti-horário).",
            Self::Text => "Clique para inserir; edite o texto nas propriedades.",
            Self::Special => "Clique para inserir; escolha o campo nas propriedades.",
            Self::Image => "Clique para posicionar a imagem escolhida (40 mm de largura).",
        }
    }
}

const FIELDS: [&str; 16] = [
    "Nome do projeto",
    "Descrição do projeto",
    "Identificação do projeto",
    "Propriedade do projeto",
    "Documento",
    "Número da página",
    "Descrição da página",
    "Função (=)",
    "Localização (+)",
    "Posição da página",
    "Total de páginas do documento",
    "Total de páginas do projeto",
    "Página anterior",
    "Página seguinte",
    "Propriedade da página",
    "Nome da moldura",
];
fn field_index(field: &SpecialField) -> (usize, String) {
    use SpecialField as F;
    match field {
        F::ProjectName => (0, String::new()),
        F::ProjectDescription => (1, String::new()),
        F::ProjectIdentification => (2, String::new()),
        F::ProjectProperty(k) => (3, k.as_str().to_owned()),
        F::DocumentName => (4, String::new()),
        F::PageNumber => (5, String::new()),
        F::PageName => (6, String::new()),
        F::PageFunctionalPath => (7, String::new()),
        F::PageLocationPath => (8, String::new()),
        F::PagePosition => (9, String::new()),
        F::PageCountInDocument => (10, String::new()),
        F::PageCountInProject => (11, String::new()),
        F::PreviousPageNumber => (12, String::new()),
        F::NextPageNumber => (13, String::new()),
        F::PageProperty(k) => (14, k.as_str().to_owned()),
        F::FrameName => (15, String::new()),
    }
}
fn field_from(index: usize, key: &str) -> Result<SpecialField, String> {
    use SpecialField as F;
    let key = || Text::new(key.trim()).map_err(|_| "informe a chave da propriedade".to_owned());
    Ok(match index {
        0 => F::ProjectName,
        1 => F::ProjectDescription,
        2 => F::ProjectIdentification,
        3 => F::ProjectProperty(key()?),
        4 => F::DocumentName,
        5 => F::PageNumber,
        6 => F::PageName,
        7 => F::PageFunctionalPath,
        8 => F::PageLocationPath,
        9 => F::PagePosition,
        10 => F::PageCountInDocument,
        11 => F::PageCountInProject,
        12 => F::PreviousPageNumber,
        13 => F::NextPageNumber,
        14 => F::PageProperty(key()?),
        _ => F::FrameName,
    })
}
const ALIGNS: [(TextAlign, &str); 9] = [
    (TextAlign::BottomLeft, "Inferior esquerdo"),
    (TextAlign::BottomCenter, "Inferior centro"),
    (TextAlign::BottomRight, "Inferior direito"),
    (TextAlign::MiddleLeft, "Meio esquerdo"),
    (TextAlign::Center, "Centro"),
    (TextAlign::MiddleRight, "Meio direito"),
    (TextAlign::TopLeft, "Superior esquerdo"),
    (TextAlign::TopCenter, "Superior centro"),
    (TextAlign::TopRight, "Superior direito"),
];

/// Formulário do elemento selecionado (valores em mm e graus).
#[derive(Clone, Debug, Default)]
struct ElementForm {
    nums: [f64; 6],
    text: String,
    translations: String,
    field: usize,
    key: String,
    index: u32,
    style: Option<FrameTextStyle>,
    keep_aspect: bool,
}
/// Formulário da moldura.
#[derive(Clone, Debug, Default)]
struct FrameForm {
    name: String,
    description: String,
    sheet: [f64; 2],
    usable: [f64; 4],
    axes: [AxisForm; 2],
}
#[derive(Clone, Debug, Default)]
struct AxisForm {
    count: usize,
    size: f64,
    custom: String,
    letters: bool,
    start: u32,
    labels_start: bool,
    labels_end: bool,
}
impl AxisForm {
    fn from(axis: &ZoneAxis) -> Self {
        let uniform = axis.sizes.windows(2).all(|w| (w[0] - w[1]).abs() < 1e-9);
        let (letters, start) = match axis.style {
            ZoneLabelStyle::Numbers { start } => (false, start),
            ZoneLabelStyle::Letters { start } => (true, start),
        };
        Self {
            count: axis.sizes.len(),
            size: axis.sizes.first().copied().unwrap_or(40.0),
            custom: if uniform {
                String::new()
            } else {
                axis.sizes
                    .iter()
                    .map(|s| format!("{s}"))
                    .collect::<Vec<_>>()
                    .join("; ")
            },
            letters,
            start,
            labels_start: axis.labels_start,
            labels_end: axis.labels_end,
        }
    }
    fn build(&self) -> Result<ZoneAxis, String> {
        let sizes = if self.custom.trim().is_empty() {
            vec![self.size; self.count]
        } else {
            self.custom
                .split(';')
                .map(|s| s.trim().replace(',', ".").parse::<f64>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| "tamanhos individuais: números separados por ;".to_owned())?
        };
        Ok(ZoneAxis {
            sizes,
            style: if self.letters {
                ZoneLabelStyle::Letters { start: self.start }
            } else {
                ZoneLabelStyle::Numbers { start: self.start }
            },
            labels_start: self.labels_start,
            labels_end: self.labels_end,
        })
    }
}

#[derive(Default)]
pub struct FrameEditor {
    /// Moldura aberta; `None` = editor fechado.
    pub frame: Option<FrameTemplateId>,
    pub camera: Camera,
    pub needs_fit: bool,
    pub tool: FrameTool,
    pub selection: BTreeSet<usize>,
    /// Área de desenho do último quadro (testes e automação).
    pub viewport: Option<Rect>,
    pending: Vec<Point2D>,
    drag: Option<(Point2D, Point2D)>,
    pending_image: Option<ImageAsset>,
    element_form: ElementForm,
    frame_form: FrameForm,
    form_key: Option<(FrameTemplateId, u32, Vec<usize>)>,
}
impl FrameEditor {
    pub fn is_open(&self) -> bool {
        self.frame.is_some()
    }
    /// Cancela ferramenta e pontos pendentes; `true` se havia algo a cancelar.
    pub fn cancel(&mut self) -> bool {
        let busy =
            !self.pending.is_empty() || self.drag.is_some() || self.tool != FrameTool::Select;
        self.pending.clear();
        self.drag = None;
        self.pending_image = None;
        self.tool = FrameTool::Select;
        busy
    }
}

fn pt(x: f64, y: f64) -> Option<Point2D> {
    Point2D::new(x, y).ok()
}
fn dist_segment(p: Point2D, a: Point2D, b: Point2D) -> f64 {
    let (dx, dy) = (b.x() - a.x(), b.y() - a.y());
    let len2 = dx * dx + dy * dy;
    let t = if len2 == 0.0 {
        0.0
    } else {
        (((p.x() - a.x()) * dx + (p.y() - a.y()) * dy) / len2).clamp(0.0, 1.0)
    };
    let (x, y) = (a.x() + t * dx, a.y() + t * dy);
    ((p.x() - x).powi(2) + (p.y() - y).powi(2)).sqrt()
}
fn dist_rect(p: Point2D, r: Rect2D, filled: bool) -> f64 {
    let (a, b) = (r.origin(), r.max());
    if filled && r.contains(p) {
        return 0.0;
    }
    let c = [
        pt(a.x(), a.y()),
        pt(b.x(), a.y()),
        pt(b.x(), b.y()),
        pt(a.x(), b.y()),
    ];
    (0..4)
        .filter_map(|i| Some(dist_segment(p, c[i]?, c[(i + 1) % 4]?)))
        .fold(f64::INFINITY, f64::min)
}
/// Caixa aproximada do texto (mesmo critério do desenho dos símbolos).
fn text_box(origin: Point2D, chars: usize, style: &FrameTextStyle) -> Option<Rect2D> {
    let (w, h) = (style.height * 0.6 * chars.max(1) as f64, style.height);
    let (fx, fy) = style.align.fractions();
    Rect2D::new(
        pt(origin.x() - fx * w, origin.y() - fy * h)?,
        Size2D::new(w, h).ok()?,
    )
    .ok()
}
/// Distância (mm) do ponto ao elemento, para seleção.
pub fn element_distance(project: &Project, e: &FrameElement, p: Point2D) -> f64 {
    match e {
        FrameElement::Line { start, end, .. } => dist_segment(p, *start, *end),
        FrameElement::Rectangle { rect, .. } => dist_rect(p, *rect, false),
        FrameElement::Image { rect, .. } => dist_rect(p, *rect, true),
        FrameElement::Circle { center, radius, .. } | FrameElement::Arc { center, radius, .. } => {
            (center.distance_to(p).unwrap_or(f64::INFINITY) - radius).abs()
        }
        FrameElement::Text {
            origin,
            text,
            style,
        } => text_box(*origin, text.default.chars().count(), style)
            .map_or(f64::INFINITY, |r| dist_rect(p, r, true)),
        FrameElement::Special {
            origin,
            field,
            style,
            ..
        } => {
            let _ = project;
            text_box(*origin, field.label().chars().count() + 2, style)
                .map_or(f64::INFINITY, |r| dist_rect(p, r, true))
        }
    }
}

/// Lê e valida uma imagem: limites e cabeçalho (Core) e decodificação completa.
pub fn load_image_file(path: &std::path::Path) -> Result<ImageAsset, String> {
    let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
    if meta.len() > MAX_IMAGE_BYTES as u64 {
        return Err("imagem acima de 10 MB".into());
    }
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    frame_view::decode_image(&bytes)?;
    ImageAsset::from_bytes(bytes).map_err(|e| e.to_string())
}

impl CadApp {
    pub fn open_frame_editor(&mut self, id: FrameTemplateId) {
        self.symbol_editor = crate::symbol_editor::SymbolEditor::default();
        self.frame_editor = FrameEditor {
            frame: Some(id),
            needs_fit: true,
            ..Default::default()
        };
        self.backstage = None;
    }
    pub fn close_frame_editor(&mut self) {
        self.frame_editor = FrameEditor::default();
    }
    fn editor_frame(&self) -> Option<FrameTemplate> {
        let id = self.frame_editor.frame?;
        self.session.project()?.frame(id).cloned()
    }
    /// Confirma o rascunho como nova versão; `extra` vem antes (ex.: imagem).
    pub fn commit_frame_draft(&mut self, draft: FrameDraft, extra: Vec<FrameEdit>) -> bool {
        let Some(frame) = self.editor_frame() else {
            return false;
        };
        match frame.edited(draft) {
            Ok(edited) => {
                let version = edited.version();
                let mut edits = extra;
                edits.push(FrameEdit::UpsertFrame(edited));
                let edit = if edits.len() == 1 {
                    edits.pop().expect("one edit")
                } else {
                    FrameEdit::Batch(edits)
                };
                self.graphic_edit(GraphicEdit::Frame(edit));
                self.editor_frame().is_some_and(|f| f.version() == version)
            }
            Err(e) => {
                self.log(Level::Error, e.to_string());
                false
            }
        }
    }
    pub fn set_frame_tool(&mut self, tool: FrameTool) {
        self.frame_editor.pending.clear();
        self.frame_editor.drag = None;
        self.frame_editor.tool = tool;
        if tool == FrameTool::Image {
            self.frame_editor.pending_image = None;
            match self.files.picker.pick_image() {
                Some(path) => match load_image_file(&path) {
                    Ok(asset) => self.frame_editor.pending_image = Some(asset),
                    Err(e) => {
                        self.log(Level::Error, format!("{}: {e}", path.display()));
                        self.frame_editor.tool = FrameTool::Select;
                    }
                },
                None => self.frame_editor.tool = FrameTool::Select,
            }
        }
    }
    fn editor_delete_selection(&mut self) {
        let Some(frame) = self.editor_frame() else {
            return;
        };
        if self.frame_editor.selection.is_empty() {
            return;
        }
        let mut draft = frame.draft();
        let selected = std::mem::take(&mut self.frame_editor.selection);
        draft.elements = draft
            .elements
            .into_iter()
            .enumerate()
            .filter(|(i, _)| !selected.contains(i))
            .map(|(_, e)| e)
            .collect();
        self.commit_frame_draft(draft, vec![]);
    }
    fn editor_transform_selection(&mut self, duplicate: bool, dx: f64, dy: f64, align: bool) {
        let Some(frame) = self.editor_frame() else {
            return;
        };
        let mut draft = frame.draft();
        let mut added = BTreeSet::new();
        let grid = self.preferences.grid_mm;
        for &i in &self.frame_editor.selection {
            let Some(e) = draft.elements.get(i).cloned() else {
                continue;
            };
            let (mx, my) = if align {
                let a = e.anchor();
                let s = snap(a, grid, true);
                (s.x() - a.x(), s.y() - a.y())
            } else {
                (dx, dy)
            };
            let Ok(moved) = e.translated(mx, my) else {
                continue;
            };
            if duplicate {
                added.insert(draft.elements.len());
                draft.elements.push(moved);
            } else {
                draft.elements[i] = moved;
            }
        }
        if self.commit_frame_draft(draft, vec![]) && duplicate {
            self.frame_editor.selection = added;
        }
    }

    pub(crate) fn show_frame_editor(&mut self, ui: &mut Ui) {
        let Some(frame) = self.editor_frame() else {
            self.close_frame_editor();
            return;
        };
        ui.horizontal_wrapped(|ui| {
            ui.strong(format!(
                "Moldura: {} · v{}",
                frame.name().as_str(),
                frame.version()
            ));
            ui.separator();
            for tool in FrameTool::ALL {
                if ui
                    .selectable_label(self.frame_editor.tool == tool, tool.label())
                    .clicked()
                {
                    self.set_frame_tool(tool);
                }
            }
            ui.separator();
            let any = !self.frame_editor.selection.is_empty();
            if ui
                .add_enabled(any, egui::Button::new("Duplicar seleção"))
                .clicked()
            {
                self.editor_transform_selection(true, 5.0, -5.0, false);
            }
            if ui
                .add_enabled(any, egui::Button::new("Excluir seleção"))
                .clicked()
            {
                self.editor_delete_selection();
            }
            if ui
                .add_enabled(any, egui::Button::new("Alinhar à grade"))
                .clicked()
            {
                self.editor_transform_selection(false, 0.0, 0.0, true);
            }
            if ui.button("Ajustar à tela").clicked() {
                self.frame_editor.needs_fit = true;
            }
            if ui.button("Fechar editor").clicked() {
                self.close_frame_editor();
            }
        });
        if !self.frame_editor.is_open() {
            return;
        }
        ui.small(self.frame_editor.tool.hint());
        egui::SidePanel::right("frame_editor_properties")
            .resizable(false)
            .exact_width(270.0)
            .frame(egui::Frame::new().fill(theme::PANEL).inner_margin(6))
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| self.frame_properties_ui(ui, &frame));
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::TITLE))
            .show_inside(ui, |ui| self.frame_canvas(ui, &frame));
    }

    fn frame_canvas(&mut self, ui: &mut Ui, frame: &FrameTemplate) {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        self.frame_editor.viewport = Some(rect);
        let sheet = Rect2D::new(pt(0.0, 0.0).expect("origin"), frame.sheet()).expect("sheet");
        if self.frame_editor.needs_fit && rect.width() > 40.0 && rect.height() > 40.0 {
            self.frame_editor.camera.fit(sheet, rect.shrink(24.0));
            self.frame_editor.needs_fit = false;
        }
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0
                && let Some(cursor) = response.hover_pos()
            {
                self.frame_editor
                    .camera
                    .zoom_at((scroll as f64 / 200.0).exp(), cursor, rect);
            }
        }
        if response.dragged_by(PointerButton::Middle) {
            self.frame_editor.camera.pan(response.drag_delta());
        }
        let camera = self.frame_editor.camera;
        let grid = self.preferences.grid_mm;
        let snapping = self.preferences.snap_enabled;
        let world = |p: egui::Pos2| {
            camera
                .screen_to_world(p, rect)
                .map(|w| snap(w, grid, snapping))
        };
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0, theme::TITLE);
        if let Some(paper) = camera.clipped_rect(sheet, rect) {
            painter.rect_filled(paper, 0, theme::PAPER);
        }
        if self.preferences.grid_visible {
            for p in grid_points(&camera, rect, grid) {
                if camera
                    .screen_to_world(p, rect)
                    .is_some_and(|w| sheet.contains(w))
                {
                    painter.circle_filled(p, 0.8, theme::GRID_DOT);
                }
            }
        }
        // Área útil e divisão em zonas (guias do editor).
        let u = frame.usable();
        if let Some(r) = camera.clipped_rect(u, rect) {
            painter.rect_stroke(
                r,
                0,
                Stroke::new(1.0_f32, theme::GRID),
                egui::StrokeKind::Inside,
            );
        }
        let Some(project) = self.session.project() else {
            return;
        };
        let (marks, rest): (Vec<_>, Vec<_>) = frame
            .elements()
            .iter()
            .enumerate()
            .partition(|(_, e)| matches!(e, FrameElement::Text { style, .. } | FrameElement::Special { style, .. } if style.watermark));
        let drag = self.frame_editor.drag;
        for (i, e) in marks.into_iter().chain(rest) {
            let selected = self.frame_editor.selection.contains(&i);
            let shown = match (selected, drag) {
                (true, Some((a, b))) => e
                    .translated(b.x() - a.x(), b.y() - a.y())
                    .unwrap_or(e.clone()),
                _ => e.clone(),
            };
            frame_view::paint_element(
                &painter,
                project,
                None,
                &shown,
                camera,
                rect,
                selected.then_some(theme::ACCENT),
            );
        }
        frame_view::paint_zone_labels(&painter, frame, camera, rect);
        // Pontos pendentes e prévia.
        let cursor = response.hover_pos().and_then(world);
        let mut preview = self.frame_editor.pending.clone();
        if let Some(c) = cursor
            && self.frame_editor.tool != FrameTool::Select
        {
            preview.push(c);
        }
        if let Some(element) = self.build_tool_element(&preview, true) {
            frame_view::paint_element(
                &painter,
                project,
                None,
                &element,
                camera,
                rect,
                Some(theme::ACCENT),
            );
        }
        for p in &self.frame_editor.pending {
            if let Some(s) = camera.world_to_screen(*p, rect) {
                painter.circle_stroke(s, 3.0, Stroke::new(1.0_f32, theme::ACCENT));
            }
        }
        // Interação.
        let ctrl = ui.input(|i| i.modifiers.command);
        let tool = self.frame_editor.tool;
        if tool == FrameTool::Select {
            // O arrasto começa onde o botão foi pressionado, não onde o egui detecta o movimento.
            if response.drag_started_by(PointerButton::Primary)
                && let Some(start) = ui
                    .input(|i| i.pointer.press_origin())
                    .or(response.interact_pointer_pos())
                    .and_then(world)
            {
                let hit = self.hit(frame, start);
                if let Some(i) = hit {
                    if !self.frame_editor.selection.contains(&i) {
                        if !ctrl {
                            self.frame_editor.selection.clear();
                        }
                        self.frame_editor.selection.insert(i);
                    }
                    self.frame_editor.drag = Some((start, start));
                }
            }
            if let Some((start, _)) = self.frame_editor.drag
                && let Some(now) = response.interact_pointer_pos().and_then(world)
            {
                self.frame_editor.drag = Some((start, now));
            }
            if response.drag_stopped_by(PointerButton::Primary)
                && let Some((a, b)) = self.frame_editor.drag.take()
                && (a.x() != b.x() || a.y() != b.y())
            {
                self.editor_transform_selection(false, b.x() - a.x(), b.y() - a.y(), false);
            }
            if response.clicked_by(PointerButton::Primary)
                && let Some(p) = response.interact_pointer_pos().and_then(world)
            {
                match self.hit(frame, p) {
                    Some(i) if ctrl => {
                        if !self.frame_editor.selection.remove(&i) {
                            self.frame_editor.selection.insert(i);
                        }
                    }
                    Some(i) => self.frame_editor.selection = BTreeSet::from([i]),
                    None if !ctrl => self.frame_editor.selection.clear(),
                    None => {}
                }
            }
        } else if response.clicked_by(PointerButton::Primary)
            && let Some(p) = response.interact_pointer_pos().and_then(world)
        {
            self.frame_editor.pending.push(p);
            if self.frame_editor.pending.len() >= tool.clicks() {
                let points = std::mem::take(&mut self.frame_editor.pending);
                self.place_tool_element(&points);
            }
        }
        if !ui.ctx().wants_keyboard_input() && response.hovered() {
            if ui.input(|i| i.key_pressed(Key::Delete)) {
                self.editor_delete_selection();
            }
            if ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, Key::D)) {
                self.editor_transform_selection(true, 5.0, -5.0, false);
            }
        }
    }

    fn hit(&self, frame: &FrameTemplate, p: Point2D) -> Option<usize> {
        let project = self.session.project()?;
        let tolerance = (6.0 / self.frame_editor.camera.scale()).max(0.5);
        frame
            .elements()
            .iter()
            .enumerate()
            .map(|(i, e)| (i, element_distance(project, e, p)))
            .filter(|(_, d)| *d <= tolerance)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)
    }

    /// Elemento da ferramenta atual com os pontos dados (prévia aceita menos).
    fn build_tool_element(&self, points: &[Point2D], preview: bool) -> Option<FrameElement> {
        let tool = self.frame_editor.tool;
        if points.is_empty() || (!preview && points.len() < tool.clicks()) {
            return None;
        }
        let a = points[0];
        let b = *points.get(1).unwrap_or(&a);
        Some(match tool {
            FrameTool::Select => return None,
            FrameTool::Line if points.len() >= 2 && a != b => FrameElement::Line {
                start: a,
                end: b,
                width: 0.25,
            },
            FrameTool::Rectangle if points.len() >= 2 => {
                let (x0, y0) = (a.x().min(b.x()), a.y().min(b.y()));
                let (w, h) = ((a.x() - b.x()).abs(), (a.y() - b.y()).abs());
                FrameElement::Rectangle {
                    rect: Rect2D::new(pt(x0, y0)?, Size2D::new(w, h).ok()?).ok()?,
                    width: 0.35,
                }
            }
            FrameTool::Circle if points.len() >= 2 => FrameElement::Circle {
                center: a,
                radius: a.distance_to(b).ok()?,
                width: 0.25,
            },
            FrameTool::Arc if points.len() >= 3 => {
                let c = points[2];
                let angle = |p: Point2D| (p.y() - a.y()).atan2(p.x() - a.x()).to_degrees();
                let start = angle(b);
                let mut sweep = angle(c) - start;
                if sweep <= 0.0 {
                    sweep += 360.0;
                }
                FrameElement::Arc {
                    center: a,
                    radius: a.distance_to(b).ok()?,
                    start_degrees: start,
                    sweep_degrees: sweep,
                    width: 0.25,
                }
            }
            FrameTool::Text => FrameElement::Text {
                origin: a,
                text: LocalizedText::new("Texto"),
                style: FrameTextStyle::new(3.5, TextAlign::BottomLeft),
            },
            FrameTool::Special => FrameElement::Special {
                origin: a,
                field: SpecialField::ProjectName,
                index: None,
                style: FrameTextStyle::new(3.0, TextAlign::BottomLeft),
            },
            FrameTool::Image => {
                let asset = self.frame_editor.pending_image.as_ref()?;
                let w = 40.0;
                FrameElement::Image {
                    rect: Rect2D::new(a, Size2D::new(w, w / asset.aspect()).ok()?).ok()?,
                    asset: asset.id(),
                }
            }
            _ => return None,
        })
    }

    fn place_tool_element(&mut self, points: &[Point2D]) {
        let Some(frame) = self.editor_frame() else {
            return;
        };
        let Some(mut element) = self.build_tool_element(points, false) else {
            self.log(
                Level::Warning,
                "Elemento degenerado; tente pontos diferentes.",
            );
            return;
        };
        let mut extra = vec![];
        if let FrameElement::Image { asset, .. } = &mut element
            && let Some(image) = self.frame_editor.pending_image.clone()
        {
            let bytes = image.bytes().unwrap_or_default();
            match self.session.project().and_then(|p| {
                p.assets()
                    .find(|a| a.same_content(bytes))
                    .map(ImageAsset::id)
            }) {
                // Imagem igual já está no projeto: reaproveita.
                Some(existing) => *asset = existing,
                None => extra.push(FrameEdit::AddAsset(image)),
            }
        }
        let mut draft = frame.draft();
        draft.elements.push(element);
        let index = draft.elements.len() - 1;
        if self.commit_frame_draft(draft, extra) {
            self.frame_editor.selection = BTreeSet::from([index]);
            if self.frame_editor.tool == FrameTool::Image {
                // A mesma imagem pode ser colocada de novo; o projeto já a tem.
                if let Some(id) = self.session.project().and_then(|p| {
                    let bytes = self.frame_editor.pending_image.as_ref()?.bytes()?;
                    p.assets()
                        .find(|a| a.same_content(bytes))
                        .map(ImageAsset::id)
                }) && let Some(img) = &mut self.frame_editor.pending_image
                {
                    *img = img.clone().with_id(id);
                }
            }
        }
    }

    // -------------------------------------------------------- propriedades

    fn sync_forms(&mut self, frame: &FrameTemplate) {
        let key = (
            frame.id(),
            frame.version(),
            self.frame_editor
                .selection
                .iter()
                .copied()
                .collect::<Vec<_>>(),
        );
        if self.frame_editor.form_key.as_ref() == Some(&key) {
            return;
        }
        self.frame_editor
            .selection
            .retain(|i| *i < frame.elements().len());
        let u = frame.usable();
        self.frame_editor.frame_form = FrameForm {
            name: frame.name().as_str().to_owned(),
            description: frame.description().to_owned(),
            sheet: [frame.sheet().width(), frame.sheet().height()],
            usable: [
                u.origin().x(),
                u.origin().y(),
                u.size().width(),
                u.size().height(),
            ],
            axes: [
                AxisForm::from(&frame.zones().columns),
                AxisForm::from(&frame.zones().rows),
            ],
        };
        if let [i] = key.2.as_slice()
            && let Some(e) = frame.elements().get(*i)
        {
            self.frame_editor.element_form = element_form(e);
        }
        self.frame_editor.form_key = Some(key);
    }

    fn frame_properties_ui(&mut self, ui: &mut Ui, frame: &FrameTemplate) {
        self.sync_forms(frame);
        let selection: Vec<usize> = self.frame_editor.selection.iter().copied().collect();
        if let [i] = selection.as_slice()
            && let Some(e) = frame.elements().get(*i).cloned()
        {
            ui.heading(e.kind_label());
            let mut apply = false;
            element_form_ui(ui, &e, &mut self.frame_editor.element_form, &mut apply);
            if apply {
                let image_aspect = e
                    .asset()
                    .and_then(|a| self.session.project()?.asset(a))
                    .map(ImageAsset::aspect);
                match apply_element_form(&e, &self.frame_editor.element_form, image_aspect) {
                    Ok(new) => {
                        let mut draft = frame.draft();
                        draft.elements[*i] = new;
                        self.commit_frame_draft(draft, vec![]);
                    }
                    Err(e) => self.log(Level::Error, e),
                }
            }
            return;
        }
        if selection.len() > 1 {
            ui.label(format!("{} elementos selecionados.", selection.len()));
            ui.weak("Use Duplicar, Excluir ou Alinhar à grade, ou arraste para mover.");
            ui.separator();
        }
        ui.heading("Moldura");
        let form = &mut self.frame_editor.frame_form;
        let mut apply = false;
        egui::Grid::new("frame_form").num_columns(2).show(ui, |ui| {
            ui.label("Nome");
            ui.add(egui::TextEdit::singleline(&mut form.name).desired_width(140.0));
            ui.end_row();
            ui.label("Descrição");
            ui.add(egui::TextEdit::singleline(&mut form.description).desired_width(140.0));
            ui.end_row();
            ui.label("Formato");
            let current = Size2D::new(form.sheet[0], form.sheet[1])
                .ok()
                .and_then(PaperFormat::matching)
                .map(|(f, l)| format!("{} {}", f.label(), if l { "paisagem" } else { "retrato" }))
                .unwrap_or_else(|| "Personalizado".into());
            egui::ComboBox::from_id_salt("frame_sheet_format")
                .width(120.0)
                .selected_text(current)
                .show_ui(ui, |ui| {
                    for f in PaperFormat::ALL {
                        for landscape in [true, false] {
                            let label = format!(
                                "{} {}",
                                f.label(),
                                if landscape { "paisagem" } else { "retrato" }
                            );
                            if ui.selectable_label(false, label).clicked() {
                                let s = f.size(landscape);
                                form.sheet = [s.width(), s.height()];
                            }
                        }
                    }
                });
            ui.end_row();
            for (label, value) in [("Largura da folha", 0usize), ("Altura da folha", 1)] {
                ui.label(label);
                ui.add(
                    egui::DragValue::new(&mut form.sheet[value])
                        .speed(1.0)
                        .suffix(" mm"),
                );
                ui.end_row();
            }
            for (label, value) in [
                ("Área útil X", 0usize),
                ("Área útil Y", 1),
                ("Área útil largura", 2),
                ("Área útil altura", 3),
            ] {
                ui.label(label);
                ui.add(
                    egui::DragValue::new(&mut form.usable[value])
                        .speed(0.5)
                        .suffix(" mm"),
                );
                ui.end_row();
            }
        });
        for (axis, title, start_side, end_side) in [
            (0usize, "Colunas", "Rótulos em cima", "Rótulos embaixo"),
            (1, "Linhas", "Rótulos à esquerda", "Rótulos à direita"),
        ] {
            ui.separator();
            ui.strong(title);
            let a = &mut form.axes[axis];
            egui::Grid::new(("frame_axis", axis))
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Quantidade");
                    ui.add(egui::DragValue::new(&mut a.count).range(0..=200));
                    ui.end_row();
                    ui.label("Tamanho uniforme");
                    ui.add(egui::DragValue::new(&mut a.size).speed(0.5).suffix(" mm"));
                    ui.end_row();
                    ui.label("Individuais (;)");
                    ui.add(egui::TextEdit::singleline(&mut a.custom).desired_width(140.0));
                    ui.end_row();
                    ui.label("Rótulo");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut a.letters, false, "1, 2, 3");
                        ui.selectable_value(&mut a.letters, true, "A, B, C");
                    });
                    ui.end_row();
                    ui.label("Início");
                    ui.add(egui::DragValue::new(&mut a.start).range(0..=999));
                    ui.end_row();
                });
            ui.checkbox(&mut a.labels_start, start_side);
            ui.checkbox(&mut a.labels_end, end_side);
        }
        ui.add_space(6.0);
        if ui.button("Aplicar moldura").clicked() {
            apply = true;
        }
        if apply {
            match build_frame_draft(frame, &self.frame_editor.frame_form) {
                Ok(draft) => {
                    self.commit_frame_draft(draft, vec![]);
                }
                Err(e) => self.log(Level::Error, e),
            }
        }
    }

    // ------------------------------------------------- comandos de arquivo

    pub(crate) fn frame_command(&mut self, id: crate::commands::CommandId) {
        use crate::commands::CommandId as C;
        let Some(project) = self.session.project() else {
            return;
        };
        match id {
            C::FrameEditor => {
                let target = self
                    .session
                    .active
                    .and_then(|page| project.page_frame(page))
                    .map(FrameTemplate::id)
                    .or(project.default_frame())
                    .or(project.frames().next().map(FrameTemplate::id));
                match target {
                    Some(id) => self.open_frame_editor(id),
                    None => self.log(Level::Warning, "Projeto sem molduras: use Nova moldura."),
                }
            }
            C::FrameNew => {
                let size = self
                    .session
                    .active
                    .and_then(|p| project.page(p))
                    .map(Page::size)
                    .unwrap_or_else(|| PaperFormat::A3.size(true));
                let name = format!("Moldura {}", project.frames().count() + 1);
                match FrameTemplate::blank(name, size) {
                    Ok(frame) => {
                        let id = frame.id();
                        self.graphic_edit(GraphicEdit::Frame(FrameEdit::UpsertFrame(frame)));
                        if self
                            .session
                            .project()
                            .is_some_and(|p| p.frame(id).is_some())
                        {
                            self.open_frame_editor(id);
                        }
                    }
                    Err(e) => self.log(Level::Error, e.to_string()),
                }
            }
            C::FrameDuplicate => {
                let Some(frame) = self.editor_frame() else {
                    self.log(Level::Warning, "Abra uma moldura no editor para duplicar.");
                    return;
                };
                match frame.duplicate(format!("{} (cópia)", frame.name().as_str())) {
                    Ok(copy) => {
                        let id = copy.id();
                        self.graphic_edit(GraphicEdit::Frame(FrameEdit::UpsertFrame(copy)));
                        self.open_frame_editor(id);
                    }
                    Err(e) => self.log(Level::Error, e.to_string()),
                }
            }
            C::FrameDelete => {
                let Some(id) = self.frame_editor.frame else {
                    self.log(Level::Warning, "Abra uma moldura no editor para excluir.");
                    return;
                };
                self.graphic_edit(GraphicEdit::Frame(FrameEdit::RemoveFrame(id)));
                if self
                    .session
                    .project()
                    .is_some_and(|p| p.frame(id).is_none())
                {
                    self.close_frame_editor();
                }
            }
            C::FrameExport => {
                let Some(id) = self.frame_editor.frame else {
                    self.log(Level::Warning, "Abra uma moldura no editor para exportar.");
                    return;
                };
                let result = project
                    .frame_package(id)
                    .map_err(|e| e.to_string())
                    .and_then(|pkg| {
                        cad_craft::persistence::write_frame_package(&pkg)
                            .map(|b| (b, pkg.frame.name().as_str().to_owned()))
                            .map_err(|e| e.to_string())
                    });
                match result {
                    Ok((bytes, name)) => {
                        let library = self.frame_library();
                        if let Some(path) = self.files.picker.save_frame(&name, library.as_deref())
                        {
                            let path = with_frame_extension(path);
                            match crate::project_file::write_atomic(&path, &mut bytes.as_slice()) {
                                Ok(()) => self.log(
                                    Level::Info,
                                    format!("Moldura exportada em {}.", path.display()),
                                ),
                                Err(e) => {
                                    self.log(Level::Error, format!("{}: {e}", path.display()))
                                }
                            }
                        }
                    }
                    Err(e) => self.log(Level::Error, e),
                }
            }
            C::FrameImport => {
                let library = self.frame_library();
                let Some(path) = self.files.picker.open_frame(library.as_deref()) else {
                    return;
                };
                let result = read_frame_file(&path).and_then(|pkg| {
                    // As imagens do pacote também precisam decodificar.
                    for a in &pkg.assets {
                        frame_view::decode_image(a.bytes().unwrap_or_default())?;
                    }
                    project.frame_import_edits(&pkg).map_err(|e| e.to_string())
                });
                match result {
                    Ok((edits, id)) => {
                        self.graphic_edit(GraphicEdit::Frame(FrameEdit::Batch(edits)));
                        if self
                            .session
                            .project()
                            .is_some_and(|p| p.frame(id).is_some())
                        {
                            self.log(
                                Level::Info,
                                format!("Moldura importada de {}.", path.display()),
                            );
                            self.open_frame_editor(id);
                        }
                    }
                    Err(e) => self.log(Level::Error, format!("{}: {e}", path.display())),
                }
            }
            C::FrameImportExternal => {
                let library = self.frame_library();
                let Some(path) = self.files.picker.open_external_frame(library.as_deref()) else {
                    return;
                };
                let result = read_external_frame(
                    &path,
                    self.session
                        .active
                        .and_then(|id| project.page(id))
                        .map(Page::size),
                    None,
                    None,
                );
                match result {
                    Ok(imported) => {
                        self.dialog = Some(crate::dialogs::Dialog::ExternalFrame {
                            path,
                            imported: Box::new(imported),
                            page: self.session.active,
                            dxf_unit_mm: None,
                            dxf_sheet: None,
                            preview_dirty: false,
                        })
                    }
                    Err(e) => self.log(Level::Error, format!("{}: {e}", path.display())),
                }
            }
            C::FrameExportDxf => {
                let Some(frame) = self.editor_frame() else {
                    self.log(
                        Level::Warning,
                        "Abra uma moldura no editor para exportar DXF.",
                    );
                    return;
                };
                let (bytes, report) = cad_craft::interchange::dxf::export(&frame);
                let library = self.frame_library();
                if let Some(path) = self
                    .files
                    .picker
                    .save_dxf(frame.name().as_str(), library.as_deref())
                {
                    let path = if path
                        .extension()
                        .is_some_and(|e| e.eq_ignore_ascii_case("dxf"))
                    {
                        path
                    } else {
                        path.with_extension("dxf")
                    };
                    match crate::project_file::write_atomic(&path, &mut bytes.as_slice()) {
                        Ok(()) => {
                            self.log(Level::Info, format!("DXF exportado em {}.", path.display()));
                            for line in report.lines() {
                                self.log(Level::Warning, line);
                            }
                        }
                        Err(e) => self.log(Level::Error, format!("{}: {e}", path.display())),
                    }
                }
            }
            _ => {}
        }
    }
    fn frame_library(&self) -> Option<std::path::PathBuf> {
        let dir = self.preferences.frame_library.trim();
        (!dir.is_empty()).then(|| std::path::PathBuf::from(dir))
    }
}

pub(crate) fn read_external_frame(
    path: &std::path::Path,
    page_size: Option<Size2D>,
    dxf_unit_mm: Option<f64>,
    dxf_sheet: Option<Size2D>,
) -> Result<cad_craft::interchange::ImportedFrame, String> {
    use cad_craft::interchange::{self, SourceFormat};
    let format = path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(SourceFormat::from_extension)
        .ok_or_else(|| "Extensão de moldura não suportada".to_owned())?;
    let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
    if metadata.len() > interchange::MAX_IMPORT_BYTES as u64 {
        return Err("arquivo acima de 64 MB".into());
    }
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let imported = match format {
        SourceFormat::EplanFn1 => {
            let mut resolve = |name: &str| -> Option<Vec<u8>> {
                let filename = std::path::Path::new(name.replace('\\', "/").as_str())
                    .file_name()?
                    .to_owned();
                let company = path.parent()?.file_name()?;
                let data = path.parent()?.parent()?.parent()?;
                let image = data.join("Imagens").join(company).join(filename);
                (image.metadata().ok()?.len() <= MAX_IMAGE_BYTES as u64)
                    .then(|| std::fs::read(image).ok())
                    .flatten()
            };
            interchange::fn1::import(&bytes, &mut resolve)
        }
        SourceFormat::QetTitleblock => interchange::titleblock::import(
            &bytes,
            interchange::titleblock::TitleblockOptions::new(
                page_size.unwrap_or_else(|| PaperFormat::A3.size(true)),
            ),
        ),
        SourceFormat::Dxf => {
            let mut options = interchange::dxf::DxfImportOptions::new(
                path.file_stem().unwrap_or_default().to_string_lossy(),
            );
            options.unit_mm = dxf_unit_mm;
            options.sheet = dxf_sheet;
            interchange::dxf::import(&bytes, &options)
        }
    }
    .map_err(|e| e.to_string())?;
    for asset in &imported.package.assets {
        frame_view::decode_image(asset.bytes().unwrap_or_default())?;
    }
    Ok(imported)
}

fn with_frame_extension(path: std::path::PathBuf) -> std::path::PathBuf {
    let ext = cad_craft::persistence::FRAME_EXTENSION;
    if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(ext))
    {
        path
    } else {
        let mut name = path.into_os_string();
        name.push(".");
        name.push(ext);
        name.into()
    }
}
fn read_frame_file(path: &std::path::Path) -> Result<FramePackage, String> {
    let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
    if meta.len() > 256 * 1024 * 1024 {
        return Err("arquivo de moldura acima de 256 MiB".into());
    }
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    cad_craft::persistence::read_frame_package(&bytes).map_err(|e| e.to_string())
}

fn element_form(e: &FrameElement) -> ElementForm {
    let mut f = ElementForm::default();
    match e {
        FrameElement::Line { start, end, width } => {
            f.nums = [start.x(), start.y(), end.x(), end.y(), *width, 0.0];
        }
        FrameElement::Rectangle { rect, width } => {
            f.nums = [
                rect.origin().x(),
                rect.origin().y(),
                rect.size().width(),
                rect.size().height(),
                *width,
                0.0,
            ];
        }
        FrameElement::Circle {
            center,
            radius,
            width,
        } => {
            f.nums = [center.x(), center.y(), *radius, *width, 0.0, 0.0];
        }
        FrameElement::Arc {
            center,
            radius,
            start_degrees,
            sweep_degrees,
            width,
        } => {
            f.nums = [
                center.x(),
                center.y(),
                *radius,
                *start_degrees,
                *sweep_degrees,
                *width,
            ];
        }
        FrameElement::Text {
            origin,
            text,
            style,
        } => {
            f.nums = [origin.x(), origin.y(), 0.0, 0.0, 0.0, 0.0];
            f.text = text.default.clone();
            f.translations = text
                .translations
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("; ");
            f.style = Some(*style);
        }
        FrameElement::Special {
            origin,
            field,
            index,
            style,
        } => {
            f.nums = [origin.x(), origin.y(), 0.0, 0.0, 0.0, 0.0];
            (f.field, f.key) = field_index(field);
            f.index = index.unwrap_or(0);
            f.style = Some(*style);
        }
        FrameElement::Image { rect, .. } => {
            f.nums = [
                rect.origin().x(),
                rect.origin().y(),
                rect.size().width(),
                rect.size().height(),
                0.0,
                0.0,
            ];
            f.keep_aspect = true;
        }
    }
    f
}

fn num(ui: &mut Ui, label: &str, value: &mut f64, suffix: &str) {
    ui.label(label);
    ui.add(
        egui::DragValue::new(value)
            .speed(0.5)
            .max_decimals(3)
            .suffix(suffix),
    );
    ui.end_row();
}
fn element_form_ui(ui: &mut Ui, e: &FrameElement, f: &mut ElementForm, apply: &mut bool) {
    let mm = " mm";
    egui::Grid::new("frame_element_form")
        .num_columns(2)
        .show(ui, |ui| {
            let labels: &[(&str, &str)] = match e {
                FrameElement::Line { .. } => &[
                    ("X inicial", mm),
                    ("Y inicial", mm),
                    ("X final", mm),
                    ("Y final", mm),
                    ("Espessura", mm),
                ],
                FrameElement::Rectangle { .. } => &[
                    ("X", mm),
                    ("Y", mm),
                    ("Largura", mm),
                    ("Altura", mm),
                    ("Espessura", mm),
                ],
                FrameElement::Circle { .. } => &[
                    ("X centro", mm),
                    ("Y centro", mm),
                    ("Raio", mm),
                    ("Espessura", mm),
                ],
                FrameElement::Arc { .. } => &[
                    ("X centro", mm),
                    ("Y centro", mm),
                    ("Raio", mm),
                    ("Início", "°"),
                    ("Varredura", "°"),
                    ("Espessura", mm),
                ],
                FrameElement::Text { .. } | FrameElement::Special { .. } => &[("X", mm), ("Y", mm)],
                FrameElement::Image { .. } => {
                    &[("X", mm), ("Y", mm), ("Largura", mm), ("Altura", mm)]
                }
            };
            for (i, (label, suffix)) in labels.iter().enumerate() {
                num(ui, label, &mut f.nums[i], suffix);
            }
            match e {
                FrameElement::Text { .. } => {
                    ui.label("Texto");
                    ui.add(egui::TextEdit::singleline(&mut f.text).desired_width(140.0));
                    ui.end_row();
                    ui.label("Traduções");
                    ui.add(
                        egui::TextEdit::singleline(&mut f.translations)
                            .hint_text("en-US=Page; es-ES=Hoja")
                            .desired_width(140.0),
                    );
                    ui.end_row();
                }
                FrameElement::Special { .. } => {
                    ui.label("Campo");
                    egui::ComboBox::from_id_salt("frame_special_field")
                        .width(140.0)
                        .height(420.0)
                        .selected_text(FIELDS[f.field])
                        .show_ui(ui, |ui| {
                            for (i, name) in FIELDS.iter().enumerate() {
                                ui.selectable_value(&mut f.field, i, *name);
                            }
                        });
                    ui.end_row();
                    if f.field == 3 || f.field == 14 {
                        ui.label("Chave");
                        ui.add(
                            egui::TextEdit::singleline(&mut f.key)
                                .hint_text("Autor, Data, Revisão…")
                                .desired_width(140.0),
                        );
                        ui.end_row();
                        ui.label("Índice (0 = sem)");
                        ui.add(egui::DragValue::new(&mut f.index).range(0..=999));
                        ui.end_row();
                    }
                }
                FrameElement::Image { .. } => {
                    ui.label("Manter proporção");
                    ui.checkbox(&mut f.keep_aspect, "");
                    ui.end_row();
                }
                _ => {}
            }
            if let Some(style) = &mut f.style {
                num(ui, "Altura do texto", &mut style.height, mm);
                ui.label("Alinhamento");
                egui::ComboBox::from_id_salt("frame_text_align")
                    .width(140.0)
                    .selected_text(
                        ALIGNS
                            .iter()
                            .find(|a| a.0 == style.align)
                            .map_or("", |a| a.1),
                    )
                    .show_ui(ui, |ui| {
                        for (align, name) in ALIGNS {
                            ui.selectable_value(&mut style.align, align, name);
                        }
                    });
                ui.end_row();
                num(ui, "Rotação", &mut style.rotation_degrees, "°");
                ui.label("Marca d'água");
                ui.checkbox(&mut style.watermark, "");
                ui.end_row();
            }
        });
    if ui.button("Aplicar elemento").clicked() {
        *apply = true;
    }
}

fn apply_element_form(
    e: &FrameElement,
    f: &ElementForm,
    image_aspect: Option<f64>,
) -> Result<FrameElement, String> {
    let n = f.nums;
    let p = |x: f64, y: f64| Point2D::new(x, y).map_err(|e| e.to_string());
    let r = |x: f64, y: f64, w: f64, h: f64| {
        Rect2D::new(p(x, y)?, Size2D::new(w, h).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())
    };
    let style = || f.style.ok_or_else(|| "estilo ausente".to_owned());
    Ok(match e {
        FrameElement::Line { .. } => FrameElement::Line {
            start: p(n[0], n[1])?,
            end: p(n[2], n[3])?,
            width: n[4],
        },
        FrameElement::Rectangle { .. } => FrameElement::Rectangle {
            rect: r(n[0], n[1], n[2], n[3])?,
            width: n[4],
        },
        FrameElement::Circle { .. } => FrameElement::Circle {
            center: p(n[0], n[1])?,
            radius: n[2],
            width: n[3],
        },
        FrameElement::Arc { .. } => FrameElement::Arc {
            center: p(n[0], n[1])?,
            radius: n[2],
            start_degrees: n[3],
            sweep_degrees: n[4],
            width: n[5],
        },
        FrameElement::Text { .. } => {
            let mut text = LocalizedText::new(f.text.clone());
            for pair in f.translations.split(';').filter(|s| !s.trim().is_empty()) {
                let (lang, value) = pair
                    .split_once('=')
                    .ok_or_else(|| "traduções no formato idioma=texto; …".to_owned())?;
                text.translations
                    .insert(lang.trim().to_owned(), value.trim().to_owned());
            }
            FrameElement::Text {
                origin: p(n[0], n[1])?,
                text,
                style: style()?,
            }
        }
        FrameElement::Special { .. } => FrameElement::Special {
            origin: p(n[0], n[1])?,
            field: field_from(f.field, &f.key)?,
            index: (f.index > 0).then_some(f.index),
            style: style()?,
        },
        FrameElement::Image { asset, .. } => {
            let height = match (f.keep_aspect, image_aspect) {
                (true, Some(aspect)) => n[2] / aspect,
                _ => n[3],
            };
            FrameElement::Image {
                rect: r(n[0], n[1], n[2], height)?,
                asset: *asset,
            }
        }
    })
}

fn build_frame_draft(frame: &FrameTemplate, f: &FrameForm) -> Result<FrameDraft, String> {
    let e = |e: cad_craft::error::DomainError| e.to_string();
    let mut draft = frame.draft();
    draft.name = f.name.clone();
    draft.description = f.description.clone();
    draft.sheet = Size2D::new(f.sheet[0], f.sheet[1]).map_err(e)?;
    draft.usable = Rect2D::new(
        Point2D::new(f.usable[0], f.usable[1]).map_err(e)?,
        Size2D::new(f.usable[2], f.usable[3]).map_err(e)?,
    )
    .map_err(e)?;
    draft.zones = ZoneLayout {
        columns: f.axes[0].build()?,
        rows: f.axes[1].build()?,
    };
    Ok(draft)
}
