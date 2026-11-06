// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Reference navigation and marker insertion through the common dispatcher.
use crate::{
    camera::Camera,
    editing::GraphicEdit,
    state::{Selection, Session},
};
use cad_craft::{
    domain::*,
    error::{DomainError, Result},
};

pub fn marker_definition(potential: bool) -> Result<SymbolDefinition> {
    let p = |x, y| Point2D::new(x, y);
    let primitives = if potential {
        vec![
            SymbolPrimitive::Line {
                start: p(-4., 0.)?,
                end: p(4., 0.)?,
            },
            SymbolPrimitive::Circle {
                center: p(0., 3.)?,
                radius: 3.,
            },
        ]
    } else {
        vec![SymbolPrimitive::Polyline {
            points: vec![p(-5., -3.)?, p(0., 0.)?, p(-5., 3.)?],
            closed: false,
        }]
    };
    SymbolDefinition::new(
        SymbolDefinitionId::new(if potential {
            "cad-craft.potential"
        } else {
            "cad-craft.interruption"
        })?,
        1,
        if potential {
            "Identificação de potencial"
        } else {
            "Ponto de interrupção"
        },
        "Referências",
        primitives,
        vec![SymbolPort::new("p", p(0., 0.)?, 0., "p", None)?],
    )
}
impl Session {
    pub fn insert_electrical_marker(
        &mut self,
        kind: MarkerKind,
        name: Text,
        point: Option<ConnectionPointId>,
        position: Point2D,
    ) -> Result<RepresentationId> {
        let page = self.active.ok_or(DomainError::InvalidRelation(
            crate::strings::OPEN_PAGE_TO_INSERT,
        ))?;
        let project = self.project().ok_or(DomainError::InvalidRelation(
            crate::strings::NO_OPEN_PROJECT,
        ))?;
        let definition = marker_definition(matches!(kind, MarkerKind::Potential { .. }))?;
        let representation = Representation::new_symbol(
            project.id(),
            page,
            &definition,
            point.map(RepresentationTarget::ConnectionPoint),
            Transform2D::new(position, 0., false, false)?,
        )?;
        let id = representation.id();
        let marker = ElectricalMarker::new(id, name, kind, point);
        self.dispatch_graphic(GraphicEdit::Marker(MarkerEdit::Insert {
            marker,
            representation: Box::new(representation),
            definition: Box::new(definition),
        }))?;
        Ok(id)
    }
    /// Resolve before changing tabs; navigation never mutates the document.
    pub fn navigate_reference(&mut self, id: RepresentationId) -> Result<()> {
        let project = self.project().ok_or(DomainError::InvalidRelation(
            crate::strings::NO_OPEN_PROJECT,
        ))?;
        let r = project
            .representation(id)
            .ok_or(DomainError::MissingRepresentation(id))?;
        let page = r.page_id();
        let position = r.transform().translation();
        let scale = self
            .tabs
            .iter()
            .find(|t| t.page == page)
            .map(|t| t.camera.scale())
            .unwrap_or(crate::camera::SCALE_100);
        let camera = Camera::new([position.x(), position.y()], scale).ok_or(
            DomainError::InvalidRelation("destino fora da faixa de navegação"),
        )?;
        self.open_page(page)?;
        if let Some(tab) = self.active_tab_mut() {
            tab.previous = Some(tab.camera);
            tab.camera = camera;
            tab.needs_fit = false;
        }
        self.select(Selection::Representation(id));
        Ok(())
    }
    pub fn selected_reference_device(&self) -> Option<DeviceId> {
        let p = self.project()?;
        let target = match self.selection {
            Selection::Electrical(t) => t,
            Selection::Representation(id) => p.representation(id)?.target(),
            _ => return None,
        };
        p.device_for_target(target)
    }
    pub fn reference_destinations(&self) -> Vec<RepresentationId> {
        let Some(p) = self.project() else {
            return vec![];
        };
        if let Selection::Representation(id) = self.selection {
            if p.marker(id).is_some() {
                return p.marker_references(id).unwrap_or_default();
            }
            if let Some(device) = self.selected_reference_device() {
                return p
                    .device_references(device)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|r| r.representation)
                    .filter(|r| *r != id)
                    .collect();
            }
            return p
                .related_representations(id)
                .unwrap_or_default()
                .iter()
                .map(|r| r.id())
                .collect();
        }
        self.selected_reference_device()
            .and_then(|d| p.device_references(d).ok())
            .unwrap_or_default()
            .into_iter()
            .map(|r| r.representation)
            .collect()
    }
}
