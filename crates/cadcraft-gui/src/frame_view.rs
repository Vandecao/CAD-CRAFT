// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Paint frames, zone labels and cross-references on the sheet. Start with the page-frame renderer. Core data is read-only; only screen conversion is performed here.
use crate::{camera::Camera, theme};
use cad_craft::domain::*;
use eframe::egui::{self, Rect, Stroke, vec2};

const LABEL_HEIGHT: f64 = 2.5;

fn stroke(camera: Camera, width_mm: f64) -> Stroke {
    Stroke::new(
        (width_mm * camera.scale()).clamp(0.6, 3.0) as f32,
        theme::INK,
    )
}
fn pt(x: f64, y: f64) -> Option<Point2D> {
    Point2D::new(x, y).ok()
}
fn segment(
    painter: &egui::Painter,
    camera: Camera,
    viewport: Rect,
    a: Point2D,
    b: Point2D,
    s: Stroke,
) {
    if let Some(line) = camera.clipped_segment(a, b, viewport) {
        painter.line_segment(line, s);
    }
}
fn arc(
    painter: &egui::Painter,
    camera: Camera,
    viewport: Rect,
    (center, radius, start, sweep): (Point2D, f64, f64, f64),
    s: Stroke,
) {
    let steps = ((sweep.abs() / 360.0) * 64.0).ceil().max(4.0) as usize;
    let at = |i: usize| {
        let a = (start + sweep * i as f64 / steps as f64).to_radians();
        pt(center.x() + radius * a.cos(), center.y() + radius * a.sin())
    };
    for i in 0..steps {
        if let (Some(a), Some(b)) = (at(i), at(i + 1)) {
            segment(painter, camera, viewport, a, b, s);
        }
    }
}

/// Text in millimetres with nine-point alignment and counterclockwise rotation.
pub(crate) fn paint_text(
    painter: &egui::Painter,
    camera: Camera,
    viewport: Rect,
    origin: Point2D,
    text: &str,
    style: FrameTextStyle,
    color: egui::Color32,
) {
    let size = style.height * camera.scale();
    if text.trim().is_empty() || !(3.0..=400.0).contains(&size) {
        return;
    }
    let Some(anchor) = camera.world_to_screen(origin, viewport) else {
        return;
    };
    let galley = painter.layout_no_wrap(
        text.to_owned(),
        egui::FontId::proportional(size as f32),
        color,
    );
    let (w, h) = (galley.size().x, galley.size().y);
    if !viewport.expand(w.max(h) + 4.0).contains(anchor) {
        return;
    }
    let (fx, fy) = style.align.fractions();
    let offset = vec2(-(fx as f32) * w, -((1.0 - fy) as f32) * h);
    let angle = -(style.rotation_degrees.to_radians()) as f32;
    let (sin, cos) = angle.sin_cos();
    let rotated = vec2(
        offset.x * cos - offset.y * sin,
        offset.x * sin + offset.y * cos,
    );
    painter.add(egui::epaint::TextShape::new(anchor + rotated, galley, color).with_angle(angle));
}

/// Light watermark color.
pub(crate) const WATERMARK: egui::Color32 = egui::Color32::from_rgb(200, 205, 212);

fn is_watermark(element: &FrameElement) -> bool {
    matches!(
        element,
        FrameElement::Text { style, .. } | FrameElement::Special { style, .. } if style.watermark
    )
}

/// Complete page frame: watermarks first, then elements, special text and zone labels.
pub(crate) fn paint_frame(
    painter: &egui::Painter,
    project: &Project,
    page: PageId,
    frame: &FrameTemplate,
    camera: Camera,
    viewport: Rect,
) {
    let (marks, rest): (Vec<_>, Vec<_>) = frame.elements().iter().partition(|e| is_watermark(e));
    for element in marks.into_iter().chain(rest) {
        paint_element(
            painter,
            project,
            Some(page),
            element,
            camera,
            viewport,
            None,
        );
    }
    for (a, b) in frame.zone_marks() {
        segment(
            painter,
            camera,
            viewport,
            a,
            b,
            stroke(camera, cad_craft::domain::ZONE_MARK_WIDTH),
        );
    }
    paint_zone_labels(painter, frame, camera, viewport);
}

pub(crate) fn paint_zone_labels(
    painter: &egui::Painter,
    frame: &FrameTemplate,
    camera: Camera,
    viewport: Rect,
) {
    for (text, origin) in zone_labels(frame) {
        paint_text(
            painter,
            camera,
            viewport,
            origin,
            &text,
            FrameTextStyle::new(LABEL_HEIGHT, TextAlign::Center),
            theme::INK,
        );
    }
}

