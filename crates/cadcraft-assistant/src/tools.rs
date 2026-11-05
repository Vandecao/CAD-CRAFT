// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Closed catalogue of read-only queries the model may ask for.
//!
//! Queries never change the project. Results are bounded and returned as
//! JSON data, which the conversation neutralises before sending.
use crate::help::{self, CommandInfo};
use cadcraft_model::domain::{CHECK_RULES, CheckScope, PageId, Project};
use serde_json::{Map, Value, json};

/// Most items returned by one query.
pub const MAX_ITEMS: usize = 30;

/// Description of one query (sent to the model).
#[derive(Clone, Copy, Debug)]
pub struct QueryTool {
    pub name: &'static str,
    pub args: &'static str,
    pub description: &'static str,
}

pub const QUERY_TOOLS: &[QueryTool] = &[
    QueryTool {
        name: "ajuda",
        args: "{\"pergunta\": \"texto\"}",
        description: "Como fazer uma tarefa no CAD-CRAFT e quais comandos usar.",
    },
    QueryTool {
        name: "localizar_comando",
        args: "{\"texto\": \"texto\"}",
        description: "Onde fica um comando: aba, grupo, atalho e descrição.",
    },
    QueryTool {
        name: "resumo_projeto",
        args: "{}",
        description: "Documentos, páginas, dispositivos, ligações e contagens de verificação.",
    },
    QueryTool {
        name: "listar_paginas",
        args: "{}",
        description: "Páginas na ordem do projeto (número, nome, documento).",
    },
    QueryTool {
        name: "buscar_dispositivo",
        args: "{\"texto\": \"designação ou parte dela\"}",
        description: "Dispositivos pela designação: classificação, funções e páginas onde aparecem.",
    },
    QueryTool {
        name: "buscar_fio",
        args: "{\"numero\": \"número do fio ou vazio para os sem número\"}",
        description: "Ligações pelo número do fio, com travamento e páginas.",
    },
    QueryTool {
        name: "verificacoes",
        args: "{\"pagina\": \"número (opcional)\"}",
        description: "Mensagens das verificações de consistência (regra, gravidade, texto).",
    },
    QueryTool {
        name: "explicar_regra",
        args: "{\"codigo\": \"VER-001\"}",
        description: "Título e gravidade de uma regra de verificação.",
    },
];

/// Text list of the queries for the instructions.
pub fn describe() -> String {
    QUERY_TOOLS
        .iter()
        .map(|t| format!("- {} {}: {}", t.name, t.args, t.description))
        .collect::<Vec<_>>()
        .join("\n")
}

fn arg<'a>(args: &'a Map<String, Value>, key: &str) -> &'a str {
    args.get(key).and_then(Value::as_str).unwrap_or("").trim()
}

fn page_label(project: &Project, page: PageId) -> String {
    project
        .page(page)
        .map(|p| p.number().as_str().to_owned())
        .unwrap_or_default()
}

/// Page of every occurrence of a device.
fn device_pages(project: &Project, device: cadcraft_model::domain::DeviceId) -> Vec<String> {
    let mut pages: Vec<String> = project
        .representations()
        .filter(|r| project.device_for_target(r.target()) == Some(device))
        .map(|r| page_label(project, r.page_id()))
        .collect();
    pages.sort();
    pages.dedup();
    pages
}

/// Page number by its identification text.
pub fn page_by_number(project: &Project, number: &str) -> Option<PageId> {
    project
        .pages()
        .find(|p| p.number().as_str().eq_ignore_ascii_case(number.trim()))
        .map(|p| p.id())
}

/// Runs one query. Unknown queries and bad arguments are errors, reported
/// back to the model as data.
pub fn run(
    project: Option<&Project>,
    catalog: &[CommandInfo],
    tool: &str,
    args: &Map<String, Value>,
) -> Result<Value, String> {
    match tool {
        "ajuda" => {
            let q = arg(args, "pergunta");
            let topics: Vec<Value> = help::find_topics(q, 3)
                .into_iter()
                .map(|t| json!({ "titulo": t.title, "texto": t.text }))
                .collect();
            let commands: Vec<Value> = help::find_commands(catalog, q, 5)
                .into_iter()
                .map(command_json)
                .collect();
            Ok(json!({ "topicos": topics, "comandos": commands }))
        }
        "localizar_comando" => {
            let q = arg(args, "texto");
            if q.is_empty() {
                return Err("informe o texto".into());
            }
            let found: Vec<Value> = help::find_commands(catalog, q, 8)
                .into_iter()
                .map(command_json)
                .collect();
            Ok(json!({ "comandos": found }))
        }
        "explicar_regra" => {
            let code = arg(args, "codigo").to_uppercase();
            CHECK_RULES
                .iter()
                .find(|r| r.code == code)
                .map(|r| {
                    json!({
                        "codigo": r.code,
                        "titulo": r.title,
                        "gravidade": r.severity.label(),
                        "ativa_por_padrao": r.enabled,
                    })
                })
                .ok_or_else(|| format!("regra desconhecida: {code}"))
        }
        _ => {
            let project = project.ok_or("nenhum projeto aberto")?;
            run_on_project(project, tool, args)
        }
    }
}

