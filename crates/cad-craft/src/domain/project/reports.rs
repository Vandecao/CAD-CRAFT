// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::*;
use std::collections::BTreeSet;

fn row(cells: Vec<ReportCell>) -> ReportRow {
    ReportRow { cells }
}
fn cell(value: impl Into<String>) -> ReportCell {
    ReportCell::plain(value)
}
fn col(title: &'static str, width_mm: f64) -> ReportColumn {
    ReportColumn { title, width_mm }
}
fn index_includes_new_report(filter: &ReportFilter, document: DocumentId) -> bool {
    filter.document.is_none_or(|id| id == document)
        && filter.page.is_none()
        && filter.function_path.is_empty()
        && filter.location_path.is_empty()
}
fn page_count(rows: usize, per_page: usize, self_in_index: bool) -> usize {
    let mut count = rows.div_ceil(per_page).max(1);
    if self_in_index {
        loop {
            let needed = (rows + count).div_ceil(per_page).max(1);
            if needed == count {
                break;
            }
            count = needed;
        }
    }
    count
}

impl Project {
    pub(crate) fn report_rows_per_page(
        &self,
        size: Size2D,
        frame: Option<FrameTemplateId>,
    ) -> Result<usize> {
        let usable_height = if let Some(id) = frame.or(self.default_frame) {
            let frame = self
                .frame(id)
                .ok_or(DomainError::InvalidRelation("moldura do relatório ausente"))?;
            if frame.sheet() != size {
                return Err(DomainError::InvalidRelation(
                    "moldura e papel do relatório incompatíveis",
                ));
            }
            frame.usable().size().height()
        } else {
            size.height() - 20.0
        };
        let available = usable_height - 16.0;
        if available < 6.0 {
            return Err(DomainError::InvalidRelation(
                "área útil insuficiente para relatório",
            ));
        }
        Ok((available / 6.0).floor() as usize)
    }

    pub fn preview_report(&self, request: &ReportRequest) -> Result<ReportPreview> {
        self.require(EntityRef::Document(request.document))?;
        let table = self.report_table(request.kind, &request.filter)?;
        let rows_per_page = self.report_rows_per_page(request.size, request.frame)?;
        let self_in_index = request.kind == ReportKind::PageIndex
            && index_includes_new_report(&request.filter, request.document);
        let pages = page_count(table.rows.len(), rows_per_page, self_in_index);
        Ok(ReportPreview {
            rows: table.rows.len(),
            rows_per_page,
            pages,
        })
    }

    pub fn generate_report(&mut self, request: &ReportRequest) -> Result<ReportDelta> {
        let preview = self.preview_report(request)?;
        let group = ReportId::new();
        let after = (0..preview.pages)
            .map(|index| {
                let report = ReportDefinition {
                    group,
                    kind: request.kind,
                    filter: request.filter.clone(),
                    sort: request.sort,
                    columns: request.columns.clone(),
                    index: index as u32,
                };
                let mut page = Page::new(
                    self.id,
                    request.document,
                    request.kind.label(),
                    format!("R{}", self.pages.len() + index + 1),
                    PageKind::TechnicalDocumentation,
                    request.size,
                )?;
                page.frame = request.frame;
                page.report = Some(report);
                Ok(page)
            })
            .collect::<Result<Vec<_>>>()?;
        let delta = ReportDelta {
            project: self.id,
            before: Vec::new(),
            after,
        };
        self.restore_report(&delta, true)?;
        Ok(delta)
    }

    pub fn report_stale(&self, group: ReportId) -> Result<bool> {
        let pages: Vec<_> = self
            .pages
            .values()
            .filter(|p| p.report().is_some_and(|r| r.group == group))
            .collect();
        let first = pages
            .first()
            .ok_or(DomainError::InvalidRelation("grupo de relatório ausente"))?;
        let def = first.report().expect("grupo selecionado tem definição");
        let rows_per_page = self.report_rows_per_page(first.size(), first.frame())?;
        let current_rows = self.report_table(def.kind, &def.filter)?.rows.len();
        let self_in_index = def.kind == ReportKind::PageIndex
            && index_includes_new_report(&def.filter, first.document_id());
        let base_rows = if self_in_index {
            current_rows.saturating_sub(pages.len())
        } else {
            current_rows
        };
        let needed = page_count(base_rows, rows_per_page, self_in_index);
        Ok(pages.len() != needed)
    }

