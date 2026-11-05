// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::*;
use std::collections::BTreeSet;

fn invalid() -> DomainError {
    DomainError::InvalidRelation("identificação, associação funcional ou histórico inconsistente")
}
fn changed<T: PartialEq>(before: T, after: T) -> Option<IdentificationChange<T>> {
    (before != after).then_some(IdentificationChange { before, after })
}

impl Project {
    /// Unicidade para novas escritas; snapshots antigos podem conservar conflitos
    /// que continuam visíveis em designation_conflicts, sem fundir identidades.
    pub(super) fn check_new_device_designation(&self, device: &Device) -> Result<()> {
        if self
            .devices()
            .any(|d| d.id() != device.id() && d.designation() == device.designation())
        {
            return Err(DomainError::InvalidRelation(
                "identificação completa de dispositivo já utilizada",
            ));
        }
        Ok(())
    }
    pub fn designation_conflicts(&self) -> Vec<Vec<DeviceId>> {
        let mut names: BTreeMap<String, Vec<DeviceId>> = BTreeMap::new();
        for d in self.devices() {
            names
                .entry(d.designation().to_string())
                .or_default()
                .push(d.id());
        }
        names.into_values().filter(|ids| ids.len() > 1).collect()
    }
    pub(super) fn validate_function_roles(&self) -> Result<()> {
        let mut main = BTreeSet::new();
        for f in self.functions() {
            if f.role() == FunctionRole::Main && !main.insert(f.device_id()) {
                return Err(invalid());
            }
        }
        Ok(())
    }
    pub fn main_function(&self, device: DeviceId) -> Result<Option<&Function>> {
        self.require(EntityRef::Device(device))?;
        Ok(self
            .functions_for_device(device)
            .find(|f| f.role() == FunctionRole::Main))
    }
    pub fn function_for_target(&self, target: RepresentationTarget) -> Option<FunctionId> {
        match target {
            RepresentationTarget::Function(id) => self.function(id).map(Function::id),
            RepresentationTarget::Component(id) => self.component(id)?.function_id(),
            RepresentationTarget::ConnectionPoint(id) => {
                Some(self.connection_point(id)?.function_id())
            }
            RepresentationTarget::Terminal(id) => {
                let ids: BTreeSet<_> = self
                    .connection_points()
                    .filter(|p| p.terminal_id() == Some(id))
                    .map(ConnectionPoint::function_id)
                    .collect();
                if ids.len() == 1 {
                    ids.first().copied()
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    pub fn device_for_target(&self, target: RepresentationTarget) -> Option<DeviceId> {
        match target {
            RepresentationTarget::Device(id) => self.device(id).map(Device::id),
            RepresentationTarget::Component(id) => self.component(id)?.device_id(),
            RepresentationTarget::Terminal(id) => match self.terminal(id)?.owner() {
                TerminalOwner::Device(id) => Some(id),
                TerminalOwner::Component(id) => self.component(id)?.device_id(),
            },
            _ => self
                .function_for_target(target)
                .and_then(|id| self.function(id))
                .map(Function::device_id),
        }
    }
    pub fn device_references(&self, device: DeviceId) -> Result<Vec<DeviceReference>> {
        self.require(EntityRef::Device(device))?;
        let mut result = Vec::new();
        for r in self
            .representations()
            .filter(|r| self.device_for_target(r.target()) == Some(device))
        {
            result.push(self.describe_device_reference(device, r)?);
        }
        result.sort_by_key(|r| (r.page, r.representation));
        Ok(result)
    }
    pub fn function_references(&self, function: FunctionId) -> Result<Vec<DeviceReference>> {
        let f = self
            .function(function)
            .ok_or(DomainError::MissingEntity(EntityRef::Function(function)))?;
        Ok(self
            .device_references(f.device_id())?
            .into_iter()
            .filter(|r| r.function == Some(function))
            .collect())
    }
    pub fn contact_mirror(&self, device: DeviceId) -> Result<Vec<ContactMirrorRow>> {
        self.require(EntityRef::Device(device))?;
        let mut rows = Vec::new();
        for function in self.functions_for_device(device) {
            if !matches!(
                function.kind(),
                FunctionKind::AuxiliaryContact | FunctionKind::MainContact
            ) {
                continue;
            }
            let terminals = self
                .connection_points()
                .filter(|p| p.function_id() == function.id())
                .map(|p| p.designation().as_str().to_owned())
                .collect();
            rows.push(ContactMirrorRow {
                function: function.id(),
                kind: function.kind().clone(),
                terminals,
                references: self.function_references(function.id())?,
            });
        }
        rows.sort_by_key(|r| r.function);
        Ok(rows)
    }
    fn describe_device_reference(
        &self,
        device: DeviceId,
        r: &Representation,
    ) -> Result<DeviceReference> {
        let d = self.device(device).ok_or(invalid())?;
        let function = self
            .function_for_target(r.target())
            .and_then(|id| self.function(id));
        let page = self.page(r.page_id()).ok_or(invalid())?;
        let terminals = function
            .map(|f| {
                self.connection_points()
                    .filter(|cp| cp.function_id() == f.id())
                    .map(|cp| cp.designation().as_str().to_string())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect()
            })
            .unwrap_or_default();
        Ok(DeviceReference {
            device,
            designation: d.designation().clone(),
            function: function.map(Function::id),
            function_designation: function.map(|f| f.designation().clone()),
            function_kind: function.map(|f| f.kind().clone()),
            role: function.map_or(FunctionRole::Unspecified, Function::role),
            representation: r.id(),
            page: page.id(),
            position: r.transform().translation(),
            page_number: page.number().as_str().into(),
            document_structure: page.structure().clone(),
            reference: self.reference_label(page.id(), r.transform().translation()),
            terminals,
        })
    }
    pub fn edit_device(&mut self, edit: DeviceEdit) -> Result<DeviceDelta> {
        let mut delta = DeviceDelta::new(self.id());
        match edit {
            DeviceEdit::SetDesignationLocked { device, locked } => {
                let before = self
                    .device(device)
                    .ok_or(DomainError::MissingEntity(EntityRef::Device(device)))?
                    .clone();
                let mut after = before.clone();
                after.designation_locked = locked;
                delta.devices.extend(changed(before, after));
            }
            DeviceEdit::Update {
                device,
                designation,
                classification,
                properties,
            } => {
                let before = self
                    .device(device)
                    .ok_or(DomainError::MissingEntity(EntityRef::Device(device)))?
                    .clone();
                let mut after = before.clone();
                after.designation = designation;
                after.classification = classification;
                after.properties = properties;
                if before.designation != after.designation {
                    self.check_new_device_designation(&after)?;
                }
                delta.devices.extend(changed(before, after));
            }
            DeviceEdit::Rename {
                device,
                designation,
            } => {
                let before = self
                    .device(device)
                    .ok_or(DomainError::MissingEntity(EntityRef::Device(device)))?
                    .clone();
                let mut after = before.clone();
                after.designation = designation;
                if before != after {
                    self.check_new_device_designation(&after)?;
                }
                delta.legacy_conflicts.extend(
                    self.devices()
                        .filter(|d| d.id() != device && d.designation() == before.designation())
                        .map(|d| (device, d.id())),
                );
                delta.devices.extend(changed(before, after));
            }
            DeviceEdit::SetClassification {
                device,
                classification,
            } => {
                let before = self
                    .device(device)
                    .ok_or(DomainError::MissingEntity(EntityRef::Device(device)))?
                    .clone();
                let mut after = before.clone();
                after.classification = classification;
                delta.devices.extend(changed(before, after));
            }
            DeviceEdit::RenameFunction {
                function,
                designation,
            } => {
                let before = self
                    .function(function)
                    .ok_or(DomainError::MissingEntity(EntityRef::Function(function)))?
                    .clone();
                let mut after = before.clone();
                after.designation = designation;
                delta.functions.extend(changed(before, after));
            }
            DeviceEdit::AssociateFunction {
                function,
                device,
                role,
            } => {
                self.require(EntityRef::Device(device))?;
                let before = self
                    .function(function)
                    .ok_or(DomainError::MissingEntity(EntityRef::Function(function)))?
                    .clone();
                let mut after = before.clone();
                after.device_id = device;
                after.role = role;
                delta.functions.extend(changed(before, after));
            }
            DeviceEdit::SetMainFunction { device, function } => {
                self.require(EntityRef::Device(device))?;
                if let Some(id) = function {
                    let f = self
                        .function(id)
                        .ok_or(DomainError::MissingEntity(EntityRef::Function(id)))?;
                    if f.device_id() != device {
                        return Err(invalid());
                    }
                }
                for before in self.functions_for_device(device) {
                    let mut after = before.clone();
                    if function == Some(before.id()) {
                        after.role = FunctionRole::Main;
                    } else if before.role() == FunctionRole::Main {
                        after.role = FunctionRole::Auxiliary;
                    }
                    delta.functions.extend(changed(before.clone(), after));
                }
            }
            DeviceEdit::MoveRepresentation {
                representation,
                page,
                transform,
            } => {
                let before = self
                    .representation(representation)
                    .ok_or(DomainError::MissingRepresentation(representation))?
                    .clone();
                let mut after = before.clone();
                after.page_id = page;
                after.transform = transform;
                if before != after && self.drawing_references_representation(representation) {
                    return Err(drawing_error());
                }
                self.check_representation(&after)?;
                delta.representations.extend(changed(before, after));
            }
            DeviceEdit::PageName { page, name } => {
                let before = self
                    .page(page)
                    .ok_or(DomainError::MissingEntity(EntityRef::Page(page)))?
                    .clone();
                let mut after = before.clone();
                after.name = name;
                delta.pages.extend(changed(before, after));
            }
            DeviceEdit::PageNumber { page, number } => {
                let before = self
                    .page(page)
                    .ok_or(DomainError::MissingEntity(EntityRef::Page(page)))?
                    .clone();
                let mut after = before.clone();
                after.number = number;
                delta.pages.extend(changed(before, after));
            }
            DeviceEdit::PageStructure { page, structure } => {
                let before = self
                    .page(page)
                    .ok_or(DomainError::MissingEntity(EntityRef::Page(page)))?
                    .clone();
                let mut after = before.clone();
                after.structure = structure;
                delta.pages.extend(changed(before, after));
            }
        }
        self.restore_device_edit(&delta, true)?;
        Ok(delta)
    }
    pub fn restore_device_edit(&mut self, delta: &DeviceDelta, forward: bool) -> Result<()> {
        self.check_project(delta.project)?;
        macro_rules! check {
            ($changes:expr,$map:expr) => {
                for c in $changes {
                    let expected = if forward { &c.before } else { &c.after };
                    if $map.get(&expected.id()) != Some(expected) {
                        return Err(invalid());
                    }
                }
            };
        }
        check!(&delta.devices, self.devices);
        check!(&delta.functions, self.functions);
        check!(&delta.representations, self.representations);
        check!(&delta.pages, self.pages);
        if delta.is_empty() {
            return Ok(());
        }
        // A replay may restore a legacy conflicting name, but cannot create a
        // new collision caused by an unrelated edit since this delta was made.
        for c in &delta.devices {
            let (from, to) = if forward {
                (&c.before, &c.after)
            } else {
                (&c.after, &c.before)
            };
            if from.designation() != to.designation() {
                let unexpected = self
                    .devices()
                    .filter(|d| d.id() != to.id() && d.designation() == to.designation())
                    .any(|d| forward || !delta.legacy_conflicts.contains(&(to.id(), d.id())));
                // Reverse replay may restore only the recorded legacy conflicts,
                // never a new conflicting device introduced since the edit.
                if unexpected {
                    return Err(invalid());
                }
            }
        }
        self.write_device_delta(delta, forward);
        let new_index = match self.build_document_index() {
            Ok(index) => index,
            Err(error) => {
                self.write_device_delta(delta, !forward);
                return Err(error);
            }
        };
        let old_index = std::mem::replace(&mut self.document_index, new_index);
        if let Err(error) = self.validate() {
            self.write_device_delta(delta, !forward);
            self.document_index = old_index;
            return Err(error);
        }
        Ok(())
    }
    fn write_device_delta(&mut self, delta: &DeviceDelta, forward: bool) {
        macro_rules! write {
            ($changes:expr,$map:expr) => {
                for c in $changes {
                    let next = if forward { &c.after } else { &c.before };
                    $map.insert(next.id(), next.clone());
                }
            };
        }
        write!(&delta.devices, self.devices);
        write!(&delta.functions, self.functions);
        write!(&delta.representations, self.representations);
        write!(&delta.pages, self.pages);
    }
}
