// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_craft::{domain::*, persistence};
use cad_gui::{
    app::CadApp,
    commands::{CommandId, registry},
    interaction::DevicePlan,
    preferences::RibbonTab,
};
use eframe::egui::{self, Context, Event, Modifiers, PointerButton, Pos2, Rect, vec2};

fn frame(app: &mut CadApp, ctx: &Context, events: Vec<Event>) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1600., 900.))),
            events,
            ..Default::default()
        },
        |ctx| app.show(ctx),
    )
}
fn nodes(output: &egui::FullOutput) -> Vec<(String, Pos2)> {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .into_iter()
        .flat_map(|u| u.nodes.iter())
        .filter_map(|(_, n)| {
            let b = n.bounds()?;
            Some((
                n.label().unwrap_or_default().to_owned(),
                Pos2::new(((b.x0 + b.x1) / 2.) as f32, ((b.y0 + b.y1) / 2.) as f32),
            ))
        })
        .collect()
}
fn click(app: &mut CadApp, ctx: &Context, label: &str) {
    let mut out = frame(app, ctx, vec![]);
    for _ in 0..7 {
        if nodes(&out)
            .iter()
            .any(|(name, p)| name == label && (170.0..890.0).contains(&p.y))
        {
            break;
        }
        out = frame(
            app,
            ctx,
            vec![
                Event::PointerMoved(Pos2::new(700.0, 500.0)),
                Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: vec2(0.0, -280.0),
                    modifiers: Modifiers::NONE,
                },
            ],
        );
    }
    let available = nodes(&out);
    let at = available
        .iter()
        .find(|(name, p)| name == label && (170.0..890.0).contains(&p.y))
        .map(|(_, p)| *p)
        .unwrap_or_else(|| {
            panic!(
                "widget {label:?} não encontrado: {:?}",
                available
                    .iter()
                    .filter(|(n, _)| n == label)
                    .collect::<Vec<_>>()
            )
        });
    let event = |pressed| Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    frame(app, ctx, vec![Event::PointerMoved(at), event(true)]);
    frame(app, ctx, vec![event(false)]);
    frame(app, ctx, vec![]);
}

#[test]
fn part_window_assigns_library_part_and_undo_restores_project() {
    let mut project = Project::new("P").unwrap();
    let device = Device::new(project.id(), Designation::new("K1").unwrap(), "Contator").unwrap();
    project.insert_device(device).unwrap();
    let part = Part::new("ACME.K1", "ACME", "K1", "Contator", "Contator")
        .unwrap()
        .with_functions(vec![
            PartFunction::new(FunctionKind::Coil, FunctionRole::Main, ["A1", "A2"]).unwrap(),
            PartFunction::new(
                FunctionKind::MainContact,
                FunctionRole::Auxiliary,
                ["1", "2"],
            )
            .unwrap(),
            PartFunction::new(
                FunctionKind::MainContact,
                FunctionRole::Auxiliary,
                ["3", "4"],
            )
            .unwrap(),
            PartFunction::new(
                FunctionKind::MainContact,
                FunctionRole::Auxiliary,
                ["5", "6"],
            )
            .unwrap(),
            PartFunction::new(
                FunctionKind::AuxiliaryContact,
                FunctionRole::Auxiliary,
                ["13", "14"],
            )
            .unwrap(),
        ])
        .unwrap();
    let dir = std::env::temp_dir().join(format!(
        "cadcraft-parts-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("K1.cadpart"),
        persistence::write_part_package(&part).unwrap(),
    )
    .unwrap();
    let mut app = CadApp::default();
    app.session.replace(project, false).unwrap();
    app.preferences.part_library = dir.to_string_lossy().into_owned();
    app.part_editor.open = true;
    let ctx = Context::default();
    ctx.enable_accesskit();
    frame(&mut app, &ctx, vec![]);
    click(&mut app, &ctx, "Dispositivo -K1");
    click(&mut app, &ctx, "Peça ACME.K1");
    // The preview lists what the assignment creates before anything changes.
    let out = frame(&mut app, &ctx, vec![]);
    let shown: Vec<String> = out
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, n)| n.label().or(n.value()).map(str::to_owned))
        .collect();
    assert!(
        shown.iter().any(|t| t == "Bobina (A1, A2) → nova função"),
        "{:?}",
        shown.iter().filter(|t| t.contains('→')).collect::<Vec<_>>()
    );
    assert_eq!(app.session.project().unwrap().functions().count(), 0);
    click(&mut app, &ctx, "Atribuir peça");
    assert_eq!(app.session.project().unwrap().bom_rows()[0].quantity, 1);
    assert_eq!(app.session.project().unwrap().functions().count(), 5);
    assert_eq!(
        app.session.project().unwrap().connection_points().count(),
        10
    );
    assert_eq!(app.session.project().unwrap().representations().count(), 0);
    frame(
        &mut app,
        &ctx,
        vec![Event::Key {
            key: egui::Key::Z,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::CTRL,
        }],
    );
    assert!(app.session.project().unwrap().bom_rows().is_empty());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn parts_command_is_in_master_data() {
    assert_eq!(
        registry()
            .iter()
            .find(|c| c.id == CommandId::PartEditor)
            .unwrap()
            .tab,
        RibbonTab::MasterData
    );
}

