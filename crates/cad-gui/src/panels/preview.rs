// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use crate::{
    app::CadApp, camera::Camera, preferences::PanelId, state::Selection, theme, viewport::draw_page,
};
use eframe::egui::{self, Sense, Ui};
impl CadApp {
    pub(crate) fn show_preview(&mut self, ui: &mut Ui) {
        if !super::header(
            ui,
            &mut self.preferences,
            PanelId::Preview,
            "Pré-visualização",
        ) {
            return;
        }
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.preferences.thumbnails_grid, true, "Grade");
            ui.selectable_value(&mut self.preferences.thumbnails_grid, false, "Lista");
        });
        let Some(project) = self.session.project() else {
            ui.weak("Nenhuma página disponível.");
            return;
        };
        let mut selection = None;
        let mut open = None;
        egui::ScrollArea::vertical()
            .id_salt("thumbnails")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let columns = if self.preferences.thumbnails_grid {
                    (ui.available_width() / 105.0).floor().max(1.0) as usize
                } else {
                    1
                };
                let width = ((ui.available_width() - 6.0 * (columns - 1) as f32) / columns as f32)
                    .max(40.0);
                let row_height = 95.0;
                let count = self.index.rows.len().div_ceil(columns);
                // Only visible thumbnail rows are rendered.
                egui::ScrollArea::vertical()
                    .id_salt("thumbnail_rows")
                    .show_rows(ui, row_height, count, |ui, range| {
                        for row in range {
                            ui.horizontal(|ui| {
                                for page in self.index.rows.iter().skip(row * columns).take(columns)
                                {
                                    ui.vertical(|ui| {
                                        let (rect, response) = ui.allocate_exact_size(
                                            egui::vec2(width, 70.0),
                                            Sense::click(),
                                        );
                                        let painter = ui.painter_at(rect);
                                        painter.rect_filled(rect, 1, theme::CANVAS);
                                        if let Ok(bounds) = project.document_bounds(page.id) {
                                            let mut camera = Camera::default();
                                            camera.fit(bounds.sheet, rect.shrink(4.0));
                                            draw_page(
                                                &painter,
                                                project,
                                                page.id,
                                                camera,
                                                rect,
                                                Selection::None,
                                                None,
                                            );
                                        }
                                        if self.session.selected_page() == Some(page.id) {
                                            painter.rect_stroke(
                                                rect,
                                                1,
                                                egui::Stroke::new(2.0_f32, theme::ACCENT),
                                                egui::StrokeKind::Inside,
                                            );
                                        }
                                        if response.clicked() {
                                            selection = Some(page.id);
                                        }
                                        if response.double_clicked() {
                                            open = Some(page.id);
                                        }
                                        ui.add_sized(
                                            [width, 17.0],
                                            egui::Label::new(
                                                egui::RichText::new(format!(
                                                    "{} · {}",
                                                    page.number, page.name
                                                ))
                                                .small(),
                                            )
                                            .truncate(),
                                        );
                                    });
                                }
                            });
                        }
                    });
            });
        if let Some(id) = selection {
            self.session.select(Selection::Page(id));
        }
        if let Some(id) = open {
            self.open_page(id);
        }
    }
}
