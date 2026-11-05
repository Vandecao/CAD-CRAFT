// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Change proposals from the assistant.
//!
//! A proposal is prepared against the current project and validated before
//! the user sees it: numbering uses the same previews as the numbering
//! dialog, and the other changes are applied to a copy of the project. The
//! application applies a [`Change`] through its normal undoable operations,
//! only when the user clicks "Aplicar".
use cadcraft_model::domain::{
    ConnectionId, ConnectionNumberPreview, DeviceNumberPreview, Document, DocumentEdit, DocumentId,
    NumberingFilter, Page, PageKind, PageNumberPreview, PaperFormat, Project, Size2D, Text,
};
use serde_json::Value;

/// Validated change, ready to apply through the session.
#[derive(Clone, Debug, PartialEq)]
pub enum Change {
    NumberConnections(ConnectionNumberPreview),
    NumberDevices(DeviceNumberPreview),
    NumberPages(PageNumberPreview),
    WireNumber {
        connection: ConnectionId,
        number: Text,
        locked: bool,
    },
    NewDocument {
        name: String,
    },
    NewPage {
        document: DocumentId,
        name: String,
        number: String,
        size: Size2D,
    },
}

/// What the user sees in the proposal card.
#[derive(Clone, Debug, PartialEq)]
pub struct Proposal {
    pub title: String,
    /// Preview lines (bounded).
    pub lines: Vec<String>,
    pub change: Change,
}

const MAX_LINES: usize = 12;

fn text(value: &Value, key: &str) -> Option<String> {
    let s = value.get(key)?.as_str()?.trim();
    (!s.is_empty() && s.chars().count() <= 120).then(|| s.to_owned())
}

fn document_by_name(project: &Project, name: Option<String>) -> Result<DocumentId, String> {
    let documents: Vec<_> = project.documents().collect();
    match name {
        Some(n) => documents
            .iter()
            .find(|d| d.name().as_str().eq_ignore_ascii_case(&n))
            .map(|d| d.id())
            .ok_or_else(|| format!("documento não encontrado: {n}")),
        None if documents.len() == 1 => Ok(documents[0].id()),
        None => Err("informe o documento".into()),
    }
}

fn bounded(mut lines: Vec<String>, total: usize) -> Vec<String> {
    if lines.len() > MAX_LINES {
        lines.truncate(MAX_LINES);
        lines.push(format!("… e mais {} alteração(ões)", total - MAX_LINES));
    }
    lines
}

