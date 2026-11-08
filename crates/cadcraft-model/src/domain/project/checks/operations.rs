// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Pure registry checks in deterministic order; start with Project::check. Includes navigation
//! targets and reversible project check-scheme edits.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

/// Check scheme replacement with exact undo and redo.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckSchemeDelta {
    project: ProjectId,
    before: CheckScheme,
    after: CheckScheme,
}
impl CheckSchemeDelta {
    /// Whether this collection or recorded edit contains no effective entries or changes.
    pub fn is_empty(&self) -> bool {
        self.before == self.after
    }
}

impl Project {
    /// Return the check scheme.
    pub fn check_scheme(&self) -> &CheckScheme {
        &self.check_scheme
    }

    /// Validate and replace the check configuration, returning the reversible change.
    pub fn set_check_scheme(&mut self, scheme: CheckScheme) -> Result<CheckSchemeDelta> {
        scheme.validate()?;
        let delta = CheckSchemeDelta {
            project: self.id,
            before: self.check_scheme.clone(),
            after: scheme,
        };
        self.restore_check_scheme(&delta, true)?;
        Ok(delta)
    }

    /// Replay the recorded check scheme change; forward reapplies it and false restores the prior
    /// state. Reject a mismatched expected state.
    pub fn restore_check_scheme(&mut self, delta: &CheckSchemeDelta, forward: bool) -> Result<()> {
        self.check_project(delta.project)?;
        let (expected, next) = if forward {
            (&delta.before, &delta.after)
        } else {
            (&delta.after, &delta.before)
        };
        if &self.check_scheme != expected {
            return Err(DomainError::InvalidRelation(
                "histórico de verificação inconsistente",
            ));
        }
        next.validate()?;
        self.check_scheme = next.clone();
        Ok(())
    }

    /// Check the project or a page using the project's scheme.
    pub fn check(&self, scope: CheckScope) -> CheckReport {
        let scheme = &self.check_scheme;
        let mut report = CheckReport::default();
        for message in self.all_check_messages() {
            let setting = scheme.setting(message.rule);
            if !setting.enabled {
                continue;
            }
            if let CheckScope::Page(page) = scope
                && !message.targets.iter().any(|t| {
                    self.check_target_location(*t)
                        .is_some_and(|(p, _)| p == page)
                })
            {
                continue;
            }
            if scheme.is_ignored(&message.key()) {
                report.ignored += 1;
                continue;
            }
            report.messages.push(CheckMessage {
                severity: setting.severity,
                ..message
            });
        }
        report
    }

    /// First page and occurrence displaying the target, in stable order.
    pub fn check_target_location(
        &self,
        target: CheckTarget,
    ) -> Option<(PageId, Option<RepresentationId>)> {
        let rep =
            |id: RepresentationId| self.representation(id).map(|r| (r.page_id(), Some(r.id())));
        let first = |pred: &dyn Fn(&Representation) -> bool| {
            self.representations()
                .filter(|r| pred(r))
                .map(Representation::id)
                .min()
                .and_then(rep)
        };
        match target {
            CheckTarget::Page(id) => self.page(id).map(|p| (p.id(), None)),
            CheckTarget::Representation(id) => rep(id),
            CheckTarget::Device(id) => first(&|r| self.device_for_target(r.target()) == Some(id)),
            CheckTarget::Function(id) => {
                first(&|r| self.function_for_target(r.target()) == Some(id))
            }
            CheckTarget::Connection(id) => self
                .connection_routes
                .values()
                .filter(|r| r.connection_id() == id)
                .map(ConnectionRoute::id)
                .min()
                .and_then(rep),
            CheckTarget::ConnectionPoint(id) => self
                .port_bindings
                .iter()
                .filter(|(_, point)| **point == id)
                .map(|(port, _)| port.representation_id())
                .min()
                .and_then(rep),
            CheckTarget::Terminal(id) => self
                .connection_points()
                .filter(|p| p.terminal_id() == Some(id))
                .find_map(|p| self.check_target_location(CheckTarget::ConnectionPoint(p.id()))),
            CheckTarget::Cable(_) | CheckTarget::Connector(_) => None,
        }
    }

