// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! ChatGPT provider: model list and streamed Responses API calls, using
//! the ChatGPT plan of the signed-in user (`store: false`, `stream: true`).
use super::oauth::{self, Tokens};
use cad_craft::assistant::conversation::{Provider, Request, Role};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub const MODELS_URL: &str = "https://api.openai.com/v1/models";
pub const RESPONSES_URL: &str = "https://api.openai.com/v1/responses";

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(180)))
        .build()
        .into()
}

/// Valid access token, refreshing it first when needed.
fn access_token(tokens: &Mutex<Option<Tokens>>) -> Result<String, String> {
    let mut guard = tokens
        .lock()
        .map_err(|_| crate::strings::SESSION_UNAVAILABLE)?;
    let current = guard.as_ref().ok_or("entre com a conta ChatGPT")?;
    if current.expired() {
        let next = oauth::refresh(current)?;
        *guard = Some(next);
    }
    Ok(guard.as_ref().expect("tokens").access_token.clone())
}

/// Models available to the account: (slug, display name), in server order.
pub fn list_models(tokens: &Mutex<Option<Tokens>>) -> Result<Vec<(String, String)>, String> {
    let token = access_token(tokens)?;
    let mut response = agent()
        .get(MODELS_URL)
        .header("Authorization", &format!("Bearer {token}"))
        .call()
        .map_err(|e| format!("falha de rede: {e}"))?;
    let status = response.status().as_u16();
    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("resposta ilegível: {e}"))?;
    if status != 200 {
        return Err(format!("lista de modelos indisponível (HTTP {status})"));
    }
    let json: Value = serde_json::from_str(&body).map_err(|_| "lista de modelos inválida")?;
    let models = json["models"]
        .as_array()
        .or_else(|| json["data"].as_array())
        .cloned()
        .unwrap_or_default();
    Ok(models
        .iter()
        .filter(|m| m["visibility"].as_str().is_none_or(|v| v == "list"))
        .filter_map(|m| {
            let slug = m["slug"].as_str().or_else(|| m["id"].as_str())?.to_owned();
            let name = m["display_name"].as_str().unwrap_or(&slug).to_owned();
            Some((slug, name))
        })
        .collect())
}

/// Friendly text for plan errors.
fn plan_error(code: &str, message: &str) -> String {
    match code {
        "subscription_sharing_usage_limit_exceeded" => {
            "limite de uso do plano ChatGPT atingido; tente mais tarde".into()
        }
        "subscription_sharing_usage_unavailable" => {
            "uso do plano ChatGPT indisponível para esta conta agora".into()
        }
        _ if !message.is_empty() => format!("o modelo recusou a resposta: {message}"),
        _ => format!("o modelo recusou a resposta ({code})"),
    }
}

pub struct ChatGpt {
    pub tokens: Arc<Mutex<Option<Tokens>>>,
    pub model: String,
}

impl Provider for ChatGpt {
    fn respond(
        &mut self,
        request: &Request<'_>,
        on_text: &mut dyn FnMut(&str),
        cancel: &AtomicBool,
    ) -> Result<String, String> {
        let token = access_token(&self.tokens)?;
        let input: Vec<Value> = request
            .input
            .iter()
            .map(|m| {
                json!({
                    "role": match m.role { Role::User => "user", Role::Assistant => "assistant" },
                    "content": m.content,
                })
            })
            .collect();
        let body = json!({
            "model": self.model,
            "instructions": request.instructions,
            "input": input,
            "store": false,
            "stream": true,
        });
        let response = agent()
            .post(RESPONSES_URL)
            .header("Authorization", &format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .send(&serde_json::to_string(&body).map_err(|e| e.to_string())?)
            .map_err(|e| format!("falha de rede: {e}"))?;
        let status = response.status().as_u16();
        let mut body = response.into_body();
        if status != 200 {
            let text = body.read_to_string().unwrap_or_default();
            let json: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
            let code = json["error"]["code"].as_str().unwrap_or("");
            let message = json["error"]["message"].as_str().unwrap_or("");
            return Err(if status == 401 {
                crate::strings::SESSION_EXPIRED.into()
            } else {
                plan_error(code, format!("HTTP {status} {message}").trim())
            });
        }
        let mut answer = String::new();
        let mut completed = false;
        for line in BufReader::new(body.into_reader()).lines() {
            if cancel.load(Ordering::Relaxed) {
                return Err("interrompido".into());
            }
            let line = line.map_err(|e| format!("conexão interrompida: {e}"))?;
            let Some(data) = line.strip_prefix("data:") else {
                continue;
            };
            let Ok(event) = serde_json::from_str::<Value>(data.trim()) else {
                continue;
            };
            match event["type"].as_str().unwrap_or("") {
                "response.output_text.delta" => {
                    if let Some(delta) = event["delta"].as_str() {
                        answer.push_str(delta);
                        on_text(delta);
                    }
                }
                "response.completed" => {
                    completed = true;
                    break;
                }
                "response.failed" => {
                    let error = &event["response"]["error"];
                    return Err(plan_error(
                        error["code"].as_str().unwrap_or(""),
                        error["message"].as_str().unwrap_or(""),
                    ));
                }
                "response.incomplete" => return Err("resposta incompleta do modelo".into()),
                _ => {}
            }
        }
        if !completed {
            return Err("a resposta terminou antes de concluir".into());
        }
        Ok(answer)
    }
}
