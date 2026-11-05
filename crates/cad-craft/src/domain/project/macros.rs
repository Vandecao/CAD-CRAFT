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

impl Project {
    /// Captura a seleção de uma página como conteúdo de macro.
    pub fn capture_macro(&self, ids: &[RepresentationId]) -> Result<(MacroContent, MacroReport)> {
        if ids.is_empty() {
            return Err(macro_error("selecione símbolos, fios ou conteúdo livre"));
        }
        if ids.len() > MAX_MACRO_ITEMS {
            return Err(macro_error("seleção grande demais para uma macro"));
        }
        let mut page = None;
        for id in ids {
            let r = self
                .representation(*id)
                .ok_or(DomainError::MissingRepresentation(*id))?;
            if *page.get_or_insert(r.page_id()) != r.page_id() {
                return Err(macro_error("a seleção deve estar numa única página"));
            }
        }
        let page = page.expect("seleção não vazia");
        self.capture_on(page, ids.iter().copied().collect())
    }

    /// Captura a página inteira (sem a moldura, que é propriedade da página).
    pub fn capture_page_macro(&self, page: PageId) -> Result<(MacroContent, MacroReport)> {
        self.require(EntityRef::Page(page))?;
        let ids: BTreeSet<_> = self
            .representations
            .values()
            .filter(|r| r.page_id() == page)
            .map(Representation::id)
            .collect();
        if ids.is_empty() {
            return Err(macro_error("página sem conteúdo para a macro"));
        }
        self.capture_on(page, ids)
    }

