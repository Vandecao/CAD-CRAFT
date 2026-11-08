// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! input support for the symbol editor; start at the parent module.

use super::*;

impl SymbolEditor {
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
    pub(in crate::symbol_editor) fn move_selection(&mut self, dx: f64, dy: f64) -> bool {
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
    /// Item under the point, with tolerance in millimetres.
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
    pub(in crate::symbol_editor) fn field(&self) -> SymbolTextField {
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
    /// Tool primitive from clicks, for preview or final insertion.
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
    /// Handle a tool click; finish when enough points are present.
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
    /// Finish a polyline or polygon.
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
    pub(in crate::symbol_editor) fn start_symbol_editor(
        &mut self,
        base: Option<SymbolDefinition>,
        id: SymbolDefinitionId,
        draft: Draft,
    ) {
        if self.frame_editor.is_open() {
            self.close_frame_editor();
        }
        // Zoom and position carry over from the previous symbol once fitted.
        let (camera, fitted) = (self.symbol_editor.camera, self.symbol_editor.fitted);
        self.symbol_editor = SymbolEditor {
            open: true,
            camera,
            fitted,
            base,
            id: Some(id),
            draft: Some(draft),
            needs_fit: !fitted,
            new_text: "Texto".into(),
            ..Default::default()
        };
        self.session.graphics.cancel();
    }
    /// New symbol with a unique `usuario.<uuid>` ID.
    pub fn open_symbol_editor_new(&mut self) {
        let id = SymbolDefinitionId::new(format!("usuario.{}", RepresentationId::new()))
            .expect("ID gerado");
        self.start_symbol_editor(
            None,
            id,
            Draft {
                name: crate::strings::NEW_SYMBOL.into(),
                category: "Usuário".into(),
                primitives: vec![],
                styles: BTreeMap::new(),
                ports: vec![],
                kind: SymbolFunctionKind::Other,
                prefix: String::new(),
            },
        );
    }
    /// Edit an existing symbol; saving increments its version.
    pub fn open_symbol_editor(&mut self, definition: SymbolDefinition) {
        let draft = Draft {
            name: definition.name().to_owned(),
            category: definition.category().to_owned(),
            primitives: definition.primitives().to_vec(),
            styles: definition.styles().clone(),
            ports: definition.ports().to_vec(),
            kind: definition.symbol_function_kind(),
            prefix: definition
                .designation_prefix()
                .unwrap_or_default()
                .to_owned(),
        };
        let id = definition.id().clone();
        self.start_symbol_editor(Some(definition), id, draft);
    }
    pub fn close_symbol_editor(&mut self) {
        let (camera, fitted) = (self.symbol_editor.camera, self.symbol_editor.fitted);
        self.symbol_editor = SymbolEditor {
            camera,
            fitted,
            ..SymbolEditor::default()
        };
    }
    /// Save the symbol to the directory or in-memory library.
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
    /// Replace the definition used by the project, checking compatible ports, in one undo step.
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
}
