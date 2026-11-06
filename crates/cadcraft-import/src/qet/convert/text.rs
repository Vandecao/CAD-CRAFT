// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Converts free page and occurrence texts after electrical content; start at `convert`.
use super::*;

pub(super) fn convert(importer: &mut Importer<'_>, state: &mut DiagramState) {
    let texts = std::mem::take(&mut state.texts);
    let DiagramState {
        pages, instances, ..
    } = state;
    let s = importer.s;
    let world = |page, x, y| super::world(pages, s, page, x, y);
    // Textos livres: os da folha e os de usuário das ocorrências (posição
    // local levada à folha pela transformação da ocorrência).
    let mut free: Vec<(usize, Point2D, String, f64)> = vec![];
    for (page, x, y, text, size) in texts {
        if let Ok(origin) = Point2D::new(x * s, (pages[page].1.height_px() - y) * s) {
            free.push((page, origin, text, size * s * 0.8));
        }
    }
    for inst in instances {
        if inst.user_texts.is_empty() {
            continue;
        }
        let Ok(t) = world(inst.page, inst.x, inst.y).and_then(|p| {
            Transform2D::new(p, rotation(inst.orientation), false, false).map_err(domain)
        }) else {
            continue;
        };
        for (lx, ly, text, height) in &inst.user_texts {
            if let Ok(origin) = Point2D::new(*lx, *ly).and_then(|p| t.apply(p)) {
                free.push((inst.page, origin, text.clone(), *height));
            }
        }
    }
    for (page, origin, text, height) in free {
        let graphic = PageGraphic::new(
            SymbolPrimitive::Text {
                origin: Point2D::new(0.0, 0.0).expect("origem"),
                text: text.chars().take(400).collect(),
                height: height.clamp(1.0, 50.0),
            },
            0.25,
            TextAlignment::Left,
        );
        let rep = graphic.and_then(|g| {
            Representation::new_graphic(
                importer.project.id(),
                pages[page].0,
                g,
                Transform2D::new(origin, 0.0, false, false)?,
            )
        });
        match rep.and_then(|r| importer.project.insert_representation(r)) {
            Ok(_) => importer.stats.texts += 1,
            Err(e) => importer.report.ignored(format!("texto livre: {e}")),
        }
    }
}
