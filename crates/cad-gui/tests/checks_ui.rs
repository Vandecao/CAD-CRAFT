// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 24: painel Verificações com eventos egui reais: Verificar projeto,
//! duplo clique navega, F8/Shift+F8, reavaliação após correção, Ignorar com
//! justificativa (Desfazer) e exportação CSV.
use cad_craft::domain::*;
use cad_gui::{app::CadApp, commands::CommandId, project_file::ScriptedPicker, state::Selection};
use eframe::egui::{self, Context, Event, Key, Modifiers, PointerButton, Pos2, Rect, vec2};

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
fn nodes(out: &egui::FullOutput) -> Vec<(String, egui::accesskit::Role, Option<Pos2>)> {
    out.platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .map(|(_, n)| {
            (
                n.label()
                    .or_else(|| n.value())
                    .unwrap_or_default()
                    .to_owned(),
                n.role(),
                n.bounds()
                    .map(|b| Pos2::new(((b.x0 + b.x1) / 2.) as f32, ((b.y0 + b.y1) / 2.) as f32)),
            )
        })
        .collect()
}
/// Posição do controle com o rótulo (ignora títulos e textos; o mais baixo
/// quando a faixa repete o nome do comando).
fn control(out: &egui::FullOutput, label: &str, starts: bool) -> Pos2 {
    use egui::accesskit::Role;
    nodes(out)
        .into_iter()
        .filter(|(l, role, _)| {
            (if starts {
                l.starts_with(label)
            } else {
                l == label
            }) && !matches!(role, Role::Label | Role::TextRun | Role::Window)
        })
        .filter_map(|(_, _, p)| p)
        .max_by(|a, b| a.y.total_cmp(&b.y))
        .unwrap_or_else(|| {
            panic!(
                "controle {label} ausente: {:?}",
                nodes(out)
                    .into_iter()
                    .map(|n| n.0)
                    .filter(|l| !l.is_empty())
                    .collect::<Vec<_>>()
            )
        })
}
fn press(app: &mut CadApp, ctx: &Context, pos: Pos2, clicks: usize) {
    let event = |pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    frame(app, ctx, vec![Event::PointerMoved(pos)]);
    for _ in 0..clicks {
        frame(app, ctx, vec![event(true)]);
        frame(app, ctx, vec![event(false)]);
    }
    frame(app, ctx, vec![]);
}
fn click(app: &mut CadApp, ctx: &Context, label: &str) {
    frame(app, ctx, vec![]);
    let out = frame(app, ctx, vec![]);
    let pos = control(&out, label, false);
    press(app, ctx, pos, 1);
}
fn key(app: &mut CadApp, ctx: &Context, key: Key, modifiers: Modifiers) {
    let ev = |pressed| Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers,
    };
    frame(app, ctx, vec![ev(true)]);
    frame(app, ctx, vec![ev(false)]);
    frame(app, ctx, vec![]);
}
fn wait(app: &mut CadApp, ctx: &Context) {
    for _ in 0..1000 {
        frame(app, ctx, vec![]);
        if !app.checks.is_running() && app.checks.report.is_some() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("verificação não terminou");
}
fn rules(app: &CadApp) -> Vec<&'static str> {
    app.checks
        .report
        .as_ref()
        .unwrap()
        .messages
        .iter()
        .map(|m| m.rule)
        .collect()
}

/// Duas páginas; na 2, uma bobina sem fios (VER-007 ×2) e dois fios lógicos,
/// um numerado (VER-008 para o outro). Dispositivos sem peça (VER-018)
/// desativados no esquema para a lista ficar curta.
fn project() -> (Project, PageId, RepresentationId, ConnectionId) {
    let mut p = Project::new("Verificar").unwrap();
    let doc = p
        .insert_document(Document::new(p.id(), "Esquema").unwrap())
        .unwrap();
    let pages = ["1", "2"].map(|n| {
        p.insert_page(
            Page::new(
                p.id(),
                doc,
                format!("P{n}"),
                n,
                PageKind::MultiLine,
                PaperFormat::A4.size(true),
            )
            .unwrap(),
        )
        .unwrap()
    });
    let coil = SymbolLibrary::builtin()
        .get(&SymbolDefinitionId::new("cad-craft.coil").unwrap())
        .unwrap()
        .clone();
    let placed = p
        .place_symbol(SymbolPlacement {
            page: pages[1],
            definition: coil,
            transform: Transform2D::new(Point2D::new(100.0, 100.0).unwrap(), 0.0, false, false)
                .unwrap(),
            device: PlacementDevice::New {
                designation: Designation::new("K1").unwrap(),
                classification: "Contator".into(),
            },
            function_kind: FunctionKind::Coil,
        })
        .unwrap();
    let device = p
        .insert_device(Device::new(p.id(), Designation::new("K9").unwrap(), "teste").unwrap())
        .unwrap();
    let function = p
        .insert_function(Function::new(
            p.id(),
            device,
            Designation::new("K9").unwrap(),
            FunctionKind::Coil,
        ))
        .unwrap();
    let pts: Vec<_> = (1..=4)
        .map(|i| {
            p.insert_connection_point(ConnectionPoint::new(
                p.id(),
                function,
                DesignationPart::new(i.to_string()).unwrap(),
            ))
            .unwrap()
        })
        .collect();
    let a = p
        .insert_connection(Connection::new(p.id(), pts[0], pts[1]).unwrap())
        .unwrap();
    let b = p
        .insert_connection(Connection::new(p.id(), pts[2], pts[3]).unwrap())
        .unwrap();
    p.set_connection_number(a, Some(Text::new("1").unwrap()), false)
        .unwrap();
    let mut scheme = p.check_scheme().clone();
    scheme.rules.insert(
        "VER-018".into(),
        RuleSetting {
            enabled: false,
            severity: CheckSeverity::Info,
        },
    );
    p.set_check_scheme(scheme).unwrap();
    (p, pages[1], placed.representation_id(), b)
}

