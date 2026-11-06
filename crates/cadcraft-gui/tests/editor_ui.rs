// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Fixture helpers use validated synthetic inputs and fail immediately on setup errors.
#![allow(clippy::unwrap_used)]
use cad_craft::domain::*;
use cad_gui::{
    app::CadApp,
    commands::{CommandId, registry},
    editing::GraphicEdit,
    preferences::RibbonTab,
};
use eframe::egui::{self, Context, Event, Key, Modifiers, PointerButton, Pos2, Rect, vec2};
fn frame(app: &mut CadApp, ctx: &Context, events: Vec<Event>, ctrl: bool) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1600.0, 900.0))),
            events,
            modifiers: if ctrl {
                Modifiers::CTRL
            } else {
                Modifiers::NONE
            },
            ..Default::default()
        },
        |ctx| app.show(ctx),
    )
}
fn setup() -> (CadApp, Context, RepresentationId, RepresentationId) {
    let mut app = CadApp::default();
    let ctx = Context::default();
    app.session
        .replace(cad_gui::demo::project().unwrap(), true)
        .unwrap();
    let page = app
        .session
        .project()
        .unwrap()
        .representations()
        .next()
        .unwrap()
        .page_id();
    let ids: Vec<_> = app
        .session
        .project()
        .unwrap()
        .representations_on_page(page)
        .unwrap()
        .iter()
        .take(2)
        .map(|r| r.id())
        .collect();
    app.session.open_page(page).unwrap();
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![], false);
    }
    (app, ctx, ids[0], ids[1])
}
fn pos(app: &CadApp, id: RepresentationId) -> Pos2 {
    let t = app
        .session
        .project()
        .unwrap()
        .representation(id)
        .unwrap()
        .transform();
    app.session.tabs[0]
        .camera
        .world_to_screen(t.translation(), app.last_viewport.unwrap())
        .unwrap()
}
fn button(p: Pos2, button: PointerButton, pressed: bool, ctrl: bool) -> Event {
    Event::PointerButton {
        pos: p,
        button,
        pressed,
        modifiers: if ctrl {
            Modifiers::CTRL
        } else {
            Modifiers::NONE
        },
    }
}
fn click(app: &mut CadApp, ctx: &Context, p: Pos2, ctrl: bool) {
    frame(
        app,
        ctx,
        vec![
            Event::PointerMoved(p),
            button(p, PointerButton::Primary, true, ctrl),
        ],
        ctrl,
    );
    frame(
        app,
        ctx,
        vec![button(p, PointerButton::Primary, false, ctrl)],
        ctrl,
    );
}
#[test]
fn real_click_ctrl_hover_and_empty_click_do_not_edit_document() {
    let (mut a, c, x, y) = setup();
    let before = a.session.snapshot().unwrap();
    let px = pos(&a, x);
    let py = pos(&a, y);
    click(&mut a, &c, px, false);
    assert!(a.session.graphics.selection.ids().contains(&x));
    click(&mut a, &c, py, true);
    assert_eq!(a.session.graphics.selection.ids().len(), 2);
    frame(&mut a, &c, vec![Event::PointerMoved(px)], false);
    assert_eq!(a.session.graphics.hover, Some(x));
    click(&mut a, &c, px, true);
    assert!(!a.session.graphics.selection.ids().contains(&x));
    let empty = a.last_viewport.unwrap().left_bottom() + vec2(20.0, -20.0);
    click(&mut a, &c, empty, false);
    assert!(a.session.graphics.selection.ids().is_empty());
    assert_eq!(before, a.session.snapshot().unwrap());
}
#[test]
fn real_drag_previews_then_commits_once_and_undo_redo_use_dispatcher() {
    let (mut a, c, x, _) = setup();
    let before = a.session.snapshot().unwrap();
    let p = pos(&a, x);
    click(&mut a, &c, p, false);
    frame(
        &mut a,
        &c,
        vec![button(p, PointerButton::Primary, true, false)],
        false,
    );
    frame(
        &mut a,
        &c,
        vec![Event::PointerMoved(p + vec2(30.0, 20.0))],
        false,
    );
    assert!(a.session.graphics.is_active());
    assert_eq!(before, a.session.snapshot().unwrap());
    frame(
        &mut a,
        &c,
        vec![button(
            p + vec2(30.0, 20.0),
            PointerButton::Primary,
            false,
            false,
        )],
        false,
    );
    let after = a.session.snapshot().unwrap();
    assert_ne!(before, after);
    assert_eq!(a.session.history.undo_len(), 1);
    let key = |key| Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::CTRL,
    };
    frame(&mut a, &c, vec![key(Key::Z)], true);
    assert_eq!(before, a.session.snapshot().unwrap());
    frame(&mut a, &c, vec![key(Key::Y)], true);
    assert_eq!(after, a.session.snapshot().unwrap());
}
#[test]
fn escape_and_middle_button_cancel_preview_without_moving_model() {
    for middle in [false, true] {
        let (mut a, c, x, _) = setup();
        let before = a.session.snapshot().unwrap();
        let p = pos(&a, x);
        frame(
            &mut a,
            &c,
            vec![
                Event::PointerMoved(p),
                button(p, PointerButton::Primary, true, false),
            ],
            false,
        );
        frame(
            &mut a,
            &c,
            vec![Event::PointerMoved(p + vec2(40.0, 10.0))],
            false,
        );
        assert!(a.session.graphics.is_active());
        let event = if middle {
            button(p + vec2(40.0, 10.0), PointerButton::Middle, true, false)
        } else {
            Event::Key {
                key: Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }
        };
        frame(&mut a, &c, vec![event], false);
        assert!(!a.session.graphics.is_active());
        frame(
            &mut a,
            &c,
            vec![button(
                p + vec2(40.0, 10.0),
                PointerButton::Primary,
                false,
                false,
            )],
            false,
        );
        assert_eq!(before, a.session.snapshot().unwrap());
        assert_eq!(a.session.history.undo_len(), 0);
    }
}
#[test]
fn command_registry_enables_real_transforms_and_history_only_with_context() {
    let (mut a, c, x, _) = setup();
    let r = registry();
    let available =
        |id, s: &cad_gui::state::Session| r.iter().find(|r| r.id == id).unwrap().available(s);
    assert!(!available(CommandId::Rotate90, &a.session));
    a.session.select_graphics([x]);
    assert!(available(CommandId::Rotate90, &a.session));
    let old = a.session.snapshot().unwrap();
    a.execute(CommandId::Rotate90, &c);
    assert_ne!(old, a.session.snapshot().unwrap());
    assert!(available(CommandId::Undo, &a.session));
    a.execute(CommandId::Undo, &c);
    assert_eq!(old, a.session.snapshot().unwrap());
    assert!(available(CommandId::Redo, &a.session));
    a.preferences.ribbon_tab = RibbonTab::Edit;
    frame(&mut a, &c, vec![], false);
}
#[test]
fn switching_pages_cancels_gesture_and_keeps_history_and_cameras() {
    let (mut a, _, x, _) = setup();
    a.session.select_graphics([x]);
    a.session
        .dispatch_graphic(GraphicEdit::Translate(1.0, 0.0))
        .unwrap();
    let next = a
        .session
        .project()
        .unwrap()
        .pages()
        .find(|p| Some(p.id()) != a.session.active)
        .unwrap()
        .id();
    let camera = a.session.tabs[0].camera;
    a.session.open_page(next).unwrap();
    assert!(!a.session.graphics.is_active());
    assert!(a.session.graphics.selection.ids().is_empty());
    assert!(a.session.history.can_undo());
    assert_eq!(a.session.tabs[0].camera, camera);
}

