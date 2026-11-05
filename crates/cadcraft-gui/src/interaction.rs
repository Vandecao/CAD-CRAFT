// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Transient editor state. Only confirmed batches cross into the document.
use cad_craft::{
    domain::*,
    error::{DomainError, Result},
};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Default, Debug)]
pub struct SelectionState {
    ids: BTreeSet<RepresentationId>,
    primary: Option<RepresentationId>,
}
impl SelectionState {
    pub fn ids(&self) -> &BTreeSet<RepresentationId> {
        &self.ids
    }
    pub fn primary(&self) -> Option<RepresentationId> {
        self.primary
    }
    pub fn replace(&mut self, ids: impl IntoIterator<Item = RepresentationId>) {
        self.ids = ids.into_iter().collect();
        self.primary = self.ids.first().copied();
    }
    fn click(&mut self, id: Option<RepresentationId>, additive: bool) {
        if !additive {
            self.ids.clear();
        }
        if let Some(id) = id {
            if additive && !self.ids.insert(id) {
                self.ids.remove(&id);
            } else {
                self.ids.insert(id);
            }
        }
        self.primary = id
            .filter(|id| self.ids.contains(id))
            .or_else(|| self.ids.first().copied());
    }
}
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum Tool {
    #[default]
    Select,
    Move,
    InsertSymbol,
    InsertInterruption,
    InsertPotential,
    DrawConnection,
    InsertJunction,
    EditRoute,
    AddVertex,
    RemoveVertex,
    InsertGraphic(GraphicKind),
    InsertText,
    Paste,
    /// Etapa 19: inserir a macro de `Interaction::macro_draft`.
    InsertMacro,
}
/// Etapa 19: macro em posicionamento (giro e espelho antes do clique).
#[derive(Clone, Debug)]
pub struct MacroDraft {
    pub definition: MacroDefinition,
    pub rotation: f64,
    pub mirror: bool,
}
impl MacroDraft {
    pub fn new(definition: MacroDefinition) -> Self {
        Self {
            definition,
            rotation: 0.0,
            mirror: false,
        }
    }
    pub fn transform(&self, at: Point2D) -> Result<Transform2D> {
        Transform2D::new(at, self.rotation, self.mirror, false)
    }
}
/// Etapa 12: cadastro elétrico criado ao confirmar cada inserção.
#[derive(Clone, Debug, PartialEq)]
pub enum DevicePlan {
    /// Novo dispositivo com designação `{prefixo}{n}` livre no momento do clique.
    New { prefix: String },
    /// Nova função (com pontos de conexão) em dispositivo existente.
    Existing(DeviceId),
    /// Função já cadastrada, ainda sem representação na folha.
    ExistingFunction(FunctionId),
}
/// Placement draft has no occurrence ID and never owns electrical entities.
#[derive(Clone, Debug)]
pub struct InsertionDraft {
    pub definition: SymbolDefinitionId,
    pub page: PageId,
    pub target: Option<RepresentationTarget>,
    pub transform: Transform2D,
    pub positioned: bool,
    /// Quando presente, a confirmação cria o cadastro em vez de usar `target`.
    pub device: Option<DevicePlan>,
}
#[derive(Debug)]
pub struct DragSession {
    start: Point2D,
    anchor: Point2D,
    originals: Vec<(RepresentationId, Transform2D)>,
    preview: BTreeMap<RepresentationId, Transform2D>,
}
#[derive(Debug)]
pub enum Gesture {
    Move(DragSession),
    Rectangle {
        start: Point2D,
        current: Point2D,
        additive: bool,
    },
}
#[derive(Default, Debug)]
pub struct Interaction {
    pub selection: SelectionState,
    pub hover: Option<RepresentationId>,
    pub tool: Tool,
    pub gesture: Option<Gesture>,
    pub insertion: Option<InsertionDraft>,
    pub graphic_start: Option<Point2D>,
    pub connection: Option<crate::connection_editor::ConnectionDraft>,
    pub route_drag: Option<crate::connection_editor::RouteDrag>,
    pub connection_snap: Option<crate::connection_editor::ConnectionSnap>,
    pub junction_selection: Option<ConnectionJunctionId>,
    pub junction_outline: bool,
    pub highlighted_routes: BTreeSet<RepresentationId>,
    pub connection_suggestion: Option<[Point2D; 2]>,
    pub macro_draft: Option<MacroDraft>,
}
impl Interaction {
    pub fn click(&mut self, id: Option<RepresentationId>, additive: bool) {
        self.junction_selection = None;
        self.selection.click(id, additive);
    }
    pub fn cancel(&mut self) {
        self.gesture = None;
        self.insertion = None;
        self.graphic_start = None;
        self.connection = None;
        self.route_drag = None;
        self.connection_snap = None;
        self.connection_suggestion = None;
        self.macro_draft = None;
        self.tool = Tool::Select;
    }
    pub fn is_active(&self) -> bool {
        self.gesture.is_some()
            || self.insertion.is_some()
            || self.connection.is_some()
            || self.route_drag.is_some()
            || matches!(
                self.tool,
                Tool::InsertJunction
                    | Tool::EditRoute
                    | Tool::AddVertex
                    | Tool::RemoveVertex
                    | Tool::InsertInterruption
                    | Tool::InsertPotential
                    | Tool::InsertGraphic(_)
                    | Tool::InsertText
                    | Tool::Paste
                    | Tool::InsertMacro
            )
    }
    pub fn update_insertion(&mut self, cursor: Point2D, spacing: f64, snap: bool) -> Result<()> {
        if let Some(draft) = &mut self.insertion {
            let t = draft.transform;
            draft.transform = Transform2D::new(
                crate::camera::snap(cursor, spacing, snap),
                t.rotation_degrees(),
                t.mirror_x(),
                t.mirror_y(),
            )?;
            draft.positioned = true;
        }
        Ok(())
    }
    pub fn begin_move(
        &mut self,
        p: &Project,
        page: PageId,
        grabbed: RepresentationId,
        start: Point2D,
    ) -> Result<()> {
        let r = p
            .representation(grabbed)
            .ok_or(DomainError::MissingRepresentation(grabbed))?;
        if !self.selection.ids.contains(&grabbed) {
            self.click(Some(grabbed), false);
        }
        let mut originals = Vec::with_capacity(self.selection.ids.len());
        for &id in self.selection.ids() {
            let r = p
                .representation(id)
                .ok_or(DomainError::MissingRepresentation(id))?;
            if r.page_id() != page || !r.visual().visible {
                return Err(DomainError::InvalidRelation(
                    "seleção fora da página ativa ou oculta",
                ));
            }
            originals.push((id, r.transform()));
        }
        self.gesture = Some(Gesture::Move(DragSession {
            start,
            anchor: r.transform().translation(),
            preview: originals.iter().copied().collect(),
            originals,
        }));
        Ok(())
    }
    pub fn update_move(&mut self, cursor: Point2D, spacing: f64, snap: bool) -> Result<()> {
        let Some(Gesture::Move(d)) = &mut self.gesture else {
            return Ok(());
        };
        let result = (|| {
            let dx = cursor.x() - d.start.x();
            let dy = cursor.y() - d.start.y();
            // Returning to the exact grab position is a no-op even off grid.
            let target = if dx == 0.0 && dy == 0.0 {
                d.anchor
            } else {
                crate::camera::snap(d.anchor.translated(dx, dy)?, spacing, snap)
            };
            let (dx, dy) = (target.x() - d.anchor.x(), target.y() - d.anchor.y());
            for &(id, t) in &d.originals {
                let new = Transform2D::new(
                    t.translation().translated(dx, dy)?,
                    t.rotation_degrees(),
                    t.mirror_x(),
                    t.mirror_y(),
                )?;
                d.preview.insert(id, new);
            }
            Ok(())
        })();
        if result.is_err() {
            d.preview.clear();
        }
        result
    }
    pub fn preview(&self, id: RepresentationId) -> Option<Transform2D> {
        if let Some(Gesture::Move(d)) = &self.gesture {
            d.preview.get(&id).copied()
        } else {
            None
        }
    }
    pub fn preview_transforms(&self) -> Option<&BTreeMap<RepresentationId, Transform2D>> {
        match &self.gesture {
            Some(Gesture::Move(d)) => Some(&d.preview),
            _ => None,
        }
    }
    pub fn finish_move(&mut self) -> Option<Vec<(RepresentationId, Transform2D)>> {
        self.tool = Tool::Select;
        match self.gesture.take() {
            Some(Gesture::Move(d)) if d.preview.len() == d.originals.len() => {
                Some(d.preview.into_iter().collect())
            }
            _ => None,
        }
    }
    pub fn begin_rectangle(&mut self, start: Point2D, additive: bool) {
        self.gesture = Some(Gesture::Rectangle {
            start,
            current: start,
            additive,
        });
    }
    pub fn update_rectangle(&mut self, point: Point2D) {
        if let Some(Gesture::Rectangle { current, .. }) = &mut self.gesture {
            *current = point;
        }
    }
    pub fn finish_rectangle(&mut self, p: &Project, page: PageId) {
        if let Some(Gesture::Rectangle {
            start,
            current,
            additive,
        }) = self.gesture.take()
        {
            let mut ids = crate::hit_test::rectangle_selection(
                p,
                page,
                start,
                current,
                current.x() < start.x(),
            );
            if additive {
                ids.extend(self.selection.ids.iter().copied());
            }
            self.selection.replace(ids);
        }
    }
}
