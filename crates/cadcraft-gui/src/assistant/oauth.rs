// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! "Sign in with ChatGPT" for open-source, locally hosted apps.
//!
//! OAuth 2.0 authorization code with PKCE (S256) in the system browser, a
//! loopback callback on `127.0.0.1`, dynamic client registration and token
//! refresh. Nothing about the account is written to disk: tokens and the
//! issued client id live only in memory. The only persisted value is the
//! opaque host id required by OpenAI (`urn:uuid:…`), which identifies the
//! installation, not the user. CAD-CRAFT never sees or stores a password and
//! never asks for an API key.
use base64::Engine as _;
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub const AUTHORIZE_URL: &str = "https://auth.openai.com/api/accounts/authorize";
pub const TOKEN_URL: &str = "https://auth.openai.com/api/accounts/oauth/token";
pub const RESOURCE: &str = "https://api.openai.com/v1";
pub const SCOPE: &str =
    "openid profile email offline_access resource.invoke chatgpt.tokens.use.direct";
/// Scope that authorizes ChatGPT plan usage (checked after the exchange).
pub const PLAN_SCOPE: &str = "chatgpt.tokens.use.direct";
pub const AGENT_NAME: &str = "CAD-CRAFT";
const CALLBACK_PATH: &str = "/auth/callback";
/// How long the browser sign-in may take.
pub const SIGN_IN_TIMEOUT: Duration = Duration::from_secs(300);

/// Tokens of the signed-in session (memory only).
#[derive(Clone)]
pub struct Tokens {
    pub client_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Instant,
}
impl std::fmt::Debug for Tokens {
    // Never print secrets.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tokens").finish_non_exhaustive()
    }
}
impl Tokens {
    pub fn expired(&self) -> bool {
        Instant::now() + Duration::from_secs(60) >= self.expires_at
    }
}

fn random_bytes(n: usize) -> Result<Vec<u8>, String> {
    let mut buf = vec![0u8; n];
    getrandom::fill(&mut buf).map_err(|e| format!("gerador aleatório: {e}"))?;
    Ok(buf)
}
fn b64url(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// S256 challenge of a PKCE verifier (RFC 7636).
pub fn s256(verifier: &str) -> String {
    b64url(&Sha256::digest(verifier.as_bytes()))
}

/// PKCE verifier and its S256 challenge.
pub fn pkce() -> Result<(String, String), String> {
    let verifier = b64url(&random_bytes(32)?);
    let challenge = s256(&verifier);
    Ok((verifier, challenge))
}

/// New opaque host id (`urn:uuid:` with a random version-4 UUID).
pub fn new_host_id() -> Result<String, String> {
    let mut b = random_bytes(16)?;
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    let h: String = b.iter().map(|x| format!("{x:02x}")).collect();
    Ok(format!(
        "urn:uuid:{}-{}-{}-{}-{}",
        &h[0..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..32]
    ))
}

/// Percent-encoding for query strings (RFC 3986 unreserved kept).
pub fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}
fn decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => {
                if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                    out.push(v);
                    i += 2;
                } else {
                    out.push(b'%');
                }
            }
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Authorization URL for the system browser.
pub fn authorize_url(
    host_id: &str,
    redirect_uri: &str,
    state: &str,
    nonce: &str,
    challenge: &str,
) -> String {
    let params = [
        ("client_id", "dynamic_agent_client"),
        ("agent_name_hint", AGENT_NAME),
        ("ext_agent_host_id", host_id),
        ("response_type", "code"),
        ("redirect_uri", redirect_uri),
        ("scope", SCOPE),
        ("resource", RESOURCE),
        ("state", state),
        ("nonce", nonce),
        ("code_challenge_method", "S256"),
        ("code_challenge", challenge),
    ];
    let query: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("{k}={}", encode(v)))
        .collect();
    format!("{AUTHORIZE_URL}?{}", query.join("&"))
}

/// Parameters of the loopback callback.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Callback {
    pub code: Option<String>,
    pub state: Option<String>,
    pub client_id: Option<String>,
    pub scope: Option<String>,
    pub error: Option<String>,
}

/// Parses the request line `GET /auth/callback?… HTTP/1.1`.
pub fn parse_callback(request_line: &str) -> Option<Callback> {
    let target = request_line.split_whitespace().nth(1)?;
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    if path != CALLBACK_PATH {
        return None;
    }
    let mut cb = Callback::default();
    for pair in query.split('&') {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        let v = Some(decode(v));
        match k {
            "code" => cb.code = v,
            "state" => cb.state = v,
            "client_id" => cb.client_id = v,
            "scope" => cb.scope = v,
            "error" => cb.error = v,
            _ => {}
        }
    }
    Some(cb)
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(60)))
        .build()
        .into()
}

