// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Numbering previews and reversible application; start with preview_device_numbering.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
/// Proposed designation change for one device, including any conflict.
#[derive(Clone, Debug, PartialEq)]
pub struct DeviceNumberRow {
    /// Associated logical device.
    pub device: DeviceId,
    /// Recorded state before the operation.
    pub before: Designation,
    /// Recorded state after the operation.
    pub after: Designation,
    /// Reason this numbering proposal cannot be applied, when present.
    pub conflict: Option<String>,
}
/// Proposed device designation changes calculated without mutating the project.
#[derive(Clone, Debug, PartialEq)]
pub struct DeviceNumberPreview {
    /// Proposed numbering changes in the configured ordering.
    pub rows: Vec<DeviceNumberRow>,
}
/// Prior and resulting device designations retained for reversible numbering.
#[derive(Clone, Debug, PartialEq)]
pub struct DeviceNumberDelta {
    project: ProjectId,
    devices: Vec<IdentificationChange<Device>>,
    functions: Vec<IdentificationChange<Function>>,
}
impl DeviceNumberDelta {
    /// Return the number of contained entries or entity changes recorded by this value.
    pub fn len(&self) -> usize {
        self.devices.len() + self.functions.len()
    }
    /// Whether this collection or recorded edit contains no effective entries or changes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
/// Prior and resulting number and lock state of one logical connection.
#[derive(Clone, Debug, PartialEq)]
pub struct ConnectionNumberDelta {
    project: ProjectId,
    before: Connection,
    after: Connection,
}
impl ConnectionNumberDelta {
    /// No change when the requested number and lock already match.
    pub fn is_empty(&self) -> bool {
        self.before == self.after
    }
}
/// Proposed wire-number change for one logical connection.
#[derive(Clone, Debug, PartialEq)]
pub struct ConnectionNumberRow {
    /// Explicit logical connection associated with this record.
    pub connection: ConnectionId,
    /// Recorded state before the operation.
    pub before: Option<Text>,
    /// Recorded state after the operation.
    pub after: Option<Text>,
    /// Reason this numbering proposal cannot be applied, when present.
    pub conflict: Option<String>,
}
/// Proposed wire numbers and conflicts calculated without mutation.
#[derive(Clone, Debug, PartialEq)]
pub struct ConnectionNumberPreview {
    /// Proposed numbering changes in the configured ordering.
    pub rows: Vec<ConnectionNumberRow>,
}
/// Wire-number and lock changes retained for exact undo and redo.
#[derive(Clone, Debug, PartialEq)]
pub struct ConnectionNumberingDelta {
    project: ProjectId,
    changes: Vec<IdentificationChange<Connection>>,
}
impl ConnectionNumberingDelta {
    /// Return the number of contained entries or entity changes recorded by this value.
    pub fn len(&self) -> usize {
        self.changes.len()
    }
    /// Whether this collection or recorded edit contains no effective entries or changes.
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}
/// Proposed documentary number change for one page.
#[derive(Clone, Debug, PartialEq)]
pub struct PageNumberRow {
    /// Page identifier used by this operation or filter.
    pub page: PageId,
    /// Recorded state before the operation.
    pub before: Text,
    /// Recorded state after the operation.
    pub after: Text,
}
/// Proposed page numbers calculated without mutation.
#[derive(Clone, Debug, PartialEq)]
pub struct PageNumberPreview {
    /// Proposed numbering changes in the configured ordering.
    pub rows: Vec<PageNumberRow>,
}
/// Prior and resulting page numbers retained for reversible numbering.
#[derive(Clone, Debug, PartialEq)]
pub struct PageNumberDelta {
    project: ProjectId,
    changes: Vec<IdentificationChange<Page>>,
}
impl PageNumberDelta {
    /// Return the number of contained entries or entity changes recorded by this value.
    pub fn len(&self) -> usize {
        self.changes.len()
    }
    /// Whether this collection or recorded edit contains no effective entries or changes.
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}
impl Project {
    /// Position of each page in document and page order.
    fn page_ranks(&self) -> Result<BTreeMap<PageId, usize>> {
        let mut page_rank = BTreeMap::new();
        for document in self.documents() {
            for page in self.ordered_pages(document.id())? {
                let rank = page_rank.len();
                page_rank.insert(page.id(), rank);
            }
        }
        Ok(page_rank)
    }
    /// Find a free device designation using the numbering scheme and placement position.
    pub fn next_device_designation_at(
        &self,
        prefix: &str,
        page: PageId,
        point: Point2D,
    ) -> Result<Designation> {
        if !self.numbering_scheme.use_on_insert {
            return self.next_device_designation(prefix);
        }
        let page = self
            .page(page)
            .ok_or(DomainError::MissingEntity(EntityRef::Page(page)))?;
        let prefix = prefix.trim();
        let base = match self.numbering_scheme.device_format {
            DeviceNumberFormat::Sequential => None,
            DeviceNumberFormat::PageColumn => {
                let column = self
                    .zone_at(page.id(), point)
                    .and_then(|zone| zone.column)
                    .unwrap_or_else(|| "1".into());
                Some(format!("{prefix}{}.{}", page.number().as_str(), column))
            }
        };
        let mut number = self.numbering_scheme.start;
        loop {
            let name = match &base {
                Some(base) if number == self.numbering_scheme.start => base.clone(),
                Some(base) => format!("{base}.{number}"),
                None => format!("{prefix}{number}"),
            };
            let designation = Designation::new(name)?;
            if self.devices().all(|d| d.designation() != &designation) {
                return Ok(designation);
            }
            number = number
                .checked_add(self.numbering_scheme.step)
                .ok_or(DomainError::InvalidRelation("numeração excedeu o limite"))?;
        }
    }
    /// Calculate proposed page numbers without changing the project.
    pub fn preview_page_numbering(
        &self,
        document: DocumentId,
        prefix: &str,
        start: u32,
    ) -> Result<PageNumberPreview> {
        if start == 0 {
            return Err(DomainError::InvalidRelation(
                "início de página deve ser positivo",
            ));
        }
        let mut rows = Vec::new();
        for (offset, page) in self.ordered_pages(document)?.into_iter().enumerate() {
            let number = start
                .checked_add(
                    u32::try_from(offset)
                        .map_err(|_| DomainError::InvalidRelation("muitas páginas"))?,
                )
                .ok_or(DomainError::InvalidRelation(
                    "numeração de página excedeu o limite",
                ))?;
            rows.push(PageNumberRow {
                page: page.id(),
                before: page.number().clone(),
                after: Text::new(format!("{prefix}{number}"))?,
            });
        }
        Ok(PageNumberPreview { rows })
    }
    /// Validate and apply a page-number preview, retaining prior values for replay.
    pub fn apply_page_numbering(&mut self, preview: &PageNumberPreview) -> Result<PageNumberDelta> {
        let mut changes = Vec::new();
        let mut seen = BTreeSet::new();
        for row in &preview.rows {
            if !seen.insert(row.page) {
                return Err(DomainError::InvalidRelation("página duplicada na prévia"));
            }
            let before = self
                .page(row.page)
                .ok_or(DomainError::MissingEntity(EntityRef::Page(row.page)))?
                .clone();
            if before.number() != &row.before {
                return Err(DomainError::InvalidRelation(
                    "prévia de página desatualizada",
                ));
            }
            let mut after = before.clone();
            after.number = row.after.clone();
            if before != after {
                changes.push(IdentificationChange { before, after });
            }
        }
        let delta = PageNumberDelta {
            project: self.id,
            changes,
        };
        self.restore_page_numbering(&delta, true)?;
        Ok(delta)
    }
    /// Replay the recorded page numbering change; forward reapplies it and false restores the prior
    /// state. Reject a mismatched expected state.
    pub fn restore_page_numbering(&mut self, delta: &PageNumberDelta, forward: bool) -> Result<()> {
        self.check_project(delta.project)?;
        let mut candidate = self.clone();
        for change in &delta.changes {
            let (expected, next) = if forward {
                (&change.before, &change.after)
            } else {
                (&change.after, &change.before)
            };
            if candidate.pages.get(&expected.id()) != Some(expected) {
                return Err(DomainError::InvalidRelation(
                    "histórico de página inconsistente",
                ));
            }
            candidate.pages.insert(next.id(), next.clone());
        }
        candidate.document_index = candidate.build_document_index()?;
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }
    /// Return the numbering scheme.
    pub fn numbering_scheme(&self) -> &NumberingScheme {
        &self.numbering_scheme
    }
    /// Validate and replace the project numbering configuration.
    pub fn set_numbering_scheme(&mut self, scheme: NumberingScheme) -> Result<()> {
        if scheme.start == 0 || scheme.step == 0 {
            return Err(DomainError::InvalidRelation(
                "início e passo devem ser positivos",
            ));
        }
        self.numbering_scheme = scheme;
        Ok(())
    }
    /// Replace the wire number and numbering lock through a reversible change.
    pub fn set_connection_number(
        &mut self,
        id: ConnectionId,
        number: Option<Text>,
        locked: bool,
    ) -> Result<ConnectionNumberDelta> {
        let before = self
            .connection(id)
            .ok_or(DomainError::InvalidRelation("conexão ausente"))?
            .clone();
        let mut after = before.clone();
        after.number = number;
        after.number_locked = locked;
        let delta = ConnectionNumberDelta {
            project: self.id,
            before,
            after,
        };
        self.restore_connection_number(&delta, true)?;
        Ok(delta)
    }
    /// Replay the recorded connection number change; forward reapplies it and false restores the prior
    /// state. Reject a mismatched expected state.
    pub fn restore_connection_number(
        &mut self,
        delta: &ConnectionNumberDelta,
        forward: bool,
    ) -> Result<()> {
        self.check_project(delta.project)?;
        let (expected, next) = if forward {
            (&delta.before, &delta.after)
        } else {
            (&delta.after, &delta.before)
        };
        if self.connection(expected.id()) != Some(expected) {
            return Err(DomainError::InvalidRelation(
                "histórico de numeração de fio inconsistente",
            ));
        }
        let mut candidate = self.clone();
        candidate.connections.insert(next.id(), next.clone());
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }
    /// Calculate proposed wire numbers without changing the project.
    pub fn preview_connection_numbering(&self) -> Result<ConnectionNumberPreview> {
        let scheme = &self.numbering_scheme;
        let order = scheme.order;
        let page_rank = self.page_ranks()?;
        // First drawn segment of each connection in rule order.
        let mut first_route: BTreeMap<ConnectionId, (usize, Point2D, PageId)> = BTreeMap::new();
        for route in self.connection_routes.values() {
            let Some(point) = route.points().first().copied() else {
                continue;
            };
            let rank = page_rank
                .get(&route.page_id())
                .copied()
                .unwrap_or(usize::MAX);
            let earlier = first_route.get(&route.connection_id()).is_none_or(|old| {
                position_order(order, (rank, point), (old.0, old.1)) == std::cmp::Ordering::Less
            });
            if earlier {
                first_route.insert(route.connection_id(), (rank, point, route.page_id()));
            }
        }
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
        struct Net {
            connections: Vec<ConnectionId>,
            fixed: Option<String>,
            conflict: Option<String>,
            at: Option<(usize, Point2D, PageId)>,
        }
        let mut nets = Vec::new();
        for network in self.networks() {
            if network.connections().is_empty() {
                continue;
            }
            let named: BTreeSet<String> = network
                .points()
                .iter()
                .filter_map(|point| potentials.get(point))
                .flatten()
                .cloned()
                .collect();
            let locked: BTreeSet<String> = network
                .connections()
                .iter()
                .filter_map(|id| self.connection(*id))
                .filter(|c| c.number_locked())
                .filter_map(|c| c.number().map(|n| n.as_str().to_owned()))
                .collect();
            let conflict = (named.len() > 1
                || locked.len() > 1
                || (!named.is_empty() && !locked.is_empty() && named != locked))
                .then(|| "potenciais ou números travados conflitantes".to_owned());
            // Potential names take precedence, followed by locked numbers.
            let fixed = named
                .iter()
                .next()
                .or_else(|| locked.iter().next())
                .cloned();
            let at = network
                .connections()
                .iter()
                .filter_map(|id| first_route.get(id))
                .copied()
                .min_by(|a, b| position_order(order, (a.0, a.1), (b.0, b.1)));
            nets.push(Net {
                connections: network.connections().to_vec(),
                fixed,
                conflict,
                at,
            });
        }
        nets.sort_by(|a, b| match (a.at, b.at) {
            (Some(x), Some(y)) => position_order(order, (x.0, x.1), (y.0, y.1))
                .then_with(|| a.connections.cmp(&b.connections)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.connections.cmp(&b.connections),
        });
        // Reserve potential names and locked numbers.
        let mut used: BTreeSet<String> = nets.iter().filter_map(|n| n.fixed.clone()).collect();
        let step = |n: u32| {
            n.checked_add(scheme.step)
                .ok_or(DomainError::InvalidRelation("numeração excedeu o limite"))
        };
        let mut next = scheme.start;
        let mut per_base: BTreeMap<String, u32> = BTreeMap::new();
        let mut rows = Vec::new();
        for net in &nets {
            let number = match (&net.fixed, scheme.wire_format, net.at) {
                (Some(fixed), _, _) => fixed.clone(),
                (None, WireNumberFormat::PageColumn, Some((_, point, page))) => {
                    let page_number = self
                        .page(page)
                        .ok_or(DomainError::MissingEntity(EntityRef::Page(page)))?
                        .number()
                        .as_str()
                        .to_owned();
                    let column = self
                        .zone_at(page, point)
                        .and_then(|z| z.column)
                        .unwrap_or_else(|| "1".into());
                    let base = format!("{page_number}.{column}");
                    loop {
                        let n = per_base.entry(base.clone()).or_insert(scheme.start);
                        let candidate = if *n == scheme.start {
                            base.clone()
                        } else {
                            format!("{base}.{n}")
                        };
                        *n = step(*n)?;
                        if !used.contains(&candidate) {
                            break candidate;
                        }
                    }
                }
                (None, _, _) => loop {
                    let candidate = next.to_string();
                    next = step(next)?;
                    if !used.contains(&candidate) {
                        break candidate;
                    }
                },
            };
            used.insert(number.clone());
            let number = Some(Text::new(number)?);
            for id in &net.connections {
                let connection = self.connection(*id).expect("network connection exists");
                rows.push(ConnectionNumberRow {
                    connection: *id,
                    before: connection.number().cloned(),
                    after: number.clone(),
                    conflict: net.conflict.clone(),
                });
            }
        }
        Ok(ConnectionNumberPreview { rows })
    }
    /// Validate and apply a wire-number preview, retaining prior values for replay.
    pub fn apply_connection_numbering(
        &mut self,
        preview: &ConnectionNumberPreview,
    ) -> Result<ConnectionNumberingDelta> {
        if preview.rows.iter().any(|row| row.conflict.is_some()) {
            return Err(DomainError::InvalidRelation(
                "prévia de fios contém conflito",
            ));
        }
        let mut changes = Vec::new();
        let mut seen = BTreeSet::new();
        for row in &preview.rows {
            if !seen.insert(row.connection) {
                return Err(DomainError::InvalidRelation("conexão repetida na prévia"));
            }
            let before = self
                .connection(row.connection)
                .ok_or(DomainError::InvalidRelation("conexão ausente"))?
                .clone();
            if before.number() != row.before.as_ref()
                || (before.number_locked() && before.number() != row.after.as_ref())
            {
                return Err(DomainError::InvalidRelation("prévia de fios desatualizada"));
            }
            let mut after = before.clone();
            after.number = row.after.clone();
            if before != after {
                changes.push(IdentificationChange { before, after });
            }
        }
        let delta = ConnectionNumberingDelta {
            project: self.id,
            changes,
        };
        self.restore_connection_numbering(&delta, true)?;
        Ok(delta)
    }
    /// Replay the recorded connection numbering change; forward reapplies it and false restores the
    /// prior state. Reject a mismatched expected state.
    pub fn restore_connection_numbering(
        &mut self,
        delta: &ConnectionNumberingDelta,
        forward: bool,
    ) -> Result<()> {
        self.check_project(delta.project)?;
        let mut candidate = self.clone();
        for change in &delta.changes {
            let (expected, next) = if forward {
                (&change.before, &change.after)
            } else {
                (&change.after, &change.before)
            };
            if candidate.connections.get(&expected.id()) != Some(expected) {
                return Err(DomainError::InvalidRelation(
                    "histórico de fios inconsistente",
                ));
            }
            candidate.connections.insert(next.id(), next.clone());
        }
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }
    /// Set the device automatic-numbering lock through a reversible change.
    pub fn set_designation_locked(
        &mut self,
        id: DeviceId,
        locked: bool,
    ) -> Result<DeviceNumberDelta> {
        let before = self
            .device(id)
            .ok_or(DomainError::MissingEntity(EntityRef::Device(id)))?
            .clone();
        let mut after = before.clone();
        after.designation_locked = locked;
        let delta = DeviceNumberDelta {
            project: self.id,
            devices: vec![IdentificationChange { before, after }],
            functions: vec![],
        };
        self.restore_device_numbering(&delta, true)?;
        Ok(delta)
    }
    /// Calculate proposed device designations for the requested filter without mutation.
    pub fn preview_device_numbering(&self, filter: NumberingFilter) -> Result<DeviceNumberPreview> {
        let page_rank = self.page_ranks()?;
        let selected: BTreeSet<_> = match filter {
            NumberingFilter::Project => self.devices.keys().copied().collect(),
            NumberingFilter::Document(id) => {
                self.require(EntityRef::Document(id))?;
                self.representations()
                    .filter(|r| {
                        self.page(r.page_id())
                            .is_some_and(|p| p.document_id() == id)
                    })
                    .filter_map(|r| self.device_for_target(r.target()))
                    .collect()
            }
            NumberingFilter::Pages(ids) => {
                for id in &ids {
                    self.require(EntityRef::Page(*id))?;
                }
                let pages: BTreeSet<_> = ids.into_iter().collect();
                self.representations()
                    .filter(|r| pages.contains(&r.page_id()))
                    .filter_map(|r| self.device_for_target(r.target()))
                    .collect()
            }
            NumberingFilter::Selection(ids) => {
                for id in &ids {
                    self.require(EntityRef::Device(*id))?;
                }
                ids.into_iter().collect()
            }
        };
        // First occurrence of each device in rule order, computed once to avoid repeated representation
        // scans during comparison.
        let order = self.numbering_scheme.order;
        let before_in_order = |a, b| position_order(order, a, b);
        let mut location: BTreeMap<DeviceId, (usize, Point2D, PageId)> = BTreeMap::new();
        for r in self.representations() {
            let Some(device) = self.device_for_target(r.target()) else {
                continue;
            };
            let rank = page_rank.get(&r.page_id()).copied().unwrap_or(usize::MAX);
            let point = r.transform().translation();
            let replace = location.get(&device).is_none_or(|old| {
                before_in_order((rank, point), (old.0, old.1)) == std::cmp::Ordering::Less
            });
            if replace {
                location.insert(device, (rank, point, r.page_id()));
            }
        }
        let mut devices: Vec<_> = selected.into_iter().collect();
        devices.sort_by(|a, b| match (location.get(a), location.get(b)) {
            (Some(x), Some(y)) => before_in_order((x.0, x.1), (y.0, y.1)).then_with(|| a.cmp(b)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.cmp(b),
        });
        let scheme = &self.numbering_scheme;
        let step = |n: u32| {
            n.checked_add(scheme.step)
                .ok_or(DomainError::InvalidRelation("numeração excedeu o limite"))
        };
        let mut counters: BTreeMap<String, u32> = BTreeMap::new();
        let mut rows = Vec::new();
        for id in devices {
            let device = self.device(id).expect("selected device exists");
            let before = device.designation().clone();
            let raw = before.device().as_str();
            let prefix: String = raw.chars().take_while(|c| c.is_alphabetic()).collect();
            let fixed = device.designation_locked() && scheme.keep_locked;
            // No alphabetic prefix means no applicable rule; retain the designation.
            let name = if fixed || prefix.is_empty() {
                raw.to_owned()
            } else {
                let page_column = location
                    .get(&id)
                    .filter(|_| scheme.device_format == DeviceNumberFormat::PageColumn)
                    .and_then(|(_, point, page)| {
                        let number = self.page(*page)?.number().as_str().to_owned();
                        let column = self
                            .zone_at(*page, *point)
                            .and_then(|z| z.column)
                            .unwrap_or_else(|| "1".into());
                        Some(format!("{prefix}{number}.{column}"))
                    });
                match page_column {
                    // Use the same collision policy as insertion: -K2.4, then -K2.4.2 and so on.
                    Some(base) => {
                        let next = counters.entry(base.clone()).or_insert(scheme.start);
                        let name = if *next == scheme.start {
                            base
                        } else {
                            format!("{base}.{next}")
                        };
                        *next = step(*next)?;
                        name
                    }
                    None => {
                        let next = counters.entry(prefix.clone()).or_insert(scheme.start);
                        let name = format!("{prefix}{next}");
                        *next = step(*next)?;
                        name
                    }
                }
            };
            let after = Designation::new(name)?
                .with_functional_path(before.functional_path().to_vec())
                .with_location_path(before.location_path().to_vec());
            rows.push(DeviceNumberRow {
                device: id,
                before,
                after,
                conflict: None,
            });
        }
        let renamed: BTreeMap<DeviceId, &Designation> =
            rows.iter().map(|row| (row.device, &row.after)).collect();
        let mut final_names: BTreeMap<String, usize> = BTreeMap::new();
        for device in self.devices() {
            let name = renamed
                .get(&device.id())
                .copied()
                .unwrap_or(device.designation());
            *final_names.entry(name.to_string()).or_default() += 1;
        }
        for row in &mut rows {
            if final_names
                .get(&row.after.to_string())
                .is_some_and(|count| *count > 1)
            {
                row.conflict = Some("designação já utilizada ou travada".into());
            }
        }
        Ok(DeviceNumberPreview { rows })
    }
    /// Validate and apply a device-numbering preview, retaining prior values for replay.
    pub fn apply_device_numbering(
        &mut self,
        preview: &DeviceNumberPreview,
    ) -> Result<DeviceNumberDelta> {
        if preview.rows.iter().any(|row| row.conflict.is_some()) {
            return Err(DomainError::InvalidRelation(
                "prévia de numeração contém conflito",
            ));
        }
        let mut delta = DeviceNumberDelta {
            project: self.id,
            devices: vec![],
            functions: vec![],
        };
        let mut seen = BTreeSet::new();
        for row in &preview.rows {
            if !seen.insert(row.device) {
                return Err(DomainError::InvalidRelation(
                    "dispositivo duplicado na prévia",
                ));
            }
            let before = self
                .device(row.device)
                .ok_or(DomainError::MissingEntity(EntityRef::Device(row.device)))?
                .clone();
            if before.designation != row.before
                || (before.designation_locked
                    && self.numbering_scheme.keep_locked
                    && row.after != row.before)
            {
                return Err(DomainError::InvalidRelation(
                    "prévia de numeração desatualizada",
                ));
            }
            let mut after = before.clone();
            after.designation = row.after.clone();
            if before != after {
                delta.devices.push(IdentificationChange { before, after });
            }
            for function in self.functions_for_device(row.device) {
                let before = function.clone();
                let mut after = before.clone();
                after.designation = row.after.clone();
                if before != after {
                    delta.functions.push(IdentificationChange { before, after });
                }
            }
        }
        let mut names = BTreeSet::new();
        for device in self.devices() {
            let name = delta
                .devices
                .iter()
                .find(|c| c.before.id() == device.id())
                .map_or(device.designation(), |c| c.after.designation());
            if !names.insert(name.to_string()) {
                return Err(DomainError::InvalidRelation(
                    "designação duplicada na numeração",
                ));
            }
        }
        self.restore_device_numbering(&delta, true)?;
        Ok(delta)
    }
    /// Replay the recorded device numbering change; forward reapplies it and false restores the prior
    /// state. Reject a mismatched expected state.
    pub fn restore_device_numbering(
        &mut self,
        delta: &DeviceNumberDelta,
        forward: bool,
    ) -> Result<()> {
        self.check_project(delta.project)?;
        let mut candidate = self.clone();
        for change in &delta.devices {
            let (expected, next) = if forward {
                (&change.before, &change.after)
            } else {
                (&change.after, &change.before)
            };
            if candidate.devices.get(&expected.id()) != Some(expected) {
                return Err(DomainError::InvalidRelation(
                    "histórico de dispositivo inconsistente",
                ));
            }
            candidate.devices.insert(next.id(), next.clone());
        }
        for change in &delta.functions {
            let (expected, next) = if forward {
                (&change.before, &change.after)
            } else {
                (&change.after, &change.before)
            };
            if candidate.functions.get(&expected.id()) != Some(expected) {
                return Err(DomainError::InvalidRelation(
                    "histórico de função inconsistente",
                ));
            }
            candidate.functions.insert(next.id(), next.clone());
        }
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }
}
/// Project numbering order: page, then sheet position.
fn position_order(
    order: NumberingOrder,
    a: (usize, Point2D),
    b: (usize, Point2D),
) -> std::cmp::Ordering {
    a.0.cmp(&b.0).then_with(|| match order {
        // Column by column, increasing X, from top to bottom.
        NumberingOrder::PageColumnRow => {
            a.1.x()
                .total_cmp(&b.1.x())
                .then_with(|| b.1.y().total_cmp(&a.1.y()))
        }
        // Row by row, decreasing Y, from left to right.
        NumberingOrder::PositionXY => {
            b.1.y()
                .total_cmp(&a.1.y())
                .then_with(|| a.1.x().total_cmp(&b.1.x()))
        }
    })
}
