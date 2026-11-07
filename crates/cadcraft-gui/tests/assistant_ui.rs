// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Assistant tab with real egui events and a scripted provider
//! (no network): open from the status bar, ask, see the answer, show a
//! command in the ribbon, go to a page, apply a proposal and undo it; and
//! the OAuth helpers (PKCE, callback, authorization URL, no secrets kept).
// Fixture helpers use validated synthetic inputs and fail immediately on setup errors.
#![allow(clippy::unwrap_used)]
use cad_craft::assistant::conversation::{Provider, Request};
use cad_gui::assistant::oauth;
use cad_gui::{app::CadApp, editing::GraphicEdit, preferences::RibbonTab};
use eframe::egui::{self, Context, Event, Modifiers, PointerButton, Pos2, Rect, vec2};
use std::sync::atomic::AtomicBool;

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
fn click(app: &mut CadApp, ctx: &Context, label: &str) {
    use egui::accesskit::Role;
    frame(app, ctx, vec![]);
    let out = frame(app, ctx, vec![]);
    let pos = nodes(&out)
        .into_iter()
        .filter(|(l, role, _)| {
            l == label && !matches!(role, Role::Label | Role::TextRun | Role::Window)
        })
        .filter_map(|(_, _, p)| p)
        .max_by(|a, b| a.y.total_cmp(&b.y))
        .unwrap_or_else(|| {
            panic!(
                "controle {label} ausente: {:?}",
                nodes(&out)
                    .into_iter()
                    .map(|n| n.0)
                    .filter(|l| !l.is_empty())
                    .collect::<Vec<_>>()
            )
        });
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
    nodes(&out).iter().any(|(l, _, _)| l.contains(text))
}
fn wait(app: &mut CadApp, ctx: &Context) {
    for _ in 0..1000 {
        frame(app, ctx, vec![]);
        if !app.assistant.busy() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("resposta não chegou");
}

/// Provider that answers with a fixed script per question.
struct Script(Vec<String>);
impl Provider for Script {
    fn respond(
        &mut self,
        _: &Request<'_>,
        on_text: &mut dyn FnMut(&str),
        _: &AtomicBool,
    ) -> Result<String, String> {
        let answer = if self.0.is_empty() {
            String::new()
        } else {
            self.0.remove(0)
        };
        on_text(&answer);
        Ok(answer)
    }
}

fn block(json: &str) -> String {
    format!("```cadcraft\n{json}\n```")
}

fn app_with_project() -> (CadApp, Context) {
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut app = CadApp::default();
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../cad-craft/tests/fixtures/etapa24-v9.cadcraft"
    );
    let project = cad_craft::persistence::read_project_from_slice(&std::fs::read(path).unwrap())
        .unwrap()
        .project;
    app.session.replace(project, false).unwrap();
    frame(&mut app, &ctx, vec![]);
    (app, ctx)
}