/// Prepares and validates a proposal. Errors are shown to the user instead
/// of the card; nothing changes in the project.
pub fn prepare(project: &Project, request: &Value) -> Result<Proposal, String> {
    let kind = text(request, "tipo").ok_or("proposta sem tipo")?;
    match kind.as_str() {
        "numerar_fios" => {
            let preview = project
                .preview_connection_numbering()
                .map_err(|e| e.to_string())?;
            let changed: Vec<_> = preview
                .rows
                .iter()
                .filter(|r| r.before != r.after)
                .collect();
            if changed.is_empty() {
                return Err("nenhum fio para numerar".into());
            }
            if let Some(c) = preview.rows.iter().find_map(|r| r.conflict.clone()) {
                return Err(format!("numeração com conflito: {c}"));
            }
            let lines = changed
                .iter()
                .map(|r| {
                    format!(
                        "Fio {} → {}",
                        r.before.as_ref().map_or("(sem número)", Text::as_str),
                        r.after.as_ref().map_or("(sem número)", Text::as_str)
                    )
                })
                .collect();
            Ok(Proposal {
                title: format!("Numerar {} fio(s)", changed.len()),
                lines: bounded(lines, changed.len()),
                change: Change::NumberConnections(preview),
            })
        }
        "numerar_dispositivos" => {
            let preview = project
                .preview_device_numbering(NumberingFilter::Project)
                .map_err(|e| e.to_string())?;
            let changed: Vec<_> = preview
                .rows
                .iter()
                .filter(|r| r.before != r.after)
                .collect();
            if changed.is_empty() {
                return Err("nenhum dispositivo para renumerar".into());
            }
            if let Some(c) = preview.rows.iter().find_map(|r| r.conflict.clone()) {
                return Err(format!("numeração com conflito: {c}"));
            }
            let lines = changed
                .iter()
                .map(|r| format!("{} → {}", r.before, r.after))
                .collect();
            Ok(Proposal {
                title: format!("Renumerar {} dispositivo(s)", changed.len()),
                lines: bounded(lines, changed.len()),
                change: Change::NumberDevices(preview),
            })
        }
        "numerar_paginas" => {
            let document = document_by_name(project, text(request, "documento"))?;
            let prefix = request
                .get("prefixo")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .to_owned();
            let start = request.get("inicio").and_then(Value::as_u64).unwrap_or(1);
            let start = u32::try_from(start).map_err(|_| "início inválido")?;
            let preview = project
                .preview_page_numbering(document, &prefix, start)
                .map_err(|e| e.to_string())?;
            let changed: Vec<_> = preview
                .rows
                .iter()
                .filter(|r| r.before != r.after)
                .collect();
            if changed.is_empty() {
                return Err("as páginas já estão nessa numeração".into());
            }
            let lines = changed
                .iter()
                .map(|r| format!("Página {} → {}", r.before.as_str(), r.after.as_str()))
                .collect();
            Ok(Proposal {
                title: format!("Renumerar {} página(s)", changed.len()),
                lines: bounded(lines, changed.len()),
                change: Change::NumberPages(preview),
            })
        }
        "numero_fio" => {
            let wire = text(request, "fio").ok_or("informe o fio")?;
            let number = text(request, "numero").ok_or("informe o número")?;
            let locked = request
                .get("travado")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            let connection = project
                .connections()
                .find(|c| {
                    c.id().to_string() == wire
                        || c.number()
                            .is_some_and(|n| n.as_str().eq_ignore_ascii_case(&wire))
                })
                .map(|c| c.id())
                .ok_or_else(|| format!("fio não encontrado: {wire}"))?;
            let number = Text::new(number).map_err(|e| e.to_string())?;
            let mut copy = project.clone();
            copy.set_connection_number(connection, Some(number.clone()), locked)
                .map_err(|e| e.to_string())?;
            copy.validate().map_err(|e| e.to_string())?;
            Ok(Proposal {
                title: format!("Número do fio {wire} → {}", number.as_str()),
                lines: vec![format!(
                    "Fio {wire} recebe o número {}{}",
                    number.as_str(),
                    if locked { " (travado)" } else { "" }
                )],
                change: Change::WireNumber {
                    connection,
                    number,
                    locked,
                },
            })
        }
        "novo_documento" => {
            let name = text(request, "nome").ok_or("informe o nome do documento")?;
            let mut copy = project.clone();
            let document = Document::new(copy.id(), name.as_str()).map_err(|e| e.to_string())?;
            copy.edit_document(DocumentEdit::InsertDocument(document))
                .map_err(|e| e.to_string())?;
            Ok(Proposal {
                title: format!("Novo documento \"{name}\""),
                lines: vec![format!("Cria o documento \"{name}\", ainda sem páginas")],
                change: Change::NewDocument { name },
            })
        }
        "nova_pagina" => {
            let document = document_by_name(project, text(request, "documento"))?;
            let name = text(request, "nome").ok_or("informe o nome da página")?;
            let number = text(request, "numero").ok_or("informe o número da página")?;
            let format = text(request, "formato").unwrap_or_else(|| "A3".into());
            let paper = PaperFormat::ALL
                .into_iter()
                .find(|f| f.label().eq_ignore_ascii_case(&format))
                .ok_or_else(|| format!("formato desconhecido: {format}"))?;
            let size = paper.size(true);
            let mut copy = project.clone();
            let page = Page::new(
                copy.id(),
                document,
                name.as_str(),
                number.as_str(),
                PageKind::MultiLine,
                size,
            )
            .map_err(|e| e.to_string())?;
            copy.edit_document(DocumentEdit::InsertPage(Box::new(page)))
                .map_err(|e| e.to_string())?;
            Ok(Proposal {
                title: format!("Nova página {number} \"{name}\""),
                lines: vec![format!(
                    "Página {number} \"{name}\", {} paisagem, multifilar",
                    paper.label()
                )],
                change: Change::NewPage {
                    document,
                    name,
                    number,
                    size,
                },
            })
        }
        other => Err(format!("tipo de proposta desconhecido: {other}")),
    }
}
