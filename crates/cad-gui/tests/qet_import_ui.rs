// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 25: Arquivo > Importar > Projeto QElectroTech com eventos egui
//! reais: Procurar, Analisar em segundo plano, prévia, criar projeto novo e
//! abrir as Verificações; arquivo inválido; confirmação com alterações
//! pendentes.
use cad_gui::{app::CadApp, commands::CommandId, project_file::ScriptedPicker};
use eframe::egui::{self, Context, Event, Modifiers, PointerButton, Pos2, Rect, vec2};
use std::path::PathBuf;
use std::sync::Mutex;

static TEMP_LOCK: Mutex<()> = Mutex::new(());

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
fn labels(out: &egui::FullOutput) -> Vec<(String, egui::accesskit::Role, Option<Pos2>)> {
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
fn click(app: &mut CadApp, ctx: &Context, label: &str) {
    use egui::accesskit::Role;
    frame(app, ctx, vec![]);
    let out = frame(app, ctx, vec![]);
    let pos = labels(&out)
        .into_iter()
        .filter(|(l, role, _)| {
            l == label && !matches!(role, Role::Label | Role::TextRun | Role::Window)
        })
        .filter_map(|(_, _, p)| p)
        .max_by(|a, b| a.y.total_cmp(&b.y))
        .unwrap_or_else(|| panic!("controle {label} ausente"));
    let event = |pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    frame(app, ctx, vec![Event::PointerMoved(pos)]);
    frame(app, ctx, vec![event(true)]);
    frame(app, ctx, vec![event(false)]);
    frame(app, ctx, vec![]);
}
fn shows(app: &mut CadApp, ctx: &Context, text: &str) -> bool {
    let out = frame(app, ctx, vec![]);
    labels(&out).iter().any(|(l, _, _)| l.contains(text))
}
fn wait_analysis(app: &mut CadApp, ctx: &Context) {
    for _ in 0..1000 {
        frame(app, ctx, vec![]);
        if !app.qet.busy() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("análise não terminou");
}

const QET: &str = r#"<project title="Painel QET" version="0.90">
<diagram order="1" title="Força" cols="10" colsize="60" rows="6" rowsize="80"><elements>
<element type="embed://import/lamp.elmt" uuid="{b0000000-0000-0000-0000-000000000001}" x="100" y="200" orientation="0"><terminals><terminal id="0" x="0" y="-20" orientation="0"/><terminal id="1" x="0" y="20" orientation="2"/></terminals><elementInformations><elementInformation name="label">-H1</elementInformation></elementInformations></element>
<element type="embed://import/lamp.elmt" uuid="{b0000000-0000-0000-0000-000000000002}" x="300" y="200" orientation="0"><terminals><terminal id="2" x="0" y="-20" orientation="0"/><terminal id="3" x="0" y="20" orientation="2"/></terminals><elementInformations><elementInformation name="label">-H2</elementInformation></elementInformations></element>
</elements><conductors><conductor terminal1="1" terminal2="2" num="5"/></conductors></diagram>
<diagram order="2" title="Comando" cols="10" colsize="60" rows="6" rowsize="80"/>
<collection><category name="import"><element name="lamp.elmt"><definition version="0.90" type="element" link_type="simple" width="20" height="40" hotspot_x="10" hotspot_y="20"><names><name lang="en">lamp</name></names><description><rect x="-5" y="-10" width="10" height="20" style="line-style:normal;line-weight:normal;filling:none;color:black"/><terminal name="X1" x="0" y="-20" orientation="n"/><terminal name="X2" x="0" y="20" orientation="s"/></description></definition></element></category></collection>
</project>"#;

fn temp(name: &str, contents: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cad-craft-qet-ui-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, contents).unwrap();
    path
}

fn app_with(paths: Vec<PathBuf>) -> (CadApp, Context) {
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut app = CadApp::default();
    app.files.picker = Box::new(ScriptedPicker {
        qet_open: paths.into(),
        ..Default::default()
    });
    frame(&mut app, &ctx, vec![]);
    (app, ctx)
}

/// Procurar → Analisar (segundo plano) → prévia → Criar projeto importado →
/// projeto novo aberto na 1ª folha, pendente de salvar, Verificações abertas.
#[test]
fn browse_analyse_preview_and_create_project_with_checks() {
    let _guard = TEMP_LOCK.lock().unwrap();
    let path = temp("painel.qet", QET);
    let (mut app, ctx) = app_with(vec![path.clone()]);
    app.execute(CommandId::ImportQet, &ctx);
    assert!(shows(&mut app, &ctx, "Importar projeto QElectroTech"));
    click(&mut app, &ctx, "Procurar…");
    assert_eq!(app.qet.path, path.display().to_string());
    click(&mut app, &ctx, "Analisar");
    wait_analysis(&mut app, &ctx);
    assert!(
        shows(&mut app, &ctx, "2 folha(s), 2 dispositivo(s), 1 fio(s)"),
        "{}",
        app.qet.status
    );
    assert!(shows(&mut app, &ctx, "Projeto \"Painel QET\""));
    assert!(app.session.project().is_none(), "a prévia não cria projeto");
    click(&mut app, &ctx, "Criar projeto importado");
    let project = app.session.project().expect("projeto importado");
    assert_eq!(project.name().as_str(), "Painel QET");
    assert_eq!(project.devices().count(), 2);
    assert!(app.session.active.is_some(), "1ª folha aberta");
    assert!(app.session.dirty, "projeto novo, ainda sem arquivo");
    assert!(app.session.file.is_none());
    for _ in 0..1000 {
        frame(&mut app, &ctx, vec![]);
        if !app.checks.is_running() && app.checks.report.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(app.checks.report.is_some(), "Verificações executadas");
}

/// Arquivo inválido: falha relatada, sem prévia nem projeto.
#[test]
fn invalid_file_reports_failure_without_preview() {
    let _guard = TEMP_LOCK.lock().unwrap();
    let path = temp("quebrado.qet", "<project><nada/></project>");
    let (mut app, ctx) = app_with(vec![]);
    app.execute(CommandId::ImportQet, &ctx);
    app.qet.path = path.display().to_string();
    click(&mut app, &ctx, "Analisar");
    wait_analysis(&mut app, &ctx);
    assert!(app.qet.preview.is_none());
    assert!(shows(&mut app, &ctx, "Falha:"), "{}", app.qet.status);
    assert!(app.session.project().is_none());
    // Extensão errada também é recusada.
    app.qet.path = temp("x.txt", QET).display().to_string();
    click(&mut app, &ctx, "Analisar");
    wait_analysis(&mut app, &ctx);
    assert!(app.qet.status.contains(".qet"), "{}", app.qet.status);
}

/// Com alterações pendentes, criar o projeto importado pede confirmação e
/// mantém o projeto atual até a resposta.
#[test]
fn pending_changes_ask_before_replacing_the_session() {
    let _guard = TEMP_LOCK.lock().unwrap();
    let path = temp("painel2.qet", QET);
    let (mut app, ctx) = app_with(vec![]);
    app.session
        .replace(cad_craft::domain::Project::new("Atual").unwrap(), false)
        .unwrap();
    app.session.dirty = true;
    app.qet.open_checks = false;
    app.execute(CommandId::ImportQet, &ctx);
    app.qet.path = path.display().to_string();
    click(&mut app, &ctx, "Analisar");
    wait_analysis(&mut app, &ctx);
    click(&mut app, &ctx, "Criar projeto importado");
    assert_eq!(app.session.project().unwrap().name().as_str(), "Atual");
    assert!(app.dialog_open(), "confirmação de descarte");
    assert!(app.checks.report.is_none());
}
