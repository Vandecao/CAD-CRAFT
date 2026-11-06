// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Inspect real electrical data; all writes go through the graphical dispatcher.
use crate::{
    app::{CadApp, Level},
    commands::CommandId,
    connection_geometry::orthogonal_route,
    editing::GraphicEdit,
    state::Selection,
};
use cad_craft::domain::*;
use eframe::egui::{self, Ui};

#[derive(Default)]
pub(crate) struct ConnectionInspector {
    key: Option<(RepresentationId, u64, u64)>,
    points: Vec<Point2D>,
    information: Vec<(String, String)>,
    label: String,
    number_label_x: f64,
    number_label_y: f64,
    number: String,
    number_locked: bool,
}
fn endpoint_description(p: &Project, port: &PortRef) -> String {
    let Some(point) = p.port_binding(port).and_then(|id| p.connection_point(id)) else {
        return "Sem associação".into();
    };
    let device = p
        .function(point.function_id())
        .and_then(|f| p.device(f.device_id()));
    format!(
        "{} · {} · porta {}",
        device
            .map(|d| d.designation().to_string())
            .unwrap_or_default(),
        point.designation().as_str(),
        port.port_id()
    )
}
pub(crate) fn port_controls(
    ui: &mut Ui,
    p: &Project,
    id: RepresentationId,
) -> Option<ConnectionDrawingEdit> {
    let r = p.representation(id)?;
    let d = p.symbol_definition(r.symbol()?)?;
    let mut edit = None;
    egui::CollapsingHeader::new("Vínculos elétricos das portas").default_open(true).show(ui,|ui| {
        if r.target()==RepresentationTarget::Unassociated {ui.weak("Sem associação elétrica. Insira o símbolo com um alvo existente para vincular suas portas.");}
        for port in d.ports() {
            let Ok(reference)=PortRef::new(id,port.id())else{continue;};
            let old=p.port_binding(&reference);let mut selected=old;
            ui.label(format!("Porta {} · {}",port.id(),port.identification()));
            egui::ComboBox::from_id_salt(("port_binding",id,port.id())).width(ui.available_width().max(50.))
                .selected_text(old.and_then(|id|p.connection_point(id)).map(|p|format!("Ponto {}",p.designation().as_str())).unwrap_or_else(||"Sem vínculo".into()))
                .show_ui(ui,|ui| {
                    ui.selectable_value(&mut selected,None,"Sem vínculo");
                    for point in p.port_binding_candidates(&reference).unwrap_or_default() {
                        if let Some(cp)=p.connection_point(point) {ui.selectable_value(&mut selected,Some(point),format!("{} · {}",cp.designation().as_str(),cp.id()));}
                    }
                });
            if selected!=old {edit=Some(ConnectionDrawingEdit::BindPort{port:reference,point:selected});}
        }
    });
    edit
}
/// Physical wiring rows for a logical connection. Route, logical connection,
/// conductor and cable are reported as distinct layers.
pub(crate) fn physical_rows(p: &Project, connection: ConnectionId) -> Vec<(String, String)> {
    if p.terminal_bridge(connection).is_some() {
        return vec![(
            "Fiação física".into(),
            "Ponte explícita de régua; não recebe condutor".into(),
        )];
    }
    let wires = p
        .physical_for_connection(connection)
        .map(|e| e.conductors().to_vec())
        .unwrap_or_default();
    if wires.is_empty() {
        return vec![(
            "Fiação física".into(),
            "Nenhum condutor físico associado".into(),
        )];
    }
    let mut rows = vec![("Condutores vinculados".into(), wires.len().to_string())];
    for id in wires {
        let Some(wire) = p.conductor(id) else {
            continue;
        };
        let (a, b) = wire.endpoints();
        let cable = p
            .core_for_conductor(id)
            .and_then(|core| {
                p.cable(core.cable_id()).map(|cable| {
                    format!(
                        " · cabo {} veia {}",
                        cable.designation().as_str(),
                        core.identification().as_str()
                    )
                })
            })
            .unwrap_or_else(|| " · fio individual".into());
        rows.push((
            format!("Condutor {}", wire.designation().as_str()),
            format!(
                "{}/{} → {}/{} · {} mm²{cable}",
                p.terminal_label(a.terminal_id()),
                a.position().as_str(),
                p.terminal_label(b.terminal_id()),
                b.position().as_str(),
                wire.cross_section().get()
            ),
        ));
    }
    rows
}
impl CadApp {
    pub(crate) fn show_connection_properties(&mut self, ui: &mut Ui) -> bool {
        if let Some(id) = self.session.graphics.junction_selection {
            let Some(j) = self
                .session
                .project()
                .and_then(|p| p.connection_junction(id))
            else {
                return false;
            };
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Junção elétrica");
                ui.label(format!("ID: {}", j.id()));
                ui.label(format!("Página: {}", j.page_id()));
                ui.label(format!(
                    "X {:.3} · Y {:.3} mm",
                    j.position().x(),
                    j.position().y()
                ));
                ui.label(format!(
                    "{} rotas · {} uniões explícitas",
                    j.routes().len(),
                    j.connections().len()
                ));
                ui.weak("O ponto une as rotas indicadas. Cruzamentos sem marca não criam conexão.");
            });
            if self.command_button(ui, CommandId::DeleteJunction, false, false) {
                self.execute(CommandId::DeleteJunction, ui.ctx());
            }
            return true;
        }
        let Selection::Representation(id) = self.session.selection else {
            return false;
        };
        let Some(p) = self.session.project() else {
            return false;
        };
        let Some(route) = p.connection_route(id) else {
            return false;
        };
        let key = (id, self.session.revision, self.session.selection_revision);
        let mut inspector = std::mem::take(&mut self.panels_state.connection_inspector);
        if inspector.key != Some(key) {
            inspector.key = Some(key);
            inspector.points = route.points().to_vec();
            let number_label = route
                .number_label_position()
                .or_else(|| route.points().first().copied());
            inspector.number_label_x = number_label.map_or(0.0, Point2D::x);
            inspector.number_label_y = number_label.map_or(0.0, Point2D::y);
            let connection = p.connection(route.connection_id());
            inspector.number = connection
                .and_then(Connection::number)
                .map(|n| n.as_str().to_owned())
                .unwrap_or_default();
            inspector.number_locked = connection.is_some_and(Connection::number_locked);
            inspector.label = p
                .representation(id)
                .and_then(|r| r.visual().label.as_ref())
                .map(|t| t.as_str().to_string())
                .unwrap_or_default();
            inspector.information = vec![
                ("Representação".into(), id.to_string()),
                ("Conexão".into(), route.connection_id().to_string()),
                (
                    "Página".into(),
                    p.page(route.page_id())
                        .map(|p| format!("{} · {}", p.number().as_str(), p.name().as_str()))
                        .unwrap_or_default(),
                ),
                ("Tipo".into(), "Conexão lógica multifilar".into()),
                ("Origem".into(), endpoint_description(p, route.source())),
                ("Destino".into(), endpoint_description(p, route.target())),
            ];
            if let Some(point) = p.port_binding(route.source())
                && let Ok(net) = p.network_for_point(point)
            {
                inspector.information.push((
                    "Rede derivada".into(),
                    format!(
                        "{} pontos · {} conexões",
                        net.points().len(),
                        net.connections().len()
                    ),
                ));
                inspector.information.push((
                    "Referência da rede".into(),
                    net.points()
                        .first()
                        .map(ToString::to_string)
                        .unwrap_or_default(),
                ));
                inspector.information.push((
                    "Dispositivos".into(),
                    net.devices()
                        .iter()
                        .filter_map(|id| p.device(*id))
                        .map(|d| d.designation().to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                ));
            }
            if let Some(c) = p.connection(route.connection_id()) {
                let (a, b) = c.endpoints();
                inspector
                    .information
                    .push(("Endpoint A".into(), a.to_string()));
                inspector
                    .information
                    .push(("Endpoint B".into(), b.to_string()));
            }
            inspector
                .information
                .extend(physical_rows(p, route.connection_id()));
        }
        let mut edit = None;
        let mut appearance = None;
        let mut number_edit: Option<(ConnectionId, String, bool)> = None;
        let mut command = None;
        egui::ScrollArea::vertical().auto_shrink([false,false]).show(ui,|ui| {
            ui.strong("Conexão elétrica");
            for (label,value) in &inspector.information {ui.weak(label);ui.add(egui::Label::new(value).wrap().selectable(true));}
            ui.separator();ui.label("Identificação gráfica");ui.text_edit_singleline(&mut inspector.label);
            ui.label("Número do fio");
            ui.text_edit_singleline(&mut inspector.number);
            ui.checkbox(&mut inspector.number_locked, "Travar número do fio");
            if ui.button("Aplicar número do fio").clicked() {
                number_edit = Some((route.connection_id(), inspector.number.trim().to_owned(), inspector.number_locked));
            }
            ui.horizontal(|ui| { ui.label("Rótulo X/Y (mm)"); ui.add(egui::DragValue::new(&mut inspector.number_label_x).speed(1.0)); ui.add(egui::DragValue::new(&mut inspector.number_label_y).speed(1.0)); });
            if ui.button("Posicionar número do fio").clicked()
                && let Ok(position) = Point2D::new(inspector.number_label_x, inspector.number_label_y) {
                edit = Some(ConnectionDrawingEdit::SetNumberLabelPosition { id, position: Some(position) });
            }
            if ui.button("Aplicar identificação").clicked() {
                let mut visual=p.representation(id).expect("route representation").visual().clone();
                visual.label=if inspector.label.trim().is_empty(){None}else{Text::new(inspector.label.clone()).ok()};appearance=Some(visual);
            }
            ui.separator();ui.strong("Rota em milímetros");
            ui.weak("Extremos vinculados. Arraste segmentos/vértices no desenho ou ajuste os pontos intermediários abaixo.");
            let len=inspector.points.len();
            for (i,point) in inspector.points.iter_mut().enumerate() {
                ui.push_id(i,|ui|{ui.horizontal(|ui| {
                    ui.label(format!("{i}"));let mut x=point.x();let mut y=point.y();
                    ui.add_enabled(i>0&&i+1<len,egui::DragValue::new(&mut x).speed(1.).prefix("X "));
                    ui.add_enabled(i>0&&i+1<len,egui::DragValue::new(&mut y).speed(1.).prefix("Y "));
                    if let Ok(updated)=Point2D::new(x,y){*point=updated;}
                });});
            }
            if ui.button("Aplicar rota ortogonal").clicked(){edit=Some(ConnectionDrawingEdit::SetRoute{id,points:orthogonal_route(&inspector.points,true)});}
            for (id,label) in [(CommandId::WiringEditor,"Atribuir fiação física"),(CommandId::EditConnection,"Editar rota"),(CommandId::AddConnectionVertex,"Adicionar vértice"),(CommandId::RemoveConnectionVertex,"Remover vértice"),(CommandId::HighlightNetwork,"Destacar rede"),(CommandId::DeleteConnection,"Excluir conexão")] {
                if ui.button(label).clicked(){command=Some(id);}
            }
        });
        self.panels_state.connection_inspector = inspector;
        if let Some(edit) = edit {
            self.graphic_edit(GraphicEdit::Drawing(edit));
        }
        if let Some((connection, number, locked)) = number_edit {
            // Empty input clears the number; names never create connectivity.
            let result = if number.is_empty() {
                Ok(None)
            } else {
                Text::new(number).map(Some)
            }
            .and_then(|number| {
                self.session
                    .set_connection_number(connection, number, locked)
            });
            match result {
                Ok(true) => self.log(Level::Info, "Número do fio registrado no histórico."),
                Ok(false) => {}
                Err(e) => self.log(Level::Error, e.to_string()),
            }
        }
        if let Some(visual) = appearance {
            self.graphic_edit(GraphicEdit::Visual(id, visual));
        }
        if let Some(command) = command {
            self.execute(command, ui.ctx());
        }
        true
    }
}
