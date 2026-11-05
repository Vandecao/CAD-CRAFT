// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use eframe::egui::{self, Color32, FontId, Stroke, TextStyle};
pub const TITLE: Color32 = Color32::from_rgb(17, 18, 20);
pub const PANEL: Color32 = Color32::from_rgb(43, 45, 48);
pub const RIBBON: Color32 = Color32::from_rgb(52, 54, 57);
pub const CANVAS: Color32 = Color32::from_rgb(225, 228, 231);
pub const PAPER: Color32 = Color32::WHITE;
pub const INK: Color32 = Color32::from_rgb(38, 48, 57);
pub const GRID: Color32 = Color32::from_rgb(202, 209, 216);
pub const GRID_DOT: Color32 = Color32::from_rgb(139, 149, 160);
pub const LINE: Color32 = Color32::from_rgb(77, 80, 84);
pub const TEXT: Color32 = Color32::from_rgb(226, 230, 234);
pub const MUTED: Color32 = Color32::from_rgb(160, 168, 176);
pub const ACCENT: Color32 = Color32::from_rgb(38, 161, 228);
pub const SELECTION: Color32 = Color32::from_rgb(23, 81, 117);
pub const WARNING: Color32 = Color32::from_rgb(239, 185, 75);
pub const ERROR: Color32 = Color32::from_rgb(239, 108, 106);
pub const SUCCESS: Color32 = Color32::from_rgb(105, 198, 151);
pub fn apply(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::Dark);
    request_native_dark(ctx);
    // Styling egui's client area does not request dark native decorations.
    // Keep the safe backend API and the OS-provided caption/window controls.
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = PANEL;
    style.visuals.window_fill = PANEL;
    style.visuals.extreme_bg_color = TITLE;
    style.visuals.override_text_color = Some(TEXT);
    style.visuals.selection.bg_fill = SELECTION;
    style.visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT);
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, LINE);
    style.visuals.widgets.inactive.bg_fill = RIBBON;
    style.visuals.widgets.inactive.corner_radius = 2.into();
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(69, 75, 81);
    style.visuals.widgets.hovered.corner_radius = 2.into();
    style.visuals.widgets.active.bg_fill = SELECTION;
    style.visuals.widgets.active.corner_radius = 2.into();
    style.visuals.window_corner_radius = 4.into();
    style.spacing.item_spacing = egui::vec2(5.0, 4.0);
    style.spacing.button_padding = egui::vec2(5.0, 3.0);
    style.spacing.interact_size = egui::vec2(24.0, 22.0);
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(12.0));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(12.0));
    style
        .text_styles
        .insert(TextStyle::Small, FontId::proportional(10.0));
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::proportional(18.0));
    style
        .text_styles
        .insert(TextStyle::Monospace, FontId::monospace(11.0));
    ctx.set_style(style);
}

pub(crate) fn request_native_dark(_ctx: &egui::Context) {
    #[cfg(target_os = "windows")]
    _ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Dark));
}

/// Scope to the light welcome canvas, not to other CAD panels. Labels use ink;
/// interactive widgets use light foregrounds on their dark state backgrounds.
pub fn welcome(ui: &mut egui::Ui) {
    let v = ui.visuals_mut();
    v.override_text_color = None;
    // The global 50% fade becomes illegible when composited onto the light canvas.
    v.disabled_alpha = 0.85;
    v.weak_text_color = Some(INK);
    v.widgets.noninteractive.fg_stroke.color = INK;
    v.widgets.inactive.fg_stroke.color = TEXT;
    v.widgets.hovered.fg_stroke.color = Color32::WHITE;
    v.widgets.active.fg_stroke.color = Color32::WHITE;
    v.widgets.inactive.weak_bg_fill = RIBBON;
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(69, 75, 81);
    v.widgets.active.weak_bg_fill = SELECTION;
}

