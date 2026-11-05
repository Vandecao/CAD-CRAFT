// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 21: lista de exibição e PDF multipágina com links e marcadores.
use cad_craft::display::{DisplayColor, DisplayItem};
use cad_craft::domain::*;
use cad_craft::pdf::{self, PaperFit, PdfOptions, PdfScope};
use std::time::Instant;

mod output_common;
use output_common::*;

fn texts(project: &Project, page: PageId) -> Vec<String> {
    project
        .page_display_list(page)
        .unwrap()
        .texts()
        .map(|t| t.text.clone())
        .collect()
}

fn all(options: PdfOptions, project: &Project) -> pdf::PdfOutput {
    pdf::export_pdf(project, &options, &mut |_, _| true).unwrap()
}

fn page_contents(doc: &lopdf::Document) -> Vec<Vec<u8>> {
    doc.get_pages()
        .values()
        .map(|id| doc.get_page_content(*id))
        .collect()
}
/// Textos (bytes WinAnsi) dos operadores Tj de um conteúdo.
fn shown(content: &[u8]) -> Vec<Vec<u8>> {
    lopdf::content::Content::decode(content)
        .unwrap()
        .operations
        .into_iter()
        .filter(|op| op.operator == "Tj")
        .filter_map(|op| match op.operands.first() {
            Some(lopdf::Object::String(bytes, _)) => Some(bytes.clone()),
            _ => None,
        })
        .collect()
}
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// Critério 1: a lista de exibição tem moldura, textos especiais, designação,
/// número do fio, referência e imagem; a referência do contato leva à bobina.
#[test]
fn display_list_reproduces_the_sheet() {
    let f = fixture("Painel CCM");
    let list = f.project.page_display_list(f.pages[0]).unwrap();
    assert_eq!((list.width, list.height), (420.0, 297.0));
    let first = texts(&f.project, f.pages[0]);
    for expected in ["Painel CCM", "-K1", "-K2", "1"] {
        assert!(first.iter().any(|t| t == expected), "{expected}: {first:?}");
    }
    assert!(
        list.items
            .iter()
            .any(|i| matches!(i, DisplayItem::Image { .. }))
    );
    // Bobina principal: espelho com os pontos do contato e a referência da página 2.
    let contact_ref = f.project.short_reference(f.pages[1], p(60.0, 160.0));
    assert!(
        first
            .iter()
            .any(|t| t.contains(&contact_ref) && t.contains("1-2")),
        "{first:?}"
    );
    let second = texts(&f.project, f.pages[1]);
    let coil_ref = f.project.short_reference(f.pages[0], p(130.0, 150.0));
    assert!(second.iter().any(|t| t == &coil_ref), "{second:?}");
    let links: Vec<_> = f
        .project
        .page_display_list(f.pages[1])
        .unwrap()
        .links()
        .map(|(_, target)| target)
        .collect();
    assert!(links.contains(&f.coil), "{links:?}");
    // Referências em cor própria; a tela usa os mesmos textos.
    assert!(list.texts().any(|t| t.color == DisplayColor::Reference));
    let references: Vec<_> = f
        .project
        .sheet_references(f.pages[1])
        .iter()
        .map(SheetReference::text)
        .collect();
    assert!(references.contains(&coil_ref));
}