    fn all_check_messages(&self) -> Vec<CheckMessage> {
        let mut out = vec![];
        self.check_designations(&mut out);
        self.check_functions(&mut out);
        self.check_connections(&mut out);
        self.check_markers(&mut out);
        self.check_physical(&mut out);
        self.check_parts_and_reports(&mut out);
        out
    }

    fn check_designations(&self, out: &mut Vec<CheckMessage>) {
        for ids in self.designation_conflicts() {
            let name = self
                .device(ids[0])
                .map(|d| d.designation().to_string())
                .unwrap_or_default();
            out.push(CheckMessage::new(
                "VER-001",
                format!("{name}: {} dispositivos com a mesma designação", ids.len()),
                ids.into_iter().map(CheckTarget::Device).collect(),
            ));
        }
        // CON-001: equal names never merge networks; report a warning only.
        let mut connectors: BTreeMap<String, Vec<ConnectorId>> = BTreeMap::new();
        for c in self.connectors() {
            connectors
                .entry(c.designation().as_str().to_owned())
                .or_default()
                .push(c.id());
        }
        for (name, ids) in connectors.into_iter().filter(|(_, ids)| ids.len() > 1) {
            out.push(CheckMessage::new(
                "VER-002",
                format!(
                    "Conector -{name}: {} conectores com a mesma designação",
                    ids.len()
                ),
                ids.into_iter().map(CheckTarget::Connector).collect(),
            ));
        }
        for document in self.documents() {
            let mut numbers: BTreeMap<&str, Vec<PageId>> = BTreeMap::new();
            for page in self.ordered_pages(document.id()).unwrap_or_default() {
                numbers
                    .entry(page.number().as_str())
                    .or_default()
                    .push(page.id());
            }
            for (number, ids) in numbers.into_iter().filter(|(_, ids)| ids.len() > 1) {
                out.push(CheckMessage::new(
                    "VER-003",
                    format!(
                        "Página {number} repetida no documento {}",
                        document.name().as_str()
                    ),
                    ids.into_iter().map(CheckTarget::Page).collect(),
                ));
            }
        }
    }

    fn check_functions(&self, out: &mut Vec<CheckMessage>) {
        // Build indexes once to avoid scanning functions and parts per device.
        let mut by_device: BTreeMap<DeviceId, Vec<&Function>> = BTreeMap::new();
        for f in self.functions() {
            by_device.entry(f.device_id()).or_default().push(f);
        }
        let with_part: BTreeSet<DeviceId> = self
            .components()
            .filter(|c| c.part_id().is_some())
            .filter_map(Component::device_id)
            .collect();
        let mut points_by_function: BTreeMap<FunctionId, Vec<&ConnectionPoint>> = BTreeMap::new();
        for point in self.connection_points() {
            points_by_function
                .entry(point.function_id())
                .or_default()
                .push(point);
        }
        for device in self.devices() {
            let functions = by_device.remove(&device.id()).unwrap_or_default();
            // VER-021: two points of one device with the same designation
            // (two contacts numbered 13-14 on -K1).
            let mut labels: BTreeMap<&str, Vec<ConnectionPointId>> = BTreeMap::new();
            for f in &functions {
                for point in points_by_function.get(&f.id()).into_iter().flatten() {
                    labels
                        .entry(point.designation().as_str())
                        .or_default()
                        .push(point.id());
                }
            }
            for (label, ids) in labels.into_iter().filter(|(_, ids)| ids.len() > 1) {
                let mut targets = vec![CheckTarget::Device(device.id())];
                targets.extend(ids.iter().map(|id| CheckTarget::ConnectionPoint(*id)));
                out.push(CheckMessage::new(
                    "VER-021",
                    format!(
                        "{}: ponto {label} aparece {} vezes; renumere os contatos",
                        device.designation(),
                        ids.len()
                    ),
                    targets,
                ));
            }
            let has_main = functions.iter().any(|f| f.role() == FunctionRole::Main);
            let auxiliaries: Vec<_> = functions
                .iter()
                .filter(|f| f.role() == FunctionRole::Auxiliary)
                // Strip terminals (every point on a terminal) are not
                // auxiliaries of a missing main function.
                .filter(|f| {
                    let points = points_by_function.get(&f.id());
                    !points.is_some_and(|p| {
                        !p.is_empty() && p.iter().all(|pt| pt.terminal_id().is_some())
                    })
                })
                .collect();
            if !has_main && !auxiliaries.is_empty() {
                out.push(CheckMessage::new(
                    "VER-004",
                    format!(
                        "{}: {} função(ões) auxiliar(es) sem função principal",
                        device.designation(),
                        auxiliaries.len()
                    ),
                    vec![CheckTarget::Device(device.id())],
                ));
            }
            let has_part = with_part.contains(&device.id());
            if has_part {
                for f in &functions {
                    if self.function_references(f.id()).is_ok_and(|r| r.is_empty()) {
                        out.push(CheckMessage::new(
                            "VER-006",
                            format!(
                                "{}: função {} da peça não representada",
                                device.designation(),
                                f.kind().label()
                            ),
                            vec![
                                CheckTarget::Device(device.id()),
                                CheckTarget::Function(f.id()),
                            ],
                        ));
                    }
                }
            } else {
                out.push(CheckMessage::new(
                    "VER-018",
                    format!("{}: dispositivo sem peça atribuída", device.designation()),
                    vec![CheckTarget::Device(device.id())],
                ));
            }
        }
    }

