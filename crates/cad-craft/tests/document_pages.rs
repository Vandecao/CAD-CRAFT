// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_craft::domain::*;

#[test]
fn reorder_cannot_cross_documents_and_deleted_orders_can_be_reassigned() {
    let (mut p, d, ids) = setup();
    let other = p
        .insert_document(Document::new(p.id(), "Other").unwrap())
        .unwrap();
    let foreign = p
        .insert_page(
            Page::new(
                p.id(),
                other,
                "Other",
                "1",
                PageKind::Overview,
                Size2D::new(1.0, 1.0).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let before = p.clone();
    let mut bad = ids.clone();
    bad[0] = foreign;
    assert!(p.reorder_pages(d, &bad).is_err());
    assert_eq!(p, before);
    p.reorder_pages(d, &ids).unwrap();
    p.remove_page(ids[0]).unwrap();
    p.reorder_pages(d, &ids[1..]).unwrap();
    assert_eq!(p.page(ids[1]).unwrap().order(), Some(0));
    assert!(p.page(foreign).unwrap().order().is_none());
    let before = p.clone();
    assert!(p.rename_page(PageId::new(), "Missing").is_err());
    assert!(p.identify_page(ids[1], "").is_err());
    assert_eq!(p, before);
}

fn setup() -> (Project, DocumentId, Vec<PageId>) {
    let mut p = Project::new("P").unwrap();
    let d = p
        .insert_document(Document::new(p.id(), "D").unwrap())
        .unwrap();
    let mut ids = vec![];
    for kind in [
        PageKind::MultiLine,
        PageKind::SingleLine,
        PageKind::Overview,
        PageKind::PanelLayout,
        PageKind::TechnicalDocumentation,
    ] {
        ids.push(
            p.insert_page(
                Page::new(
                    p.id(),
                    d,
                    "Folha",
                    "1",
                    kind,
                    Size2D::new(297.0, 210.0).unwrap(),
                )
                .unwrap(),
            )
            .unwrap(),
        );
    }
    (p, d, ids)
}

#[test]
fn reorder_is_complete_per_document_and_atomic() {
    let (mut p, d, ids) = setup();
    let mut reversed = ids.clone();
    reversed.reverse();
    p.reorder_pages(d, &reversed).unwrap();
    assert_eq!(
        p.ordered_pages(d)
            .unwrap()
            .iter()
            .map(|p| p.id())
            .collect::<Vec<_>>(),
        reversed
    );
    let before = p.clone();
    for invalid in [vec![ids[0]], vec![ids[0]; 5], vec![PageId::new(); 5]] {
        assert!(p.reorder_pages(d, &invalid).is_err());
        assert_eq!(p, before);
    }
    assert!(p.ordered_pages(DocumentId::new()).is_err());
}

#[test]
fn rename_identification_and_format_preserve_identity() {
    let (mut p, _, ids) = setup();
    p.rename_page(ids[0], "Comando").unwrap();
    p.identify_page(ids[0], "A-10").unwrap();
    p.set_page_format(
        ids[0],
        PageKind::SingleLine,
        Size2D::new(210.0, 297.0).unwrap(),
    )
    .unwrap();
    assert_eq!(p.page(ids[0]).unwrap().name().as_str(), "Comando");
    assert_eq!(p.page(ids[0]).unwrap().number().as_str(), "A-10");
    assert_eq!(
        p.page(ids[0]).unwrap().orientation(),
        PageOrientation::Portrait
    );
    let before = p.clone();
    assert!(p.rename_page(ids[0], " ").is_err());
    assert_eq!(p, before);
    p.set_identification(Some(Text::new("PROJ-001").unwrap()));
    assert_eq!(p.identification().unwrap().as_str(), "PROJ-001");
    p.set_identification(None);
    assert!(p.identification().is_none());
}

#[test]
fn independent_function_and_location_trees_include_ancestors() {
    let (mut p, d, ids) = setup();
    p.set_page_structure(
        ids[0],
        DocumentStructure::new(
            vec![
                DesignationPart::new("Drive").unwrap(),
                DesignationPart::new("Power").unwrap(),
            ],
            vec![DesignationPart::new("Panel1").unwrap()],
        ),
    )
    .unwrap();
    let f = p.page_tree(d, StructureAxis::Function).unwrap();
    assert_eq!(f.len(), 3);
    assert_eq!(f[0].pages.len(), 4);
    assert_eq!(f[1].path[0].as_str(), "Drive");
    assert!(f[1].pages.is_empty());
    assert_eq!(f[2].pages, vec![ids[0]]);
    let l = p.page_tree(d, StructureAxis::Location).unwrap();
    assert_eq!(l.len(), 2);
    assert_eq!(l[1].path[0].as_str(), "Panel1");
    assert_eq!(p.devices().count(), 0);
    assert_eq!(p.functions().count(), 0);
}

#[test]
fn page_removal_rejects_external_property_references_without_partial_change() {
    let (mut p, _, ids) = setup();
    let mut props = TechnicalProperties::default();
    props
        .set_custom("page", PropertyValue::Reference(EntityRef::Page(ids[0])))
        .unwrap();
    p.set_properties(EntityRef::Project(p.id()), props.clone())
        .unwrap();
    let before = p.clone();
    assert!(p.remove_page(ids[0]).is_err());
    assert_eq!(p, before);
    p.set_properties(EntityRef::Project(p.id()), TechnicalProperties::default())
        .unwrap();
    // A referência da própria página desaparece com ela e não bloqueia remoção.
    p.set_properties(EntityRef::Page(ids[0]), props).unwrap();
    assert_eq!(p.remove_page(ids[0]).unwrap().id(), ids[0]);
    assert!(p.remove_page(ids[0]).is_err());
    p.validate().unwrap();
}
