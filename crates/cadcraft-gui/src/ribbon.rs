// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Ribbon tabs and command buttons; start with the ribbon renderer.
use crate::{
    app::CadApp,
    commands::{CommandId, Icon},
    icons,
    preferences::{PanelId, RibbonTab},
    theme,
};
use eframe::egui::{self, Align, Context, Layout, RichText, ScrollArea};
impl CadApp {
    pub(crate) fn top_bar(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("quick_access")
            .frame(egui::Frame::new().fill(theme::TITLE).inner_margin(4))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    icons::logo(ui, 22.0);
                    ui.strong("CAD-CRAFT");
                    ui.separator();
                    for id in [
                        CommandId::NewProject,
                        CommandId::OpenProject,
                        CommandId::SaveProject,
                        CommandId::Undo,
                        CommandId::Redo,
                        CommandId::Repeat,
                        CommandId::Settings,
                    ] {
                        if self.command_button(ui, id, false, true) {
                            self.execute(id, ctx);
                        }
                    }
                    ui.separator();
                    let name = self
                        .session
                        .project()
                        .map(|p| p.name().as_str())
                        .unwrap_or("Nenhum projeto aberto");
                    ui.add(
                        egui::Label::new(RichText::new(name).small().color(theme::MUTED))
                            .truncate(),
                    );
                    if self.session.dirty {
                        ui.label(
                            RichText::new("• Alterações em memória")
                                .small()
                                .color(theme::WARNING),
                        );
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let response = ui.add_sized(
                            [180.0, 23.0],
                            egui::TextEdit::singleline(&mut self.chrome.command_query)
                                .id(egui::Id::new("command_search"))
                                .hint_text("Procurar comando"),
                        );
                        if !self.chrome.command_query.is_empty() {
                            response.on_hover_text(
                                "Busca por nome, palavras relacionadas, guia e grupo",
                            );
                        }
                    });
                });
            });
        if !self.chrome.command_query.is_empty() {
            let mut action = None;
            egui::Window::new("Resultados de comandos")
                .id(egui::Id::new("command_results"))
                .collapsible(false)
                .resizable(true)
                .default_width(580.0)
                .show(ctx, |ui| {
                    ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                        let mut found = false;
                        for c in self
                            .chrome
                            .commands
                            .iter()
                            .filter(|c| c.matches(&self.chrome.command_query))
                        {
                            found = true;
                            let label = format!("{}   /   {} · {}", c.name, c.tab.label(), c.group);
                            let tip = format!(
                                "{}\n{}",
                                c.description,
                                c.reason(&self.session).unwrap_or("Disponível")
                            );
                            if ui
                                .add_enabled(c.available(&self.session), egui::Button::new(label))
                                .on_hover_text(&tip)
                                .on_disabled_hover_text(&tip)
                                .clicked()
                            {
                                action = Some(c.id);
                            }
                        }
                        if !found {
                            ui.label("Nenhum comando encontrado.");
                        }
                    });
                    if ui.button("Fechar busca").clicked() {
                        self.chrome.command_query.clear();
                    }
                });
            if let Some(id) = action {
                self.chrome.command_query.clear();
                self.execute(id, ctx);
            }
        }
    }
    pub(crate) fn show_ribbon(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("ribbon")
            // Do not reuse the previous frame's collapsed height as a clip rectangle.
            .exact_height(if self.preferences.ribbon_collapsed {
                31.0
            } else {
                137.0
            })
            .frame(egui::Frame::new().fill(theme::RIBBON).inner_margin(3))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add(egui::Button::new("Arquivo").fill(theme::SELECTION))
                        .clicked()
                    {
                        self.dialogs_state.backstage = Some("Informações");
                    }
                    let tabs_width = (ui.available_width()
                        - if self.preferences.ribbon_collapsed {
                            90.0
                        } else {
                            32.0
                        })
                    .max(50.0);
                    ui.allocate_ui_with_layout(
                        egui::vec2(tabs_width, 25.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ScrollArea::horizontal()
                                .id_salt("ribbon_tabs")
                                .auto_shrink([false, true])
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        for tab in RibbonTab::ALL {
                                            let active = tab == self.preferences.ribbon_tab;
                                            let response = ui.selectable_label(active, tab.label());
                                            if response.clicked() {
                                                self.preferences.ribbon_tab = tab;
                                            }
                                            // egui counts rapid clicks across widgets as a double click.
                                            // Switching tabs must never change the explicit collapsed state.
                                            if active && response.double_clicked() {
                                                self.preferences.ribbon_collapsed =
                                                    !self.preferences.ribbon_collapsed;
                                            }
                                            if active {
                                                ui.painter().line_segment(
                                                    [
                                                        response.rect.left_bottom(),
                                                        response.rect.right_bottom(),
                                                    ],
                                                    egui::Stroke::new(2.0_f32, theme::ACCENT),
                                                );
                                            }
                                        }
                                    });
                                });
                        },
                    );
                    if icons::button(
                        ui,
                        Icon::Collapse,
                        if self.preferences.ribbon_collapsed {
                            "Expandir"
                        } else {
                            ""
                        },
                        if self.preferences.ribbon_collapsed {
                            "Ribbon recolhida — expandir com Ctrl+F1"
                        } else {
                            "Recolher Ribbon — Ctrl+F1"
                        },
                        true,
                        false,
                        false,
                    )
                    .clicked()
                    {
                        self.preferences.ribbon_collapsed = !self.preferences.ribbon_collapsed;
                    }
                });
                if !self.preferences.ribbon_collapsed {
                    let mut selected = None;
                    ScrollArea::horizontal()
                        .id_salt(("ribbon_content", self.preferences.ribbon_tab))
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            ui.horizontal_top(|ui| {
                                let mut groups: Vec<&str> = Vec::new();
                                for c in self
                                    .chrome
                                    .commands
                                    .iter()
                                    .filter(|c| c.tab == self.preferences.ribbon_tab)
                                {
                                    if !groups.contains(&c.group) {
                                        groups.push(c.group);
                                    }
                                }
                                // Match reference emphasis: page and editing groups before session management.
                                if self.preferences.ribbon_tab == RibbonTab::Home {
                                    groups.sort_by_key(|g| match *g {
                                        "Área de transferência" => 0,
                                        "Página" => 1,
                                        "Espaço do layout 3D" => 2,
                                        "Visualização gráfica" => 3,
                                        "Texto" => 4,
                                        "Editar" => 5,
                                        "Procurar" => 6,
                                        _ => 7,
                                    });
                                }
                                for group in groups {
                                    let commands: Vec<_> = self
                                        .chrome
                                        .commands
                                        .iter()
                                        .filter(|c| {
                                            c.tab == self.preferences.ribbon_tab && c.group == group
                                        })
                                        .collect();
                                    ui.vertical(|ui| {
                                        ui.set_min_height(88.0);
                                        ui.horizontal_top(|ui| {
                                            if let Some(first) = commands.first()
                                                && self.command_button(ui, first.id, true, false)
                                            {
                                                selected = Some(first.id);
                                            }
                                            for column in commands
                                                .iter()
                                                .skip(1)
                                                .collect::<Vec<_>>()
                                                .chunks(3)
                                            {
                                                ui.vertical(|ui| {
                                                    for c in column {
                                                        if self
                                                            .command_button(ui, c.id, false, false)
                                                        {
                                                            selected = Some(c.id);
                                                        }
                                                    }
                                                });
                                            }
                                        });
                                        ui.with_layout(
                                            Layout::left_to_right(Align::Center),
                                            |ui| {
                                                ui.add(egui::Label::new(
                                                    RichText::new(group)
                                                        .size(10.0)
                                                        .color(theme::MUTED),
                                                ));
                                                icons::menu(
                                                    ui,
                                                    Icon::More,
                                                    "Comandos do grupo",
                                                    |ui| {
                                                        for c in &commands {
                                                            if self.command_button(
                                                                ui, c.id, false, false,
                                                            ) {
                                                                selected = Some(c.id);
                                                                ui.close();
                                                            }
                                                        }
                                                    },
                                                );
                                            },
                                        );
                                    });
                                    ui.separator();
                                }
                            });
                        });
                    if let Some(id) = selected {
                        self.execute(id, ctx);
                    }
                }
            });
    }
    pub(crate) fn global_status(&mut self, ctx: &Context) {
        egui::TopBottomPanel::bottom("global_status")
            .exact_height(25.0)
            .frame(egui::Frame::new().fill(theme::TITLE).inner_margin(3))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(if self.session.demonstration {
                            "DEMONSTRAÇÃO"
                        } else if self.session.project().is_some() {
                            "SESSÃO EM MEMÓRIA"
                        } else {
                            "PRONTO"
                        })
                        .small()
                        .color(theme::ACCENT),
                    );
                    if let Some(message) = self.panels_state.messages.back() {
                        ui.add(
                            egui::Label::new(
                                RichText::new(&message.text).small().color(theme::MUTED),
                            )
                            .truncate(),
                        );
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .selectable_label(self.assistant.open, "✦ Assistente")
                            .on_hover_text("Assistente de IA e ajuda do CAD-CRAFT")
                            .clicked()
                        {
                            self.toggle_assistant();
                        }
                        ui.menu_button("Painéis", |ui| {
                            for id in PanelId::ALL {
                                ui.checkbox(
                                    &mut self.preferences.panel_mut(id).visible,
                                    id.label(),
                                );
                            }
                            if ui.button("Restaurar layout padrão").clicked() {
                                self.preferences.restore_layout();
                                self.chrome.layout_generation += 1;
                                ui.close();
                            }
                        });
                    });
                });
            });
    }
}