    fn check_connections(&self, out: &mut Vec<CheckMessage>) {
        // Symbol ports whose points have no connections.
        // Symbols joined by single-line links are documentation of the
        // single-line diagram; their ports are not wired.
        let single_line: BTreeSet<RepresentationId> = self
            .single_lines()
            .flat_map(|l| [l.source(), l.target()])
            .collect();
        let mut seen = BTreeSet::new();
        let mut bindings: Vec<_> = self.port_bindings.iter().collect();
        bindings.sort_by_key(|(port, _)| (port.representation_id(), *port));
        for (port, point) in bindings {
            if self.marker(port.representation_id()).is_some()
                || single_line.contains(&port.representation_id())
                || !seen.insert(*point)
            {
                continue;
            }
            if self.graph.neighbors(*point).next().is_none() {
                let label = self.connection_point_label(*point);
                out.push(CheckMessage::new(
                    "VER-007",
                    format!("Porta {label} sem conexão"),
                    vec![
                        CheckTarget::Representation(port.representation_id()),
                        CheckTarget::ConnectionPoint(*point),
                    ],
                ));
            }
        }
        let numbered = self.connections.values().any(|c| c.number().is_some());
        let mut potentials: BTreeMap<ConnectionPointId, Vec<String>> = BTreeMap::new();
        for marker in self.markers.values() {
            if matches!(marker.kind(), MarkerKind::Potential { .. })
                && let Some(point) = marker.point()
            {
                potentials
                    .entry(point)
                    .or_default()
                    .push(marker.name().as_str().to_owned());
            }
        }
        for network in self.networks() {
            if network.connections().is_empty() {
                continue;
            }
            let targets: Vec<_> = network
                .connections()
                .iter()
                .map(|c| CheckTarget::Connection(*c))
                .collect();
            if numbered
                && network
                    .connections()
                    .iter()
                    .all(|c| self.connection(*c).is_some_and(|c| c.number().is_none()))
            {
                out.push(CheckMessage::new(
                    "VER-008",
                    format!(
                        "Rede com {} fio(s) sem número (outras já foram numeradas)",
                        network.connections().len()
                    ),
                    targets.clone(),
                ));
            }
            let names: BTreeSet<_> = network
                .points()
                .iter()
                .filter_map(|p| potentials.get(p))
                .flatten()
                .cloned()
                .collect();
            if names.len() > 1 {
                out.push(CheckMessage::new(
                    "VER-009",
                    format!(
                        "Rede com potenciais diferentes: {}",
                        names.into_iter().collect::<Vec<_>>().join(", ")
                    ),
                    targets,
                ));
            }
        }
    }