/// Critérios 1, 2 e 4: três páginas A3, leitura independente, fonte e
/// imagem declaradas, marcadores documento → páginas.
#[test]
fn pdf_has_three_real_size_pages_fonts_images_and_bookmarks() {
    let f = fixture("Painel CCM");
    let out = all(PdfOptions::default(), &f.project);
    assert!(out.bytes.starts_with(b"%PDF-"));
    assert_eq!(out.report.pages, 3);
    let doc = lopdf::Document::load_mem(&out.bytes).unwrap();
    let pages = doc.get_pages();
    assert_eq!(pages.len(), 3);
    for id in pages.values() {
        let page = doc.get_dictionary(*id).unwrap();
        let media = page.get(b"MediaBox").unwrap().as_array().unwrap();
        let w = media[2].as_float().unwrap();
        let h = media[3].as_float().unwrap();
        assert!(
            (w - 1190.55).abs() < 0.1 && (h - 841.89).abs() < 0.1,
            "{w}×{h}"
        );
        let resources = page.get(b"Resources").unwrap().as_dict().unwrap();
        assert!(resources.get(b"Font").is_ok());
        assert!(
            resources.get(b"XObject").is_ok(),
            "logotipo em todas as páginas"
        );
    }
    let helvetica = doc.objects.values().any(|o| {
        o.as_dict().is_ok_and(|d| {
            d.get(b"BaseFont")
                .is_ok_and(|n| n.as_name().ok() == Some(b"Helvetica"))
        })
    });
    assert!(helvetica);
    // Marcadores: raiz → documento → 3 páginas, com os títulos certos.
    let catalog = doc.catalog().unwrap();
    let outlines = catalog.get(b"Outlines").unwrap().as_reference().unwrap();
    let root = doc.get_dictionary(outlines).unwrap();
    let document = doc
        .get_dictionary(root.get(b"First").unwrap().as_reference().unwrap())
        .unwrap();
    let title =
        |d: &lopdf::Dictionary| lopdf::decode_text_string(d.get(b"Title").unwrap()).unwrap();
    assert_eq!(title(document), "Esquema elétrico");
    let mut child = doc
        .get_dictionary(document.get(b"First").unwrap().as_reference().unwrap())
        .unwrap();
    let mut titles = vec![title(child)];
    while let Ok(next) = child.get(b"Next") {
        child = doc.get_dictionary(next.as_reference().unwrap()).unwrap();
        titles.push(title(child));
    }
    assert_eq!(titles, ["1 — Força", "2 — Comando", "3 — Sinalização"]);
    let info = doc
        .get_dictionary(doc.trailer.get(b"Info").unwrap().as_reference().unwrap())
        .unwrap();
    assert_eq!(title(info), "Painel CCM");
    assert!(info.get(b"CreationDate").is_ok());
}

/// Critério 3: links do contato para a bobina e da bobina (espelho) para o
/// contato; interrupções entre si; destino fora do alcance não vira link.
#[test]
fn links_follow_cross_references_and_are_omitted_outside_scope() {
    let f = fixture("Links");
    let out = all(PdfOptions::default(), &f.project);
    let doc = lopdf::Document::load_mem(&out.bytes).unwrap();
    let pages: Vec<_> = doc.get_pages().values().copied().collect();
    let destinations = |page: lopdf::ObjectId| -> Vec<lopdf::ObjectId> {
        let dict = doc.get_dictionary(page).unwrap();
        let Ok(annots) = dict.get(b"Annots") else {
            return vec![];
        };
        annots
            .as_array()
            .unwrap()
            .iter()
            .map(|a| {
                let a = doc.get_dictionary(a.as_reference().unwrap()).unwrap();
                assert_eq!(a.get(b"Subtype").unwrap().as_name().unwrap(), b"Link");
                let action = a.get(b"A").unwrap().as_dict().unwrap();
                assert_eq!(action.get(b"S").unwrap().as_name().unwrap(), b"GoTo");
                action.get(b"D").unwrap().as_array().unwrap()[0]
                    .as_reference()
                    .unwrap()
            })
            .collect()
    };
    // Página 1 (bobina) → página 2 (contato); página 2 → páginas 1 e 3.
    assert!(destinations(pages[0]).contains(&pages[1]));
    let from_second = destinations(pages[1]);
    assert!(from_second.contains(&pages[0]) && from_second.contains(&pages[2]));
    assert!(destinations(pages[2]).contains(&pages[1]));
    assert!(out.report.links >= 4);
    assert_eq!(out.report.omitted_links, 0);
    // Só a página 2: os destinos 1 e 3 ficam fora e são relatados.
    let partial = all(
        PdfOptions {
            scope: PdfScope::Pages(vec![f.pages[1]]),
            ..PdfOptions::default()
        },
        &f.project,
    );
    assert_eq!(partial.report.pages, 1);
    assert!(partial.report.omitted_links >= 2);
    assert_eq!(partial.report.links, 0);
    assert!(partial.report.summary().contains("sem link"));
    let _ = f.contact;
}