    fn capture_on(
        &self,
        page: PageId,
        selected: BTreeSet<RepresentationId>,
    ) -> Result<(MacroContent, MacroReport)> {
        let mut report = MacroReport::default();
        let selected_routes: BTreeSet<_> = selected
            .iter()
            .copied()
            .filter(|id| self.connection_routes.contains_key(id))
            .collect();
        // 1. Ocorrências de símbolo/conteúdo livre admitidas.
        let mut kept: BTreeSet<RepresentationId> = BTreeSet::new();
        let mut functions: BTreeSet<FunctionId> = BTreeSet::new();
        let mut devices: BTreeSet<DeviceId> = BTreeSet::new();
        let mut marker_ids = Vec::new();
        for id in selected.difference(&selected_routes) {
            let r = self
                .representation(*id)
                .ok_or(DomainError::MissingRepresentation(*id))?;
            if self.markers.contains_key(id) {
                marker_ids.push(*id);
                continue;
            }
            if r.graphic().is_some() {
                kept.insert(*id);
                continue;
            }
            if r.symbol().is_none() {
                report.dropped_physical += 1;
                continue;
            }
            match r.target() {
                RepresentationTarget::Unassociated => {}
                RepresentationTarget::Function(f) => {
                    let function = self
                        .function(f)
                        .ok_or(DomainError::MissingEntity(EntityRef::Function(f)))?;
                    functions.insert(f);
                    devices.insert(function.device_id());
                }
                RepresentationTarget::Device(d) => {
                    self.require(EntityRef::Device(d))?;
                    devices.insert(d);
                }
                _ => {
                    report.dropped_physical += 1;
                    continue;
                }
            }
            kept.insert(*id);
        }
        // 2. Dispositivos levam todas as funções e pontos (as não representadas
        //    na seleção ficam "não representadas" no destino).
        for d in &devices {
            functions.extend(self.functions_for_device(*d).map(Function::id));
        }
        let points: BTreeSet<ConnectionPointId> = self
            .connection_points
            .values()
            .filter(|p| functions.contains(&p.function_id()))
            .map(ConnectionPoint::id)
            .collect();
        // 3. Marcadores só com o ponto dentro da seleção.
        let mut markers = Vec::new();
        for id in marker_ids {
            let m = self.markers.get(&id).expect("marcador conferido");
            if m.point().is_none_or(|p| points.contains(&p)) {
                kept.insert(id);
                markers.push(m.clone());
            } else {
                report.dropped_markers += 1;
            }
        }
        // 4. Fios internos: as duas pontas em ocorrências mantidas.
        let mut routes = Vec::new();
        for r in self.connection_routes.values() {
            if r.page_id() != page {
                continue;
            }
            let inside = kept.contains(&r.source().representation_id())
                && kept.contains(&r.target().representation_id());
            if inside {
                routes.push(r.clone());
            } else if selected_routes.contains(&r.id()) {
                report.dropped_routes += 1;
            }
        }
        let route_ids: BTreeSet<_> = routes.iter().map(ConnectionRoute::id).collect();
        let mut junctions = Vec::new();
        for j in self.connection_junctions.values() {
            if j.page_id() != page {
                continue;
            }
            let inside = j.routes().iter().filter(|r| route_ids.contains(r)).count();
            if inside == j.routes().len() {
                junctions.push(j.clone());
            } else if inside > 0 {
                report.dropped_junctions += 1;
            }
        }
        let mut connection_ids: BTreeSet<ConnectionId> =
            routes.iter().map(ConnectionRoute::connection_id).collect();
        for j in &junctions {
            connection_ids.extend(j.connections().iter().copied());
        }
        // 5. Cadastro.
        let mut point_entities = Vec::new();
        for id in &points {
            let mut p = self.connection_points[id].clone();
            if p.terminal_id.is_some() || p.terminal_level.is_some() {
                p.terminal_id = None;
                p.terminal_level = None;
                report.cleared_terminals += 1;
            }
            point_entities.push(p);
        }
        let components: Vec<Component> = self
            .components
            .values()
            .filter(|c| c.device_id().is_some_and(|d| devices.contains(&d)))
            .cloned()
            .collect();
        let part_ids: BTreeSet<PartId> = components
            .iter()
            .filter_map(|c| c.part_id().cloned())
            .collect();
        let parts = part_ids
            .iter()
            .map(|id| {
                self.parts
                    .get(id)
                    .cloned()
                    .ok_or(macro_error("peça atribuída ausente no projeto"))
            })
            .collect::<Result<Vec<_>>>()?;
        let representations: Vec<Representation> = kept
            .iter()
            .map(|id| self.representations[id].clone())
            .collect();
        let definition_ids: BTreeSet<SymbolDefinitionId> = representations
            .iter()
            .filter_map(|r| r.symbol().cloned())
            .collect();
        let definitions = definition_ids
            .iter()
            .map(|id| {
                self.symbol_definition(id)
                    .cloned()
                    .ok_or(macro_error("definição de símbolo ausente"))
            })
            .collect::<Result<Vec<_>>>()?;
        let bindings = self
            .port_bindings
            .iter()
            .filter(|(port, _)| kept.contains(&port.representation_id()))
            .map(|(port, &point)| MacroBinding {
                port: port.clone(),
                point,
            })
            .collect();
        let content = MacroContent {
            project: self.id,
            page,
            devices: devices.iter().map(|d| self.devices[d].clone()).collect(),
            functions: functions
                .iter()
                .map(|f| self.functions[f].clone())
                .collect(),
            points: point_entities,
            components,
            parts,
            definitions,
            representations,
            bindings,
            markers,
            connections: connection_ids
                .iter()
                .map(|c| {
                    self.connections
                        .get(c)
                        .cloned()
                        .ok_or(macro_error("conexão de fio ausente"))
                })
                .collect::<Result<Vec<_>>>()?,
            routes,
            junctions,
        };
        if content.is_empty() {
            return Err(macro_error(
                "nada na seleção pode entrar na macro (só engenharia física?)",
            ));
        }
        if content.len() > MAX_MACRO_ITEMS {
            return Err(macro_error("seleção grande demais para uma macro"));
        }
        // Propriedades personalizadas não podem apontar para fora da macro.
        let inside = |r: &EntityRef| match r {
            EntityRef::Device(id) => devices.contains(id),
            EntityRef::Function(id) => functions.contains(id),
            EntityRef::ConnectionPoint(id) => points.contains(id),
            EntityRef::Component(id) => content.components.iter().any(|c| c.id() == *id),
            _ => false,
        };
        let outside = content
            .devices
            .iter()
            .flat_map(|d| d.properties().references())
            .chain(
                content
                    .functions
                    .iter()
                    .flat_map(|f| f.properties().references()),
            )
            .chain(
                content
                    .points
                    .iter()
                    .flat_map(|p| p.properties().references()),
            )
            .chain(
                content
                    .components
                    .iter()
                    .flat_map(|c| c.properties().references()),
            )
            .any(|r| !inside(&r));
        if outside {
            return Err(macro_error(
                "propriedade personalizada referencia entidade fora da seleção",
            ));
        }
        Ok((content, report))
    }

