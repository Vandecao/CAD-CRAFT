// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! PDF document and page assembly; start with export_pdf.

use super::*;

/// Data atual (UTC) para os metadados.
fn today() -> Date {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    // Dias desde 1970-01-01 → data civil (algoritmo de H. Hinnant).
    let z = (secs / 86_400) as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    Date::new(year.clamp(0, 9999) as u16)
        .month(month as u8)
        .day(day as u8)
}

/// Exporta o alcance para PDF. `progress(feitas, total)` é chamado a cada
/// página; devolver `false` cancela.
pub fn export_pdf(
    project: &Project,
    options: &PdfOptions,
    progress: &mut dyn FnMut(usize, usize) -> bool,
) -> Result<PdfOutput, PdfError> {
    let pages = pages_in_scope(project, &options.scope)?;
    if pages.is_empty() {
        return Err(PdfError::Empty);
    }
    let mut w = Writer {
        project,
        options,
        pdf: Pdf::new(),
        refs: Refs(0),
        images: BTreeMap::new(),
        report: PdfReport::default(),
    };
    let catalog = w.refs.next();
    let tree = w.refs.next();
    let font_id = w.refs.next();
    let info = w.refs.next();
    // Referência e posição de cada página exportada (destinos dos links).
    let mut index: BTreeMap<PageId, (Ref, Placement)> = BTreeMap::new();
    for page in &pages {
        let p = project
            .page(*page)
            .ok_or(DomainError::MissingEntity(EntityRef::Page(*page)))?;
        let place = Placement::new(p.size().width(), p.size().height(), options.paper);
        index.insert(*page, (w.refs.next(), place));
    }
    let total = pages.len();
    for (done, page) in pages.iter().enumerate() {
        if !progress(done, total) {
            return Err(PdfError::Cancelled);
        }
        let list = project.page_display_list(*page)?;
        let (page_ref, place) = index[page];
        let (data, images) = w.content(&list, &place);
        let content_id = w.refs.next();
        w.pdf.stream(content_id, &data).filter(Filter::FlateDecode);
        // Links: destino na página exportada, na posição da ocorrência.
        let mut annotations = vec![];
        for (rect, target) in list.links() {
            let destination = project.representation(target).and_then(|r| {
                let (dest_ref, dest_place) = index.get(&r.page_id())?;
                let at = r.transform().translation();
                Some((*dest_ref, dest_place.point(at.x() - 20.0, at.y() + 20.0)))
            });
            let Some((dest_ref, (left, top))) = destination else {
                w.report.omitted_links += 1;
                continue;
            };
            let id = w.refs.next();
            let mut a = w.pdf.annotation(id);
            a.subtype(AnnotationType::Link);
            a.rect(place.rect(rect));
            a.border(0.0, 0.0, 0.0, None);
            a.action()
                .action_type(ActionType::GoTo)
                .destination()
                .page(dest_ref)
                .xyz(left.max(0.0), top, None);
            a.finish();
            annotations.push(id);
            w.report.links += 1;
        }
        let mut pdf_page = w.pdf.page(page_ref);
        pdf_page.parent(tree);
        pdf_page.media_box(Rect::new(
            0.0,
            0.0,
            place.media.0 as f32,
            place.media.1 as f32,
        ));
        pdf_page.contents(content_id);
        if !annotations.is_empty() {
            pdf_page.annotations(annotations);
        }
        let mut resources = pdf_page.resources();
        resources.fonts().pair(FONT, font_id);
        if !images.is_empty() {
            let mut x = resources.x_objects();
            for (name, id) in &images {
                x.pair(Name(name.as_bytes()), *id);
            }
        }
        resources.finish();
        pdf_page.finish();
        w.report.pages += 1;
    }
    progress(total, total);
    w.pdf
        .pages(tree)
        .kids(pages.iter().map(|p| index[p].0))
        .count(total as i32);
    w.pdf
        .type1_font(font_id)
        .base_font(Name(b"Helvetica"))
        .encoding_predefined(Name(b"WinAnsiEncoding"));
    let outline = if options.bookmarks {
        write_outline(&mut w, &pages, &index)?
    } else {
        None
    };
    let mut cat = w.pdf.catalog(catalog);
    cat.pages(tree);
    if let Some(root) = outline {
        cat.outlines(root);
        cat.page_mode(PageMode::UseOutlines);
    }
    cat.finish();
    let title = project.name().as_str().to_owned();
    let author = project
        .metadata()
        .iter()
        .find(|(k, _)| {
            let k = k.as_str().to_lowercase();
            k.contains("autor") || k.contains("author")
        })
        .map(|(_, v)| v.clone());
    let mut doc = w.pdf.document_info(info);
    doc.title(TextStr(&title));
    if let Some(author) = &author {
        doc.author(TextStr(author));
    }
    doc.creator(TextStr("CAD-CRAFT"));
    doc.producer(TextStr("CAD-CRAFT (pdf-writer)"));
    doc.creation_date(today());
    doc.finish();
    let report = w.report;
    Ok(PdfOutput {
        bytes: w.pdf.finish(),
        report,
    })
}

/// Marcadores: um item por documento com as suas páginas exportadas.
fn write_outline(
    w: &mut Writer<'_>,
    pages: &[PageId],
    index: &BTreeMap<PageId, (Ref, Placement)>,
) -> Result<Option<Ref>, PdfError> {
    let project = w.project;
    let mut groups: Vec<(String, Vec<(String, Ref)>)> = vec![];
    for document in project.documents() {
        let entries: Vec<_> = project
            .ordered_pages(document.id())?
            .into_iter()
            .filter(|p| pages.contains(&p.id()))
            .map(|p| {
                (
                    format!("{} — {}", p.number().as_str(), p.name().as_str()),
                    index[&p.id()].0,
                )
            })
            .collect();
        if !entries.is_empty() {
            groups.push((document.name().as_str().to_owned(), entries));
        }
    }
    if groups.is_empty() {
        return Ok(None);
    }
    let root = w.refs.next();
    let doc_refs: Vec<Ref> = groups.iter().map(|_| w.refs.next()).collect();
    let page_refs: Vec<Vec<Ref>> = groups
        .iter()
        .map(|(_, e)| e.iter().map(|_| w.refs.next()).collect())
        .collect();
    let visible: usize = groups.iter().map(|(_, e)| 1 + e.len()).sum();
    w.pdf
        .outline(root)
        .first(doc_refs[0])
        .last(*doc_refs.last().expect("grupos"))
        .count(visible as i32);
    for (i, (name, entries)) in groups.iter().enumerate() {
        let mut item = w.pdf.outline_item(doc_refs[i]);
        item.title(TextStr(name));
        item.parent(root);
        if i > 0 {
            item.prev(doc_refs[i - 1]);
        }
        if i + 1 < doc_refs.len() {
            item.next(doc_refs[i + 1]);
        }
        item.first(page_refs[i][0]);
        item.last(*page_refs[i].last().expect("páginas"));
        item.count(entries.len() as i32);
        item.dest().page(entries[0].1).fit();
        item.finish();
        for (j, (title, page_ref)) in entries.iter().enumerate() {
            let mut child = w.pdf.outline_item(page_refs[i][j]);
            child.title(TextStr(title));
            child.parent(doc_refs[i]);
            if j > 0 {
                child.prev(page_refs[i][j - 1]);
            }
            if j + 1 < entries.len() {
                child.next(page_refs[i][j + 1]);
            }
            child.dest().page(*page_ref).fit();
            child.finish();
        }
    }
    Ok(Some(root))
}