fn token_request(form: &[(&str, &str)], client_id: &str) -> Result<Tokens, String> {
    let mut response = agent()
        .post(TOKEN_URL)
        .send_form(form.iter().copied())
        .map_err(|e| format!("falha de rede no login: {e}"))?;
    let status = response.status().as_u16();
    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("resposta do login ilegível: {e}"))?;
    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|_| format!("resposta do login inválida (HTTP {status})"))?;
    if status != 200 {
        let code = json["error"].as_str().unwrap_or("erro");
        return Err(format!("login recusado ({code})"));
    }
    let scope = json["scope"].as_str().unwrap_or("");
    if !scope.split_whitespace().any(|s| s == PLAN_SCOPE) {
        return Err("a conta não autorizou o uso do plano ChatGPT pelo CAD-CRAFT".into());
    }
    let access_token = json["access_token"]
        .as_str()
        .ok_or("resposta do login sem token")?
        .to_owned();
    Ok(Tokens {
        client_id: client_id.to_owned(),
        access_token,
        refresh_token: json["refresh_token"].as_str().map(str::to_owned),
        expires_at: Instant::now()
            + Duration::from_secs(json["expires_in"].as_u64().unwrap_or(3600)),
    })
}

/// Renews the access token (rotating refresh token).
pub fn refresh(tokens: &Tokens) -> Result<Tokens, String> {
    let refresh = tokens
        .refresh_token
        .as_deref()
        .ok_or("sessão expirada: entre de novo")?;
    let mut next = token_request(
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh),
            ("client_id", &tokens.client_id),
            ("resource", RESOURCE),
        ],
        &tokens.client_id,
    )?;
    if next.refresh_token.is_none() {
        next.refresh_token = tokens.refresh_token.clone();
    }
    Ok(next)
}

const PAGE_OK: &str = "<!doctype html><meta charset=utf-8><title>CAD-CRAFT</title><body style=\"font-family:sans-serif;padding:2em\"><h2>Login concluído</h2><p>Pode fechar esta janela e voltar ao CAD-CRAFT.</p></body>";
const PAGE_FAIL: &str = "<!doctype html><meta charset=utf-8><title>CAD-CRAFT</title><body style=\"font-family:sans-serif;padding:2em\"><h2>Login não concluído</h2><p>Volte ao CAD-CRAFT e tente de novo.</p></body>";

/// Full sign-in: opens the browser, waits for the callback (until
/// [`SIGN_IN_TIMEOUT`] or `cancel`) and exchanges the code.
pub fn sign_in(host_id: &str, cancel: &AtomicBool) -> Result<Tokens, String> {
    let listener =
        TcpListener::bind("127.0.0.1:0").map_err(|e| format!("porta local indisponível: {e}"))?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let redirect_uri = format!("http://127.0.0.1:{port}{CALLBACK_PATH}");
    let (verifier, challenge) = pkce()?;
    let state = b64url(&random_bytes(24)?);
    let nonce = b64url(&random_bytes(24)?);
    let url = authorize_url(host_id, &redirect_uri, &state, &nonce, &challenge);
    webbrowser::open(&url).map_err(|e| format!("não foi possível abrir o navegador: {e}"))?;
    let started = Instant::now();
    let callback = loop {
        if cancel.load(Ordering::Relaxed) {
            return Err("login cancelado".into());
        }
        if started.elapsed() > SIGN_IN_TIMEOUT {
            return Err("tempo do login esgotado".into());
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                let _ = stream.set_nonblocking(false);
                let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
                let mut line = String::new();
                let _ = BufReader::new(&stream).read_line(&mut line);
                let Some(cb) = parse_callback(&line) else {
                    let _ = stream.write_all(
                        b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    );
                    continue;
                };
                let ok = cb.error.is_none() && cb.state.as_deref() == Some(state.as_str());
                let page = if ok { PAGE_OK } else { PAGE_FAIL };
                let _ = stream.write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{page}",
                        page.len()
                    )
                    .as_bytes(),
                );
                break cb;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(format!("retorno do login falhou: {e}")),
        }
    };
    if callback.state.as_deref() != Some(state.as_str()) {
        return Err("retorno do login com estado inválido".into());
    }
    if let Some(error) = callback.error {
        return Err(if error == "access_denied" {
            "login cancelado no navegador".into()
        } else {
            format!("login recusado ({error})")
        });
    }
    let code = callback.code.ok_or("retorno do login sem código")?;
    let client_id = callback
        .client_id
        .ok_or("registro do CAD-CRAFT incompleto no retorno do login")?;
    token_request(
        &[
            ("grant_type", "authorization_code"),
            ("client_id", &client_id),
            ("code", &code),
            ("code_verifier", &verifier),
            ("redirect_uri", &redirect_uri),
            ("resource", RESOURCE),
        ],
        &client_id,
    )
}