/// Fonte reserva do sistema para símbolos que as fontes padrão do egui não
/// têm (→, Ω, │, ☐…). Encontrada no teste no executável da Etapa 22: sem ela
/// esses caracteres apareciam como quadrados. Sem o arquivo, nada muda.
pub fn install_symbol_font(ctx: &egui::Context) {
    let windows = std::env::var_os("WINDIR").map(|dir| {
        std::path::PathBuf::from(dir)
            .join("Fonts")
            .join("seguisym.ttf")
    });
    let candidates = windows.into_iter().chain([
        std::path::PathBuf::from("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"),
        std::path::PathBuf::from("/System/Library/Fonts/Apple Symbols.ttf"),
    ]);
    let Some(bytes) = candidates.into_iter().find_map(|p| std::fs::read(p).ok()) else {
        return;
    };
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "cad-craft-simbolos".into(),
        std::sync::Arc::new(egui::FontData::from_owned(bytes)),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .push("cad-craft-simbolos".into());
    }
    ctx.set_fonts(fonts);
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Event, Pos2, Rect, Shape, Vec2};
    fn luminance(c: Color32) -> f32 {
        let f = |v: u8| {
            let v = v as f32 / 255.0;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * f(c.r()) + 0.7152 * f(c.g()) + 0.0722 * f(c.b())
    }
    fn over(c: Color32, bg: Color32) -> Color32 {
        let remaining = 1.0 - c.a() as f32 / 255.0;
        Color32::from_rgb(
            (c.r() as f32 + bg.r() as f32 * remaining).min(255.0) as u8,
            (c.g() as f32 + bg.g() as f32 * remaining).min(255.0) as u8,
            (c.b() as f32 + bg.b() as f32 * remaining).min(255.0) as u8,
        )
    }
    #[test]
    fn welcome_button_states_and_auxiliary_labels_are_readable_without_global_style_leak() {
        for state in ["normal", "hover", "pressed", "focus", "disabled"] {
            let ctx = egui::Context::default();
            apply(&ctx);
            let original = ctx.style().visuals.clone();
            let mut rect = Rect::NOTHING;
            let mut output = egui::FullOutput::default();
            for frame in 0..4 {
                let mut events = vec![];
                if frame > 0 && ["hover", "pressed"].contains(&state) {
                    events.push(Event::PointerMoved(rect.center()));
                }
                if frame == 1 && state == "pressed" {
                    events.push(Event::PointerButton {
                        pos: rect.center(),
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
                output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 400.0))),
                        events,
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default()
                            .frame(egui::Frame::new().fill(CANVAS))
                            .show(ctx, |ui| {
                                ui.scope(|ui| {
                                    welcome(ui);
                                    ui.label("Instruções");
                                    let response = ui.add_enabled(
                                        state != "disabled",
                                        egui::Button::new("Ação"),
                                    );
                                    rect = response.rect;
                                    if state == "focus" {
                                        response.request_focus();
                                    }
                                });
                            });
                    },
                );
            }
            assert_eq!(ctx.style().visuals, original);
            for label in ["Ação", "Instruções"] {
                let text = output
                    .shapes
                    .iter()
                    .find_map(|s| match &s.shape {
                        Shape::Text(t) if t.galley.text() == label => Some(t),
                        _ => None,
                    })
                    .unwrap();
                let center = text.pos + text.galley.size() * 0.5;
                let fill = output
                    .shapes
                    .iter()
                    .rev()
                    .find_map(|s| match &s.shape {
                        Shape::Rect(r) if r.rect.contains(center) => Some(r.fill),
                        _ => None,
                    })
                    .unwrap();
                let section = text.galley.job.sections[0].format.color;
                let color =
                    text.override_text_color
                        .unwrap_or(if section == Color32::PLACEHOLDER {
                            text.fallback_color
                        } else {
                            section
                        });
                let fill = over(fill, CANVAS);
                let color = over(color.gamma_multiply(text.opacity_factor), fill);
                let ratio = (luminance(color).max(luminance(fill)) + 0.05)
                    / (luminance(color).min(luminance(fill)) + 0.05);
                assert!(
                    ratio >= 4.5,
                    "{state}/{label}: contrast {ratio}, {color:?}/{fill:?}"
                );
            }
        }
    }
}
