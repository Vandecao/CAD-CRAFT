// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Action blocks in the model's answer.
//!
//! The model writes normal text and, when it wants CAD-CRAFT to do
//! something, a fenced block:
//!
//! ````text
//! ```cadcraft
//! {"consulta": "buscar_dispositivo", "args": {"texto": "K1"}}
//! ```
//! ````
//!
//! Only the model's own text is parsed; tool results and project data are
//! neutralised before they reach the model ([`crate::prompt::neutralise`]),
//! so they can never form a block. Anything outside the closed set of
//! actions is rejected and reported.
use serde_json::{Map, Value};

/// Opening of an action block.
pub const FENCE: &str = "```cadcraft";
/// Most actions accepted from one answer.
pub const MAX_ACTIONS: usize = 6;
/// Largest accepted block (bytes).
pub const MAX_BLOCK: usize = 4096;

/// Where "navegar" goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavTarget {
    /// Page by its number (identification), e.g. "5".
    Page(String),
    /// Device by designation, e.g. "-K1".
    Device(String),
    /// Wire by its number, e.g. "511".
    Wire(String),
}

/// One validated action.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    /// Read-only query, run by CAD-CRAFT; the result goes back to the model.
    Query {
        tool: String,
        args: Map<String, Value>,
    },
    /// Open a page and select an occurrence (does not change the project).
    Navigate(NavTarget),
    /// Highlight a command in the ribbon (does not change the project).
    ShowCommand(String),
    /// Change proposal; shown as a card, applied only by the user.
    Propose(Value),
}

/// Answer split into text and actions.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Parsed {
    /// Text without the action blocks.
    pub text: String,
    pub actions: Vec<Action>,
    /// Blocks refused, with the reason (shown to the user, never executed).
    pub rejected: Vec<String>,
}

fn text_arg(v: &Value) -> Option<String> {
    let s = v.as_str()?.trim();
    (!s.is_empty() && s.chars().count() <= 200).then(|| s.to_owned())
}

fn action(value: &Value) -> Result<Action, String> {
    let obj = value.as_object().ok_or("bloco não é um objeto JSON")?;
    if let Some(tool) = obj.get("consulta") {
        let tool = text_arg(tool).ok_or("consulta sem nome")?;
        if !crate::tools::QUERY_TOOLS.iter().any(|t| t.name == tool) {
            return Err(format!("consulta desconhecida: {tool}"));
        }
        let args = match obj.get("args") {
            None => Map::new(),
            Some(Value::Object(m)) => m.clone(),
            Some(_) => return Err("args precisa ser um objeto".into()),
        };
        return Ok(Action::Query { tool, args });
    }
    if let Some(nav) = obj.get("navegar") {
        let nav = nav.as_object().ok_or("navegar precisa ser um objeto")?;
        let target = if let Some(p) = nav.get("pagina").and_then(text_arg) {
            NavTarget::Page(p)
        } else if let Some(d) = nav.get("dispositivo").and_then(text_arg) {
            NavTarget::Device(d)
        } else if let Some(w) = nav.get("fio").and_then(text_arg) {
            NavTarget::Wire(w)
        } else {
            return Err("navegar precisa de pagina, dispositivo ou fio".into());
        };
        return Ok(Action::Navigate(target));
    }
    if let Some(cmd) = obj.get("mostrar_comando") {
        return Ok(Action::ShowCommand(
            text_arg(cmd).ok_or("mostrar_comando sem nome")?,
        ));
    }
    if let Some(p) = obj.get("propor") {
        if !p.is_object() {
            return Err("propor precisa ser um objeto".into());
        }
        return Ok(Action::Propose(p.clone()));
    }
    Err("ação desconhecida".into())
}

/// Splits the model's answer into text and validated actions.
pub fn parse(answer: &str) -> Parsed {
    let mut out = Parsed::default();
    let mut rest = answer;
    while let Some(start) = rest.find(FENCE) {
        out.text.push_str(&rest[..start]);
        let after = &rest[start + FENCE.len()..];
        let Some(end) = after.find("```") else {
            out.rejected.push("bloco de ação sem fechamento".into());
            rest = "";
            break;
        };
        let body = after[..end].trim();
        rest = &after[end + 3..];
        if body.len() > MAX_BLOCK {
            out.rejected.push("bloco de ação grande demais".into());
            continue;
        }
        if out.actions.len() >= MAX_ACTIONS {
            out.rejected.push("ações demais numa resposta".into());
            continue;
        }
        match serde_json::from_str::<Value>(body) {
            Ok(value) => match action(&value) {
                Ok(a) => out.actions.push(a),
                Err(e) => out.rejected.push(e),
            },
            Err(_) => out.rejected.push("bloco de ação com JSON inválido".into()),
        }
    }
    out.text.push_str(rest);
    out.text = out.text.trim().to_owned();
    out
}