    /// Insere o conteúdo na página indicada (uma transação).
    pub fn insert_macro_content(
        &mut self,
        content: &MacroContent,
        placement: MacroPlacement,
    ) -> Result<(MacroInsertDelta, MacroReport)> {
        self.require(EntityRef::Page(placement.page))?;
        let (batch, report) = self.prepare_batch(content, placement)?;
        Ok((
            MacroInsertDelta {
                project: self.id,
                page: None,
                batch,
            },
            report,
        ))
    }

    /// Insere uma macro de janela.
    pub fn insert_macro(
        &mut self,
        definition: &MacroDefinition,
        page: PageId,
        transform: Transform2D,
    ) -> Result<(MacroInsertDelta, MacroReport)> {
        self.insert_macro_content(
            definition.content(),
            MacroPlacement {
                page,
                base: definition.base(),
                transform,
            },
        )
    }

    /// Macro de página: cria a página no documento (no fim) e insere o
    /// conteúdo nas mesmas coordenadas da página de origem.
    pub fn insert_page_macro(
        &mut self,
        definition: &MacroDefinition,
        document: DocumentId,
        number: &str,
        frame: Option<FrameTemplateId>,
    ) -> Result<(MacroInsertDelta, MacroReport)> {
        let MacroKind::Page { name, kind, size } = definition.kind() else {
            return Err(macro_error("a macro não é de página"));
        };
        let page = Page::new(
            self.id,
            document,
            name.as_str(),
            number,
            kind.clone(),
            *size,
        )?
        .with_frame(frame);
        let page_id = page.id();
        let page_delta = self.edit_document(DocumentEdit::InsertPage(Box::new(page)))?;
        let identity = Transform2D::new(Point2D::new(0.0, 0.0)?, 0.0, false, false)?;
        match self.prepare_batch(
            definition.content(),
            MacroPlacement {
                page: page_id,
                base: Point2D::new(0.0, 0.0)?,
                transform: identity,
            },
        ) {
            Ok((batch, report)) => Ok((
                MacroInsertDelta {
                    project: self.id,
                    page: Some(page_delta),
                    batch,
                },
                report,
            )),
            Err(error) => {
                let _ = self.restore_document_edit(&page_delta, false);
                Err(error)
            }
        }
    }

    /// Validação completa do conteúdo: inserção num projeto de rascunho.
    pub(crate) fn check_macro_content(content: &MacroContent) -> Result<()> {
        let mut scratch = Project::new("macro")?;
        let document = Document::new(scratch.id, "macro")?;
        let document_id = document.id();
        scratch.edit_document(DocumentEdit::InsertDocument(document))?;
        let page = Page::new(
            scratch.id,
            document_id,
            "macro",
            "1",
            PageKind::MultiLine,
            Size2D::new(420.0, 297.0)?,
        )?;
        let page_id = page.id();
        scratch.edit_document(DocumentEdit::InsertPage(Box::new(page)))?;
        scratch.prepare_batch(
            content,
            MacroPlacement {
                page: page_id,
                base: Point2D::new(0.0, 0.0)?,
                transform: Transform2D::new(Point2D::new(0.0, 0.0)?, 0.0, false, false)?,
            },
        )?;
        Ok(())
    }

