// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Occurrence reference property controls; start with the reference inspector renderer.
use crate::{
    app::{CadApp, Level},
    device_properties::DeviceInspector,
    editing::GraphicEdit,
    preferences::PanelId,
    state::Selection,
};
use cad_craft::domain::*;
use eframe::egui::{self, Ui};
pub(crate) struct ReferenceInspector {
    pub details: bool,
    pub device: DeviceInspector,
    key: Option<(Selection, u64, u64)>,
    name: String,
    kind: String,
    description: String,
    point: Option<ConnectionPointId>,
    mode: ReferenceMode,
    members: Vec<RepresentationId>,
    continuity: bool,
    group: Option<ReferenceGroupId>,
    potential_conflict: bool,
}
impl Default for ReferenceInspector {
    fn default() -> Self {
        Self {
            details: false,
            device: DeviceInspector::default(),
            key: None,
            name: String::new(),
            kind: String::new(),
            description: String::new(),
            point: None,
            mode: ReferenceMode::Star,
            members: vec![],
            continuity: false,
            group: None,
            potential_conflict: false,
        }
    }
}
impl CadApp {
    pub(crate) fn open_reference_properties(&mut self) {
        self.panels_state.reference_inspector.details = true;
        let p = self.preferences.panel_mut(PanelId::Properties);
        p.visible = true;
        p.collapsed = false;
    }
    pub(crate) fn navigate_to_reference(&mut self, id: RepresentationId) {
        match self.session.navigate_reference(id) {
            Ok(()) => self.open_reference_properties(),
            Err(e) => self.log(Level::Error, e.to_string()),
        }
    }
    pub(crate) fn show_reference_properties(&mut self, ui: &mut Ui) -> bool {
        let selection = self.session.selection;
        let eligible = match selection {
            Selection::Electrical(_) => self.session.selected_reference_device().is_some(),
            Selection::Representation(id) => self.session.project().is_some_and(|p| {
                p.marker(id).is_some() || self.session.selected_reference_device().is_some()
            }),
            _ => false,
        };
        if !eligible {
            return false;
        }
        let key = (
            selection,
            self.session.revision,
            self.session.selection_revision,
        );
        let mut inspector = std::mem::take(&mut self.panels_state.reference_inspector);
        if inspector.key.map(|k| k.0) != Some(selection) {
            inspector.details = matches!(selection, Selection::Electrical(_))
                || matches!(selection,Selection::Representation(id) if self.session.project().is_some_and(|p|p.marker(id).is_some()));
        }
        if inspector.key != Some(key) {
            inspector.key = Some(key);
            if let Selection::Representation(id) = selection
                && let Some(p) = self.session.project()
                && let Some(m) = p.marker(id)
            {
                inspector.potential_conflict = p
                    .potential_conflicts()
                    .iter()
                    .any(|c| c.first == id || c.second == id);
                inspector.name = m.name().as_str().into();
                inspector.point = m.point();
                if let MarkerKind::Potential { kind, description } = m.kind() {
                    inspector.kind = kind.as_ref().map(|v| v.as_str()).unwrap_or("").into();
                    inspector.description = description.clone();
                }
                if let Some(g) = p.marker_groups().find(|g| g.members().contains(&id)) {
                    inspector.group = Some(g.id());
                    inspector.mode = g.mode();
                    inspector.members = g.members().to_vec();
                    inspector.continuity = g.continuity();
                } else {
                    inspector.group = None;
                    inspector.mode = ReferenceMode::Star;
                    inspector.members = vec![id];
                    inspector.continuity = false;
                }
            }
        }
        if !inspector.details {
            if ui.button("Identificação e referências").clicked() {
                inspector.details = true;
            }
            self.panels_state.reference_inspector = inspector;
            return false;
        }
        if ui.button("Geometria e aparência").clicked() {
            inspector.details = false;
            self.panels_state.reference_inspector = inspector;
            return false;
        }
        let mut device_edit = None;
        let mut marker_edit = None;
        let mut nav = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Selection::Representation(id) = selection
                && let Some(p) = self.session.project()
                && let Some(m) = p.marker(id)
            {
                ui.heading(match m.kind() {
                    MarkerKind::Interruption => "Ponto de interrupção",
                    MarkerKind::Potential { .. } => "Identificação de potencial",
                });
                ui.label(format!("ID: {}", id));
                if inspector.potential_conflict {
                    ui.colored_label(
                        ui.visuals().error_fg_color,
                        "Conflito: identificações de potencial incompatíveis na rede elétrica.",
                    );
                }
                if let Some(r) = p.representation(id) {
                    ui.label(format!(
                        "Posição: {}",
                        p.reference_label(r.page_id(), r.transform().translation())
                    ));
                }
                ui.strong("Referências");
                let refs = p.marker_references(id).unwrap_or_default();
                if refs.is_empty() {
                    ui.weak("Sem destino vinculado.");
                }
                for target in refs {
                    if let (Some(dest), Some(rep)) = (p.marker(target), p.representation(target))
                        && ui
                            .link(format!(
                                "Ir: {} · {}",
                                dest.name().as_str(),
                                p.reference_label(rep.page_id(), rep.transform().translation())
                            ))
                            .clicked()
                    {
                        nav = Some(target);
                    }
                }
                ui.separator();
                ui.label("Identificação");
                ui.text_edit_singleline(&mut inspector.name);
                if matches!(m.kind(), MarkerKind::Potential { .. }) {
                    ui.label("Tipo");
                    ui.text_edit_singleline(&mut inspector.kind);
                    ui.label("Descrição");
                    ui.text_edit_multiline(&mut inspector.description);
                }
                ui.label("Ponto elétrico associado");
                egui::ComboBox::from_id_salt(("marker_point", id))
                    .selected_text(
                        inspector
                            .point
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "Sem vínculo".into()),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut inspector.point, None, "Sem vínculo");
                        for cp in p.connection_points() {
                            ui.selectable_value(
                                &mut inspector.point,
                                Some(cp.id()),
                                format!("{} · {}", cp.designation().as_str(), cp.id()),
                            );
                        }
                    });
                if ui.button("Aplicar marcador").clicked() {
                    let kind = match m.kind() {
                        MarkerKind::Interruption => MarkerKind::Interruption,
                        MarkerKind::Potential { .. } => MarkerKind::Potential {
                            kind: if inspector.kind.trim().is_empty() {
                                None
                            } else {
                                Text::new(&inspector.kind).ok()
                            },
                            description: inspector.description.clone(),
                        },
                    };
                    if let Ok(name) = Text::new(&inspector.name) {
                        marker_edit = Some(MarkerEdit::Update {
                            id,
                            name,
                            kind,
                            point: inspector.point,
                        });
                    }
                }
                if ui.button("Excluir marcador").clicked() {
                    marker_edit = Some(MarkerEdit::Delete { id });
                }
                if matches!(m.kind(), MarkerKind::Interruption) {
                    ui.separator();
                    ui.strong("Grupo de referências");
                    egui::ComboBox::from_id_salt(("marker_mode", id))
                        .selected_text(format!("{:?}", inspector.mode))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut inspector.mode,
                                ReferenceMode::Star,
                                "Estrela",
                            );
                            ui.selectable_value(
                                &mut inspector.mode,
                                ReferenceMode::Chain,
                                "Cadeia",
                            );
                        });
                    ui.checkbox(&mut inspector.continuity, "Continuidade elétrica explícita");
                    for candidate in p
                        .markers()
                        .filter(|m| matches!(m.kind(), MarkerKind::Interruption))
                    {
                        let present = inspector.members.contains(&candidate.id());
                        let mut selected = present;
                        ui.checkbox(
                            &mut selected,
                            format!("{} · {}", candidate.name().as_str(), candidate.id()),
                        );
                        if selected && !present {
                            inspector.members.push(candidate.id());
                        }
                        if !selected && present {
                            inspector.members.retain(|v| *v != candidate.id());
                        }
                    }
                    ui.label("Ordem dos pontos (primeiro = principal na estrela)");
                    let mut swap = None;
                    for (index, member) in inspector.members.iter().enumerate() {
                        let name = p
                            .marker(*member)
                            .map(|m| m.name().as_str())
                            .unwrap_or("Destino ausente");
                        ui.horizontal(|ui| {
                            ui.label(format!("{}. {}", index + 1, name));
                            if ui.add_enabled(index > 0, egui::Button::new("↑")).clicked() {
                                swap = Some((index, index - 1));
                            }
                            if ui
                                .add_enabled(
                                    index + 1 < inspector.members.len(),
                                    egui::Button::new("↓"),
                                )
                                .clicked()
                            {
                                swap = Some((index, index + 1));
                            }
                        });
                    }
                    if let Some((a, b)) = swap {
                        inspector.members.swap(a, b);
                    }
                    if ui.button("Aplicar grupo").clicked() {
                        marker_edit = Some(MarkerEdit::SetGroup {
                            id: inspector.group.unwrap_or_default(),
                            mode: inspector.mode,
                            members: inspector.members.clone(),
                            continuity: inspector.continuity,
                        });
                    }
                    if let Some(group) = inspector.group
                        && ui.button("Remover grupo").clicked()
                    {
                        marker_edit = Some(MarkerEdit::DeleteGroup { id: group });
                    }
                }
            }
            if self.session.selected_reference_device().is_some() {
                let (edit, destination) =
                    crate::device_properties::show(ui, &self.session, &mut inspector.device);
                device_edit = edit;
                nav = nav.or(destination);
            }
        });
        self.panels_state.reference_inspector = inspector;
        if let Some(edit) = device_edit {
            match edit {
                Ok(edit) => self.graphic_edit(GraphicEdit::Device(edit)),
                Err(e) => self.log(Level::Error, e.to_string()),
            }
        }
        if let Some(edit) = marker_edit {
            self.graphic_edit(GraphicEdit::Marker(edit));
        }
        if let Some(dest) = nav {
            self.navigate_to_reference(dest);
        }
        true
    }
}
