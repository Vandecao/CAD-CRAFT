// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Sheet rendering and pointer interaction; start with draw_page and the viewport renderer.
use crate::{
    app::{CadApp, Level},
    camera::{Camera, SCALE_100, grid_points, snap},
    commands::{CommandId, Icon},
    icons,
    state::Selection,
    theme,
};
use cad_craft::display::{DisplayColor, DisplayItem, DisplayLayer};
use cad_craft::domain::*;
use eframe::egui::{self, Align2, FontId, PointerButton, Rect, Sense, Stroke, Ui, vec2};
impl CadApp {
    pub(crate) fn camera_command(&mut self, id: CommandId) {
        let Some(view) = self.last_viewport else {
            return;
        };
        let Some(page) = self.session.active else {
            return;
        };
        let bounds = self
            .session
            .project()
            .and_then(|p| p.document_bounds(page).ok());
        let Some(tab) = self.session.active_tab_mut() else {
            return;
        };
        let old = tab.camera;
        let mut ok = true;
        match id {
            CommandId::FitPage | CommandId::FitContent => {
                if let Some(bounds) = bounds {
                    let target = if id == CommandId::FitContent {
                        bounds.content.unwrap_or(bounds.sheet)
                    } else {
                        bounds.sheet
                    };
                    ok = tab.camera.fit(target, view);
                    tab.needs_fit = false;
                } else {
                    ok = false;
                }
            }
            CommandId::ZoomIn => {
                ok = tab.camera.zoom_at(1.25, view.center(), view);
            }
            CommandId::ZoomOut => {
                ok = tab.camera.zoom_at(0.8, view.center(), view);
            }
            CommandId::Zoom100 => {
                ok = tab
                    .camera
                    .zoom_at(SCALE_100 / tab.camera.scale(), view.center(), view);
            }
            CommandId::PreviousView => {
                if let Some(previous) = tab.previous {
                    tab.camera = previous;
                }
            }
            _ => {}
        }
        if ok {
            tab.previous = Some(old);
        } else {
            self.log(
                Level::Warning,
                "Não foi possível enquadrar estes limites na faixa numérica da câmera.",
            );
        }
    }
    pub(crate) fn show_documents(&mut self, ui: &mut Ui) {
        let mut open = None;
        let mut close = None;
        egui::ScrollArea::horizontal()
            .id_salt("document_tabs")
            .max_height(29.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for tab in &self.session.tabs {
                        if let Some(page) = self.session.project().and_then(|p| p.page(tab.page)) {
                            let active = self.session.active == Some(tab.page);
                            let response = ui.selectable_label(
                                active,
                                format!("{} · {}", page.number().as_str(), page.name().as_str()),
                            );
                            if active {
                                ui.painter().line_segment(
                                    [response.rect.left_top(), response.rect.right_top()],
                                    Stroke::new(2.0_f32, theme::ACCENT),
                                );
                            }
                            if response.clicked() {
                                open = Some(tab.page);
                            }
                            if icons::tiny(
                                ui,
                                Icon::Close,
                                "Fechar aba — mantém a página no projeto",
                            )
                            .clicked()
                            {
                                close = Some(tab.page);
                            }
                        }
                    }
                    if self.session.tabs.is_empty() {
                        ui.label(
                            egui::RichText::new("Área de trabalho")
                                .small()
                                .color(theme::MUTED),
                        );
                    }
                });
            });
        if let Some(id) = open {
            self.open_page(id);
        }
        if let Some(id) = close {
            self.session.close_tab(id);
        }
        egui::TopBottomPanel::bottom("viewport_status")
            .frame(egui::Frame::new().fill(theme::RIBBON).inner_margin(3))
            .show_inside(ui, |ui| self.viewport_status(ui));
        let rect = ui.available_rect_before_wrap();
        self.last_viewport = Some(rect);
        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0, theme::CANVAS);
        if let Some(page_id) = self.session.active {
            let Some(tab_index) = self.session.tabs.iter().position(|t| t.page == page_id) else {
                return;
            };
            if self.session.tabs[tab_index].needs_fit {
                if let Some(sheet) = self
                    .session
                    .project()
                    .and_then(|p| p.document_bounds(page_id).ok())
                    .map(|b| b.sheet)
                {
                    self.session.tabs[tab_index].camera.fit(sheet, rect);
                }
                self.session.tabs[tab_index].needs_fit = false;
            }
            let tab = &mut self.session.tabs[tab_index];
            if response.hovered()
                && self.dialogs_state.dialog.is_none()
                && let Some(cursor) = response.hover_pos()
            {
                let scroll = ui.ctx().input(|i| i.smooth_scroll_delta.y);
                if scroll != 0.0 {
                    let old = tab.camera;
                    if tab.camera.zoom_at(
                        (scroll as f64 * 0.004).clamp(-2.0, 2.0).exp(),
                        cursor,
                        rect,
                    ) {
                        tab.previous = Some(old);
                    }
                    ui.ctx()
                        .input_mut(|i| i.smooth_scroll_delta = egui::Vec2::ZERO);
                }
                self.chrome.cursor_world = tab.camera.screen_to_world(cursor, rect);
            } else {
                self.chrome.cursor_world = None;
            }
            if response.drag_started_by(PointerButton::Middle)
                && self.dialogs_state.dialog.is_none()
            {
                tab.previous = Some(tab.camera);
            }
            if response.dragged_by(PointerButton::Middle) && self.dialogs_state.dialog.is_none() {
                tab.camera.pan(ui.ctx().input(|i| i.pointer.delta()));
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            } else if response.hovered()
                && ui
                    .ctx()
                    .input(|i| i.pointer.button_down(PointerButton::Middle))
            {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
            }
            let camera = tab.camera;
            self.interact_editor(ui, &response, page_id, camera, rect);
            if let Some(project) = self.session.project() {
                draw_editor_page(
                    &painter,
                    project,
                    page_id,
                    camera,
                    rect,
                    &self.session.graphics,
                    self.preferences
                        .grid_visible
                        .then_some(self.preferences.grid_mm),
                );
            }
            self.paint_selection_window(&painter, camera, rect);
            if let Some(draft) = &self.session.graphics.insertion
                && draft.positioned
                && let Some(d) = self.session.find_symbol(&draft.definition)
            {
                let d = &d;
                crate::symbol_view::paint_symbol(
                    &painter,
                    d,
                    draft.transform,
                    camera,
                    rect,
                    Stroke::new(1.5_f32, theme::ACCENT.linear_multiply(0.6)),
                );
                paint_symbol_ports(&painter, d, draft.transform, camera, rect, theme::ACCENT);
                painter.text(
                    rect.left_top() + vec2(10.0, 28.0),
                    Align2::LEFT_TOP,
                    "Inserir · clique para repetir · Esc cancela",
                    FontId::proportional(12.0),
                    theme::ACCENT,
                );
            }
            if self.preferences.snap_enabled
                && let Some(world) = self.chrome.cursor_world
            {
                let snapped = snap(world, self.preferences.grid_mm, true);
                if let Some(screen) = camera.world_to_screen(snapped, rect) {
                    painter.circle_stroke(screen, 4.0, Stroke::new(1.0_f32, theme::ACCENT));
                }
            }
        } else {
            ui.scope_builder(egui::UiBuilder::new().max_rect(rect.shrink(20.0)), |ui| {
                theme::welcome(ui);
                egui::ScrollArea::vertical()
                    .id_salt("welcome_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(((rect.height() - 250.0) * 0.3).max(4.0));
                            icons::logo(ui, 42.0);
                            ui.heading("CAD-CRAFT");
                            ui.label(if self.session.project().is_some() {
                        "Selecione uma página no navegador e abra com duplo clique ou Enter."
                    } else {
                        "Engenharia elétrica · Ambiente documental"
                    });
                            ui.add_space(12.0);
                            for (id, label) in [
                                (CommandId::NewProject, "Criar sessão em memória"),
                                (CommandId::OpenProject, "Abrir projeto .cadcraft"),
                                (CommandId::Demo, "Abrir demonstração CAD-CRAFT"),
                                (CommandId::OpenSnapshot, "Importar snapshot técnico"),
                            ] {
                                if ui.button(label).clicked() {
                                    self.execute(id, ui.ctx());
                                }
                            }
                            ui.add_space(12.0);
                            ui.small(
                                "Roda: zoom no cursor  ·  Botão central: pan  ·  Ctrl+F1: Ribbon",
                            );
                        });
                    });
            });
        }
    }
    fn viewport_status(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        ui.horizontal_wrapped(|ui| {
            let coordinates = self
                .chrome
                .cursor_world
                .map(|p| format!("X: {:.2} mm  |  Y: {:.2} mm", p.x(), p.y()))
                .unwrap_or_else(|| "X: — mm  |  Y: — mm".into());
            ui.label(egui::RichText::new(coordinates).small());
            icons::button(
                ui,
                Icon::Tools,
                "",
                "Modo de construção — ferramenta futura",
                false,
                false,
                false,
            );
            icons::button(
                ui,
                Icon::Pin,
                "",
                "Fixar objeto — bloqueio de edição futuro",
                false,
                false,
                false,
            );
            icons::button(
                ui,
                Icon::Connection,
                "",
                "Visualização lógica — camada futura",
                false,
                false,
                false,
            );
            if icons::button(
                ui,
                Icon::Snap,
                "",
                "Snap: ajusta a âncora do arraste em milímetros, mesmo com a grade oculta",
                true,
                self.preferences.snap_enabled,
                false,
            )
            .clicked()
            {
                self.execute(CommandId::ToggleSnap, &ctx);
            }
            if icons::button(
                ui,
                Icon::Grid,
                "",
                "Mostrar/ocultar grade",
                true,
                self.preferences.grid_visible,
                false,
            )
            .clicked()
            {
                self.execute(CommandId::ToggleGrid, &ctx);
            }
            egui::ComboBox::from_id_salt("grid_spacing")
                .selected_text(format!("{:.1} mm", self.preferences.grid_mm))
                .width(70.0)
                .show_ui(ui, |ui| {
                    for value in [1.0, 2.0, 2.5, 4.0, 8.0, 16.0] {
                        ui.selectable_value(
                            &mut self.preferences.grid_mm,
                            value,
                            format!("{value:.1} mm"),
                        );
                    }
                })
                .response
                .on_hover_text("Espaçamento real da grade em milímetros");
            if let Some(tab) = self.session.active_tab_mut() {
                ui.label(egui::RichText::new(format!("Zoom {:.1}%", tab.camera.percent())).small())
                    .on_hover_text(
                        "Zoom de tela; não altera escala documental nem dimensões em mm",
                    );
            }
            for id in [
                CommandId::ZoomOut,
                CommandId::ZoomIn,
                CommandId::Zoom100,
                CommandId::FitPage,
                CommandId::FitContent,
                CommandId::PreviousView,
            ] {
                if self.command_button(ui, id, false, true) {
                    self.execute(id, &ctx);
                }
            }
        });
    }
}

