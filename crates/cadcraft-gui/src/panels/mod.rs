// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
mod insertion;
mod messages;
mod navigator;
mod preview;
mod properties;
use crate::{
    commands::Icon,
    icons,
    preferences::{PanelId, Preferences},
    theme,
};
use eframe::egui::{self, Align, Layout, Ui};
pub(crate) use insertion::InsertionState;
#[cfg(test)]
pub(crate) use navigator::physical_tree;
pub(crate) use navigator::{NavigatorState, NavigatorTab};
pub(crate) use properties::Inspector;
pub(crate) fn header(ui: &mut Ui, preferences: &mut Preferences, id: PanelId, title: &str) -> bool {
    header_with(ui, preferences, id, |ui| {
        ui.add(egui::Label::new(egui::RichText::new(title).size(11.0)).truncate());
    })
}

/// Panel header whose left side is drawn by `left` (a title or tabs), so
/// tabs share the header row and take no extra height.
pub(crate) fn header_with(
    ui: &mut Ui,
    preferences: &mut Preferences,
    id: PanelId,
    left: impl FnOnce(&mut Ui),
) -> bool {
    let compact = ui.available_width() < 200.0;
    ui.horizontal(|ui| {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 1.0;
            if !compact {
                if icons::tiny(ui, Icon::Close, &format!("Fechar {}", id.label())).clicked() {
                    preferences.panel_mut(id).visible = false;
                }
                if icons::tiny(ui, Icon::Collapse, &format!("Recolher {}", id.label())).clicked() {
                    preferences.panel_mut(id).collapsed = true;
                }
                let p = preferences.panel_mut(id);
                if icons::button(
                    ui,
                    Icon::Pin,
                    "",
                    "Fixar painel: preserva ao recolher painéis desafixados",
                    true,
                    p.pinned,
                    false,
                )
                .clicked()
                {
                    p.pinned = !p.pinned;
                }
            }
            icons::menu(ui, Icon::More, "Opções do painel", |ui| {
                ui.checkbox(&mut preferences.panel_mut(id).pinned, "Fixar painel");
                if ui.button("Recolher painel").clicked() {
                    preferences.panel_mut(id).collapsed = true;
                    ui.close();
                }
                if ui.button("Fechar painel").clicked() {
                    preferences.panel_mut(id).visible = false;
                    ui.close();
                }
                if ui.button("Recolher painéis desafixados").clicked() {
                    for panel in PanelId::ALL {
                        let p = preferences.panel_mut(panel);
                        if !p.pinned {
                            p.collapsed = true;
                        }
                    }
                    ui.close();
                }
            });
            ui.with_layout(Layout::left_to_right(Align::Center), left);
        });
    });
    ui.painter().hline(
        ui.max_rect().x_range(),
        ui.cursor().top(),
        egui::Stroke::new(1.0_f32, theme::LINE),
    );
    preferences.panel(id).visible && !preferences.panel(id).collapsed
}
