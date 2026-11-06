// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Converts pages and frames while collecting raw instances, wires and texts; start at `convert`.
use super::*;

pub(super) fn convert(
    importer: &mut Importer<'_>,
    root: &Element,
    doc: DocumentId,
) -> ImportResult<DiagramState> {
    let s = importer.s;
    let defaults = root.child("newdiagrams").and_then(|n| n.child("border"));
    let mut diagrams: Vec<&Element> = root.children_named("diagram").collect();
    diagrams.sort_by_key(|d| num(d, "order").map_or(usize::MAX, |o| o as usize));
    if diagrams.is_empty() {
        return Err(fail("projeto sem folhas"));
    }

    // Sheets and frames.
    let mut pages = vec![];
    let mut frames: Vec<(Sheet, Option<String>, FrameTemplateId)> = vec![];
    let mut instances: Vec<Instance> = vec![];
    let mut wires: Vec<Wire> = vec![];
    let mut texts: Vec<(usize, f64, f64, String, f64)> = vec![];
    for (index, d) in diagrams.iter().enumerate() {
        let sheet = importer.sheet_of(d, defaults);
        let template = d.attr("titleblocktemplate").map(str::to_owned);
        let size = Size2D::new(sheet.width_px() * s, sheet.height_px() * s).map_err(domain)?;
        let name = d
            .attr("title")
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map_or_else(|| format!("Folha {}", index + 1), str::to_owned);
        let page = importer
            .project
            .insert_page(
                Page::new(
                    importer.project.id(),
                    doc,
                    name,
                    (index + 1).to_string(),
                    PageKind::MultiLine,
                    size,
                )
                .map_err(domain)?,
            )
            .map_err(domain)?;
        let frame = match frames
            .iter()
            .find(|(f, t, _)| *f == sheet && *t == template)
        {
            Some((_, _, id)) => Some(*id),
            None => match importer.frame_for(sheet, template.as_deref()) {
                Ok(package) => {
                    let (edits, id) = importer
                        .project
                        .frame_import_edits(&package)
                        .map_err(domain)?;
                    for edit in edits {
                        importer.project.edit_frame(edit).map_err(domain)?;
                    }
                    frames.push((sheet, template.clone(), id));
                    Some(id)
                }
                Err(e) => {
                    importer
                        .report
                        .approximated(format!("folha {}: moldura: {e}", index + 1));
                    None
                }
            },
        };
        if let Some(frame) = frame {
            importer
                .project
                .edit_frame(FrameEdit::SetPageFrame {
                    page,
                    frame: Some(frame),
                })
                .map_err(domain)?;
        }
        for (attr, key) in [
            ("author", "Autor"),
            ("date", "Data"),
            ("indexrev", "Revisão"),
        ] {
            if let Some(value) = d.attr(attr).map(str::trim).filter(|v| !v.is_empty()) {
                importer
                    .project
                    .edit_frame(FrameEdit::SetPageMetadata {
                        page,
                        key: Text::new(key).map_err(domain)?,
                        value: Some(value.to_owned()),
                    })
                    .map_err(domain)?;
            }
        }
        pages.push((page, sheet));
        for e in d
            .child("elements")
            .into_iter()
            .flat_map(|es| es.children_named("element"))
        {
            let terminals = e
                .child("terminals")
                .into_iter()
                .flat_map(|t| t.children_named("terminal"))
                .filter_map(|t| Some((t.attr("id")?.to_owned(), num(t, "x")?, num(t, "y")?)))
                .collect();
            let links = e
                .child("links_uuids")
                .into_iter()
                .flat_map(|l| l.children_named("link_uuid"))
                .filter_map(|l| l.attr("uuid").map(trim_uuid))
                .collect();
            // Occurrence dynamic texts, either standalone or grouped.
            let dynamic: Vec<&Element> = e
                .child("dynamic_texts")
                .into_iter()
                .flat_map(|d| d.children_named("dynamic_elmt_text"))
                .chain(
                    e.child("texts_groups")
                        .into_iter()
                        .flat_map(|g| g.children_named("texts_group"))
                        .flat_map(|g| g.children_named("dynamic_elmt_text")),
                )
                .collect();
            let mut label_text = None;
            let mut user_texts = vec![];
            for t in dynamic {
                let (Some(x), Some(y)) = (num(t, "x"), num(t, "y")) else {
                    continue;
                };
                let height = elmt::font_height(t, s);
                let info_name = t.child("info_name").map(|i| i.text.trim().to_owned());
                match (t.attr("text_from").unwrap_or(""), info_name.as_deref()) {
                    ("ElementInfo", Some("label")) => {
                        label_text = Point2D::new(x * s, -y * s).ok().map(|origin| {
                            SymbolPrimitive::DynamicText {
                                origin,
                                field: SymbolTextField::Designation,
                                height,
                                align: elmt::align_of(t),
                                rotation_degrees: -num(t, "rotation").unwrap_or(0.0),
                            }
                        });
                    }
                    ("UserText", _) => {
                        let text = t
                            .child("text")
                            .map(|x| plain_text(&x.text))
                            .unwrap_or_default();
                        if !text.is_empty() {
                            user_texts.push((x * s, -y * s, text, height));
                        }
                    }
                    _ => importer
                        .report
                        .ignored("texto dinâmico da ocorrência sem equivalente"),
                }
            }
            instances.push(Instance {
                page: index,
                uuid: trim_uuid(e.attr("uuid").unwrap_or("")),
                def: e.attr("type").unwrap_or("").to_owned(),
                x: num(e, "x").unwrap_or(0.0),
                y: num(e, "y").unwrap_or(0.0),
                orientation: num(e, "orientation").unwrap_or(0.0) as u32,
                label: info(e, "label"),
                prefix: e.attr("prefix").unwrap_or("").to_owned(),
                links,
                terminals,
                label_text,
                user_texts,
            });
        }
        for c in d
            .child("conductors")
            .into_iter()
            .flat_map(|cs| cs.children_named("conductor"))
        {
            let (Some(t1), Some(t2)) = (c.attr("terminal1"), c.attr("terminal2")) else {
                importer.report.ignored("condutor sem terminais");
                continue;
            };
            let (e1, e2) = (
                c.attr("element1").map(trim_uuid),
                c.attr("element2").map(trim_uuid),
            );
            let key = |t: &str, e: &Option<String>| {
                if e.is_some() {
                    trim_uuid(t)
                } else {
                    t.to_owned()
                }
            };
            wires.push(Wire {
                page: index,
                t1: key(t1, &e1),
                t2: key(t2, &e2),
                e1,
                e2,
                num: c.attr("num").unwrap_or("").trim().to_owned(),
                segments: c
                    .children_named("segment")
                    .filter_map(|g| Some((g.attr("orientation")? == "vertical", num(g, "length")?)))
                    .collect(),
            });
        }
        for input in d
            .child("inputs")
            .into_iter()
            .flat_map(|i| i.children_named("input"))
        {
            let text = plain_text(input.attr("text").unwrap_or(""));
            if let (Some(x), Some(y)) = (num(input, "x"), num(input, "y"))
                && !text.is_empty()
            {
                texts.push((index, x, y, text, num(input, "size").unwrap_or(9.0)));
            }
        }
        for (tag, what) in [("images", "imagem"), ("shapes", "forma")] {
            let n = d.child(tag).map_or(0, |c| c.children.len());
            for _ in 0..n {
                importer
                    .report
                    .ignored(format!("{what} na folha (sem suporte nesta etapa)"));
            }
        }
    }
    importer.stats.pages = pages.len();
    Ok(DiagramState {
        pages,
        instances,
        wires,
        texts,
    })
}
