// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! draw support for the physical editor; start at the parent module.

use super::*;

impl PhysicalEditor {
    pub fn show(&mut self, session: &mut Session, ctx: &Context) {
        if !self.open {
            return;
        }
        self.refresh(session);
        let mut open = self.open;
        // On small screens or high DPI scales the window (title bar included)
        // must fit the screen; it is never larger than the available area.
        let screen = ctx.content_rect().size();
        // Sizes are of the content area; title bar and frame add ~50 points.
        let fit = egui::vec2((screen.x - 40.).max(420.), (screen.y - 80.).max(240.));
        egui::Window::new("Engenharia física")
            .open(&mut open)
            .default_size(egui::vec2(850., 610.).min(fit))
            .min_size(egui::vec2(420., 240.))
            .max_size(fit)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (tab, label) in [
                        (PhysicalTab::Strips, "Réguas e bornes"),
                        (PhysicalTab::Connectors, "Conectores"),
                        (PhysicalTab::Cables, "Cabos e veias"),
                        (PhysicalTab::Wiring, "Condutores"),
                        (PhysicalTab::Lists, "Listas técnicas"),
                    ] {
                        if ui.selectable_label(self.tab == tab, label).clicked() {
                            self.tab = tab;
                        }
                    }
                });
                ui.separator();
                // A resizable egui window grows to the content measured on the
                // previous frame, and its frame/background cover only what is
                // allocated in this Ui. So the feedback line is laid out from
                // the bottom and the body fills exactly the remaining space,
                // both allocated here (panels' show_inside would not allocate).
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    if !self.feedback.is_empty() {
                        ui.label(&self.feedback);
                        ui.separator();
                    }
                    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        match self.tab {
                            // The technical lists are already virtualized scroll areas.
                            PhysicalTab::Lists => self.lists_ui(session, ui),
                            tab => {
                                ScrollArea::both()
                                    .id_salt(("physical_body", tab as u8))
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| match tab {
                                        PhysicalTab::Strips => self.strips_ui(session, ui),
                                        PhysicalTab::Connectors => self.connectors_ui(session, ui),
                                        PhysicalTab::Cables => self.cables_ui(session, ui),
                                        _ => self.wiring_ui(session, ui),
                                    });
                            }
                        }
                    });
                });
            });
        self.open = open;
    }
}
