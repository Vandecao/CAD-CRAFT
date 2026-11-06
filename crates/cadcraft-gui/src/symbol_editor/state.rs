// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! state support for the symbol editor; start at the parent module.

use super::*;

/// Grade do editor de símbolos (mm).
pub(in crate::symbol_editor) const GRID_MM: f64 = 1.0;
pub(in crate::symbol_editor) const MAX_UNDO: usize = 200;

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
    pub(in crate::symbol_editor) fn hint(self) -> &'static str {
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
    pub(in crate::symbol_editor) fn clicks(self) -> usize {
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
pub(in crate::symbol_editor) struct Draft {
    pub(in crate::symbol_editor) name: String,
    pub(in crate::symbol_editor) category: String,
    pub(in crate::symbol_editor) primitives: Vec<SymbolPrimitive>,
    pub(in crate::symbol_editor) styles: BTreeMap<usize, PrimitiveStyle>,
    pub(in crate::symbol_editor) ports: Vec<SymbolPort>,
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
    pub(in crate::symbol_editor) open: bool,
    /// Definição de origem (edição gera nova versão com o mesmo ID).
    pub(in crate::symbol_editor) base: Option<SymbolDefinition>,
    pub(in crate::symbol_editor) id: Option<SymbolDefinitionId>,
    pub(in crate::symbol_editor) draft: Option<Draft>,
    pub tool: SymbolTool,
    pub selection: Option<SymbolItem>,
    pub camera: Camera,
    pub needs_fit: bool,
    /// Área de desenho do último quadro (testes e automação).
    pub viewport: Option<Rect>,
    pub(in crate::symbol_editor) pending: Vec<Point2D>,
    pub(in crate::symbol_editor) drag: Option<(Point2D, Point2D)>,
    pub(in crate::symbol_editor) undo: Vec<Draft>,
    pub(in crate::symbol_editor) redo: Vec<Draft>,
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
    pub(in crate::symbol_editor) fn build(&self) -> Result<SymbolDefinition, String> {
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
    pub(in crate::symbol_editor) fn snapshot(&mut self) {
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
    pub(in crate::symbol_editor) fn change(&mut self, f: impl FnOnce(&mut Draft)) -> bool {
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
}