    fn check_markers(&self, out: &mut Vec<CheckMessage>) {
        for marker in self.markers.values() {
            match marker.kind() {
                MarkerKind::Interruption => {
                    if self
                        .marker_references(marker.id())
                        .is_ok_and(|r| r.is_empty())
                    {
                        out.push(CheckMessage::new(
                            "VER-010",
                            format!("Interrupção {} sem par", marker.name().as_str()),
                            vec![CheckTarget::Representation(marker.id())],
                        ));
                    }
                }
                MarkerKind::Potential { .. } if marker.point().is_none() => {
                    out.push(CheckMessage::new(
                        "VER-011",
                        format!("Potencial {} sem ponto de conexão", marker.name().as_str()),
                        vec![CheckTarget::Representation(marker.id())],
                    ));
                }
                _ => {}
            }
        }
    }

    fn check_physical(&self, out: &mut Vec<CheckMessage>) {
        // VER-013, core count exceeding cable capacity, is enforced by the physical index.
        let lists = self.physical_lists();
        for row in &lists.terminals {
            if row.strip.is_some()
                && row.connections.is_empty()
                && row.bridges.is_empty()
                && row.conductors.is_empty()
            {
                out.push(CheckMessage::new(
                    "VER-014",
                    format!("Borne {} sem destino", row.designation),
                    vec![CheckTarget::Terminal(row.terminal)],
                ));
            }
        }
        // The state is documentary; report it without changing connectivity.
        for terminal in self.terminals() {
            if terminal.disconnect_state() == Some(DisconnectState::Open) {
                out.push(CheckMessage::new(
                    "VER-016",
                    format!(
                        "Borne seccionável {} aberto (documental: a rede continua contínua)",
                        self.terminal_label(terminal.id())
                    ),
                    vec![CheckTarget::Terminal(terminal.id())],
                ));
            }
        }
        for mating in self.connector_matings.values() {
            for pair in mating.pairs() {
                let missing: Vec<_> = [pair.first(), pair.second()]
                    .into_iter()
                    .filter(|t| self.contact_point(*t).is_none())
                    .collect();
                if !missing.is_empty() {
                    out.push(CheckMessage::new(
                        "VER-017",
                        format!(
                            "Contato {} acoplado sem ponto associado: sem continuidade",
                            missing
                                .iter()
                                .map(|t| self.terminal_label(*t))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                        missing.into_iter().map(CheckTarget::Terminal).collect(),
                    ));
                }
            }
        }
    }

    fn check_parts_and_reports(&self, out: &mut Vec<CheckMessage>) {
        let mut groups: BTreeMap<ReportId, PageId> = BTreeMap::new();
        for page in self.pages.values() {
            if let Some(report) = page.report() {
                groups.entry(report.group).or_insert(page.id());
            }
        }
        for (group, page) in groups {
            if self.report_stale(group).unwrap_or(false) {
                out.push(CheckMessage::new(
                    "VER-020",
                    "Relatório desatualizado: use Ferramentas > Relatórios > Atualizar".into(),
                    vec![CheckTarget::Page(page)],
                ));
            }
        }
    }
}

/// Columns used when exporting check messages.
pub const CHECK_EXPORT_HEADERS: [&str; 5] = ["Código", "Gravidade", "Regra", "Mensagem", "Local"];

impl Project {
    /// Exportable CSV/XLSX message rows, including sheet locations.
    pub fn check_export_rows(&self, report: &CheckReport) -> Vec<Vec<String>> {
        report
            .messages
            .iter()
            .map(|m| {
                let place = m
                    .targets
                    .iter()
                    .find_map(|t| self.check_target_location(*t))
                    .map(|(page, rep)| match rep {
                        Some(rep) => self
                            .representation(rep)
                            .map(|r| self.short_reference(page, r.transform().translation()))
                            .unwrap_or_default(),
                        None => format!(
                            "/{}",
                            self.page(page).map(|p| p.number().as_str()).unwrap_or("?")
                        ),
                    })
                    .unwrap_or_else(|| "—".into());
                vec![
                    m.rule.to_owned(),
                    m.severity.label().to_owned(),
                    check_rule(m.rule).map_or("", |r| r.title).to_owned(),
                    m.text.clone(),
                    place,
                ]
            })
            .collect()
    }
}