    /// Gera IDs, designações e geometria, escreve tudo e confere o projeto.
    fn prepare_batch(
        &mut self,
        content: &MacroContent,
        placement: MacroPlacement,
    ) -> Result<(Batch, MacroReport)> {
        if content.len() > MAX_MACRO_ITEMS {
            return Err(macro_error("macro grande demais"));
        }
        if content.is_empty() {
            return Err(macro_error("macro sem conteúdo"));
        }
        let mut report = MacroReport::default();
        // 1. Mapa de IDs: cada entidade de origem ganha um UUID v7 novo.
        let mut map: BTreeMap<String, String> = BTreeMap::new();
        let fresh = || uuid::Uuid::now_v7().to_string();
        let mut add = |old: String| {
            map.entry(old).or_insert_with(fresh);
        };
        content.devices.iter().for_each(|e| add(e.id().to_string()));
        content
            .functions
            .iter()
            .for_each(|e| add(e.id().to_string()));
        content.points.iter().for_each(|e| add(e.id().to_string()));
        content
            .components
            .iter()
            .for_each(|e| add(e.id().to_string()));
        content
            .representations
            .iter()
            .for_each(|e| add(e.id().to_string()));
        content
            .connections
            .iter()
            .for_each(|e| add(e.id().to_string()));
        content.routes.iter().for_each(|e| add(e.id().to_string()));
        content
            .junctions
            .iter()
            .for_each(|e| add(e.id().to_string()));
        map.insert(content.project.to_string(), self.id.to_string());
        map.insert(content.page.to_string(), placement.page.to_string());
        // 2. Troca dos IDs no JSON e designações livres.
        let mut value = serde_json::to_value(content).map_err(json_error)?;
        let definitions = value
            .get_mut("definitions")
            .map(serde_json::Value::take)
            .unwrap_or_default();
        let parts = value
            .get_mut("parts")
            .map(serde_json::Value::take)
            .unwrap_or_default();
        remap(&mut value, &map);
        value["definitions"] = definitions;
        value["parts"] = parts;
        self.rename_devices(
            &mut value,
            placement.page,
            placement.transform.translation(),
        )?;
        let mut next: MacroContent = serde_json::from_value(value).map_err(json_error)?;
        // 3. Geometria.
        let total = placement_from_base(placement.base, placement.transform)?;
        for r in &mut next.representations {
            r.transform = r.transform.then(total)?;
        }
        let routes = next
            .routes
            .iter()
            .map(|r| Ok((r.has_auto_clearance(), r.transformed(total)?)))
            .collect::<Result<Vec<_>>>()?;
        for j in &mut next.junctions {
            j.position = total.apply(j.position)?;
        }
        // 4. Símbolos e peças: acrescenta os ausentes; mantém os do projeto.
        let mut batch = Batch::default();
        for d in next.definitions.drain(..) {
            match self.symbol_definition(d.id()) {
                None => batch.definitions.push(d),
                Some(existing) if existing != &d => report.kept_symbols.push(d.id().clone()),
                Some(_) => {}
            }
        }
        for p in next.parts.drain(..) {
            match self.parts.get(p.id()) {
                None => batch.parts.push(p),
                Some(existing) if existing != &p => report.kept_parts.push(p.id().clone()),
                Some(_) => {}
            }
        }
        batch.devices = next.devices;
        batch.functions = next.functions;
        batch.points = next.points;
        batch.components = next.components;
        batch.representations = next.representations;
        batch.bindings = next
            .bindings
            .into_iter()
            .map(|b| (b.port, b.point))
            .collect();
        batch.markers = next.markers;
        batch.connections = next.connections;
        // 5. Escreve o cadastro; os fios dependem das portas já posicionadas.
        self.preflight_batch(&batch, true)?;
        self.write_batch(&batch, true);
        let finished = (|| -> Result<Batch> {
            let mut wires = Batch::default();
            for (cleared, route) in routes {
                let a = self.port_position(route.source())?;
                let b = self.port_position(route.target())?;
                let mut after = route.with_endpoints(a, b)?;
                if cleared {
                    let points = self.clear_route_geometry(
                        &after,
                        after.points().to_vec(),
                        &BTreeMap::new(),
                    )?;
                    after = after.with_auto_clearance(points)?;
                }
                wires
                    .representations
                    .push(super::connection_drawing::route_representation(&after)?);
                wires.routes.push(after);
            }
            wires.junctions = std::mem::take(&mut next.junctions);
            self.preflight_batch(&wires, true)?;
            Ok(wires)
        })();
        let wires = match finished {
            Ok(wires) => wires,
            Err(error) => {
                self.write_batch(&batch, false);
                return Err(error);
            }
        };
        self.write_batch(&wires, true);
        batch.representations.extend(wires.representations);
        batch.routes = wires.routes;
        batch.junctions = wires.junctions;
        if let Err(error) = self.commit_batch() {
            self.write_batch(&batch, false);
            let _ = self.commit_batch();
            return Err(error);
        }
        Ok((batch, report))
    }

