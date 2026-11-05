// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 16: copiar, recortar, colar e duplicar na sessão.
use cad_craft::domain::*;
use cad_gui::{editing::GraphicEdit, state::Session};

fn session() -> (Session, PageId, PageId) {
    let mut s = Session::default();
    s.replace(cad_gui::demo::project().unwrap(), true).unwrap();
    let pages: Vec<_> = s.project().unwrap().pages().take(2).map(Page::id).collect();
    s.open_page(pages[0]).unwrap();
    (s, pages[0], pages[1])
}
fn electrical_session() -> (Session, PageId, PageId) {
    let mut project = cad_gui::connection_demo::project().unwrap();
    let source = project.pages().next().unwrap().id();
    let doc = project.documents().next().unwrap().id();
    let target = project
        .insert_page(
            Page::new(
                project.id(),
                doc,
                "Destino",
                "2",
                PageKind::MultiLine,
                Size2D::new(420.0, 297.0).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let mut s = Session::default();
    s.replace(project, false).unwrap();
    s.open_page(source).unwrap();
    (s, source, target)
}
fn text() -> PageGraphic {
    PageGraphic::new(
        SymbolPrimitive::Text {
            origin: Point2D::new(0.0, 0.0).unwrap(),
            text: "Nota Ω".into(),
            height: 4.0,
        },
        0.25,
        TextAlignment::Right,
    )
    .unwrap()
}
fn transform(x: f64, y: f64) -> Transform2D {
    Transform2D::new(Point2D::new(x, y).unwrap(), 0.0, false, false).unwrap()
}
fn function_symbol(s: &Session, page: PageId) -> Representation {
    let p = s.project().unwrap();
    p.representations_on_page(page)
        .unwrap()
        .into_iter()
        .find(|r| {
            r.symbol().is_some()
                && p.marker(r.id()).is_none()
                && matches!(r.target(), RepresentationTarget::Function(_))
        })
        .unwrap()
        .clone()
}
fn device_of(s: &Session, r: &Representation) -> Device {
    let p = s.project().unwrap();
    let RepresentationTarget::Function(f) = r.target() else {
        panic!("símbolo sem função")
    };
    p.device(p.function(f).unwrap().device_id())
        .unwrap()
        .clone()
}
fn primary(s: &Session) -> Representation {
    s.project()
        .unwrap()
        .representation(s.graphics.selection.primary().unwrap())
        .unwrap()
        .clone()
}

#[test]
fn copy_multiple_to_other_page_is_reusable_and_reversible() {
    let (mut s, source, target) = session();
    let a = s.insert_graphic(text(), transform(20.0, 30.0)).unwrap();
    let b = s
        .insert_graphic(
            PageGraphic::from_drag(
                GraphicKind::Line,
                Point2D::new(0.0, 0.0).unwrap(),
                Point2D::new(8.0, 0.0).unwrap(),
                0.3,
            )
            .unwrap(),
            transform(40.0, 30.0),
        )
        .unwrap();
    s.select_graphics([a, b]);
    s.capture_selection(false).unwrap();
    s.open_page(target).unwrap();
    let at = Point2D::new(10.0, 10.0).unwrap();
    let previews = s.clipboard_preview(at).unwrap();
    assert!(
        previews
            .iter()
            .all(|r| r.page_id() == target && r.id() != a && r.id() != b)
    );
    assert_eq!(
        previews[1].bounds().unwrap().origin().x() - previews[0].bounds().unwrap().origin().x(),
        20.0 + 4.0 * 0.6 * 6.0
    );
    let before = s.snapshot().unwrap();
    s.confirm_paste(at).unwrap();
    let pasted: Vec<_> = s.graphics.selection.ids().iter().copied().collect();
    assert_eq!(pasted.len(), 2);
    assert!(s.project().unwrap().representation(a).is_some());
    assert_eq!(
        s.project()
            .unwrap()
            .representations_on_page(source)
            .unwrap()
            .iter()
            .filter(|r| r.graphic().is_some())
            .count(),
        2
    );
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(s.snapshot().unwrap(), before);
    s.dispatch_graphic(GraphicEdit::Redo).unwrap();
    assert!(
        pasted
            .iter()
            .all(|id| s.project().unwrap().representation(*id).is_some())
    );
    s.confirm_paste(Point2D::new(80.0, 80.0).unwrap()).unwrap();
    assert!(
        s.graphics
            .selection
            .ids()
            .is_disjoint(&pasted.into_iter().collect())
    );
}

#[test]
fn cut_removes_now_and_first_paste_restores_same_id() {
    let (mut s, source, target) = session();
    let id = s.insert_graphic(text(), transform(20.0, 30.0)).unwrap();
    let before_cut = s.snapshot().unwrap();
    let undo = s.history.undo_len();
    s.capture_selection(true).unwrap();
    // Recortar remove na hora, num passo de Desfazer.
    assert!(s.project().unwrap().representation(id).is_none());
    assert_eq!(s.history.undo_len(), undo + 1);
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(s.snapshot().unwrap(), before_cut);
    s.dispatch_graphic(GraphicEdit::Redo).unwrap();
    assert!(s.project().unwrap().representation(id).is_none());
    s.open_page(target).unwrap();
    s.confirm_paste(Point2D::new(50.0, 60.0).unwrap()).unwrap();
    assert_eq!(
        s.project().unwrap().representation(id).unwrap().page_id(),
        target
    );
    // Desfazer a colagem e depois o recorte devolve a origem exata.
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert!(s.project().unwrap().representation(id).is_none());
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(s.snapshot().unwrap(), before_cut);
    assert_eq!(
        s.project().unwrap().representation(id).unwrap().page_id(),
        source
    );
    s.dispatch_graphic(GraphicEdit::Redo).unwrap();
    s.dispatch_graphic(GraphicEdit::Redo).unwrap();
    assert_eq!(
        s.project().unwrap().representation(id).unwrap().page_id(),
        target
    );
}

#[test]
fn later_pastes_of_a_cut_are_copies() {
    let (mut s, _, target) = session();
    let id = s.insert_graphic(text(), transform(20.0, 30.0)).unwrap();
    s.capture_selection(true).unwrap();
    s.open_page(target).unwrap();
    s.confirm_paste(Point2D::new(50.0, 60.0).unwrap()).unwrap();
    assert!(!s.clipboard.as_ref().unwrap().is_cut());
    s.confirm_paste(Point2D::new(70.0, 70.0).unwrap()).unwrap();
    assert_ne!(s.graphics.selection.primary().unwrap(), id);
    assert!(s.project().unwrap().representation(id).is_some());
}

#[test]
fn mixed_selection_rejection_preserves_clipboard_and_history() {
    let (mut s, _, _) = electrical_session();
    let route = s
        .project()
        .unwrap()
        .representations()
        .find(|r| s.project().unwrap().connection_route(r.id()).is_some())
        .unwrap()
        .clone();
    s.open_page(route.page_id()).unwrap();
    let graphic = s.insert_graphic(text(), transform(20.0, 30.0)).unwrap();
    s.select_graphics([graphic]);
    s.capture_selection(false).unwrap();
    let prior = s
        .clipboard_preview(Point2D::new(5.0, 5.0).unwrap())
        .unwrap();
    s.select_graphics([graphic, route.id()]);
    let history = s.history.undo_len();
    let before = s.snapshot().unwrap();
    assert!(s.capture_selection(true).is_err());
    assert!(s.capture_selection(false).is_err());
    assert_eq!(s.history.undo_len(), history);
    assert_eq!(s.snapshot().unwrap(), before);
    let after = s
        .clipboard_preview(Point2D::new(5.0, 5.0).unwrap())
        .unwrap();
    assert_eq!(after[0].graphic(), prior[0].graphic());
    assert_eq!(after[0].transform(), prior[0].transform());
}

#[test]
fn symbol_paste_creates_new_device_with_next_designation() {
    let (mut s, source, target) = electrical_session();
    let original = function_symbol(&s, source);
    let source_device = device_of(&s, &original);
    let devices = s.project().unwrap().devices().count();
    let connections = s.project().unwrap().connections().count();
    let before = s.snapshot().unwrap();
    s.select_graphics([original.id()]);
    s.capture_selection(false).unwrap();
    assert_eq!(s.snapshot().unwrap(), before);
    s.open_page(target).unwrap();
    s.confirm_paste(Point2D::new(50.0, 50.0).unwrap()).unwrap();
    let copied = primary(&s);
    let new_device = device_of(&s, &copied);
    assert_eq!(copied.page_id(), target);
    assert_eq!(copied.symbol(), original.symbol());
    assert_eq!(copied.visual(), original.visual());
    assert_ne!(new_device.id(), source_device.id());
    assert_ne!(new_device.designation(), source_device.designation());
    assert_eq!(new_device.classification(), source_device.classification());
    assert_eq!(s.project().unwrap().devices().count(), devices + 1);
    // Fios não são copiados: as ligações existentes não mudam.
    assert_eq!(s.project().unwrap().connections().count(), connections);
    let bound = s
        .project()
        .unwrap()
        .port_bindings()
        .filter(|(p, _)| p.representation_id() == copied.id())
        .count();
    assert!(bound > 0, "portas do novo símbolo ligadas a pontos novos");
    // Um passo de Desfazer remove tudo; Refazer recria com os mesmos IDs.
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(s.snapshot().unwrap(), before);
    s.dispatch_graphic(GraphicEdit::Redo).unwrap();
    assert_eq!(device_of(&s, &copied).id(), new_device.id());
    // Nova colagem: outra designação.
    s.confirm_paste(Point2D::new(120.0, 50.0).unwrap()).unwrap();
    let third = primary(&s);
    assert_ne!(
        device_of(&s, &third).designation(),
        new_device.designation()
    );
}

#[test]
fn paste_same_device_adds_function_and_reference_on_other_page() {
    let (mut s, source, target) = electrical_session();
    let original = function_symbol(&s, source);
    let device = device_of(&s, &original);
    let devices = s.project().unwrap().devices().count();
    s.select_graphics([original.id()]);
    s.capture_selection(false).unwrap();
    s.clipboard.as_mut().unwrap().keep_device = true;
    s.open_page(target).unwrap();
    s.confirm_paste(Point2D::new(60.0, 40.0).unwrap()).unwrap();
    let copied = s.graphics.selection.primary().unwrap();
    assert_eq!(s.project().unwrap().devices().count(), devices);
    let refs = s.project().unwrap().device_references(device.id()).unwrap();
    let pasted = refs.iter().find(|r| r.representation == copied).unwrap();
    assert_eq!(pasted.page, target);
    let RepresentationTarget::Function(original_function) = original.target() else {
        unreachable!()
    };
    // Função nova do mesmo dispositivo (principal só se ele ainda não tinha).
    assert_ne!(pasted.function, Some(original_function));
    assert_eq!(pasted.device, device.id());
}

#[test]
fn cut_refuses_wired_symbol_and_moves_bindings_of_unwired_one() {
    let (mut s, source, target) = electrical_session();
    let wired = function_symbol(&s, source);
    let has_route = s.project().unwrap().representations().any(|r| {
        s.project()
            .unwrap()
            .connection_route(r.id())
            .is_some_and(|c| {
                c.source().representation_id() == wired.id()
                    || c.target().representation_id() == wired.id()
            })
    });
    s.select_graphics([wired.id()]);
    if has_route {
        let before = s.snapshot().unwrap();
        let err = s.capture_selection(true).unwrap_err().to_string();
        assert!(err.contains("fios"), "{err}");
        assert_eq!(s.snapshot().unwrap(), before);
    }
    // Símbolo sem fios (colado como dispositivo novo) é recortado com vínculos.
    s.capture_selection(false).unwrap();
    s.confirm_paste(Point2D::new(300.0, 200.0).unwrap())
        .unwrap();
    let unwired = s.graphics.selection.primary().unwrap();
    let bindings = |s: &Session| -> Vec<(PortRef, ConnectionPointId)> {
        s.project()
            .unwrap()
            .port_bindings()
            .filter(|(p, _)| p.representation_id() == unwired)
            .map(|(p, c)| (p.clone(), c))
            .collect()
    };
    let before = bindings(&s);
    assert!(!before.is_empty());
    s.capture_selection(true).unwrap();
    assert!(s.project().unwrap().representation(unwired).is_none());
    assert!(bindings(&s).is_empty());
    s.open_page(target).unwrap();
    s.confirm_paste(Point2D::new(40.0, 40.0).unwrap()).unwrap();
    assert_eq!(
        s.project()
            .unwrap()
            .representation(unwired)
            .unwrap()
            .page_id(),
        target
    );
    assert_eq!(bindings(&s), before);
}

#[test]
fn duplicate_offsets_selection_in_one_step() {
    let (mut s, _, _) = session();
    let id = s.insert_graphic(text(), transform(20.0, 30.0)).unwrap();
    let before = s.snapshot().unwrap();
    let copies = s.duplicate_selection().unwrap();
    assert_eq!(copies.len(), 1);
    let t = s
        .project()
        .unwrap()
        .representation(copies[0])
        .unwrap()
        .transform()
        .translation();
    let (dx, dy) = cad_gui::state::DUPLICATE_OFFSET;
    assert_eq!((t.x(), t.y()), (20.0 + dx, 30.0 + dy));
    assert_ne!(copies[0], id);
    assert!(s.clipboard.is_none());
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(s.snapshot().unwrap(), before);
}