#[test]
fn existing_coil_is_placed_by_real_canvas_click_and_undo_restores_points() {
    let mut project = Project::new("P").unwrap();
    let document = project
        .insert_document(Document::new(project.id(), "D").unwrap())
        .unwrap();
    let page = project
        .insert_page(
            Page::new(
                project.id(),
                document,
                "P",
                "1",
                PageKind::MultiLine,
                Size2D::new(420.0, 297.0).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let device = project
        .insert_device(
            Device::new(project.id(), Designation::new("K1").unwrap(), "Contator").unwrap(),
        )
        .unwrap();
    let part = Part::new("ACME.K1", "ACME", "K1", "Contator", "Contator")
        .unwrap()
        .with_functions(vec![
            PartFunction::new(FunctionKind::Coil, FunctionRole::Main, ["A1", "A2"]).unwrap(),
        ])
        .unwrap();
    project.assign_part(device, part, 1).unwrap();
    let function = project.functions().next().unwrap().id();
    let original_points: Vec<_> = project
        .connection_points()
        .map(ConnectionPoint::id)
        .collect();
    let mut app = CadApp::default();
    app.session.replace(project, false).unwrap();
    app.session.open_page(page).unwrap();
    let ctx = Context::default();
    ctx.enable_accesskit();
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    app.session
        .begin_device_insertion(
            SymbolDefinitionId::new("cad-craft.coil").unwrap(),
            DevicePlan::ExistingFunction(function),
            Transform2D::new(Point2D::new(0.0, 0.0).unwrap(), 0.0, false, false).unwrap(),
        )
        .unwrap();
    let at = app.last_viewport.unwrap().center();
    let button = |pressed| Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    frame(&mut app, &ctx, vec![Event::PointerMoved(at)]);
    frame(&mut app, &ctx, vec![button(true)]);
    frame(&mut app, &ctx, vec![button(false)]);
    let after = app.session.project().unwrap();
    assert_eq!(after.connection_points().count(), 2);
    assert_eq!(after.function_references(function).unwrap().len(), 1);
    assert_eq!(
        after
            .connection_points()
            .map(ConnectionPoint::id)
            .collect::<Vec<_>>(),
        original_points
    );
    app.execute(CommandId::Undo, &ctx);
    assert!(
        app.session
            .project()
            .unwrap()
            .function_references(function)
            .unwrap()
            .is_empty()
    );
    app.execute(CommandId::Redo, &ctx);
    assert_eq!(
        app.session
            .project()
            .unwrap()
            .function_references(function)
            .unwrap()
            .len(),
        1
    );
}
