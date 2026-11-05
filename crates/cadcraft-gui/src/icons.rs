// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Small original line icons drawn with egui; no third-party artwork or glyphs.
use crate::{commands::Icon, theme};
use eframe::egui::{
    self, Align2, Color32, FontId, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui, Vec2,
    pos2, vec2,
};
pub fn paint(p: &egui::Painter, rect: Rect, icon: Icon, color: Color32) {
    let r = rect.shrink(rect.width() * 0.15);
    let s = Stroke::new(1.3_f32, color);
    let pt = |x: f32, y: f32| pos2(r.left() + r.width() * x, r.top() + r.height() * y);
    let line = |a: (f32, f32), b: (f32, f32)| {
        p.line_segment([pt(a.0, a.1), pt(b.0, b.1)], s);
    };
    let box_at = |a: (f32, f32), b: (f32, f32)| {
        p.rect_stroke(
            Rect::from_min_max(pt(a.0, a.1), pt(b.0, b.1)),
            0,
            s,
            StrokeKind::Inside,
        );
    };
    match icon {
        Icon::Page => {
            p.add(Shape::line(
                vec![
                    pt(0.2, 1.0),
                    pt(0.2, 0.0),
                    pt(0.65, 0.0),
                    pt(0.9, 0.25),
                    pt(0.9, 1.0),
                    pt(0.2, 1.0),
                ],
                s,
            ));
            line((0.65, 0.0), (0.65, 0.3));
            line((0.65, 0.3), (0.9, 0.3));
            line((0.35, 0.55), (0.75, 0.55));
            line((0.35, 0.75), (0.75, 0.75));
        }
        Icon::Folder => {
            box_at((0.05, 0.25), (0.95, 0.95));
            line((0.05, 0.25), (0.05, 0.05));
            line((0.05, 0.05), (0.4, 0.05));
            line((0.4, 0.05), (0.55, 0.25));
        }
        Icon::Save => {
            box_at((0.05, 0.05), (0.95, 0.95));
            box_at((0.25, 0.05), (0.7, 0.35));
            box_at((0.25, 0.55), (0.75, 0.95));
        }
        Icon::Search | Icon::ZoomIn | Icon::ZoomOut => {
            p.circle_stroke(pt(0.4, 0.4), r.width() * 0.32, s);
            line((0.65, 0.65), (1.0, 1.0));
            if icon != Icon::Search {
                line((0.22, 0.4), (0.58, 0.4));
                if icon == Icon::ZoomIn {
                    line((0.4, 0.22), (0.4, 0.58));
                }
            }
        }
        Icon::Close | Icon::Delete => {
            line((0.2, 0.2), (0.8, 0.8));
            line((0.8, 0.2), (0.2, 0.8));
            if icon == Icon::Delete {
                box_at((0.05, 0.05), (0.95, 0.95));
            }
        }
        Icon::Grid => {
            for i in 0..3 {
                for j in 0..3 {
                    p.circle_filled(
                        pt(0.15 + i as f32 * 0.35, 0.15 + j as f32 * 0.35),
                        1.2,
                        color,
                    );
                }
            }
        }
        Icon::Fit => {
            for (x, y, dx, dy) in [
                (0.0, 0.0, 0.3, 0.3),
                (1.0, 0.0, -0.3, 0.3),
                (1.0, 1.0, -0.3, -0.3),
                (0.0, 1.0, 0.3, -0.3),
            ] {
                line((x, y), (x + dx, y));
                line((x, y), (x, y + dy));
            }
        }
        Icon::Panels => {
            box_at((0.05, 0.05), (0.95, 0.95));
            line((0.33, 0.05), (0.33, 0.95));
            line((0.67, 0.05), (0.67, 0.95));
        }
        Icon::Properties => {
            box_at((0.05, 0.05), (0.95, 0.95));
            for y in [0.3, 0.55, 0.8] {
                line((0.15, y), (0.85, y));
            }
            line((0.45, 0.05), (0.45, 0.95));
        }
        Icon::Device => {
            box_at((0.25, 0.2), (0.75, 0.8));
            for x in [0.35, 0.65] {
                line((x, 0.0), (x, 0.2));
                line((x, 0.8), (x, 1.0));
            }
        }
        Icon::Connection => {
            line((0.1, 0.25), (0.5, 0.25));
            line((0.5, 0.25), (0.5, 0.75));
            line((0.5, 0.75), (0.9, 0.75));
            p.circle_stroke(pt(0.1, 0.25), 2.0, s);
            p.circle_stroke(pt(0.9, 0.75), 2.0, s);
        }
        Icon::Text => {
            line((0.1, 0.1), (0.9, 0.1));
            line((0.5, 0.1), (0.5, 0.9));
            line((0.3, 0.9), (0.7, 0.9));
        }
        Icon::Undo | Icon::ArrowLeft => {
            line((0.9, 0.5), (0.1, 0.5));
            line((0.1, 0.5), (0.4, 0.2));
            line((0.1, 0.5), (0.4, 0.8));
        }
        Icon::Redo | Icon::ArrowRight => {
            line((0.1, 0.5), (0.9, 0.5));
            line((0.9, 0.5), (0.6, 0.2));
            line((0.9, 0.5), (0.6, 0.8));
        }
        Icon::Collapse => {
            line((0.2, 0.6), (0.5, 0.3));
            line((0.5, 0.3), (0.8, 0.6));
        }
        Icon::More => {
            for x in [0.2, 0.5, 0.8] {
                p.circle_filled(pt(x, 0.5), 1.4, color);
            }
        }
        Icon::Check => {
            line((0.1, 0.5), (0.4, 0.8));
            line((0.4, 0.8), (0.9, 0.15));
        }
        Icon::Pin => {
            box_at((0.25, 0.05), (0.75, 0.4));
            line((0.5, 0.4), (0.5, 1.0));
            line((0.15, 0.6), (0.85, 0.6));
        }
        Icon::Snap => {
            p.add(Shape::line(
                vec![
                    pt(0.15, 0.0),
                    pt(0.15, 0.65),
                    pt(0.3, 0.9),
                    pt(0.7, 0.9),
                    pt(0.85, 0.65),
                    pt(0.85, 0.0),
                ],
                Stroke::new(2.0_f32, color),
            ));
        }
        Icon::Home => {
            p.add(Shape::line(
                vec![pt(0.0, 0.4), pt(0.5, 0.0), pt(1.0, 0.4)],
                s,
            ));
            box_at((0.2, 0.4), (0.8, 1.0));
        }
        Icon::Insert => {
            box_at((0.05, 0.05), (0.95, 0.95));
            line((0.5, 0.25), (0.5, 0.75));
            line((0.25, 0.5), (0.75, 0.5));
        }
        Icon::Settings | Icon::Tools => {
            for (y, x) in [(0.2, 0.35), (0.5, 0.7), (0.8, 0.3)] {
                line((0.0, y), (1.0, y));
                p.circle_filled(pt(x, y), 2.5, color);
            }
        }
    }
}
pub fn button(
    ui: &mut Ui,
    icon: Icon,
    label: &str,
    tooltip: &str,
    enabled: bool,
    selected: bool,
    large: bool,
) -> Response {
    let size = if large {
        vec2(78.0, 66.0)
    } else if label.is_empty() {
        vec2(24.0, 24.0)
    } else {
        vec2(
            (ui.fonts_mut(|f| {
                f.layout_no_wrap(label.into(), FontId::proportional(12.0), theme::TEXT)
            })
            .size()
            .x + 30.0)
                .min(220.0),
            23.0,
        )
    };
    ui.add_enabled_ui(enabled, |ui| {
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Button,
                enabled,
                if label.is_empty() { tooltip } else { label },
            )
        });
        if response.hovered() || selected {
            ui.painter().rect_filled(
                rect,
                2,
                if selected {
                    theme::SELECTION
                } else {
                    ui.visuals().widgets.hovered.bg_fill
                },
            );
        }
        let color = if enabled {
            if selected { theme::ACCENT } else { theme::TEXT }
        } else {
            theme::MUTED.gamma_multiply(0.45)
        };
        let icon_rect = if large {
            Rect::from_center_size(pos2(rect.center().x, rect.top() + 20.0), vec2(28.0, 28.0))
        } else {
            Rect::from_min_size(rect.min + vec2(3.0, 3.0), vec2(17.0, 17.0))
        };
        paint(ui.painter(), icon_rect, icon, color);
        if !label.is_empty() {
            if large {
                let galley =
                    ui.painter()
                        .layout(label.into(), FontId::proportional(11.0), color, 74.0);
                ui.painter().galley(
                    pos2(rect.center().x - galley.size().x / 2.0, rect.top() + 38.0),
                    galley,
                    color,
                );
            } else {
                ui.painter().text(
                    rect.left_center() + vec2(25.0, 0.0),
                    Align2::LEFT_CENTER,
                    label,
                    FontId::proportional(12.0),
                    color,
                );
            }
        }
        response
    })
    .inner
    .on_hover_text(tooltip)
    .on_disabled_hover_text(tooltip)
}
pub fn tiny(ui: &mut Ui, icon: Icon, tooltip: &str) -> Response {
    button(ui, icon, "", tooltip, true, false, false)
}
pub fn menu<R>(ui: &mut Ui, icon: Icon, tooltip: &str, contents: impl FnOnce(&mut Ui) -> R) {
    let response = ui.menu_button("   ", contents).response;
    paint(ui.painter(), response.rect.shrink(3.0), icon, theme::TEXT);
    response.on_hover_text(tooltip);
}
pub fn disclosure(ui: &mut Ui, collapsed: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(14.0, 18.0), Sense::click());
    let c = rect.center();
    let points = if collapsed {
        vec![
            c + vec2(-2.0, -4.0),
            c + vec2(2.0, 0.0),
            c + vec2(-2.0, 4.0),
        ]
    } else {
        vec![
            c + vec2(-4.0, -2.0),
            c + vec2(0.0, 2.0),
            c + vec2(4.0, -2.0),
        ]
    };
    ui.painter()
        .add(Shape::line(points, Stroke::new(1.2_f32, theme::MUTED)));
    response.on_hover_text(if collapsed {
        "Expandir grupo"
    } else {
        "Recolher grupo"
    })
}
pub fn logo(ui: &mut Ui, size: f32) {
    let (r, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let p = ui.painter();
    p.rect_filled(r, 3, theme::SELECTION);
    let s = Stroke::new(1.8_f32, theme::ACCENT);
    p.add(Shape::line(
        vec![
            r.min + vec2(size * 0.7, size * 0.22),
            r.min + vec2(size * 0.3, size * 0.22),
            r.min + vec2(size * 0.22, size * 0.5),
            r.min + vec2(size * 0.3, size * 0.78),
            r.min + vec2(size * 0.7, size * 0.78),
        ],
        s,
    ));
}
pub fn page_icon(ui: &mut Ui, kind: &cad_craft::domain::PageKind) -> Response {
    use cad_craft::domain::PageKind as K;
    let (icon, color) = match kind {
        K::MultiLine => (Icon::Connection, theme::WARNING),
        K::SingleLine => (Icon::Connection, theme::SUCCESS),
        K::Overview => (Icon::Panels, theme::ACCENT),
        K::PanelLayout => (Icon::Device, theme::ACCENT),
        K::TechnicalDocumentation => (Icon::Page, theme::TEXT),
        K::Other(_) => (Icon::Folder, theme::MUTED),
    };
    let (r, response) = ui.allocate_exact_size(vec2(16.0, 18.0), Sense::hover());
    paint(ui.painter(), r, icon, color);
    response
}