fn text_rect(output: &egui::FullOutput, label: &str) -> Option<Rect> {
    output.shapes.iter().find_map(|s| {
        if let egui::Shape::Text(t) = &s.shape {
            (t.galley.text() == label).then(|| {
                t.galley
                    .rect
                    .translate(t.pos.to_vec2())
                    .intersect(s.clip_rect)
            })
        } else {
            None
        }
    })
}
#[test]
fn context_menu_preserves_group_and_offers_different_empty_actions() {
    let (mut a, c, x, y) = setup();
    a.session.select_graphics([x, y]);
    let p = pos(&a, x);
    frame(
        &mut a,
        &c,
        vec![
            Event::PointerMoved(p),
            button(p, PointerButton::Secondary, true, false),
        ],
        false,
    );
    frame(
        &mut a,
        &c,
        vec![button(p, PointerButton::Secondary, false, false)],
        false,
    );
    let out = frame(&mut a, &c, vec![], false);
    let view = a.last_viewport.unwrap();
    let context_rect = |output: &egui::FullOutput, label: &str| {
        output.shapes.iter().find_map(|s| {
            if let egui::Shape::Text(t) = &s.shape {
                (t.galley.text() == label && t.pos.x < view.right() && t.pos.x > view.left()).then(
                    || {
                        t.galley
                            .rect
                            .translate(t.pos.to_vec2())
                            .intersect(s.clip_rect)
                    },
                )
            } else {
                None
            }
        })
    };
    let rotate = context_rect(&out, "Girar 90°").expect("object context action");
    assert_eq!(a.session.graphics.selection.ids().len(), 2);
    let before = a.session.snapshot().unwrap();
    click(&mut a, &c, rotate.center(), false);
    assert_ne!(before, a.session.snapshot().unwrap());
    assert_eq!(a.session.history.undo_len(), 1);
    let empty = a.last_viewport.unwrap().left_bottom() + vec2(25.0, -25.0);
    frame(
        &mut a,
        &c,
        vec![
            Event::PointerMoved(empty),
            button(empty, PointerButton::Secondary, true, false),
        ],
        false,
    );
    frame(
        &mut a,
        &c,
        vec![button(empty, PointerButton::Secondary, false, false)],
        false,
    );
    let out = frame(&mut a, &c, vec![], false);
    assert!(text_rect(&out, "Propriedades da página").is_some());
    assert!(context_rect(&out, "Girar 90°").is_none());
}
#[test]
fn rectangle_drag_selects_and_escape_preserves_prior_selection() {
    let (mut a, c, x, y) = setup();
    let view = a.last_viewport.unwrap();
    let start = view.left_top() + vec2(8.0, 8.0);
    let end = view.right_bottom() - vec2(8.0, 8.0);
    frame(
        &mut a,
        &c,
        vec![
            Event::PointerMoved(start),
            button(start, PointerButton::Primary, true, false),
        ],
        false,
    );
    frame(&mut a, &c, vec![Event::PointerMoved(end)], false);
    assert!(a.session.graphics.is_active());
    frame(
        &mut a,
        &c,
        vec![button(end, PointerButton::Primary, false, false)],
        false,
    );
    assert!(a.session.graphics.selection.ids().contains(&x));
    assert!(a.session.graphics.selection.ids().contains(&y));
    let selected = a.session.graphics.selection.ids().clone();
    frame(
        &mut a,
        &c,
        vec![
            Event::PointerMoved(start),
            button(start, PointerButton::Primary, true, false),
        ],
        false,
    );
    frame(
        &mut a,
        &c,
        vec![Event::PointerMoved(start + vec2(35.0, 35.0))],
        false,
    );
    frame(
        &mut a,
        &c,
        vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        false,
    );
    assert_eq!(a.session.graphics.selection.ids(), &selected);
    assert!(!a.session.graphics.is_active());
}
#[test]
fn hidden_grid_does_not_disable_snapped_group_movement() {
    let (mut a, c, x, y) = setup();
    a.preferences.grid_visible = false;
    a.preferences.grid_mm = 4.0;
    a.preferences.snap_enabled = true;
    a.session.select_graphics([x, y]);
    let p = pos(&a, x);
    let project = a.session.project().unwrap();
    let tx = project.representation(x).unwrap().transform().translation();
    let ty = project.representation(y).unwrap().transform().translation();
    frame(
        &mut a,
        &c,
        vec![
            Event::PointerMoved(p),
            button(p, PointerButton::Primary, true, false),
        ],
        false,
    );
    frame(
        &mut a,
        &c,
        vec![Event::PointerMoved(p + vec2(39.0, -29.0))],
        false,
    );
    frame(
        &mut a,
        &c,
        vec![button(
            p + vec2(39.0, -29.0),
            PointerButton::Primary,
            false,
            false,
        )],
        false,
    );
    let project = a.session.project().unwrap();
    let nx = project.representation(x).unwrap().transform().translation();
    let ny = project.representation(y).unwrap().transform().translation();
    assert_eq!(nx.x() % 4.0, 0.0);
    assert_eq!(nx.y() % 4.0, 0.0);
    assert!(((ny.x() - nx.x()) - (ty.x() - tx.x())).abs() < 1e-8);
    assert!(((ny.y() - nx.y()) - (ty.y() - tx.y())).abs() < 1e-8);
    assert_eq!(a.session.history.undo_len(), 1);
}
#[test]
fn loss_of_window_focus_discards_drag_and_release_outside_finishes_it() {
    for lose_focus in [false, true] {
        let (mut a, c, x, _) = setup();
        let before = a.session.snapshot().unwrap();
        let p = pos(&a, x);
        let end = a.last_viewport.unwrap().left_center() - vec2(30.0, 0.0);
        frame(
            &mut a,
            &c,
            vec![
                Event::PointerMoved(p),
                button(p, PointerButton::Primary, true, false),
            ],
            false,
        );
        frame(
            &mut a,
            &c,
            vec![Event::PointerMoved(p + vec2(40.0, 0.0))],
            false,
        );
        if lose_focus {
            frame(&mut a, &c, vec![Event::WindowFocused(false)], false);
        }
        frame(
            &mut a,
            &c,
            vec![
                Event::PointerMoved(end),
                button(end, PointerButton::Primary, false, false),
            ],
            false,
        );
        assert!(!a.session.graphics.is_active());
        if lose_focus {
            assert_eq!(before, a.session.snapshot().unwrap());
        } else {
            assert_ne!(before, a.session.snapshot().unwrap());
        }
    }
}
#[test]
fn undo_shortcut_in_text_field_does_not_undo_project() {
    let (mut a, c, x, _) = setup();
    a.session.select_graphics([x]);
    a.execute(CommandId::Rotate90, &c);
    let before = a.session.snapshot().unwrap();
    let out = frame(&mut a, &c, vec![], false);
    let search = text_rect(&out, "Procurar propriedade").expect("property search");
    click(&mut a, &c, search.center(), false);
    frame(&mut a, &c, vec![Event::Text("abc".into())], false);
    assert!(c.wants_keyboard_input());
    frame(
        &mut a,
        &c,
        vec![Event::Key {
            key: Key::Z,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::CTRL,
        }],
        true,
    );
    assert_eq!(before, a.session.snapshot().unwrap());
    assert_eq!(a.session.history.undo_len(), 1);
}
#[test]
fn multi_inspector_displays_mixed_values_and_tracks_undo() {
    let (mut a, c, x, y) = setup();
    a.session.select_graphics([x, y]);
    let out = frame(&mut a, &c, vec![], false);
    assert!(text_rect(&out, "2 representações selecionadas").is_some());
    assert!(text_rect(&out, "Valores diferentes").is_some());
    a.session
        .dispatch_graphic(GraphicEdit::Translate(4.0, 4.0))
        .unwrap();
    a.execute(CommandId::Undo, &c);
    let out = frame(&mut a, &c, vec![], false);
    assert!(text_rect(&out, "2 representações selecionadas").is_some());
}

