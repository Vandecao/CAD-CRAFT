// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Diagnostic message panel; start with the panel renderer and navigation actions.
use crate::{
    app::{CadApp, Level},
    preferences::PanelId,
    theme,
};
use eframe::egui::{self, Context, RichText};
impl CadApp {
    pub(crate) fn show_messages(&mut self, ctx: &Context) {
        if !self.preferences.panel(PanelId::Messages).visible {
            return;
        }
        let collapsed = self.preferences.panel(PanelId::Messages).collapsed;
        let max_height = (ctx.available_rect().height() * 0.3).max(55.0);
        let output =
            egui::TopBottomPanel::bottom(egui::Id::new(("messages", self.layout_generation)))
                .resizable(!collapsed)
                .default_height(if collapsed {
                    28.0
                } else {
                    self.preferences.messages_height.min(max_height)
                })
                .min_height(if collapsed { 28.0 } else { 55.0 })
                .max_height(if collapsed { 28.0 } else { max_height })
                .frame(egui::Frame::new().fill(theme::PANEL).inner_margin(3))
                .show(ctx, |ui| {
                    if collapsed {
                        if ui.button("Expandir mensagens").clicked() {
                            self.preferences.panel_mut(PanelId::Messages).collapsed = false;
                        }
                        return;
                    }
                    if !super::header(ui, &mut self.preferences, PanelId::Messages, "Mensagens") {
                        return;
                    }
                    ui.horizontal(|ui| {
                        for (filter, label) in [
                            (None, "Todos"),
                            (Some(Level::Info), "Informações"),
                            (Some(Level::Warning), "Avisos"),
                            (Some(Level::Error), "Erros"),
                        ] {
                            let count = self
                                .messages
                                .iter()
                                .filter(|m| filter.is_none_or(|level| m.level == level))
                                .count();
                            ui.selectable_value(
                                &mut self.message_filter,
                                filter,
                                format!("{label} ({count})"),
                            );
                        }
                        if ui.small_button("Limpar").clicked() {
                            self.messages.clear();
                        }
                    });
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for message in self.messages.iter().filter(|m| {
                                self.message_filter.is_none_or(|level| m.level == level)
                            }) {
                                let (label, color) = match message.level {
                                    Level::Info => ("INFO", theme::MUTED),
                                    Level::Warning => ("AVISO", theme::WARNING),
                                    Level::Error => ("ERRO", theme::ERROR),
                                };
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(label).small().color(color));
                                    ui.label(RichText::new(&message.text).small());
                                });
                            }
                        });
                });
        if !collapsed {
            self.preferences.messages_height = output.response.rect.height();
        }
    }
}
