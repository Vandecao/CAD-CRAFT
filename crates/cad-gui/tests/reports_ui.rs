// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_craft::domain::*;
use cad_gui::{app::CadApp, commands::CommandId, project_file::ScriptedPicker};
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
fn labels(out: &egui::FullOutput) -> Vec<String> {
    out.platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, n)| n.label().or_else(|| n.value()).map(str::to_owned))
        .collect()
}
fn click(app: &mut CadApp, ctx: &Context, label: &str) {
    frame(app, ctx, vec![]);
    let out = frame(app, ctx, vec![]);
    let bounds = out
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, n)| {
            n.label() == Some(label)
                && !matches!(
                    n.role(),
                    egui::accesskit::Role::Label
                        | egui::accesskit::Role::TextRun
                        | egui::accesskit::Role::Window
                )
        })
        .filter_map(|(_, n)| n.bounds())
        .max_by(|a, b| a.y0.total_cmp(&b.y0))
        .unwrap_or_else(|| panic!("{label} ausente: {:?}", labels(&out)));
    let pos = Pos2::new(
        ((bounds.x0 + bounds.x1) / 2.) as f32,
        ((bounds.y0 + bounds.y1) / 2.) as f32,
    );
    let event = |pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    frame(app, ctx, vec![Event::PointerMoved(pos), event(true)]);
    frame(app, ctx, vec![event(false)]);
    frame(app, ctx, vec![]);
}
fn app() -> (CadApp, Context) {
    let mut project = Project::new("Relatórios").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "Esquema").unwrap())
        .unwrap();
    project
        .insert_page(
            Page::new(
                project.id(),
                doc,
                "Comando",
                "1",
                PageKind::MultiLine,
                PaperFormat::A3.size(true),
            )
            .unwrap(),
        )
        .unwrap();
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut app = CadApp::default();
    app.session.replace(project, false).unwrap();
    frame(&mut app, &ctx, vec![]);
    (app, ctx)
}
fn wait_for(app: &mut CadApp, ctx: &Context, done: impl Fn(&CadApp) -> bool) {
    for _ in 0..200 {
        frame(app, ctx, vec![]);
        if done(app) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("tarefa de relatório não terminou");
}

#[test]
fn generate_and_undo_from_accesskit_clicks() {
    let (mut app, ctx) = app();
    app.execute(CommandId::GenerateReport, &ctx);
    let out = frame(&mut app, &ctx, vec![]);
    assert!(labels(&out).iter().any(|s| s.contains("Prévia: 1 página")));
    click(&mut app, &ctx, "Gerar relatório");
    wait_for(&mut app, &ctx, |app| {
        app.session
            .project()
            .unwrap()
            .pages()
            .any(|p| p.report().is_some())
    });
    assert!(
        app.session
            .active
            .and_then(|id| app.session.project().unwrap().page(id))
            .is_some_and(|p| p.report().is_some())
    );
    assert_eq!(
        app.session
            .project()
            .unwrap()
            .pages()
            .filter(|p| p.report().is_some())
            .count(),
        1
    );
    app.execute(CommandId::Undo, &ctx);
    assert_eq!(
        app.session
            .project()
            .unwrap()
            .pages()
            .filter(|p| p.report().is_some())
            .count(),
        0
    );
    app.execute(CommandId::Redo, &ctx);
    assert_eq!(
        app.session
            .project()
            .unwrap()
            .pages()
            .filter(|p| p.report().is_some())
            .count(),
        1
    );
}

#[test]
fn exports_csv_and_xlsx_through_file_picker() {
    let (mut app, ctx) = app();
    let dir = tempfile::tempdir().unwrap();
    let csv = dir.path().join("lista.csv");
    let xlsx = dir.path().join("lista.xlsx");
    app.files.picker = Box::new(ScriptedPicker {
        list_save: [csv.clone(), xlsx.clone()].into(),
        ..Default::default()
    });
    app.execute(CommandId::ExportList, &ctx);
    click(&mut app, &ctx, "Exportar lista");
    wait_for(&mut app, &ctx, |_| csv.exists());
    assert!(
        std::fs::read(&csv)
            .unwrap()
            .starts_with(&[0xEF, 0xBB, 0xBF])
    );
    click(&mut app, &ctx, "XLSX");
    click(&mut app, &ctx, "Exportar lista");
    wait_for(&mut app, &ctx, |_| xlsx.exists());
    assert!(std::fs::read(&xlsx).unwrap().starts_with(b"PK"));
}

#[test]
fn update_dialog_adds_page_and_undo_restores_it() {
    let mut project = Project::new("Atualizar").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "D").unwrap())
        .unwrap();
    let schematic = project
        .insert_page(
            Page::new(
                project.id(),
                doc,
                "P",
                "1",
                PageKind::MultiLine,
                PaperFormat::A3.size(true),
            )
            .unwrap(),
        )
        .unwrap();
    let device = project
        .insert_device(
            Device::new(project.id(), Designation::new("K1").unwrap(), "Contator").unwrap(),
        )
        .unwrap();
    let function = project
        .insert_function(Function::new(
            project.id(),
            device,
            Designation::new("K1").unwrap(),
            FunctionKind::Coil,
        ))
        .unwrap();
    let anchor = project
        .insert_connection_point(ConnectionPoint::new(
            project.id(),
            function,
            DesignationPart::new("A0").unwrap(),
        ))
        .unwrap();
    project
        .generate_report(&ReportRequest::new(
            ReportKind::Connections,
            doc,
            PaperFormat::A3.size(true),
        ))
        .unwrap();
    let report_page = project.pages().find(|p| p.report().is_some()).unwrap().id();
    for i in 0..45 {
        let point = project
            .insert_connection_point(ConnectionPoint::new(
                project.id(),
                function,
                DesignationPart::new(format!("A{}", i + 1)).unwrap(),
            ))
            .unwrap();
        project
            .insert_connection(Connection::new(project.id(), anchor, point).unwrap())
            .unwrap();
    }
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut app = CadApp::default();
    app.session.replace(project, false).unwrap();
    app.session.open_page(report_page).unwrap();
    frame(&mut app, &ctx, vec![]);
    app.execute(CommandId::UpdateReport, &ctx);
    assert!(
        labels(&frame(&mut app, &ctx, vec![]))
            .iter()
            .any(|s| s.contains("1 relatório(s) desatualizado"))
    );
    click(&mut app, &ctx, "Atualizar selecionado");
    wait_for(&mut app, &ctx, |app| {
        app.session
            .project()
            .unwrap()
            .pages()
            .filter(|p| p.report().is_some())
            .count()
            == 2
    });
    assert_eq!(
        app.session
            .project()
            .unwrap()
            .pages()
            .filter(|p| p.report().is_some())
            .count(),
        2
    );
    app.execute(CommandId::Undo, &ctx);
    assert_eq!(
        app.session
            .project()
            .unwrap()
            .pages()
            .filter(|p| p.report().is_some())
            .count(),
        1
    );
    assert!(app.session.project().unwrap().page(schematic).is_some());
}