pub(crate) fn draw_page(
    painter: &egui::Painter,
    project: &Project,
    page_id: PageId,
    camera: Camera,
    viewport: Rect,
    selection: Selection,
    grid: Option<f64>,
) {
    let mut editor = crate::interaction::Interaction::default();
    if let Selection::Representation(id) = selection {
        editor.selection.replace([id]);
    }
    draw_editor_page(painter, project, page_id, camera, viewport, &editor, grid);
}
fn draw_editor_page(
    painter: &egui::Painter,
    project: &Project,
    page_id: PageId,
    camera: Camera,
    viewport: Rect,
    editor: &crate::interaction::Interaction,
    grid: Option<f64>,
) {
    let Some(page) = project.page(page_id) else {
        return;
    };
    let origin = Point2D::new(0.0, 0.0).expect("constant");
    let upper = Point2D::new(page.size().width(), page.size().height()).expect("validated size");
    let sheet = Rect2D::new(origin, page.size()).expect("valid page");
    if let Some(visible) = camera.clipped_rect(sheet, viewport) {
        painter.rect_filled(visible, 0, theme::PAPER);
        draw_bounds(
            painter,
            camera,
            sheet,
            viewport,
            Stroke::new(1.0_f32, theme::INK),
        );
        let frame = project.page_frame(page_id);
        let margin = 10.0_f64
            .min(page.size().width() * 0.1)
            .min(page.size().height() * 0.1);
        if let Some(frame) = frame {
            // Etapa 13: a moldura substitui a margem e o cabeçalho provisórios.
            crate::frame_view::paint_frame(painter, project, page_id, frame, camera, viewport);
        } else if let Ok(inner) = Rect2D::new(
            Point2D::new(margin, margin).expect("finite margin"),
            Size2D::new(
                page.size().width() - 2.0 * margin,
                page.size().height() - 2.0 * margin,
            )
            .expect("nonnegative margin"),
        ) {
            draw_bounds(
                painter,
                camera,
                inner,
                viewport,
                Stroke::new(0.6_f32, theme::GRID),
            );
        }
        if let Some(top_left) =
            camera.world_to_screen(Point2D::new(0.0, upper.y()).expect("finite"), viewport)
            && frame.is_none()
            && visible.width() > 180.0
            && visible.height() > 80.0
        {
            painter.text(
                top_left + vec2(12.0, 8.0),
                Align2::LEFT_TOP,
                format!(
                    "{} · {}  |  {} × {} mm",
                    page.number().as_str(),
                    page.name().as_str(),
                    page.size().width(),
                    page.size().height()
                ),
                FontId::proportional(11.0),
                theme::INK,
            );
        }
    }
    if page.report().is_some() {
        paint_report_page(painter, project, page_id, camera, viewport);
        return;
    }
    if let Some(spacing) = grid {
        for point in grid_points(&camera, viewport, spacing) {
            painter.circle_filled(point, 0.8, theme::GRID_DOT);
        }
    }
    let mut reps = project.representations_on_page(page_id).unwrap_or_default();
    reps.sort_by_key(|r| {
        (
            r.visual().layer,
            u8::from(project.connection_route(r.id()).is_none()),
            r.id(),
        )
    });
    for rep in reps {
        if !rep.visual().visible {
            continue;
        }
        let selected = editor.selection.ids().contains(&rep.id());
        let hovered = editor.hover == Some(rep.id());
        let transform = editor.preview(rep.id()).unwrap_or_else(|| rep.transform());
        let color = if selected {
            theme::ACCENT
        } else if hovered {
            egui::Color32::from_rgb(30, 150, 180)
        } else {
            theme::INK
        };
        if let Some(route) = project.connection_route(rep.id()) {
            let points = crate::connection_view::display_points(project, route, editor);
            let color = if editor.highlighted_routes.contains(&rep.id()) {
                egui::Color32::from_rgb(220, 120, 20)
            } else {
                color
            };
            crate::connection_view::paint_route(
                painter,
                camera,
                viewport,
                &points,
                Stroke::new(
                    if selected || hovered {
                        2.5_f32
                    } else {
                        1.5_f32
                    },
                    color,
                ),
            );
            // Etapa 22: mesma posição do PDF/DXF, em tamanho de folha (mm).
            if let Some(number) = project
                .connection(route.connection_id())
                .and_then(Connection::number)
                && let Some((origin, align)) = cad_craft::display::wire_number_placement(
                    &points,
                    route.number_label_position(),
                )
            {
                crate::frame_view::paint_text(
                    painter,
                    camera,
                    viewport,
                    origin,
                    number.as_str(),
                    FrameTextStyle::new(cad_craft::display::LABEL_HEIGHT, align),
                    color,
                );
            }
            if selected {
                for point in points.iter().skip(1).take(points.len().saturating_sub(2)) {
                    if let Some(pos) = camera.world_to_screen(*point, viewport)
                        && viewport.contains(pos)
                    {
                        painter.rect_filled(
                            Rect::from_center_size(pos, vec2(6., 6.)),
                            0,
                            theme::ACCENT,
                        );
                    }
                }
            }
            continue;
        }
        let anchor = camera.world_to_screen(transform.translation(), viewport);
        if let Some(local) = rep.local_bounds()
            && let Ok(bounds) = transform.transform_bounds(local)
            && camera.clipped_rect(bounds, viewport.expand(12.0)).is_none()
            && !anchor.is_some_and(|p| viewport.expand(12.0).contains(p))
        {
            continue;
        }
        if let Some(graphic) = rep.graphic() {
            if let Ok(primitive) = graphic.drawable() {
                crate::symbol_view::paint_styled(
                    painter,
                    &primitive,
                    graphic.style(),
                    transform,
                    camera,
                    viewport,
                    Stroke::new((graphic.stroke_mm() * camera.scale()) as f32, color),
                    &crate::symbol_view::placeholder,
                );
            }
        } else if let Some(d) = rep.symbol().and_then(|id| project.symbol_definition(id)) {
            let rid = rep.id();
            let texts = |field: &SymbolTextField| {
                project
                    .symbol_text(rid, field)
                    .unwrap_or_else(|| field.placeholder())
            };
            crate::symbol_view::paint_symbol_with(
                painter,
                d,
                transform,
                camera,
                viewport,
                Stroke::new(
                    if selected || hovered {
                        2.0_f32
                    } else {
                        1.0_f32
                    },
                    color,
                ),
                &texts,
            );
            if selected || hovered {
                paint_symbol_ports(painter, d, transform, camera, viewport, color);
            }
        } else if let Some(points) = crate::hit_test::corners(rep, transform) {
            for i in 0..4 {
                if let Some(line) = camera.clipped_segment(points[i], points[(i + 1) % 4], viewport)
                {
                    painter.line_segment(
                        line,
                        Stroke::new(
                            if selected || hovered {
                                2.0_f32
                            } else {
                                1.0_f32
                            },
                            color,
                        ),
                    );
                }
            }
        }
        if let Some(anchor) = anchor
            && viewport.expand(8.0).contains(anchor)
        {
            painter.circle_stroke(anchor, 3.0, Stroke::new(1.0_f32, color));
            if selected {
                painter.rect_stroke(
                    Rect::from_center_size(anchor, vec2(8.0, 8.0)),
                    0,
                    Stroke::new(1.0_f32, theme::ACCENT),
                    egui::StrokeKind::Outside,
                );
            }
            if let Some(label) = &rep.visual().label
                && viewport.width() >= 200.0
            {
                painter.text(
                    anchor + vec2(6.0, 6.0),
                    Align2::LEFT_TOP,
                    label.as_str(),
                    FontId::proportional(11.0),
                    color,
                );
            }
        }
    }
    crate::frame_view::paint_cross_references(painter, project, page_id, camera, viewport);
    crate::connection_view::paint_overlay(painter, project, page_id, camera, viewport, editor);
}

