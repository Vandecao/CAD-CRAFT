// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Entity navigator rendering; start with CadApp::entity_navigator.
use super::*;

impl CadApp {
    pub(super) fn entity_navigator(&mut self, ui: &mut Ui) {
        if matches!(
            self.navigator.tab,
            NavigatorTab::Cables | NavigatorTab::Terminals
        ) {
            self.physical_navigator(ui);
            return;
        }
        let Some(p) = self.session.project() else {
            ui.weak(crate::strings::NO_OPEN_PROJECT_NOTICE);
            return;
        };
        let mut entries: Vec<(String, Option<RepresentationTarget>)> = vec![];
        let cache_key = (p.id(), self.session.revision, self.navigator.tab);
        let rebuild_references = self.navigator.reference_cache_key != Some(cache_key);
        let mut references: Vec<(String, RepresentationId)> = vec![];
        match self.navigator.tab {
            NavigatorTab::Devices => {
                if rebuild_references {
                    for device in p.devices() {
                        for r in p.device_references(device.id()).unwrap_or_default() {
                            references.push((
                                format!(
                                    "{} · {:?} · {}",
                                    device.designation(),
                                    r.role,
                                    r.reference
                                ),
                                r.representation,
                            ));
                        }
                    }
                }
                entries.extend(p.devices().map(|d| {
                    (
                        format!("{} · {}", d.designation(), d.classification().as_str()),
                        Some(RepresentationTarget::Device(d.id())),
                    )
                }));
            }
            NavigatorTab::Connections => {
                if rebuild_references {
                    for marker in p.markers() {
                        if let Some(rep) = p.representation(marker.id()) {
                            let page = p
                                .page(rep.page_id())
                                .map(|p| p.number().as_str())
                                .unwrap_or("?");
                            references.push((
                                format!(
                                    "{} · página {} · X {:.3} · Y {:.3} mm",
                                    marker.name().as_str(),
                                    page,
                                    rep.transform().translation().x(),
                                    rep.transform().translation().y()
                                ),
                                marker.id(),
                            ));
                        }
                    }
                }
                entries.extend(p.connections().map(|c| {
                    (
                        format!("Conexão {}", c.id()),
                        Some(RepresentationTarget::Connection(c.id())),
                    )
                }));
            }
            // Rendered by physical_navigator from the revision cache.
            NavigatorTab::Cables | NavigatorTab::Terminals => {}
            NavigatorTab::Structures => {
                for doc in p.documents() {
                    for axis in [StructureAxis::Function, StructureAxis::Location] {
                        for group in p.page_tree(doc.id(), axis).unwrap_or_default() {
                            if !group.path.is_empty() {
                                entries.push((
                                    format!(
                                        "{} · {}{}",
                                        doc.name().as_str(),
                                        if axis == StructureAxis::Function {
                                            "="
                                        } else {
                                            "+"
                                        },
                                        crate::navigation::path_text(&group.path)
                                    ),
                                    None,
                                ));
                            }
                        }
                    }
                }
            }
            NavigatorTab::Pages => {}
        }
        if rebuild_references {
            self.navigator.reference_cache_key = Some(cache_key);
            self.navigator.reference_cache = references.clone();
        } else {
            references = self.navigator.reference_cache.clone();
        }
        let mut target = None;
        let mut reference_target = None;
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if entries.is_empty() && references.is_empty() {
                    ui.weak("Nenhum registro disponível neste navegador.");
                }
                for (label, id) in entries {
                    if ui
                        .selectable_label(
                            id.is_some_and(|id| {
                                self.session.selection == Selection::Electrical(id)
                            }) || (id.is_none()
                                && self.navigator.selected_structure.as_ref() == Some(&label)),
                            &label,
                        )
                        .clicked()
                    {
                        if let Some(id) = id {
                            target = Some(id);
                        } else {
                            self.navigator.selected_structure = Some(label.clone());
                        }
                    }
                }
                for (label, id) in references {
                    if ui
                        .selectable_label(
                            self.session.selection == Selection::Representation(id),
                            label,
                        )
                        .clicked()
                    {
                        reference_target = Some(id);
                    }
                }
            });
        if let Some(target) = target {
            self.session.select(Selection::Electrical(target));
        }
        if let Some(id) = reference_target {
            self.navigate_to_reference(id);
        }
    }
}
