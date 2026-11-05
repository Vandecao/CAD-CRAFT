// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 19: capturar e inserir macros como transação única e reversível.
//!
//! - Capturar: a seleção (símbolos, conteúdo livre, marcadores) leva junto os
//!   dispositivos com todas as funções e pontos, peças atribuídas, definições
//!   de símbolo, vínculos porta→ponto, fios internos (as duas pontas na
//!   seleção) e junções cujas rotas entram todas. O resto é relatado.
//! - Inserir: troca todos os IDs por novos (UUID v7), dá a próxima designação
//!   livre por prefixo a cada dispositivo, aplica a transformação, acrescenta
//!   símbolos e peças ausentes (versão diferente: mantém a do projeto e avisa)
//!   e confere o projeto inteiro antes de aceitar.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

/// O que ficou de fora ou foi adaptado ao capturar ou inserir.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MacroReport {
    /// Fios com uma ponta fora da seleção.
    pub dropped_routes: usize,
    /// Junções com alguma rota fora da seleção.
    pub dropped_junctions: usize,
    /// Marcadores ligados a ponto fora da seleção.
    pub dropped_markers: usize,
    /// Representações de engenharia física (bornes, cabos, condutores...).
    pub dropped_physical: usize,
    /// Vínculos de ponto com borne descartados.
    pub cleared_terminals: usize,
    /// Símbolos do projeto mantidos com versão diferente da macro.
    pub kept_symbols: Vec<SymbolDefinitionId>,
    /// Peças do projeto mantidas com dados diferentes da macro.
    pub kept_parts: Vec<PartId>,
}
impl MacroReport {
    pub fn is_clean(&self) -> bool {
        *self == Self::default()
    }
    /// Texto curto em pt-BR para a interface.
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        let mut count = |n: usize, what: &str| {
            if n > 0 {
                parts.push(format!("{n} {what}"));
            }
        };
        count(self.dropped_routes, "fio(s) com ponta fora da seleção");
        count(self.dropped_junctions, "junção(ões) parciais");
        count(self.dropped_markers, "marcador(es) sem ponto na seleção");
        count(self.dropped_physical, "elemento(s) de engenharia física");
        count(self.cleared_terminals, "vínculo(s) de borne");
        let mut text = if parts.is_empty() {
            String::new()
        } else {
            format!("Descartados: {}.", parts.join(", "))
        };
        if !self.kept_symbols.is_empty() {
            text.push_str(&format!(
                " Mantida a versão do projeto de {} símbolo(s): {}.",
                self.kept_symbols.len(),
                self.kept_symbols
                    .iter()
                    .map(SymbolDefinitionId::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !self.kept_parts.is_empty() {
            text.push_str(&format!(
                " Mantidos os dados do projeto de {} peça(s): {}.",
                self.kept_parts.len(),
                self.kept_parts
                    .iter()
                    .map(PartId::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        text.trim().to_owned()
    }
}

/// Onde e como inserir: `placement` é aplicado ao conteúdo depois de levar o
/// ponto base à origem (a translação de `placement` é o ponto de inserção).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MacroPlacement {
    pub page: PageId,
    pub base: Point2D,
    pub transform: Transform2D,
}

/// Entidades escritas por uma inserção (forma final, com IDs novos).
#[derive(Clone, Debug, Default, PartialEq)]
struct Batch {
    definitions: Vec<SymbolDefinition>,
    parts: Vec<Part>,
    devices: Vec<Device>,
    functions: Vec<Function>,
    points: Vec<ConnectionPoint>,
    components: Vec<Component>,
    representations: Vec<Representation>,
    bindings: Vec<(PortRef, ConnectionPointId)>,
    markers: Vec<ElectricalMarker>,
    connections: Vec<Connection>,
    routes: Vec<ConnectionRoute>,
    junctions: Vec<ConnectionJunction>,
}
impl Batch {
    fn len(&self) -> usize {
        self.definitions.len()
            + self.parts.len()
            + self.devices.len()
            + self.functions.len()
            + self.points.len()
            + self.components.len()
            + self.representations.len()
            + self.bindings.len()
            + self.markers.len()
            + self.connections.len()
            + self.routes.len()
            + self.junctions.len()
    }
}

/// Resultado reversível de inserir uma macro (ou colar com fios).
#[derive(Clone, Debug, PartialEq)]
pub struct MacroInsertDelta {
    project: ProjectId,
    page: Option<DocumentDelta>,
    batch: Batch,
}
impl MacroInsertDelta {
    pub fn len(&self) -> usize {
        self.batch.len() + usize::from(self.page.is_some())
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Página criada por uma macro de página.
    pub fn created_page(&self) -> Option<PageId> {
        match &self.page {
            Some(DocumentDelta::PageInserted(page)) => Some(page.id()),
            _ => None,
        }
    }
    /// Ocorrências visíveis inseridas (símbolos, conteúdo livre, marcadores e
    /// fios), para selecionar após a inserção.
    pub fn representation_ids(&self) -> Vec<RepresentationId> {
        self.batch
            .representations
            .iter()
            .map(Representation::id)
            .collect()
    }
    pub fn device_ids(&self) -> Vec<DeviceId> {
        self.batch.devices.iter().map(Device::id).collect()
    }
    pub fn route_ids(&self) -> Vec<RepresentationId> {
        self.batch.routes.iter().map(ConnectionRoute::id).collect()
    }
}

fn macro_error(reason: &'static str) -> DomainError {
    DomainError::InvalidRelation(reason)
}
fn json_error(error: serde_json::Error) -> DomainError {
    let _ = error;
    DomainError::InvalidRelation("conteúdo de macro inconsistente")
}

/// Prefixo da designação (ex.: "K" de "K12").
fn prefix_of(device: &str) -> String {
    let prefix = device.trim_end_matches(|c: char| c.is_ascii_digit());
    if prefix.is_empty() {
        "D".to_owned()
    } else {
        prefix.to_owned()
    }
}

/// Troca recursiva de textos (IDs) segundo o mapa.
fn remap(value: &mut serde_json::Value, map: &BTreeMap<String, String>) {
    match value {
        serde_json::Value::String(s) => {
            if let Some(new) = map.get(s.as_str()) {
                *s = new.clone();
            }
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(|v| remap(v, map)),
        serde_json::Value::Object(fields) => fields.values_mut().for_each(|v| remap(v, map)),
        _ => {}
    }
}

mod capture;
mod insertion;