#[test]
fn ask_answer_show_command_navigate_apply_and_undo() {
    let (mut app, ctx) = app_with_project();
    let answer = format!(
        "Use o comando de numeração na aba Conexões.\n{}\n{}\n{}",
        block(r#"{"mostrar_comando": "Numerar conexões"}"#),
        block(r#"{"navegar": {"pagina": "1"}}"#),
        block(r#"{"propor": {"tipo": "novo_documento", "nome": "Comando"}}"#),
    );
    app.assistant.test_provider = Some(Box::new(move || Box::new(Script(vec![answer.clone()]))));
    // Open through the status bar.
    click(&mut app, &ctx, "✦ Assistente");
    assert!(app.assistant.open);
    assert!(shows(&mut app, &ctx, "ChatGPT (OpenAI)"));
    app.assistant.input = "Como numero os fios?".into();
    click(&mut app, &ctx, "Enviar");
    wait(&mut app, &ctx);
    assert!(shows(&mut app, &ctx, "Use o comando de numeração"));
    assert!(shows(&mut app, &ctx, "Proposta: Novo documento"));
    // Showing a command opens its ribbon tab and highlights its button.
    click(&mut app, &ctx, "Mostrar \"Numerar conexões\"");
    assert_eq!(app.preferences.ribbon_tab, RibbonTab::Connections);
    assert!(app.assistant.highlight.is_some());
    // Navigation opens sheet 1.
    click(&mut app, &ctx, "Ir à folha 1");
    assert!(app.session.active.is_some());
    // A proposal changes nothing until applied; applying creates content and Ctrl+Z undoes it.
    let before = app.session.project().unwrap().documents().count();
    click(&mut app, &ctx, "Aplicar");
    assert_eq!(
        app.session.project().unwrap().documents().count(),
        before + 1
    );
    assert!(shows(&mut app, &ctx, "Aplicada"));
    app.graphic_edit(GraphicEdit::Undo);
    assert_eq!(app.session.project().unwrap().documents().count(), before);
}

#[test]
fn proposal_goes_stale_when_the_project_changes_and_can_be_discarded() {
    let (mut app, ctx) = app_with_project();
    let answer = format!(
        "{}\n{}",
        block(r#"{"propor": {"tipo": "novo_documento", "nome": "A"}}"#),
        block(r#"{"propor": {"tipo": "apagar_tudo"}}"#),
    );
    app.assistant.test_provider = Some(Box::new(move || Box::new(Script(vec![answer.clone()]))));
    app.assistant.open = true;
    app.assistant.input = "crie um documento".into();
    click(&mut app, &ctx, "Enviar");
    wait(&mut app, &ctx);
    assert!(
        shows(&mut app, &ctx, "proposta recusada"),
        "proposta inválida vira nota"
    );
    // The project changes after the proposal, so it can no longer be applied.
    app.session.new_document("Outro").unwrap();
    let count = app.session.project().unwrap().documents().count();
    click(&mut app, &ctx, "Descartar");
    assert!(shows(&mut app, &ctx, "Descartada"));
    assert_eq!(app.session.project().unwrap().documents().count(), count);
}

#[test]
fn without_sign_in_the_input_is_disabled_and_no_account_data_is_kept() {
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut app = CadApp::default();
    app.assistant.open = true;
    frame(&mut app, &ctx, vec![]);
    assert!(shows(&mut app, &ctx, "Entrar com ChatGPT"));
    assert!(!app.assistant.ready());
    app.assistant.input = "oi".into();
    click(&mut app, &ctx, "✦ Assistente");
    // Nothing is sent without signing in.
    assert!(app.assistant.entries.is_empty());
    // Saved preferences contain no token, email or conversation.
    let saved = serde_json::to_string(&app.preferences).unwrap();
    for secret in ["token", "Bearer", "@", "refresh"] {
        assert!(
            !saved.to_lowercase().contains(&secret.to_lowercase()),
            "{secret}: {saved}"
        );
    }
}

#[test]
fn oauth_helpers_follow_pkce_and_the_loopback_contract() {
    // RFC 7636 example vector from Appendix B.
    assert_eq!(
        oauth::s256("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
        "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
    );
    let (verifier, challenge) = oauth::pkce().unwrap();
    assert_eq!(oauth::s256(&verifier), challenge);
    assert!(verifier.len() >= 43);
    let host = oauth::new_host_id().unwrap();
    assert!(host.starts_with("urn:uuid:") && host.len() == 45, "{host}");
    assert_ne!(host, oauth::new_host_id().unwrap());
    let url = oauth::authorize_url(
        &host,
        "http://127.0.0.1:1455/auth/callback",
        "st",
        "no",
        &challenge,
    );
    for part in [
        "client_id=dynamic_agent_client",
        "agent_name_hint=CAD-CRAFT",
        "response_type=code",
        "code_challenge_method=S256",
        "redirect_uri=http%3A%2F%2F127.0.0.1%3A1455%2Fauth%2Fcallback",
        "chatgpt.tokens.use.direct",
        "resource=https%3A%2F%2Fapi.openai.com%2Fv1",
    ] {
        assert!(url.contains(part), "{part}: {url}");
    }
    assert!(!url.contains("localhost"));
    let cb = oauth::parse_callback(
        "GET /auth/callback?code=abc&state=st&client_id=oaiapp_1&scope=openid+chatgpt.tokens.use.direct HTTP/1.1",
    )
    .unwrap();
    assert_eq!(cb.code.as_deref(), Some("abc"));
    assert_eq!(cb.client_id.as_deref(), Some("oaiapp_1"));
    assert_eq!(
        cb.scope.as_deref(),
        Some("openid chatgpt.tokens.use.direct")
    );
    assert!(oauth::parse_callback("GET /favicon.ico HTTP/1.1").is_none());
    let denied =
        oauth::parse_callback("GET /auth/callback?error=access_denied&state=st HTTP/1.1").unwrap();
    assert_eq!(denied.error.as_deref(), Some("access_denied"));
    // Tokens never appear in logs or debug messages.
    let tokens = oauth::Tokens {
        client_id: "oaiapp_1".into(),
        access_token: "segredo-de-acesso".into(),
        refresh_token: Some("segredo-de-renovacao".into()),
        expires_at: std::time::Instant::now(),
    };
    let debug = format!("{tokens:?}");
    assert!(!debug.contains("segredo"), "{debug}");
}

#[test]
fn header_tabs_switch_between_insertion_and_assistant() {
    use egui::accesskit::Role;
    let (mut app, ctx) = app_with_project();
    let top = |app: &mut CadApp, label: &str| {
        frame(app, &ctx, vec![]);
        let out = frame(app, &ctx, vec![]);
        nodes(&out)
            .into_iter()
            .filter(|(l, role, _)| l == label && !matches!(role, Role::Label | Role::TextRun))
            .filter_map(|(_, _, p)| p)
            .min_by(|a, b| a.y.total_cmp(&b.y))
            .unwrap_or_else(|| panic!("aba {label} ausente"))
    };
    let press = |app: &mut CadApp, pos: Pos2| {
        let event = |pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        frame(app, &ctx, vec![Event::PointerMoved(pos)]);
        frame(app, &ctx, vec![event(true)]);
        frame(app, &ctx, vec![event(false)]);
        frame(app, &ctx, vec![]);
    };
    let tab = top(&mut app, "✦ Assistente");
    assert!(tab.y < 200.0, "aba no cabeçalho do painel: {tab:?}");
    press(&mut app, tab);
    assert!(app.assistant.open, "a aba do cabeçalho abre o assistente");
    let back = top(&mut app, "Centro de inserção");
    press(&mut app, back);
    assert!(!app.assistant.open, "a outra aba volta à inserção");
}

#[test]
fn bold_markup_is_drawn_without_asterisks() {
    let ctx = Context::default();
    let mut texts = Vec::new();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            for text in ["Use **Conexões > Numerar** e **Ctrl+Z**.", "a ** b"] {
                texts.push(cad_gui::assistant::answer_layout(ui, text).text);
            }
        });
    });
    assert_eq!(texts[0], "Use Conexões > Numerar e Ctrl+Z.");
    assert_eq!(texts[1], "a ** b", "marcador sem par fica como texto");
}

/// Provider that waits until the request is cancelled.
struct Slow;
impl Provider for Slow {
    fn respond(
        &mut self,
        _: &Request<'_>,
        _: &mut dyn FnMut(&str),
        cancel: &AtomicBool,
    ) -> Result<String, String> {
        for _ in 0..2000 {
            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                return Err("interrompido".into());
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        Ok("tarde demais".into())
    }
}

#[test]
fn escape_interrupts_a_running_answer_as_the_panel_hint_says() {
    let (mut app, ctx) = app_with_project();
    app.assistant.test_provider = Some(Box::new(|| Box::new(Slow)));
    app.assistant.open = true;
    app.assistant.input = "pergunta longa".into();
    click(&mut app, &ctx, "Enviar");
    assert!(app.assistant.busy(), "a resposta está em andamento");
    let key = |pressed| Event::Key {
        key: egui::Key::Escape,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: Modifiers::NONE,
    };
    frame(&mut app, &ctx, vec![key(true)]);
    frame(&mut app, &ctx, vec![key(false)]);
    wait(&mut app, &ctx);
    assert!(shows(&mut app, &ctx, "interrompido"));
}