    /// Designação livre `{prefixo}{n}` por dispositivo, no mesmo caminho
    /// `=`/`+`; funções e peças instaladas acompanham o dispositivo.
    fn rename_devices(
        &self,
        value: &mut serde_json::Value,
        page: PageId,
        point: Point2D,
    ) -> Result<()> {
        let mut used: BTreeSet<(String, String, String)> = self
            .devices
            .values()
            .map(|d| {
                let des = serde_json::to_value(d.designation()).unwrap_or_default();
                (
                    des["functional_path"].to_string(),
                    des["location_path"].to_string(),
                    d.designation().device().as_str().to_owned(),
                )
            })
            .collect();
        let mut renamed: BTreeMap<String, serde_json::Value> = BTreeMap::new();
        let devices = value["devices"]
            .as_array_mut()
            .ok_or(macro_error("conteúdo de macro inconsistente"))?;
        // Ordem estável: pela designação de origem.
        let mut order: Vec<usize> = (0..devices.len()).collect();
        order.sort_by_key(|&i| {
            devices[i]["designation"]["device"]
                .as_str()
                .map(str::to_owned)
        });
        for i in order {
            let device = &mut devices[i];
            let des = &mut device["designation"];
            let name = des["device"].as_str().unwrap_or("D").to_owned();
            let prefix = if self.numbering_scheme.use_on_insert {
                name.chars()
                    .take_while(|c| c.is_alphabetic())
                    .collect::<String>()
            } else {
                prefix_of(&name)
            };
            let (fp, lp) = (
                des["functional_path"].to_string(),
                des["location_path"].to_string(),
            );
            let mut n = if self.numbering_scheme.use_on_insert {
                self.numbering_scheme.start
            } else {
                1
            };
            let base = if self.numbering_scheme.use_on_insert
                && self.numbering_scheme.device_format == DeviceNumberFormat::PageColumn
            {
                let p = self.page(page).ok_or(macro_error("página ausente"))?;
                let column = self
                    .zone_at(page, point)
                    .and_then(|z| z.column)
                    .unwrap_or_else(|| "1".into());
                Some(format!("{prefix}{}.{}", p.number().as_str(), column))
            } else {
                None
            };
            let new = loop {
                let candidate = match &base {
                    Some(base) if n == self.numbering_scheme.start => base.clone(),
                    Some(base) => format!("{base}.{n}"),
                    None => format!("{prefix}{n}"),
                };
                if !used.contains(&(fp.clone(), lp.clone(), candidate.clone())) {
                    break candidate;
                }
                n = n
                    .checked_add(if self.numbering_scheme.use_on_insert {
                        self.numbering_scheme.step
                    } else {
                        1
                    })
                    .ok_or(macro_error("numeração excedeu o limite"))?;
            };
            DesignationPart::new(new.clone())?;
            used.insert((fp, lp, new.clone()));
            des["device"] = serde_json::Value::String(new);
            renamed.insert(
                device["id"].as_str().unwrap_or_default().to_owned(),
                device["designation"].clone(),
            );
        }
        let follow = |entity: &mut serde_json::Value| {
            let Some(designation) = entity["device_id"]
                .as_str()
                .and_then(|d| renamed.get(d))
                .cloned()
            else {
                return;
            };
            let des = &mut entity["designation"];
            for key in ["functional_path", "location_path", "device"] {
                des[key] = designation[key].clone();
            }
        };
        for key in ["functions", "components"] {
            if let Some(items) = value[key].as_array_mut() {
                items.iter_mut().for_each(&follow);
            }
        }
        Ok(())
    }

    fn preflight_batch(&self, b: &Batch, forward: bool) -> Result<()> {
        fn check<K: Ord, V: PartialEq>(
            map: &BTreeMap<K, V>,
            items: impl Iterator<Item = (K, V)>,
            forward: bool,
        ) -> bool {
            let mut ok = true;
            for (k, v) in items {
                ok &= if forward {
                    !map.contains_key(&k)
                } else {
                    map.get(&k) == Some(&v)
                };
            }
            ok
        }
        let ok = check(
            &self.symbol_definitions,
            b.definitions.iter().map(|d| (d.id().clone(), d.clone())),
            forward,
        ) && check(
            &self.parts,
            b.parts.iter().map(|p| (p.id().clone(), p.clone())),
            forward,
        ) && check(
            &self.devices,
            b.devices.iter().map(|e| (e.id(), e.clone())),
            forward,
        ) && check(
            &self.functions,
            b.functions.iter().map(|e| (e.id(), e.clone())),
            forward,
        ) && check(
            &self.connection_points,
            b.points.iter().map(|e| (e.id(), e.clone())),
            forward,
        ) && check(
            &self.components,
            b.components.iter().map(|e| (e.id(), e.clone())),
            forward,
        ) && check(
            &self.representations,
            b.representations.iter().map(|e| (e.id(), e.clone())),
            forward,
        ) && check(&self.port_bindings, b.bindings.iter().cloned(), forward)
            && check(
                &self.markers,
                b.markers.iter().map(|e| (e.id(), e.clone())),
                forward,
            )
            && check(
                &self.connections,
                b.connections.iter().map(|e| (e.id(), e.clone())),
                forward,
            )
            && check(
                &self.connection_routes,
                b.routes.iter().map(|e| (e.id(), e.clone())),
                forward,
            )
            && check(
                &self.connection_junctions,
                b.junctions.iter().map(|e| (e.id(), e.clone())),
                forward,
            );
        if ok {
            Ok(())
        } else {
            Err(macro_error("histórico da macro não corresponde ao projeto"))
        }
    }