/// Paint one element. Without a page (in the editor), special text shows the field name. Color overrides the default for editor selection.
pub(crate) fn paint_element(
    painter: &egui::Painter,
    project: &Project,
    page: Option<PageId>,
    element: &FrameElement,
    camera: Camera,
    viewport: Rect,
    color: Option<egui::Color32>,
) {
    let ink = color.unwrap_or(theme::INK);
    let stroke = |w: f64| Stroke::new(stroke(camera, w).width, ink);
    match element {
        FrameElement::Line { start, end, width } => {
            segment(painter, camera, viewport, *start, *end, stroke(*width));
        }
        FrameElement::Rectangle { rect, width } => {
            let (a, b) = (rect.origin(), rect.max());
            let corners = [
                pt(a.x(), a.y()),
                pt(b.x(), a.y()),
                pt(b.x(), b.y()),
                pt(a.x(), b.y()),
            ];
            for i in 0..4 {
                if let (Some(p), Some(q)) = (corners[i], corners[(i + 1) % 4]) {
                    segment(painter, camera, viewport, p, q, stroke(*width));
                }
            }
        }
        FrameElement::Circle {
            center,
            radius,
            width,
        } => arc(
            painter,
            camera,
            viewport,
            (*center, *radius, 0.0, 360.0),
            stroke(*width),
        ),
        FrameElement::Arc {
            center,
            radius,
            start_degrees,
            sweep_degrees,
            width,
        } => arc(
            painter,
            camera,
            viewport,
            (*center, *radius, *start_degrees, *sweep_degrees),
            stroke(*width),
        ),
        FrameElement::Text {
            origin,
            text,
            style,
        } => {
            let c = color.unwrap_or(if style.watermark {
                WATERMARK
            } else {
                theme::INK
            });
            paint_text(
                painter,
                camera,
                viewport,
                *origin,
                text.get(None),
                *style,
                c,
            );
        }
        FrameElement::Special {
            origin,
            field,
            index,
            style,
        } => {
            let value = match page {
                Some(page) => project.resolve_special(page, field, *index),
                None => match index {
                    Some(i) => format!("<{} {i}>", field.label()),
                    None => format!("<{}>", field.label()),
                },
            };
            let c = color.unwrap_or(if style.watermark {
                WATERMARK
            } else {
                theme::INK
            });
            paint_text(painter, camera, viewport, *origin, &value, *style, c);
        }
        FrameElement::Image { rect, asset } => {
            paint_image(painter, project, *rect, *asset, camera, viewport, color);
        }
    }
}

// Images.

type TextureCache = std::sync::Arc<
    std::sync::Mutex<std::collections::HashMap<AssetId, Option<egui::TextureHandle>>>,
>;
fn textures(ctx: &egui::Context) -> TextureCache {
    ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<TextureCache>(egui::Id::new("cad-craft.frame-textures"))
            .clone()
    })
}
/// Fully decode PNG/JPEG; Core validation checks only the header.
pub fn decode_image(bytes: &[u8]) -> Result<egui::ColorImage, String> {
    let image = image::load_from_memory(bytes).map_err(|e| format!("imagem inválida: {e}"))?;
    let rgba = image.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        rgba.as_raw(),
    ))
}
/// Load and cache the texture for a project image.
pub fn texture(
    ctx: &egui::Context,
    project: &Project,
    asset: AssetId,
) -> Option<egui::TextureHandle> {
    let cache = textures(ctx);
    let mut cache = cache.lock().ok()?;
    if let Some(entry) = cache.get(&asset) {
        return entry.clone();
    }
    // Without project content, such as a preview before insertion, there is nothing to cache. Cache only the final decoding result.
    let bytes = project.asset(asset).and_then(ImageAsset::bytes)?;
    let handle = decode_image(bytes)
        .ok()
        .map(|img| ctx.load_texture(format!("frame-asset-{asset}"), img, Default::default()));
    cache.insert(asset, handle.clone());
    handle
}
fn paint_image(
    painter: &egui::Painter,
    project: &Project,
    rect: Rect2D,
    asset: AssetId,
    camera: Camera,
    viewport: Rect,
    color: Option<egui::Color32>,
) {
    let max = rect.max();
    let (Some(a), Some(b)) = (
        camera.world_to_screen(rect.origin(), viewport),
        pt(max.x(), max.y()).and_then(|m| camera.world_to_screen(m, viewport)),
    ) else {
        return;
    };
    let screen = Rect::from_two_pos(a, b);
    if !viewport.intersects(screen) {
        return;
    }
    match texture(painter.ctx(), project, asset) {
        Some(tex) => {
            painter.image(
                tex.id(),
                screen,
                Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
        None => {
            // Use a placeholder for missing snapshot image content or invalid images.
            let s = Stroke::new(1.0_f32, color.unwrap_or(theme::GRID));
            painter.rect_stroke(screen, 0, s, egui::StrokeKind::Inside);
            painter.line_segment([screen.left_top(), screen.right_bottom()], s);
            painter.line_segment([screen.right_top(), screen.left_bottom()], s);
        }
    }
    if let Some(c) = color {
        painter.rect_stroke(
            screen,
            0,
            Stroke::new(1.5_f32, c),
            egui::StrokeKind::Outside,
        );
    }
}

/// Zone label positions in millimetres, centered in the strip of up to 5 mm between the usable area and sheet edge on each enabled side.
pub fn zone_labels(frame: &FrameTemplate) -> Vec<(String, Point2D)> {
    cad_craft::display::frame_zone_labels(frame)
}

/// Cross-references assembled by Core, with complete text per occurrence.
pub fn cross_reference_texts(project: &Project, page: PageId) -> Vec<(RepresentationId, String)> {
    texts_of(project, page, SheetReferenceKind::CrossReference)
}

/// Derived contact mirror includes registered contacts without representations.
pub fn contact_mirror_texts(project: &Project, page: PageId) -> Vec<(RepresentationId, String)> {
    texts_of(project, page, SheetReferenceKind::ContactMirror)
}

fn texts_of(
    project: &Project,
    page: PageId,
    kind: SheetReferenceKind,
) -> Vec<(RepresentationId, String)> {
    project
        .sheet_references(page)
        .into_iter()
        .filter(|r| r.kind == kind)
        .map(|r| (r.anchor, r.text()))
        .collect()
}

pub(crate) fn paint_cross_references(
    painter: &egui::Painter,
    references: &[SheetReference],
    camera: Camera,
    viewport: Rect,
) {
    for reference in references {
        paint_text(
            painter,
            camera,
            viewport,
            reference.origin,
            &reference.text(),
            FrameTextStyle::new(SHEET_REFERENCE_HEIGHT, TextAlign::TopLeft),
            theme::ACCENT,
        );
    }
}