    pub fn update_report(&mut self, group: ReportId) -> Result<ReportDelta> {
        let mut before: Vec<_> = self
            .pages
            .values()
            .filter(|p| p.report().is_some_and(|r| r.group == group))
            .cloned()
            .collect();
        before.sort_by_key(|p| p.report().map(|r| r.index));
        let first = before
            .first()
            .ok_or(DomainError::InvalidRelation("grupo de relatório ausente"))?;
        let def = first
            .report()
            .expect("grupo selecionado tem definição")
            .clone();
        let rows_per_page = self.report_rows_per_page(first.size(), first.frame())?;
        let current_rows = self.report_table(def.kind, &def.filter)?.rows.len();
        let self_in_index = def.kind == ReportKind::PageIndex
            && index_includes_new_report(&def.filter, first.document_id());
        let base_rows = if self_in_index {
            current_rows.saturating_sub(before.len())
        } else {
            current_rows
        };
        let needed = page_count(base_rows, rows_per_page, self_in_index);
        let mut after = before.iter().take(needed).cloned().collect::<Vec<_>>();
        for index in after.len()..needed {
            let mut p = Page::new(
                self.id,
                first.document_id(),
                first.name().as_str(),
                format!("R{}", self.pages.len() + index + 1),
                PageKind::TechnicalDocumentation,
                first.size(),
            )?;
            p.frame = first.frame();
            p.report = Some(ReportDefinition {
                index: index as u32,
                ..def.clone()
            });
            after.push(p);
        }
        let delta = ReportDelta {
            project: self.id,
            before,
            after,
        };
        self.restore_report(&delta, true)?;
        Ok(delta)
    }

