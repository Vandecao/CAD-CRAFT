// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Macro insertion, validation and replay; start with Project::insert_macro_content.
use super::*;

impl Project {
    /// Insert content into the specified page in one transaction.
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

    /// Insert a window macro.
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

    /// Page macro: append a page to the document and insert content at its original coordinates.
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

    /// Validate full content by insertion into a scratch project.
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

    /// Generate IDs, designations and geometry, write everything, then validate the project.
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
        // 1. Assign a fresh UUID v7 to each source entity.
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
        // 2. Remap JSON IDs and assign free designations.
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
        // 3. Transform geometry.
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
        // 4. Add missing symbols and parts, retaining existing project versions.
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
        // 5. Write registration data; wires require already positioned ports.
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

    /// Free `{prefix}{n}` designation per device within its existing `=`/`+` path; functions and
    /// installed parts follow the device.
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
        // Stable order by source designation.
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

    /// Unchecked write; `forward = false` removes the entities.
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

    /// Rebuild derived indexes and validate the complete project.
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

    /// Undo when `forward = false`, otherwise redo the insertion.
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