fn paint_report_page(
    painter: &egui::Painter,
    project: &Project,
    page: PageId,
    camera: Camera,
    viewport: Rect,
) {
    let Ok(list) = project.page_display_list(page) else {
        return;
    };
    for item in &list.items {
        match item {
            DisplayItem::Text(t) if t.layer == DisplayLayer::Texts => {
                if let Ok(origin) = Point2D::new(t.matrix[4], t.matrix[5])
                    && let Some(pos) = camera.world_to_screen(origin, viewport)
                {
                    painter.text(
                        pos,
                        Align2::LEFT_BOTTOM,
                        &t.text,
                        FontId::proportional((t.size * camera.scale()) as f32),
                        if t.color == DisplayColor::Reference {
                            theme::ACCENT
                        } else {
                            theme::INK
                        },
                    );
                }
            }
            DisplayItem::Path {
                points,
                closed,
                stroke: Some(stroke),
                layer: DisplayLayer::Texts,
                ..
            } => {
                let color = theme::INK;
                let line = Stroke::new((stroke.width * camera.scale()).max(0.5) as f32, color);
                for pair in points.windows(2) {
                    if let (Some(a), Some(b)) = (
                        camera.world_to_screen(pair[0], viewport),
                        camera.world_to_screen(pair[1], viewport),
                    ) {
                        painter.line_segment([a, b], line);
                    }
                }
                if *closed
                    && points.len() > 2
                    && let (Some(a), Some(b)) = (
                        camera.world_to_screen(*points.last().unwrap(), viewport),
                        camera.world_to_screen(points[0], viewport),
                    )
                {
                    painter.line_segment([a, b], line);
                }
            }
            _ => {}
        }
    }
}
fn paint_symbol_ports(
    painter: &egui::Painter,
    d: &SymbolDefinition,
    t: Transform2D,
    camera: Camera,
    viewport: Rect,
    color: egui::Color32,
) {
    for port in d.ports() {
        if let Ok(p) = t.apply(port.position())
            && let Some(p) = camera.world_to_screen(p, viewport)
            && viewport.contains(p)
        {
            painter.circle_stroke(p, 3.5, Stroke::new(1.0_f32, color));
        }
    }
}
fn draw_bounds(
    painter: &egui::Painter,
    camera: Camera,
    bounds: Rect2D,
    viewport: Rect,
    stroke: Stroke,
) {
    let a = bounds.origin();
    let b = bounds.max();
    let points = [
        a,
        Point2D::new(b.x(), a.y()).expect("finite"),
        b,
        Point2D::new(a.x(), b.y()).expect("finite"),
    ];
    for i in 0..4 {
        if let Some(line) = camera.clipped_segment(points[i], points[(i + 1) % 4], viewport) {
            painter.line_segment(line, stroke);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Pos2;
    #[test]
    fn huge_paper_covering_viewport_is_rendered_even_when_corners_are_offscreen() {
        let mut project = Project::new("Extreme").unwrap();
        let doc = project
            .insert_document(Document::new(project.id(), "D").unwrap())
            .unwrap();
        let page = project
            .insert_page(
                Page::new(
                    project.id(),
                    doc,
                    "P",
                    "1",
                    PageKind::Overview,
                    Size2D::new(100_000.0, 100_000.0).unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        let rect = Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0));
        let camera = Camera::new([50_000.0, 50_000.0], 10_000.0).unwrap();
        let ctx = egui::Context::default();
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(rect),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    draw_page(
                        ui.painter(),
                        &project,
                        page,
                        camera,
                        rect,
                        Selection::None,
                        None,
                    )
                });
            },
        );
        assert!(output.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Rect(r) if r.fill==theme::PAPER&&r.rect.contains(rect.center()))));
    }
}