fn command_json(c: &CommandInfo) -> Value {
    json!({
        "nome": c.name,
        "aba": c.tab,
        "grupo": c.group,
        "atalho": c.shortcut,
        "descricao": c.description,
    })
}

fn run_on_project(
    project: &Project,
    tool: &str,
    args: &Map<String, Value>,
) -> Result<Value, String> {
    match tool {
        "resumo_projeto" => {
            let documents: Vec<Value> = project
                .documents()
                .map(|d| {
                    json!({
                        "nome": d.name().as_str(),
                        "paginas": project.ordered_pages(d.id()).map(|p| p.len()).unwrap_or(0),
                    })
                })
                .collect();
            Ok(json!({
                "projeto": project.name().as_str(),
                "documentos": documents,
                "dispositivos": project.devices().count(),
                "ligacoes": project.connections().count(),
                "ligacoes_sem_numero": project.connections().filter(|c| c.number().is_none()).count(),
            }))
        }
        "listar_paginas" => {
            let mut pages = vec![];
            for d in project.documents() {
                for p in project.ordered_pages(d.id()).map_err(|e| e.to_string())? {
                    pages.push(json!({
                        "numero": p.number().as_str(),
                        "nome": p.name().as_str(),
                        "documento": d.name().as_str(),
                    }));
                }
            }
            let total = pages.len();
            pages.truncate(MAX_ITEMS * 4);
            Ok(json!({ "total": total, "paginas": pages }))
        }
        "buscar_dispositivo" => {
            let q = arg(args, "texto").trim_start_matches('-').to_lowercase();
            if q.is_empty() {
                return Err("informe o texto".into());
            }
            let found: Vec<Value> = project
                .devices()
                .filter(|d| d.designation().to_string().to_lowercase().contains(&q))
                .take(MAX_ITEMS)
                .map(|d| {
                    let functions: Vec<String> = project
                        .functions_for_device(d.id())
                        .map(|f| format!("{:?}", f.kind()))
                        .collect();
                    json!({
                        "designacao": d.designation().to_string(),
                        "classificacao": d.classification().as_str(),
                        "funcoes": functions,
                        "paginas": device_pages(project, d.id()),
                    })
                })
                .collect();
            Ok(json!({ "dispositivos": found }))
        }
        "buscar_fio" => {
            let q = arg(args, "numero");
            let found: Vec<Value> = project
                .connections()
                .filter(|c| match c.number() {
                    Some(n) => !q.is_empty() && n.as_str().eq_ignore_ascii_case(q),
                    None => q.is_empty(),
                })
                .take(MAX_ITEMS)
                .map(|c| {
                    let mut pages: Vec<String> = project
                        .connection_routes()
                        .filter(|r| r.connection_id() == c.id())
                        .map(|r| page_label(project, r.page_id()))
                        .collect();
                    pages.sort();
                    pages.dedup();
                    json!({
                        "id": c.id().to_string(),
                        "numero": c.number().map(|n| n.as_str()),
                        "travado": c.number_locked(),
                        "paginas": pages,
                    })
                })
                .collect();
            Ok(json!({ "fios": found }))
        }
        "verificacoes" => {
            let page = arg(args, "pagina");
            let scope = if page.is_empty() {
                CheckScope::Project
            } else {
                CheckScope::Page(page_by_number(project, page).ok_or("página não encontrada")?)
            };
            let report = project.check(scope);
            let total = report.messages.len();
            let messages: Vec<Value> = report
                .messages
                .iter()
                .take(MAX_ITEMS)
                .map(|m| json!({ "regra": m.rule, "gravidade": m.severity.label(), "texto": m.text }))
                .collect();
            Ok(json!({ "total": total, "mensagens": messages }))
        }
        other => Err(format!("consulta desconhecida: {other}")),
    }
}
