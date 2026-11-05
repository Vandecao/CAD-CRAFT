// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 17: editor de símbolos (no molde do editor de molduras da Etapa 14).
//!
//! Edita um rascunho local (primitivas, estilos e portas) com Desfazer/Refazer
//! próprios. "Salvar na biblioteca" grava `.cadsym` (ou guarda em memória sem
//! pasta configurada); "Atualizar no projeto" troca a definição usada pelas
//! ocorrências quando as portas são compatíveis (um passo do histórico).
use crate::{
    app::{CadApp, Level},
    camera::{Camera, grid_points, snap},
    symbol_view, theme,
};
use cad_craft::domain::*;
use eframe::egui::{self, Key, PointerButton, Rect, Sense, Stroke, Ui};
use std::collections::BTreeMap;

/// Grade do editor de símbolos (mm).
const GRID_MM: f64 = 1.0;
const MAX_UNDO: usize = 200;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SymbolTool {
    #[default]
    Select,
    Line,
    Polyline,
    Polygon,
    Rectangle,
    Circle,
    Ellipse,
    Arc,
    Text,
    DynamicText,
    Port,
}
impl SymbolTool {
    pub const ALL: [Self; 11] = [
        Self::Select,
        Self::Line,
        Self::Polyline,
        Self::Polygon,
        Self::Rectangle,
        Self::Circle,
        Self::Ellipse,
        Self::Arc,
        Self::Text,
        Self::DynamicText,
        Self::Port,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Select => "Selecionar",
            Self::Line => "Linha",
            Self::Polyline => "Polilinha",
            Self::Polygon => "Polígono",
            Self::Rectangle => "Retângulo",
            Self::Circle => "Círculo",
            Self::Ellipse => "Elipse",
            Self::Arc => "Arco",
            Self::Text => "Texto",
            Self::DynamicText => "Texto dinâmico",
            Self::Port => "Porta",
        }
    }
    fn hint(self) -> &'static str {
        match self {
            Self::Select => {
                "Clique seleciona; arraste move; Delete exclui; Ctrl+Z/Ctrl+Y desfazem/refazem."
            }
            Self::Line => "Clique no início e no fim.",
            Self::Polyline | Self::Polygon => {
                "Clique os vértices; Enter ou duplo clique encerra; Esc cancela."
            }
            Self::Rectangle | Self::Ellipse => "Clique dois cantos opostos.",
            Self::Circle => "Clique o centro e um ponto do contorno.",
            Self::Arc => "Clique o centro, o início e o fim (sentido anti-horário).",
            Self::Text => "Clique para inserir o texto do campo ao lado.",
            Self::DynamicText => "Clique para inserir o texto dinâmico escolhido.",
            Self::Port => {
                "Clique para inserir uma porta de conexão (ajuste a orientação no painel)."
            }
        }
    }
    fn clicks(self) -> usize {
        match self {
            Self::Select => 0,
            Self::Text | Self::DynamicText | Self::Port => 1,
            Self::Line | Self::Rectangle | Self::Circle | Self::Ellipse => 2,
            Self::Arc => 3,
            Self::Polyline | Self::Polygon => usize::MAX,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolItem {
    Primitive(usize),
    Port(usize),
}

#[derive(Clone, Debug, PartialEq)]
struct Draft {
    name: String,
    category: String,
    primitives: Vec<SymbolPrimitive>,
    styles: BTreeMap<usize, PrimitiveStyle>,
    ports: Vec<SymbolPort>,
}

/// Campo escolhido para novos textos dinâmicos.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldChoice {
    #[default]
    Designation,
    FunctionKind,
    PortLabel,
    Reference,
    Property,
}

#[derive(Default)]
pub struct SymbolEditor {
    open: bool,
    /// Definição de origem (edição gera nova versão com o mesmo ID).
    base: Option<SymbolDefinition>,
    id: Option<SymbolDefinitionId>,
    draft: Option<Draft>,
    pub tool: SymbolTool,
    pub selection: Option<SymbolItem>,
    pub camera: Camera,
    pub needs_fit: bool,
    /// Área de desenho do último quadro (testes e automação).
    pub viewport: Option<Rect>,
    pending: Vec<Point2D>,
    drag: Option<(Point2D, Point2D)>,
    undo: Vec<Draft>,
    redo: Vec<Draft>,
    /// Texto usado pela ferramenta Texto.
    pub new_text: String,
    pub field: FieldChoice,
    pub field_key: String,
    pub message: Option<String>,
}

impl SymbolEditor {
    pub fn is_open(&self) -> bool {
        self.open
    }
    /// Cancela a ferramenta/pontos pendentes; `true` se havia algo.
    pub fn cancel(&mut self) -> bool {
        let busy =
            !self.pending.is_empty() || self.drag.is_some() || self.tool != SymbolTool::Select;
        self.pending.clear();
        self.drag = None;
        self.tool = SymbolTool::Select;
        busy
    }
    /// Definição resultante do rascunho (validada), com versão nova.
    pub fn definition(&self) -> Option<SymbolDefinition> {
        self.build().ok()
    }
    pub fn primitives(&self) -> &[SymbolPrimitive] {
        self.draft.as_ref().map_or(&[], |d| &d.primitives)
    }
    pub fn ports(&self) -> &[SymbolPort] {
        self.draft.as_ref().map_or(&[], |d| &d.ports)
    }
    pub fn style(&self, index: usize) -> PrimitiveStyle {
        self.draft
            .as_ref()
            .and_then(|d| d.styles.get(&index).copied())
            .unwrap_or_default()
    }
    pub fn set_name(&mut self, name: &str) {
        if let Some(d) = &mut self.draft {
            d.name = name.to_owned();
        }
    }
    fn build(&self) -> Result<SymbolDefinition, String> {
        let d = self.draft.as_ref().ok_or("editor fechado")?;
        let id = self.id.clone().ok_or("símbolo sem identificação")?;
        let result = match &self.base {
            Some(base) => base.edited(
                d.name.trim(),
                d.category.trim(),
                d.primitives.clone(),
                d.ports.clone(),
                d.styles.clone(),
            ),
            None => SymbolDefinition::new(
                id,
                1,
                d.name.trim(),
                d.category.trim(),
                d.primitives.clone(),
                d.ports.clone(),
            )
            .and_then(|s| s.with_styles(d.styles.clone())),
        };
        result.map_err(|e| e.to_string())
    }
    fn snapshot(&mut self) {
        if let Some(d) = &self.draft {
            self.undo.push(d.clone());
            if self.undo.len() > MAX_UNDO {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
    }
    pub fn undo(&mut self) -> bool {
        match (self.undo.pop(), self.draft.take()) {
            (Some(prev), Some(now)) => {
                self.redo.push(now);
                self.draft = Some(prev);
                self.selection = None;
                true
            }
            (None, now) => {
                self.draft = now;
                false
            }
            (Some(prev), None) => {
                self.draft = Some(prev);
                true
            }
        }
    }
    pub fn redo(&mut self) -> bool {
        match (self.redo.pop(), self.draft.take()) {
            (Some(next), Some(now)) => {
                self.undo.push(now);
                self.draft = Some(next);
                self.selection = None;
                true
            }
            (None, now) => {
                self.draft = now;
                false
            }
            (Some(next), None) => {
                self.draft = Some(next);
                true
            }
        }
    }
    /// Altera o rascunho com ponto de Desfazer; recusa resultado inválido.
    fn change(&mut self, f: impl FnOnce(&mut Draft)) -> bool {
        let Some(before) = self.draft.clone() else {
            return false;
        };
        let mut after = before.clone();
        f(&mut after);
        if after == before {
            return false;
        }
        // Primitivas/portas precisam continuar válidas isoladamente.
        if after.primitives.iter().any(|p| p.bounds().is_err())
            || after
                .styles
                .iter()
                .any(|(i, s)| after.primitives.get(*i).is_none_or(|p| s.check(p).is_err()))
        {
            self.message = Some("alteração inválida recusada".into());
            return false;
        }
        self.snapshot();
        self.draft = Some(after);
        self.message = None;
        true
    }
    pub fn add_primitive(&mut self, p: SymbolPrimitive) -> bool {
        self.change(|d| d.primitives.push(p))
    }
    pub fn add_port(&mut self, at: Point2D) -> bool {
        let n = (1..)
            .map(|n: usize| n.to_string())
            .find(|n| !self.ports().iter().any(|p| p.id() == n))
            .expect("infinito");
        match SymbolPort::new(n.clone(), at, 0.0, n, None) {
            Ok(port) => self.change(|d| d.ports.push(port)),
            Err(_) => false,
        }
    }
    pub fn delete_selection(&mut self) -> bool {
        let Some(item) = self.selection.take() else {
            return false;
        };
        self.change(|d| match item {
            SymbolItem::Primitive(i) if i < d.primitives.len() => {
                d.primitives.remove(i);
                d.styles = d
                    .styles
                    .iter()
                    .filter(|(k, _)| **k != i)
                    .map(|(k, s)| (if *k > i { k - 1 } else { *k }, *s))
                    .collect();
            }
            SymbolItem::Port(i) if i < d.ports.len() => {
                d.ports.remove(i);
            }
            _ => {}
        })
    }
    fn move_selection(&mut self, dx: f64, dy: f64) -> bool {
        let Some(item) = self.selection else {
            return false;
        };
        self.change(|d| match item {
            SymbolItem::Primitive(i) => {
                if let Some(p) = d.primitives.get(i).and_then(|p| p.translated(dx, dy).ok()) {
                    d.primitives[i] = p;
                }
            }
            SymbolItem::Port(i) => {
                if let Some(port) = d.ports.get(i)
                    && let Ok(moved) = port.position().translated(dx, dy).and_then(|p| {
                        SymbolPort::new(
                            port.id(),
                            p,
                            port.orientation_degrees(),
                            port.identification(),
                            port.role().map(str::to_owned),
                        )
                    })
                {
                    d.ports[i] = moved;
                }
            }
        })
    }
    /// Item sob o ponto (tolerância em mm).
    pub fn hit(&self, at: Point2D, tolerance: f64) -> Option<SymbolItem> {
        let port = self.ports().iter().enumerate().find(|(_, p)| {
            (p.position().x() - at.x()).hypot(p.position().y() - at.y()) <= tolerance * 1.5
        });
        if let Some((i, _)) = port {
            return Some(SymbolItem::Port(i));
        }
        self.primitives()
            .iter()
            .enumerate()
            .map(|(i, p)| (i, symbol_view::primitive_distance(p, at)))
            .filter(|(_, d)| *d <= tolerance)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| SymbolItem::Primitive(i))
    }
    fn field(&self) -> SymbolTextField {
        match self.field {
            FieldChoice::Designation => SymbolTextField::Designation,
            FieldChoice::FunctionKind => SymbolTextField::FunctionKind,
            FieldChoice::Reference => SymbolTextField::Reference,
            FieldChoice::PortLabel => SymbolTextField::PortLabel(
                self.ports()
                    .first()
                    .map_or_else(|| "1".to_owned(), |p| p.id().to_owned()),
            ),
            FieldChoice::Property => {
                SymbolTextField::DeviceProperty(if self.field_key.trim().is_empty() {
                    "Propriedade".into()
                } else {
                    self.field_key.trim().to_owned()
                })
            }
        }
    }
    /// Primitiva da ferramenta a partir dos cliques (prévia ou definitiva).
    pub fn build_tool_primitive(
        &self,
        points: &[Point2D],
        closing: bool,
    ) -> Option<SymbolPrimitive> {
        let pt = |x: f64, y: f64| Point2D::new(x, y).ok();
        let p = match (self.tool, points) {
            (SymbolTool::Line, [a, b, ..]) => SymbolPrimitive::Line { start: *a, end: *b },
            (SymbolTool::Rectangle, [a, b, ..]) => {
                let (x0, y0) = (a.x().min(b.x()), a.y().min(b.y()));
                SymbolPrimitive::Rectangle {
                    rect: Rect2D::new(
                        pt(x0, y0)?,
                        Size2D::new((a.x() - b.x()).abs(), (a.y() - b.y()).abs()).ok()?,
                    )
                    .ok()?,
                }
            }
            (SymbolTool::Circle, [c, r, ..]) => SymbolPrimitive::Circle {
                center: *c,
                radius: (c.x() - r.x()).hypot(c.y() - r.y()),
            },
            (SymbolTool::Ellipse, [a, b, ..]) => SymbolPrimitive::Ellipse {
                center: pt((a.x() + b.x()) / 2.0, (a.y() + b.y()) / 2.0)?,
                radius_x: (a.x() - b.x()).abs() / 2.0,
                radius_y: (a.y() - b.y()).abs() / 2.0,
                rotation_degrees: 0.0,
            },
            (SymbolTool::Arc, [c, s, e, ..]) => {
                let a0 = (s.y() - c.y()).atan2(s.x() - c.x()).to_degrees();
                let a1 = (e.y() - c.y()).atan2(e.x() - c.x()).to_degrees();
                let mut sweep = (a1 - a0).rem_euclid(360.0);
                if sweep < 1e-9 {
                    sweep = 360.0;
                }
                SymbolPrimitive::Arc {
                    center: *c,
                    radius: (c.x() - s.x()).hypot(c.y() - s.y()),
                    start_degrees: a0,
                    sweep_degrees: sweep,
                }
            }
            (SymbolTool::Polyline, pts) if pts.len() >= 2 => SymbolPrimitive::Polyline {
                points: pts.to_vec(),
                closed: false,
            },
            (SymbolTool::Polygon, pts) if pts.len() >= 3 || (!closing && pts.len() >= 2) => {
                SymbolPrimitive::Polyline {
                    points: pts.to_vec(),
                    closed: pts.len() >= 3,
                }
            }
            (SymbolTool::Text, [a, ..]) => SymbolPrimitive::Text {
                origin: *a,
                text: if self.new_text.trim().is_empty() {
                    "Texto".into()
                } else {
                    self.new_text.clone()
                },
                height: 2.5,
            },
            (SymbolTool::DynamicText, [a, ..]) => SymbolPrimitive::DynamicText {
                origin: *a,
                field: self.field(),
                height: 2.5,
                align: TextAlign::MiddleLeft,
                rotation_degrees: 0.0,
            },
            _ => return None,
        };
        p.bounds().is_ok().then_some(p)
    }
    /// Recebe um clique da ferramenta; conclui quando há pontos suficientes.
    pub fn tool_click(&mut self, at: Point2D) {
        if self.tool == SymbolTool::Port {
            self.add_port(at);
            return;
        }
        if self.pending.last() == Some(&at) {
            return;
        }
        self.pending.push(at);
        if self.pending.len() >= self.tool.clicks()
            && let Some(p) = self.build_tool_primitive(&self.pending.clone(), true)
        {
            self.pending.clear();
            self.add_primitive(p);
        }
    }
    /// Encerra polilinha/polígono.
    pub fn finish_path(&mut self) {
        if matches!(self.tool, SymbolTool::Polyline | SymbolTool::Polygon)
            && let Some(p) = self.build_tool_primitive(&self.pending.clone(), true)
        {
            self.add_primitive(p);
        }
        self.pending.clear();
    }
}

impl CadApp {
    fn start_symbol_editor(
        &mut self,
        base: Option<SymbolDefinition>,
        id: SymbolDefinitionId,
        draft: Draft,
    ) {
        if self.frame_editor.is_open() {
            self.close_frame_editor();
        }
        self.symbol_editor = SymbolEditor {
            open: true,
            base,
            id: Some(id),
            draft: Some(draft),
            needs_fit: true,
            new_text: "Texto".into(),
            ..Default::default()
        };
        self.session.graphics.cancel();
    }
    /// Novo símbolo (ID único `usuario.<uuid>`).
    pub fn open_symbol_editor_new(&mut self) {
        let id = SymbolDefinitionId::new(format!("usuario.{}", RepresentationId::new()))
            .expect("ID gerado");
        self.start_symbol_editor(
            None,
            id,
            Draft {
                name: "Novo símbolo".into(),
                category: "Usuário".into(),
                primitives: vec![],
                styles: BTreeMap::new(),
                ports: vec![],
            },
        );
    }
    /// Edita um símbolo existente (salvar gera versão + 1).
    pub fn open_symbol_editor(&mut self, definition: SymbolDefinition) {
        let draft = Draft {
            name: definition.name().to_owned(),
            category: definition.category().to_owned(),
            primitives: definition.primitives().to_vec(),
            styles: definition.styles().clone(),
            ports: definition.ports().to_vec(),
        };
        let id = definition.id().clone();
        self.start_symbol_editor(Some(definition), id, draft);
    }
    pub fn close_symbol_editor(&mut self) {
        self.symbol_editor = SymbolEditor::default();
    }
    /// Grava o símbolo na biblioteca (pasta ou memória).
    pub fn save_symbol_to_library(&mut self) -> bool {
        match self.symbol_editor.build() {
            Ok(d) => {
                let name = d.name().to_owned();
                let version = d.version();
                match self.add_to_symbol_library(d) {
                    Ok(path) => {
                        self.reload_symbol_library();
                        let place = path.map_or_else(
                            || "biblioteca da sessão".to_owned(),
                            |p| p.display().to_string(),
                        );
                        self.log(
                            Level::Info,
                            format!("Símbolo \u{201c}{name}\u{201d} v{version} salvo em {place}."),
                        );
                        self.symbol_editor.message = Some(format!("Salvo (v{version})."));
                        true
                    }
                    Err(e) => {
                        self.log(Level::Error, format!("Gravação recusada: {e}"));
                        false
                    }
                }
            }
            Err(e) => {
                self.symbol_editor.message = Some(format!("Símbolo inválido: {e}"));
                false
            }
        }
    }
    /// Troca a definição usada pelo projeto (portas compatíveis; um passo).
    pub fn update_symbol_in_project(&mut self) -> bool {
        let d = match self.symbol_editor.build() {
            Ok(d) => d,
            Err(e) => {
                self.symbol_editor.message = Some(format!("Símbolo inválido: {e}"));
                return false;
            }
        };
        let problems = self
            .session
            .project()
            .map(|p| p.symbol_update_problems(&d))
            .unwrap_or_default();
        if !problems.is_empty() {
            self.symbol_editor.message = Some(problems.join("; "));
            return false;
        }
        match self.session.update_symbols(vec![d.clone()]) {
            Ok(_) => {
                self.symbol_editor.base = Some(d.clone());
                self.symbol_editor.message =
                    Some(format!("Projeto atualizado (v{}).", d.version()));
                true
            }
            Err(e) => {
                self.symbol_editor.message = Some(e.to_string());
                false
            }
        }
    }

    pub(crate) fn show_symbol_editor(&mut self, ui: &mut Ui) {
        let title = self
            .symbol_editor
            .draft
            .as_ref()
            .map(|d| d.name.clone())
            .unwrap_or_default();
        let version = self
            .symbol_editor
            .base
            .as_ref()
            .map_or(1, |b| b.version() + 1);
        ui.horizontal_wrapped(|ui| {
            ui.strong(format!("Símbolo: {title} · v{version}"));
            ui.separator();
            for tool in SymbolTool::ALL {
                if ui
                    .selectable_label(self.symbol_editor.tool == tool, tool.label())
                    .clicked()
                {
                    self.symbol_editor.pending.clear();
                    self.symbol_editor.tool = tool;
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(
                    self.symbol_editor.selection.is_some(),
                    egui::Button::new("Excluir seleção"),
                )
                .clicked()
            {
                self.symbol_editor.delete_selection();
            }
            if ui
                .add_enabled(
                    !self.symbol_editor.undo.is_empty(),
                    egui::Button::new("Desfazer"),
                )
                .clicked()
            {
                self.symbol_editor.undo();
            }
            if ui
                .add_enabled(
                    !self.symbol_editor.redo.is_empty(),
                    egui::Button::new("Refazer"),
                )
                .clicked()
            {
                self.symbol_editor.redo();
            }
            if ui.button("Ajustar à tela").clicked() {
                self.symbol_editor.needs_fit = true;
            }
            if ui.button("Salvar na biblioteca").clicked() {
                self.save_symbol_to_library();
            }
            let in_project = self.symbol_editor.id.as_ref().is_some_and(|id| {
                self.session
                    .project()
                    .is_some_and(|p| p.symbol_definition(id).is_some())
            });
            if ui
                .add_enabled(in_project, egui::Button::new("Atualizar no projeto"))
                .clicked()
            {
                self.update_symbol_in_project();
            }
            if ui.button("Fechar editor").clicked() {
                self.close_symbol_editor();
            }
        });
        if !self.symbol_editor.is_open() {
            return;
        }
        ui.small(self.symbol_editor.tool.hint());
        if let Some(m) = &self.symbol_editor.message {
            ui.colored_label(theme::ACCENT, m);
        }
        egui::SidePanel::right("symbol_editor_properties")
            .resizable(false)
            .exact_width(270.0)
            .frame(egui::Frame::new().fill(theme::PANEL).inner_margin(6))
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| self.symbol_properties_ui(ui));
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::TITLE))
            .show_inside(ui, |ui| self.symbol_canvas(ui));
    }

    fn symbol_canvas(&mut self, ui: &mut Ui) {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        let ed = &mut self.symbol_editor;
        ed.viewport = Some(rect);
        if ed.needs_fit && rect.width() > 40.0 && rect.height() > 40.0 {
            let bounds = ed
                .build()
                .ok()
                .map(|d| d.bounds())
                .filter(|b| b.size().width() > 1.0 || b.size().height() > 1.0)
                .and_then(|b| {
                    Rect2D::new(
                        b.origin().translated(-10.0, -10.0).ok()?,
                        Size2D::new(b.size().width() + 20.0, b.size().height() + 20.0).ok()?,
                    )
                    .ok()
                })
                .unwrap_or_else(|| {
                    Rect2D::new(
                        Point2D::new(-20.0, -20.0).expect("fixo"),
                        Size2D::new(40.0, 40.0).expect("fixo"),
                    )
                    .expect("fixo")
                });
            ed.camera.fit(bounds, rect.shrink(24.0));
            ed.needs_fit = false;
        }
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0
                && let Some(cursor) = response.hover_pos()
            {
                ed.camera
                    .zoom_at((scroll as f64 / 200.0).exp(), cursor, rect);
            }
        }
        if response.dragged_by(PointerButton::Middle) {
            ed.camera.pan(response.drag_delta());
        }
        let camera = ed.camera;
        let world = |p: egui::Pos2| {
            camera
                .screen_to_world(p, rect)
                .map(|w| snap(w, GRID_MM, true))
        };
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0, theme::PAPER);
        if camera.scale() * GRID_MM >= 4.0 {
            for p in grid_points(&camera, rect, GRID_MM) {
                painter.circle_filled(p, 0.8, theme::GRID_DOT);
            }
        }
        // Origem (ponto de inserção).
        if let Some(o) = Point2D::new(0.0, 0.0)
            .ok()
            .and_then(|o| camera.world_to_screen(o, rect))
        {
            let s = Stroke::new(1.0_f32, theme::GRID);
            painter.line_segment([o - egui::vec2(8.0, 0.0), o + egui::vec2(8.0, 0.0)], s);
            painter.line_segment([o - egui::vec2(0.0, 8.0), o + egui::vec2(0.0, 8.0)], s);
        }
        let identity = Transform2D::new(Point2D::new(0.0, 0.0).expect("origem"), 0.0, false, false)
            .expect("identidade");
        let drag = ed.drag;
        let shift = |item: SymbolItem| match (ed.selection, drag) {
            (Some(sel), Some((a, b))) if sel == item => (b.x() - a.x(), b.y() - a.y()),
            _ => (0.0, 0.0),
        };
        for (i, p) in ed.primitives().iter().enumerate() {
            let (dx, dy) = shift(SymbolItem::Primitive(i));
            let shown = p.translated(dx, dy).unwrap_or_else(|_| p.clone());
            let selected = ed.selection == Some(SymbolItem::Primitive(i));
            symbol_view::paint_styled(
                &painter,
                &shown,
                ed.style(i),
                identity,
                camera,
                rect,
                Stroke::new(
                    if selected { 2.0_f32 } else { 1.2_f32 },
                    if selected { theme::ACCENT } else { theme::INK },
                ),
                &symbol_view::placeholder,
            );
        }
        for (i, port) in ed.ports().iter().enumerate() {
            let (dx, dy) = shift(SymbolItem::Port(i));
            let selected = ed.selection == Some(SymbolItem::Port(i));
            if let Some(s) = port
                .position()
                .translated(dx, dy)
                .ok()
                .and_then(|p| camera.world_to_screen(p, rect))
            {
                let color = if selected {
                    theme::ACCENT
                } else {
                    egui::Color32::from_rgb(0, 140, 70)
                };
                painter.circle_stroke(s, 4.0, Stroke::new(1.5_f32, color));
                let a = port.orientation_degrees().to_radians();
                let tip = s + egui::vec2(a.cos() as f32, -(a.sin() as f32)) * 10.0;
                painter.line_segment([s, tip], Stroke::new(1.5_f32, color));
                painter.text(
                    s + egui::vec2(6.0, -6.0),
                    egui::Align2::LEFT_BOTTOM,
                    port.identification(),
                    egui::FontId::proportional(11.0),
                    color,
                );
            }
        }
        // Prévia da ferramenta.
        let cursor = response.hover_pos().and_then(world);
        let mut preview = ed.pending.clone();
        if let Some(c) = cursor
            && ed.tool != SymbolTool::Select
        {
            preview.push(c);
        }
        if let Some(p) = ed.build_tool_primitive(&preview, false) {
            symbol_view::paint_primitive(
                &painter,
                &p,
                identity,
                camera,
                rect,
                Stroke::new(1.0_f32, theme::ACCENT),
            );
        }
        for p in &ed.pending {
            if let Some(s) = camera.world_to_screen(*p, rect) {
                painter.circle_stroke(s, 3.0, Stroke::new(1.0_f32, theme::ACCENT));
            }
        }
        // Interação.
        let tolerance = 6.0 / camera.scale();
        if ed.tool == SymbolTool::Select {
            if response.drag_started_by(PointerButton::Primary)
                && let Some(start) = ui
                    .input(|i| i.pointer.press_origin())
                    .or(response.interact_pointer_pos())
                    .and_then(world)
                && let Some(item) = ed.hit(start, tolerance)
            {
                ed.selection = Some(item);
                ed.drag = Some((start, start));
            }
            if let Some((start, _)) = ed.drag
                && let Some(now) = response.interact_pointer_pos().and_then(world)
            {
                ed.drag = Some((start, now));
            }
            if response.drag_stopped_by(PointerButton::Primary)
                && let Some((a, b)) = ed.drag.take()
                && (a != b)
            {
                ed.move_selection(b.x() - a.x(), b.y() - a.y());
            }
            if response.clicked_by(PointerButton::Primary)
                && let Some(at) = response
                    .interact_pointer_pos()
                    .and_then(|p| camera.screen_to_world(p, rect))
            {
                ed.selection = ed.hit(at, tolerance);
            }
        } else if response.double_clicked_by(PointerButton::Primary)
            && matches!(ed.tool, SymbolTool::Polyline | SymbolTool::Polygon)
        {
            // Duplo clique encerra só polilinha/polígono; nas outras
            // ferramentas dois cliques rápidos são dois pontos.
            ed.finish_path();
        } else if response.clicked_by(PointerButton::Primary)
            && let Some(at) = response.interact_pointer_pos().and_then(world)
        {
            ed.tool_click(at);
        }
        if response.secondary_clicked() {
            ed.cancel();
        }
        // Teclado (o editor de símbolos tem Desfazer próprio).
        if !ui.ctx().wants_keyboard_input() {
            let (del, enter, undo, redo, esc) = ui.input_mut(|i| {
                (
                    i.consume_key(egui::Modifiers::NONE, Key::Delete),
                    i.consume_key(egui::Modifiers::NONE, Key::Enter),
                    i.consume_key(egui::Modifiers::COMMAND, Key::Z),
                    i.consume_key(egui::Modifiers::COMMAND, Key::Y),
                    i.consume_key(egui::Modifiers::NONE, Key::Escape),
                )
            });
            if del {
                ed.delete_selection();
            }
            if enter {
                ed.finish_path();
            }
            if undo {
                ed.undo();
            }
            if redo {
                ed.redo();
            }
            if esc {
                ed.cancel();
            }
        }
    }

    fn symbol_properties_ui(&mut self, ui: &mut Ui) {
        let ed = &mut self.symbol_editor;
        ui.heading("Símbolo");
        let Some(draft) = ed.draft.clone() else {
            return;
        };
        let mut name = draft.name.clone();
        let mut category = draft.category.clone();
        egui::Grid::new("symbol_props")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Nome");
                ui.add(egui::TextEdit::singleline(&mut name).desired_width(150.0));
                ui.end_row();
                ui.label("Categoria");
                ui.add(egui::TextEdit::singleline(&mut category).desired_width(150.0));
                ui.end_row();
                ui.label("ID");
                ui.small(
                    ed.id
                        .as_ref()
                        .map(|i| i.as_str().to_owned())
                        .unwrap_or_default(),
                );
                ui.end_row();
                ui.label("Portas");
                ui.label(draft.ports.len().to_string());
                ui.end_row();
            });
        if name != draft.name || category != draft.category {
            ed.change(|d| {
                d.name = name;
                d.category = category;
            });
        }
        ui.separator();
        ui.label("Novos textos");
        ui.horizontal(|ui| {
            ui.label("Texto");
            ui.add(egui::TextEdit::singleline(&mut ed.new_text).desired_width(140.0));
        });
        egui::ComboBox::from_id_salt("symbol_field_choice")
            .selected_text(match ed.field {
                FieldChoice::Designation => "Designação",
                FieldChoice::FunctionKind => "Função",
                FieldChoice::PortLabel => "Ponto da porta",
                FieldChoice::Reference => "Referência",
                FieldChoice::Property => "Propriedade",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut ed.field, FieldChoice::Designation, "Designação");
                ui.selectable_value(&mut ed.field, FieldChoice::FunctionKind, "Função");
                ui.selectable_value(&mut ed.field, FieldChoice::PortLabel, "Ponto da porta");
                ui.selectable_value(&mut ed.field, FieldChoice::Reference, "Referência");
                ui.selectable_value(&mut ed.field, FieldChoice::Property, "Propriedade");
            });
        if ed.field == FieldChoice::Property {
            ui.horizontal(|ui| {
                ui.label("Chave");
                ui.add(egui::TextEdit::singleline(&mut ed.field_key).desired_width(120.0));
            });
        }
        ui.separator();
        match ed.selection {
            Some(SymbolItem::Primitive(i)) if i < draft.primitives.len() => {
                let p = draft.primitives[i].clone();
                ui.strong(p.kind_label());
                let mut style = ed.style(i);
                let before = style;
                egui::ComboBox::from_id_salt("symbol_dash")
                    .selected_text(style.dash.label())
                    .show_ui(ui, |ui| {
                        for dash in LineDash::ALL {
                            ui.selectable_value(&mut style.dash, dash, dash.label());
                        }
                    });
                let mut own = style.width_mm.is_some();
                let mut width = style.width_mm.unwrap_or(0.25);
                ui.horizontal(|ui| {
                    ui.checkbox(&mut own, "Espessura própria");
                    if own {
                        ui.add(
                            egui::DragValue::new(&mut width)
                                .range(0.05..=5.0)
                                .speed(0.05)
                                .suffix(" mm"),
                        );
                    }
                });
                style.width_mm = own.then_some(width);
                if p.is_closed() {
                    ui.checkbox(&mut style.fill, "Preenchimento");
                }
                if style != before {
                    ed.change(|d| {
                        if style.is_default() {
                            d.styles.remove(&i);
                        } else {
                            d.styles.insert(i, style);
                        }
                    });
                }
                let mut edited = p.clone();
                match &mut edited {
                    SymbolPrimitive::Text { text, height, .. } => {
                        ui.add(egui::TextEdit::singleline(text).desired_width(180.0));
                        ui.add(
                            egui::DragValue::new(height)
                                .range(0.5..=50.0)
                                .speed(0.1)
                                .prefix("Altura "),
                        );
                    }
                    SymbolPrimitive::DynamicText {
                        field,
                        height,
                        rotation_degrees,
                        ..
                    } => {
                        ui.label(field.label());
                        if let SymbolTextField::PortLabel(port)
                        | SymbolTextField::DeviceProperty(port) = field
                        {
                            ui.add(egui::TextEdit::singleline(port).desired_width(120.0));
                        }
                        ui.add(
                            egui::DragValue::new(height)
                                .range(0.5..=50.0)
                                .speed(0.1)
                                .prefix("Altura "),
                        );
                        ui.add(
                            egui::DragValue::new(rotation_degrees)
                                .speed(15.0)
                                .prefix("Rotação "),
                        );
                    }
                    SymbolPrimitive::Ellipse {
                        rotation_degrees, ..
                    } => {
                        ui.add(
                            egui::DragValue::new(rotation_degrees)
                                .speed(5.0)
                                .prefix("Rotação "),
                        );
                    }
                    _ => {}
                }
                if edited != p && edited.bounds().is_ok() {
                    ed.change(|d| d.primitives[i] = edited);
                }
            }
            Some(SymbolItem::Port(i)) if i < draft.ports.len() => {
                let port = draft.ports[i].clone();
                ui.strong("Porta");
                let mut id = port.id().to_owned();
                let mut ident = port.identification().to_owned();
                let mut orientation = port.orientation_degrees();
                ui.horizontal(|ui| {
                    ui.label("ID");
                    ui.add(egui::TextEdit::singleline(&mut id).desired_width(80.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Identificação");
                    ui.add(egui::TextEdit::singleline(&mut ident).desired_width(80.0));
                });
                egui::ComboBox::from_id_salt("symbol_port_orientation")
                    .selected_text(format!("{orientation}°"))
                    .show_ui(ui, |ui| {
                        for a in [0.0, 90.0, 180.0, 270.0] {
                            ui.selectable_value(&mut orientation, a, format!("{a}°"));
                        }
                    });
                let unique = !draft
                    .ports
                    .iter()
                    .enumerate()
                    .any(|(j, p)| j != i && p.id() == id);
                if (id != port.id()
                    || ident != port.identification()
                    || orientation != port.orientation_degrees())
                    && unique
                    && let Ok(new) = SymbolPort::new(
                        id,
                        port.position(),
                        orientation,
                        ident,
                        port.role().map(str::to_owned),
                    )
                {
                    ed.change(|d| d.ports[i] = new);
                }
                if !unique {
                    ui.colored_label(theme::ACCENT, "ID de porta repetido");
                }
            }
            _ => {
                ui.weak("Selecione um elemento ou uma porta.");
            }
        }
    }
}