/// Critérios 5 e 6: acentos em WinAnsi; caractere fora da fonte relatado;
/// preto e branco sem cor RGB no conteúdo.
#[test]
fn accents_are_kept_unsupported_characters_reported_and_monochrome_has_no_rgb() {
    let f = fixture("Ação elétrica Ω");
    let color = all(PdfOptions::default(), &f.project);
    assert_eq!(color.report.replaced.iter().collect::<String>(), "Ω");
    assert!(color.report.summary().contains('Ω'));
    let doc = lopdf::Document::load_mem(&color.bytes).unwrap();
    let contents = page_contents(&doc);
    // "Ação elétrica ?" em WinAnsi: ç = E7, ã = E3, é = E9.
    let expected: Vec<u8> = [b"A".as_slice(), &[0xE7, 0xE3], b"o el", &[0xE9], b"trica ?"].concat();
    assert!(
        shown(&contents[0]).contains(&expected),
        "{:?}",
        shown(&contents[0])
    );
    assert!(
        contents.iter().any(|c| contains(c, b" rg")),
        "referências em cor"
    );
    let mono = all(
        PdfOptions {
            monochrome: true,
            ..PdfOptions::default()
        },
        &f.project,
    );
    let doc = lopdf::Document::load_mem(&mono.bytes).unwrap();
    for content in page_contents(&doc) {
        assert!(!contains(&content, b" rg\n") && !contains(&content, b" RG\n"));
        assert!(!contains(&content, b" k\n") && !contains(&content, b" K\n"));
    }
}

/// Impressão: papel A4 com "ajustar" — página A4 deitada, conteúdo reduzido.
#[test]
fn paper_fit_places_the_sheet_on_the_chosen_paper() {
    let f = fixture("Papel");
    let out = all(
        PdfOptions {
            scope: PdfScope::Document(f.project.documents().next().unwrap().id()),
            paper: Some((PaperFormat::A4.size(false), PaperFit::Fit)),
            bookmarks: false,
            ..PdfOptions::default()
        },
        &f.project,
    );
    let doc = lopdf::Document::load_mem(&out.bytes).unwrap();
    let page = doc
        .get_dictionary(*doc.get_pages().values().next().unwrap())
        .unwrap();
    let media = page.get(b"MediaBox").unwrap().as_array().unwrap();
    let w = media[2].as_float().unwrap();
    let h = media[3].as_float().unwrap();
    assert!(
        (w - 841.89).abs() < 0.1 && (h - 595.28).abs() < 0.1,
        "{w}×{h}"
    );
    assert!(doc.catalog().unwrap().get(b"Outlines").is_err());
    assert!(matches!(
        pdf::export_pdf(
            &f.project,
            &PdfOptions {
                scope: PdfScope::Pages(vec![]),
                ..PdfOptions::default()
            },
            &mut |_, _| true
        ),
        Err(pdf::PdfError::Empty)
    ));
    assert!(matches!(
        pdf::export_pdf(&f.project, &PdfOptions::default(), &mut |done, _| done < 1),
        Err(pdf::PdfError::Cancelled)
    ));
}

/// Critério 8: 200 páginas com moldura, 30 símbolos e 30 fios por página.
/// Meta: < 10 s no Release; aqui o limite é folgado (Debug).
#[test]
fn two_hundred_pages_export_within_the_time_limit() {
    let project = scale_project(200);
    assert_eq!(project.pages().count(), 200);
    let start = Instant::now();
    let out = all(PdfOptions::default(), &project);
    let elapsed = start.elapsed();
    assert_eq!(out.report.pages, 200);
    println!(
        "PDF: 200 páginas, {:.1} MiB, {:?}",
        out.bytes.len() as f64 / 1_048_576.0,
        elapsed
    );
    let limit = if cfg!(debug_assertions) { 120 } else { 10 };
    assert!(elapsed.as_secs() < limit, "{elapsed:?}");
}