    /// Escrita bruta (sem conferência); `forward = false` remove.
    fn write_batch(&mut self, b: &Batch, forward: bool) {
        fn put<K: Ord, V>(
            map: &mut BTreeMap<K, V>,
            items: impl Iterator<Item = (K, V)>,
            add: bool,
        ) {
            for (k, v) in items {
                if add {
                    map.insert(k, v);
                } else {
                    map.remove(&k);
                }
            }
        }
        put(
            &mut self.symbol_definitions,
            b.definitions.iter().map(|d| (d.id().clone(), d.clone())),
            forward,
        );
        put(
            &mut self.parts,
            b.parts.iter().map(|p| (p.id().clone(), p.clone())),
            forward,
        );
        put(
            &mut self.devices,
            b.devices.iter().map(|e| (e.id(), e.clone())),
            forward,
        );
        put(
            &mut self.functions,
            b.functions.iter().map(|e| (e.id(), e.clone())),
            forward,
        );
        put(
            &mut self.connection_points,
            b.points.iter().map(|e| (e.id(), e.clone())),
            forward,
        );
        put(
            &mut self.components,
            b.components.iter().map(|e| (e.id(), e.clone())),
            forward,
        );
        put(
            &mut self.representations,
            b.representations.iter().map(|e| (e.id(), e.clone())),
            forward,
        );
        put(&mut self.port_bindings, b.bindings.iter().cloned(), forward);
        put(
            &mut self.markers,
            b.markers.iter().map(|e| (e.id(), e.clone())),
            forward,
        );
        put(
            &mut self.connections,
            b.connections.iter().map(|e| (e.id(), e.clone())),
            forward,
        );
        put(
            &mut self.connection_routes,
            b.routes.iter().map(|e| (e.id(), e.clone())),
            forward,
        );
        put(
            &mut self.connection_junctions,
            b.junctions.iter().map(|e| (e.id(), e.clone())),
            forward,
        );
    }

    /// Reconstrói os índices derivados e valida o projeto inteiro.
    fn commit_batch(&mut self) -> Result<()> {
        let document_index = self.build_document_index()?;
        let graph = self.build_graph()?;
        let physical_index = self.build_physical_index()?;
        self.document_index = document_index;
        self.graph = graph;
        self.physical_index = physical_index;
        self.validate()
    }

    fn apply_batch(&mut self, b: &Batch, forward: bool) -> Result<()> {
        self.preflight_batch(b, forward)?;
        self.write_batch(b, forward);
        if let Err(error) = self.commit_batch() {
            self.write_batch(b, !forward);
            let _ = self.commit_batch();
            return Err(error);
        }
        Ok(())
    }

    /// Desfaz (`forward = false`) ou refaz a inserção.
    pub fn restore_macro_insert(&mut self, d: &MacroInsertDelta, forward: bool) -> Result<()> {
        self.check_project(d.project)?;
        if forward {
            if let Some(page) = &d.page {
                self.restore_document_edit(page, true)?;
            }
            if let Err(error) = self.apply_batch(&d.batch, true) {
                if let Some(page) = &d.page {
                    let _ = self.restore_document_edit(page, false);
                }
                return Err(error);
            }
        } else {
            self.apply_batch(&d.batch, false)?;
            if let Some(page) = &d.page
                && let Err(error) = self.restore_document_edit(page, false)
            {
                let _ = self.apply_batch(&d.batch, true);
                return Err(error);
            }
        }
        Ok(())
    }
}
