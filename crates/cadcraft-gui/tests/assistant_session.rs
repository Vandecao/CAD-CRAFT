// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! "Manter conectado" without network or the real credential vault: an
//! in-memory vault and a scripted restore stand in for Windows and OpenAI.
// Fixture helpers use validated synthetic inputs and fail immediately on setup errors.
#![allow(clippy::unwrap_used)]
use cad_gui::app::CadApp;
use cad_gui::assistant::vault::{MemoryVault, Saved, SessionVault};
use cad_gui::assistant::{RestoreError, oauth};
use eframe::egui::{self, Context, Pos2, Rect, vec2};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn frame(app: &mut CadApp, ctx: &Context) {
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1600., 900.))),
            ..Default::default()
        },
        |ctx| app.show(ctx),
    );
}
fn wait(app: &mut CadApp, ctx: &Context) {
    for _ in 0..1000 {
        frame(app, ctx);
        if !app.assistant.busy() {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("a tarefa do assistente não terminou");
}
fn tokens(refresh: &str) -> oauth::Tokens {
    oauth::Tokens {
        client_id: "oaiapp_teste".into(),
        access_token: "acesso".into(),
        refresh_token: Some(refresh.into()),
        expires_at: Instant::now() + Duration::from_secs(3600),
    }
}
fn kept(refresh: &str) -> Saved {
    Saved {
        client_id: "oaiapp_teste".into(),
        refresh_token: refresh.into(),
    }
}
fn models() -> Vec<(String, String)> {
    vec![("gpt-teste".into(), "GPT Teste".into())]
}
/// App with the in-memory vault and "Manter conectado" on.
fn app(vault: &Arc<MemoryVault>, keep: bool) -> (CadApp, Context) {
    let mut app = CadApp::default();
    app.assistant.vault = vault.clone();
    app.preferences.assistant_keep_signed_in = keep;
    app.assistant.open = true;
    (app, Context::default())
}

#[test]
fn kept_session_is_resumed_without_browser_and_the_rotated_token_replaces_it() {
    let vault = Arc::new(MemoryVault::default());
    vault.save(&kept("antigo")).unwrap();
    let seen = Arc::new(Mutex::new(None));
    let s = seen.clone();
    let (mut app, ctx) = app(&vault, true);
    app.assistant.test_restorer = Some(Arc::new(move |saved: &Saved| {
        *s.lock().unwrap() = Some(saved.refresh_token.clone());
        Ok((tokens("novo"), models()))
    }));
    app.assistant_restore();
    wait(&mut app, &ctx);
    assert_eq!(seen.lock().unwrap().as_deref(), Some("antigo"));
    assert!(app.assistant.signed_in);
    assert_eq!(app.assistant.model, "gpt-teste");
    assert!(app.assistant.status.contains("sessão mantida"));
    assert_eq!(vault.current(), Some(kept("novo")), "token rotativo salvo");
    // A rotation during use is saved after the next answer or sync.
    app.assistant.set_tokens(Some(tokens("girado")));
    app.assistant_set_keep_signed_in(true);
    assert_eq!(vault.current(), Some(kept("girado")));
}

#[test]
fn nothing_is_resumed_or_saved_when_the_option_is_off() {
    let vault = Arc::new(MemoryVault::default());
    vault.save(&kept("antigo")).unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let c = calls.clone();
    let (mut app, ctx) = app(&vault, false);
    app.assistant.test_restorer = Some(Arc::new(move |_: &Saved| {
        c.fetch_add(1, Ordering::Relaxed);
        Ok((tokens("novo"), models()))
    }));
    app.assistant_restore();
    frame(&mut app, &ctx);
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    assert!(!app.assistant.signed_in);
    // Signed in with the option off: the session stays in memory only.
    let empty = Arc::new(MemoryVault::default());
    app.assistant.vault = empty.clone();
    app.assistant.set_tokens(Some(tokens("so-memoria")));
    app.assistant.signed_in = true;
    assert!(!app.preferences.assistant_keep_signed_in);
    assert_eq!(empty.current(), None);
}

#[test]
fn rejected_restore_deletes_the_kept_session_but_a_network_error_keeps_it() {
    let vault = Arc::new(MemoryVault::default());
    vault.save(&kept("antigo")).unwrap();
    let (mut app, ctx) = app(&vault, true);
    app.assistant.test_restorer = Some(Arc::new(|_: &Saved| {
        Err(RestoreError::Transient(
            "falha de rede no login: sem conexão".into(),
        ))
    }));
    app.assistant_restore();
    wait(&mut app, &ctx);
    assert!(!app.assistant.signed_in);
    assert!(app.assistant.restore_failed, "oferece Tentar de novo");
    assert_eq!(vault.current(), Some(kept("antigo")), "erro de rede mantém");
    app.assistant.test_restorer = Some(Arc::new(|_: &Saved| {
        Err(RestoreError::Rejected(
            "login recusado (invalid_grant)".into(),
        ))
    }));
    app.assistant_restore();
    wait(&mut app, &ctx);
    assert!(!app.assistant.signed_in);
    assert_eq!(vault.current(), None, "sessão expirada é apagada");
    assert!(app.assistant.status.contains("entre de novo"));
}

#[test]
fn sign_out_and_unticking_delete_the_kept_session() {
    let vault = Arc::new(MemoryVault::default());
    let (mut app, _ctx) = app(&vault, true);
    app.assistant.set_tokens(Some(tokens("um")));
    app.assistant.signed_in = true;
    app.assistant_set_keep_signed_in(true);
    assert_eq!(vault.current(), Some(kept("um")));
    app.assistant_set_keep_signed_in(false);
    assert_eq!(vault.current(), None, "desmarcar apaga");
    assert!(app.assistant.signed_in, "a sessão atual continua");
    app.assistant_set_keep_signed_in(true);
    assert_eq!(vault.current(), Some(kept("um")));
    app.assistant.sign_out();
    assert_eq!(vault.current(), None, "Sair apaga");
    assert!(!app.assistant.signed_in);
}

#[test]
fn unavailable_vault_disables_the_option_and_saves_nothing() {
    let vault = Arc::new(MemoryVault {
        available: false,
        ..Default::default()
    });
    vault.save(&kept("antigo")).unwrap();
    let (mut app, ctx) = app(&vault, true);
    app.assistant.test_restorer = Some(Arc::new(|_: &Saved| Ok((tokens("x"), models()))));
    app.assistant_restore();
    frame(&mut app, &ctx);
    assert!(!app.assistant.busy() && !app.assistant.signed_in);
}

#[test]
fn preferences_never_hold_the_kept_session() {
    let vault = Arc::new(MemoryVault::default());
    let (mut app, _ctx) = app(&vault, true);
    app.assistant
        .set_tokens(Some(tokens("segredo-de-renovacao")));
    app.assistant.signed_in = true;
    app.assistant_set_keep_signed_in(true);
    let saved = app.preferences.to_json().unwrap();
    for secret in ["segredo", "oaiapp_", "refresh", "token", "acesso"] {
        assert!(!saved.contains(secret), "{secret}: {saved}");
    }
    assert!(saved.contains("\"assistant_keep_signed_in\":true"));
}

#[test]
fn a_default_app_does_not_use_the_system_vault() {
    // Tests must never read or delete the developer's real credential.
    let app = CadApp::default();
    assert!(!app.assistant.vault.available());
}
