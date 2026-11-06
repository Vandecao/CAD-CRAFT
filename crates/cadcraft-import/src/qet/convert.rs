// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Converts raw QElectroTech XML in semantic order; start at `convert`.
use super::*;
mod connections;
mod definitions;
mod occurrences;
mod pages;
mod text;
struct DiagramState {
    pages: Vec<(PageId, Sheet)>,
    instances: Vec<Instance>,
    wires: Vec<Wire>,
    texts: Vec<(usize, f64, f64, String, f64)>,
}
struct PlacementState {
    by_uuid: BTreeMap<String, usize>,
    placed: BTreeMap<usize, RepresentationId>,
    reports: Vec<usize>,
}
fn world(pages: &[(PageId, Sheet)], s: f64, page: usize, x: f64, y: f64) -> ImportResult<Point2D> {
    let h = pages[page].1.height_px();
    Point2D::new(x * s, (h - y) * s).map_err(domain)
}
pub(super) fn convert(root: &Element, options: &QetOptions) -> ImportResult<QetImport> {
    let s = options.mm_per_px;
    let title = root
        .attr("title")
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .unwrap_or("Projeto QElectroTech");
    let mut importer = Importer {
        s,
        project: Project::new(title).map_err(domain)?,
        report: ImportReport::default(),
        stats: QetStats::default(),
        definitions: BTreeMap::new(),
        templates: BTreeMap::new(),
    };
    importer.report.info(format!(
        "QElectroTech {} · escala {s} mm/px · símbolos CC-BY 3.0 da coleção QElectroTech, quando houver",
        root.attr("version").unwrap_or("?")
    ));
    if let Some(templates) = root.child("titleblocktemplates") {
        for t in templates.children_named("titleblocktemplate") {
            if let Some(name) = t.attr("name") {
                importer.templates.insert(name.to_owned(), t);
            }
        }
    }
    if let Some(collection) = root.child("collection") {
        importer.collection(collection, "embed:/");
    }
    importer.stats.symbols = importer.definitions.len();
    let doc = importer
        .project
        .insert_document(Document::new(importer.project.id(), "Esquema").map_err(domain)?)
        .map_err(domain)?;
    let mut state = pages::convert(&mut importer, root, doc)?;
    let placement = occurrences::convert(&mut importer, &state)?;
    connections::convert(&mut importer, &state, &placement)?;
    text::convert(&mut importer, &mut state);
    importer.project.validate().map_err(domain)?;
    report::finish(importer)
}