#[test]
fn inspector_numeric_drag_waits_for_apply_then_undo_refreshes_coordinates() {
    let (mut a, c, x, _) = setup();
    a.session.select_graphics([x]);
    let before = a.session.snapshot().unwrap();
    let out = frame(&mut a, &c, vec![], false);
    let x_label = text_rect(&out, "X (mm)").unwrap();
    let input = out
        .shapes
        .iter()
        .find_map(|s| {
            if let egui::Shape::Text(t) = &s.shape {
                (t.pos.x > x_label.right()
                    && (t.pos.y - x_label.top()).abs() < 4.0
                    && t.galley.text().parse::<f64>().is_ok())
                .then_some(t.pos + t.galley.size() * 0.5)
            } else {
                None
            }
        })
        .expect("X numeric control");
    frame(
        &mut a,
        &c,
        vec![
            Event::PointerMoved(input),
            button(input, PointerButton::Primary, true, false),
        ],
        false,
    );
    frame(
        &mut a,
        &c,
        vec![Event::PointerMoved(input + vec2(50.0, 0.0))],
        false,
    );
    frame(
        &mut a,
        &c,
        vec![button(
            input + vec2(50.0, 0.0),
            PointerButton::Primary,
            false,
            false,
        )],
        false,
    );
    assert_eq!(a.session.snapshot().unwrap(), before);
    assert_eq!(a.session.history.undo_len(), 0);
    let out = frame(&mut a, &c, vec![], false);
    let apply = text_rect(&out, "Aplicar transformação").unwrap();
    click(&mut a, &c, apply.center(), false);
    assert_ne!(a.session.snapshot().unwrap(), before);
    assert_eq!(a.session.history.undo_len(), 1);
    a.execute(CommandId::Undo, &c);
    frame(&mut a, &c, vec![], false);
    assert_eq!(a.session.snapshot().unwrap(), before);
    let out = frame(&mut a, &c, vec![], false);
    let apply = text_rect(&out, "Aplicar transformação").unwrap();
    click(&mut a, &c, apply.center(), false);
    assert_eq!(a.session.snapshot().unwrap(), before);
    assert!(a.session.history.can_redo());
}
#[test]
fn graphical_input_keeps_hit_tolerance_in_logical_points_across_dpi_scales() {
    for scale in [1.0, 1.25, 1.5, 2.0] {
        let (mut a, c, x, _) = setup();
        c.set_zoom_factor(scale);
        for _ in 0..3 {
            frame(&mut a, &c, vec![], false);
        }
        a.execute(CommandId::FitPage, &c);
        frame(&mut a, &c, vec![], false);
        let p = pos(&a, x) - vec2(5.0, 0.0);
        click(&mut a, &c, p, false);
        assert!(
            a.session.graphics.selection.ids().contains(&x),
            "scale={scale}"
        );
    }
}
#[test]
fn contextual_cursors_distinguish_hover_drag_and_pan() {
    let (mut a, c, x, _) = setup();
    let p = pos(&a, x);
    let out = frame(&mut a, &c, vec![Event::PointerMoved(p)], false);
    assert_eq!(
        out.platform_output.cursor_icon,
        egui::CursorIcon::PointingHand
    );
    frame(
        &mut a,
        &c,
        vec![button(p, PointerButton::Primary, true, false)],
        false,
    );
    let out = frame(
        &mut a,
        &c,
        vec![Event::PointerMoved(p + vec2(25.0, 0.0))],
        false,
    );
    assert_eq!(out.platform_output.cursor_icon, egui::CursorIcon::Move);
    let out = frame(
        &mut a,
        &c,
        vec![button(
            p + vec2(25.0, 0.0),
            PointerButton::Middle,
            true,
            false,
        )],
        false,
    );
    assert!(matches!(
        out.platform_output.cursor_icon,
        egui::CursorIcon::Grab | egui::CursorIcon::Grabbing
    ));
    assert!(!a.session.graphics.is_active());
}