    pub fn update_reports(&mut self, groups: &[ReportId]) -> Result<ReportDelta> {
        let mut candidate = self.clone();
        let unique: BTreeSet<_> = groups.iter().copied().collect();
        // Um índice pode incluir páginas criadas por outro relatório do lote.
        // Recalcular até que todos os grupos tenham a contagem final.
        for _ in 0..=unique.len() {
            let mut changed = false;
            for group in &unique {
                if candidate.report_stale(*group)? {
                    candidate.update_report(*group)?;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        if unique
            .iter()
            .any(|group| candidate.report_stale(*group).unwrap_or(true))
        {
            return Err(DomainError::InvalidRelation("relatórios não convergiram"));
        }
        let before: Vec<_> = self
            .pages
            .values()
            .filter(|p| p.report().is_some_and(|r| unique.contains(&r.group)))
            .cloned()
            .collect();
        let after: Vec<_> = candidate
            .pages
            .values()
            .filter(|p| p.report().is_some_and(|r| unique.contains(&r.group)))
            .cloned()
            .collect();
        let delta = ReportDelta {
            project: self.id,
            before,
            after,
        };
        self.restore_report(&delta, true)?;
        Ok(delta)
    }

    pub fn restore_report(&mut self, delta: &ReportDelta, forward: bool) -> Result<()> {
        self.check_project(delta.project)?;
        let (expected, next) = if forward {
            (&delta.before, &delta.after)
        } else {
            (&delta.after, &delta.before)
        };
        let mut candidate = self.clone();
        for page in expected {
            if candidate.pages.get(&page.id()) != Some(page) {
                return Err(DomainError::InvalidRelation(
                    "histórico de relatório inconsistente",
                ));
            }
            candidate.pages.remove(&page.id());
        }
        for page in next {
            if candidate.pages.insert(page.id(), page.clone()).is_some() {
                return Err(DomainError::InvalidRelation(
                    "página de relatório já existe",
                ));
            }
        }
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    pub fn report_table(&self, kind: ReportKind, filter: &ReportFilter) -> Result<ReportTable> {
        if let Some(document) = filter.document {
            self.require(EntityRef::Document(document))?;
        }
        if let Some(page) = filter.page {
            self.require(EntityRef::Page(page))?;
        }
        let eligible = |page: &Page| {
            filter.document.is_none_or(|id| page.document_id() == id)
                && filter.page.is_none_or(|id| page.id() == id)
                && page.structure().functional_path().len() >= filter.function_path.len()
                && page
                    .structure()
                    .functional_path()
                    .iter()
                    .zip(&filter.function_path)
                    .all(|(p, f)| p.as_str() == f)
                && page.structure().location_path().len() >= filter.location_path.len()
                && page
                    .structure()
                    .location_path()
                    .iter()
                    .zip(&filter.location_path)
                    .all(|(p, f)| p.as_str() == f)
        };
        let page_allowed = |id: PageId| self.page(id).is_some_and(eligible);
        let all = filter.document.is_none()
            && filter.page.is_none()
            && filter.function_path.is_empty()
            && filter.location_path.is_empty();
        let (columns, rows) = match kind {
            ReportKind::PageIndex => {
                let columns = vec![
                    col("Documento", 35.0),
                    col("Número", 20.0),
                    col("Nome", 65.0),
                    col("Tipo", 35.0),
                    col("Data", 25.0),
                    col("Revisão", 20.0),
                ];
                let mut rows = Vec::new();
                for document in self.documents() {
                    for page in self
                        .ordered_pages(document.id())?
                        .into_iter()
                        .filter(|p| eligible(p))
                    {
                        let metadata = |key: &str| {
                            page.metadata()
                                .iter()
                                .find(|(k, _)| k.as_str().eq_ignore_ascii_case(key))
                                .map(|(_, v)| v.clone())
                                .unwrap_or_default()
                        };
                        let kind = match page.kind() {
                            PageKind::MultiLine => "Multifilar",
                            PageKind::SingleLine => "Unifilar",
                            PageKind::Overview => "Visão geral",
                            PageKind::PanelLayout => "Painel",
                            PageKind::TechnicalDocumentation => "Documentação técnica",
                            PageKind::Other(t) => t.as_str(),
                        };
                        rows.push(row(vec![
                            cell(document.name().as_str()),
                            cell(page.number().as_str()),
                            cell(page.name().as_str()),
                            cell(kind),
                            cell(metadata("Data")),
                            cell(metadata("Revisão")),
                        ]));
                    }
                }
                (columns, rows)
            }
            ReportKind::Devices => {
                let columns = vec![
                    col("Designação", 35.0),
                    col("Classificação", 38.0),
                    col("Funções", 38.0),
                    col("Peças", 40.0),
                    col("Ocorrências", 50.0),
                ];
                let mut rows = Vec::new();
                for device in self.devices() {
                    let refs = self.device_references(device.id())?;
                    let refs: Vec<_> = refs.into_iter().filter(|r| page_allowed(r.page)).collect();
                    if !all && refs.is_empty() {
                        continue;
                    }
                    let functions = self
                        .functions_for_device(device.id())
                        .map(|f| match f.kind() {
                            FunctionKind::Coil => "Bobina".to_owned(),
                            FunctionKind::AuxiliaryContact => "Contato auxiliar".to_owned(),
                            FunctionKind::MainContact => "Contato principal".to_owned(),
                            FunctionKind::Other(t) => t.as_str().to_owned(),
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    let parts = self
                        .bom_rows()
                        .into_iter()
                        .filter(|b| b.device == device.id())
                        .map(|b| b.code)
                        .collect::<Vec<_>>()
                        .join(", ");
                    let mut occurrence = cell(
                        refs.iter()
                            .map(|r| r.reference.as_str())
                            .collect::<Vec<_>>()
                            .join(", "),
                    );
                    occurrence.target = refs.first().map(|r| r.representation);
                    rows.push(row(vec![
                        cell(device.designation().to_string()),
                        cell(device.classification().as_str()),
                        cell(functions),
                        cell(parts),
                        occurrence,
                    ]));
                }
                rows.sort_by(|a, b| a.cells[0].text().cmp(&b.cells[0].text()));
                (columns, rows)
            }
            ReportKind::Materials => {
                let columns = vec![
                    col("Código", 40.0),
                    col("Fabricante", 38.0),
                    col("Descrição", 65.0),
                    col("Quantidade", 24.0),
                    col("Dispositivos", 53.0),
                ];
                let mut groups: BTreeMap<(String, String, String), (u64, Vec<String>)> =
                    BTreeMap::new();
                for b in self.bom_rows() {
                    if !all
                        && !self
                            .device_references(b.device)?
                            .iter()
                            .any(|r| page_allowed(r.page))
                    {
                        continue;
                    }
                    let key = if filter.materials_by_device {
                        format!("{}\0{}", b.code, b.device_designation)
                    } else {
                        b.code
                    };
                    let entry = groups
                        .entry((key, b.manufacturer, b.description))
                        .or_default();
                    entry.0 += u64::from(b.quantity);
                    entry.1.push(b.device_designation);
                }
                let rows = groups
                    .into_iter()
                    .map(|((code, maker, desc), (quantity, mut devices))| {
                        devices.sort();
                        devices.dedup();
                        row(vec![
                            cell(code.split('\0').next().unwrap_or_default()),
                            cell(maker),
                            cell(desc),
                            ReportCell::number(quantity as f64),
                            cell(devices.join(", ")),
                        ])
                    })
                    .collect();
                (columns, rows)
            }
            ReportKind::Connections => {
                let columns = vec![
                    col("Fio/Rede", 25.0),
                    col("Origem", 42.0),
                    col("Destino", 42.0),
                    col("Referência", 32.0),
                    col("Condutor", 25.0),
                    col("Seção", 22.0),
                    col("Cor", 32.0),
                ];
                let physical = self.physical_lists();
                let mut routes_by_connection: BTreeMap<ConnectionId, Vec<&ConnectionRoute>> =
                    BTreeMap::new();
                for route in self
                    .connection_routes()
                    .filter(|r| page_allowed(r.page_id()))
                {
                    routes_by_connection
                        .entry(route.connection_id())
                        .or_default()
                        .push(route);
                }
                let conductors_by_id: BTreeMap<ConductorId, &ConductorListRow> = physical
                    .conductors
                    .iter()
                    .map(|c| (c.conductor, c))
                    .collect();
                let mut network_names = BTreeMap::new();
                for (index, network) in self.networks().iter().enumerate() {
                    for connection in network.connections() {
                        network_names.insert(*connection, format!("Rede {}", index + 1));
                    }
                }
                let mut rows = Vec::new();
                for a in physical.assignments {
                    let routes = routes_by_connection.get(&a.connection);
                    if !all && routes.is_none() {
                        continue;
                    }
                    let number = self
                        .connection(a.connection)
                        .and_then(Connection::number)
                        .map_or_else(
                            || {
                                network_names
                                    .get(&a.connection)
                                    .cloned()
                                    .unwrap_or_default()
                            },
                            |t| t.as_str().to_owned(),
                        );
                    let route = routes.and_then(|r| r.first());
                    let mut reference = cell(route.map_or_else(String::new, |r| {
                        self.short_reference(r.page_id(), r.points()[0])
                    }));
                    reference.target = route.map(|r| r.id());
                    let conductors: Vec<_> = a
                        .conductors
                        .iter()
                        .filter_map(|id| conductors_by_id.get(id).copied())
                        .collect();
                    rows.push(row(vec![
                        cell(number),
                        cell(a.source),
                        cell(a.target),
                        reference,
                        cell(
                            conductors
                                .iter()
                                .map(|c| c.designation.as_str())
                                .collect::<Vec<_>>()
                                .join(", "),
                        ),
                        cell(
                            conductors
                                .first()
                                .map_or_else(String::new, |c| c.cross_section_mm2.to_string()),
                        ),
                        cell(
                            conductors
                                .iter()
                                .map(|c| c.marking.as_str())
                                .collect::<Vec<_>>()
                                .join(", "),
                        ),
                    ]));
                }
                rows.sort_by(|a, b| a.cells[0].text().cmp(&b.cells[0].text()));
                (columns, rows)
            }
            ReportKind::Terminals => {
                let columns = vec![
                    col("Régua", 28.0),
                    col("Borne", 22.0),
                    col("Ponte", 24.0),
                    col("Lado A", 38.0),
                    col("Lado B", 38.0),
                    col("Sem lado", 30.0),
                    col("Cabo/Veia", 40.0),
                ];
                let physical = self.physical_lists();
                let mut rows = Vec::new();
                for t in physical.terminals {
                    if !all
                        && self
                            .physical_occurrence(RepresentationTarget::Terminal(t.terminal))
                            .and_then(|id| self.representation(id))
                            .is_none_or(|r| !page_allowed(r.page_id()))
                    {
                        continue;
                    }
                    let mut sides: [Vec<String>; 3] = [Vec::new(), Vec::new(), Vec::new()];
                    let mut cable_cores = Vec::new();
                    for conductor in physical
                        .conductors
                        .iter()
                        .filter(|c| t.conductors.contains(&c.conductor))
                    {
                        let Some(wire) = self.conductor(conductor.conductor) else {
                            continue;
                        };
                        let (a, b) = wire.endpoints();
                        let (here, other) = if a.terminal_id() == t.terminal {
                            (a, b)
                        } else {
                            (b, a)
                        };
                        let side = self
                            .terminal(t.terminal)
                            .and_then(|terminal| terminal.side_of(here.position()));
                        let index = match side {
                            Some(TerminalSide::A) => 0,
                            Some(TerminalSide::B) => 1,
                            None => 2,
                        };
                        sides[index].push(format!(
                            "{}/{}",
                            self.terminal_label(other.terminal_id()),
                            other.position().as_str()
                        ));
                        if let Some(label) = &conductor.cable_label {
                            cable_cores.push(label.clone());
                        }
                    }
                    for values in &mut sides {
                        values.sort();
                        values.dedup();
                    }
                    cable_cores.sort();
                    cable_cores.dedup();
                    rows.push(row(vec![
                        cell(t.strip_designation.unwrap_or_default()),
                        cell(t.designation),
                        cell(t.bridges.len().to_string()),
                        cell(sides[0].join(", ")),
                        cell(sides[1].join(", ")),
                        cell(sides[2].join(", ")),
                        cell(cable_cores.join(", ")),
                    ]));
                }
                rows.sort_by(|a, b| {
                    (a.cells[0].text(), a.cells[1].text())
                        .cmp(&(b.cells[0].text(), b.cells[1].text()))
                });
                (columns, rows)
            }
            ReportKind::Cables => {
                let columns = vec![
                    col("Cabo", 32.0),
                    col("Tipo", 32.0),
                    col("Usadas", 23.0),
                    col("Livres", 23.0),
                    col("Origem", 45.0),
                    col("Destino", 45.0),
                    col("Comprimento", 25.0),
                ];
                let physical = self.physical_lists();
                let mut rows = Vec::new();
                for c in physical.cables {
                    if !all
                        && self
                            .physical_occurrence(RepresentationTarget::Cable(c.cable))
                            .and_then(|id| self.representation(id))
                            .is_none_or(|r| !page_allowed(r.page_id()))
                    {
                        continue;
                    }
                    let mut origins = Vec::new();
                    let mut destinations = Vec::new();
                    for core in physical.cores.iter().filter(|core| core.cable == c.cable) {
                        if let Some(source) = &core.source {
                            origins.push(source.clone());
                        }
                        if let Some(target) = &core.target {
                            destinations.push(target.clone());
                        }
                    }
                    origins.sort();
                    origins.dedup();
                    destinations.sort();
                    destinations.dedup();
                    let length = physical
                        .cores
                        .iter()
                        .filter(|core| core.cable == c.cable)
                        .filter_map(|core| core.conductor)
                        .filter_map(|id| self.conductor(id))
                        .filter_map(Conductor::length)
                        .map(LengthMm::get)
                        .reduce(f64::max)
                        .map_or_else(String::new, |v| v.to_string());
                    rows.push(row(vec![
                        cell(c.designation),
                        cell(format!("{:?}", c.kind)),
                        ReportCell::number(c.used_cores as f64),
                        ReportCell::number(
                            c.nominal_cores.saturating_sub(c.used_cores as u32) as f64
                        ),
                        cell(origins.join(", ")),
                        cell(destinations.join(", ")),
                        cell(length),
                    ]));
                }
                rows.sort_by(|a, b| a.cells[0].text().cmp(&b.cells[0].text()));
                (columns, rows)
            }
        };
        Ok(ReportTable {
            kind,
            columns,
            rows,
        })
    }
}
