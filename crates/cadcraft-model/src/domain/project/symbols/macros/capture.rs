// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Macro capture from selections and pages; start with Project::capture_macro.
use super::*;

impl Project {
    /// Capture a page selection as macro content.
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

    /// Capture an entire page excluding its frame, which belongs to page metadata.
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
        // 1. Include eligible symbol and free-content occurrences.
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
        // 2. Devices include every function and point; functions outside the selection remain
        // unrepresented at the destination.
        for d in &devices {
            functions.extend(self.functions_for_device(*d).map(Function::id));
        }
        let points: BTreeSet<ConnectionPointId> = self
            .connection_points
            .values()
            .filter(|p| functions.contains(&p.function_id()))
            .map(ConnectionPoint::id)
            .collect();
        // 3. Include markers only when their points are selected.
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
        // 4. Include internal wires whose endpoints belong to retained occurrences.
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
        // 5. Capture entity registration data.
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
        // Custom properties cannot refer outside the macro.
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
}