fn app() -> (CadApp, Context, PageId, RepresentationId) {
    let (project, page, rep, _) = project();
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut app = CadApp::default();
    app.session.replace(project, false).unwrap();
    frame(&mut app, &ctx, vec![]);
    (app, ctx, page, rep)
}

/// Critério 6: Verificar projeto, contadores, duplo clique navega até a
/// ocorrência, F8/Shift+F8.
#[test]
fn check_project_lists_messages_and_double_click_navigates() {
    let (mut app, ctx, page, rep) = app();
    app.execute(CommandId::CheckProject, &ctx);
    wait(&mut app, &ctx);
    let found = rules(&app);
    assert_eq!(
        found.iter().filter(|r| **r == "VER-007").count(),
        2,
        "{found:?}"
    );
    assert!(found.contains(&"VER-008"));
    let out = frame(&mut app, &ctx, vec![]);
    assert!(
        nodes(&out)
            .iter()
            .any(|(l, _, _)| l.starts_with("0 erro(s), 3 aviso(s)")),
        "{:?}",
        nodes(&out)
            .into_iter()
            .map(|n| n.0)
            .filter(|l| l.contains("erro"))
            .collect::<Vec<_>>()
    );
    let pos = control(&out, "VER-007", true);
    press(&mut app, &ctx, pos, 2);
    assert_eq!(app.session.active, Some(page));
    assert_eq!(app.session.selection, Selection::Representation(rep));
    let first = app.checks.selected.unwrap();
    key(&mut app, &ctx, Key::F8, Modifiers::NONE);
    assert_ne!(app.checks.selected, Some(first));
    key(&mut app, &ctx, Key::F8, Modifiers::SHIFT);
    assert_eq!(app.checks.selected, Some(first));
}

/// Critério 6: corrigir o problema retira a mensagem sem verificar de novo.
#[test]
fn fixing_the_problem_removes_the_message_automatically() {
    let (mut app, ctx, _, _) = app();
    app.execute(CommandId::CheckProject, &ctx);
    wait(&mut app, &ctx);
    assert!(rules(&app).contains(&"VER-008"));
    let preview = app
        .session
        .project()
        .unwrap()
        .preview_connection_numbering()
        .unwrap();
    app.session.number_connections(&preview).unwrap();
    for _ in 0..1000 {
        frame(&mut app, &ctx, vec![]);
        if !app.checks.is_running() && !rules(&app).contains(&"VER-008") {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let after = rules(&app);
    assert!(!after.contains(&"VER-008"), "{after:?}");
    assert_eq!(after.iter().filter(|r| **r == "VER-007").count(), 2);
}

/// Critério 5/6: Ignorar com justificativa pelo painel; Desfazer restaura;
/// exportar CSV.
#[test]
fn ignore_with_reason_is_undoable_and_list_exports_csv() {
    let dir = tempfile::tempdir().unwrap();
    let csv = dir.path().join("verificacoes.csv");
    let (mut app, ctx, _, _) = app();
    app.files.picker = Box::new(ScriptedPicker {
        list_save: [csv.clone()].into(),
        ..Default::default()
    });
    app.execute(CommandId::CheckProject, &ctx);
    wait(&mut app, &ctx);
    frame(&mut app, &ctx, vec![]);
    let out = frame(&mut app, &ctx, vec![]);
    let pos = control(&out, "VER-008", true);
    press(&mut app, &ctx, pos, 1);
    click(&mut app, &ctx, "Ignorar…");
    let out = frame(&mut app, &ctx, vec![]);
    // Campo na mesma linha do rótulo "Justificativa", logo à direita dele.
    let label = nodes(&out)
        .into_iter()
        .find(|(l, _, _)| l == "Justificativa")
        .and_then(|(_, _, p)| p)
        .unwrap();
    let field = nodes(&out)
        .into_iter()
        .filter(|(_, role, _)| *role == egui::accesskit::Role::TextInput)
        .filter_map(|(_, _, p)| p)
        .filter(|p| (p.y - label.y).abs() < 10.0 && p.x > label.x)
        .min_by(|a, b| a.x.total_cmp(&b.x))
        .unwrap();
    press(&mut app, &ctx, field, 1);
    frame(&mut app, &ctx, vec![Event::Text("Fio de reserva".into())]);
    click(&mut app, &ctx, "Confirmar ignorar");
    let scheme = app.session.project().unwrap().check_scheme().clone();
    assert_eq!(scheme.ignored.len(), 1);
    assert_eq!(scheme.ignored[0].reason.as_str(), "Fio de reserva");
    for _ in 0..1000 {
        frame(&mut app, &ctx, vec![]);
        if !app.checks.is_running() && !rules(&app).contains(&"VER-008") {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(!rules(&app).contains(&"VER-008"));
    app.execute(CommandId::Undo, &ctx);
    assert!(
        app.session
            .project()
            .unwrap()
            .check_scheme()
            .ignored
            .is_empty()
    );
    click(&mut app, &ctx, "Exportar CSV");
    let bytes = std::fs::read(&csv).unwrap();
    assert!(bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
    let text = String::from_utf8(bytes[3..].to_vec()).unwrap();
    assert!(text.starts_with("Código;Gravidade;Regra;Mensagem;Local"));
    assert!(text.contains("VER-007"));
}
