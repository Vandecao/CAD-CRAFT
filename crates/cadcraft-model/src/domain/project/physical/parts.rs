// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Part assignment and bill of materials; start with the Project part methods.
use super::*;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq)]
pub struct PartAssignmentDelta {
    part: Part,
    introduced_part: bool,
    component: Component,
    functions: Vec<Function>,
    points: Vec<ConnectionPoint>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PartUpdateDelta {
    before: Part,
    after: Part,
    components: Vec<(Component, Component)>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PartRemovalDelta {
    component: Component,
    removed_part: Option<Part>,
}
impl PartAssignmentDelta {
    pub fn component_id(&self) -> ComponentId {
        self.component.id()
    }
    pub fn len(&self) -> usize {
        2 + self.functions.len() + self.points.len()
    }
    pub fn is_empty(&self) -> bool {
        false
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BomRow {
    pub device: DeviceId,
    pub device_designation: String,
    pub part_id: PartId,
    pub manufacturer: String,
    pub code: String,
    pub description: String,
    pub quantity: u32,
}

impl Project {
    pub fn update_parts(&mut self, parts: Vec<Part>) -> Result<Vec<PartUpdateDelta>> {
        let mut candidate = self.clone();
        let mut seen = BTreeSet::new();
        let mut deltas = Vec::new();
        for part in parts {
            if !seen.insert(part.id().clone()) {
                return Err(DomainError::InvalidRelation(
                    "peça duplicada no lote de atualização",
                ));
            }
            part.validate()?;
            if candidate.part(part.id()) != Some(&part) {
                deltas.push(candidate.update_part(part)?);
            }
        }
        *self = candidate;
        Ok(deltas)
    }
    pub fn restore_part_updates(
        &mut self,
        deltas: &[PartUpdateDelta],
        forward: bool,
    ) -> Result<()> {
        let mut candidate = self.clone();
        if forward {
            for delta in deltas {
                candidate.restore_part_update(delta, true)?;
            }
        } else {
            for delta in deltas.iter().rev() {
                candidate.restore_part_update(delta, false)?;
            }
        }
        *self = candidate;
        Ok(())
    }
    pub fn part(&self, id: &PartId) -> Option<&Part> {
        self.parts.get(id)
    }
    pub fn parts(&self) -> impl Iterator<Item = &Part> {
        self.parts.values()
    }

    /// Atualiza a cópia interna, preservando as atribuições. Uma função
    /// representada que desapareceria do modelo impede a troca.
    pub fn update_part(&mut self, after: Part) -> Result<PartUpdateDelta> {
        after.validate()?;
        let before = self
            .part(after.id())
            .cloned()
            .ok_or(DomainError::InvalidRelation("peça ausente no projeto"))?;
        if before == after {
            return Err(DomainError::InvalidRelation("peça já atualizada"));
        }
        for component in self
            .components
            .values()
            .filter(|c| c.part_id() == Some(after.id()))
        {
            let Some(device) = component.device_id() else {
                continue;
            };
            for model in before.functions() {
                let retained = after
                    .functions()
                    .iter()
                    .any(|new| new.kind() == model.kind() && new.points() == model.points());
                if retained {
                    continue;
                }
                let represented = self
                    .functions
                    .values()
                    .filter(|f| f.device_id() == device && f.kind() == model.kind())
                    .any(|f| {
                        let labels: BTreeSet<_> = self
                            .connection_points
                            .values()
                            .filter(|p| p.function_id() == f.id())
                            .map(|p| p.designation().as_str())
                            .collect();
                        model
                            .points()
                            .iter()
                            .all(|label| labels.contains(label.as_str()))
                            && self
                                .representations
                                .values()
                                .any(|r| r.target() == RepresentationTarget::Function(f.id()))
                    });
                if represented {
                    return Err(DomainError::InvalidRelation(
                        "modelo remove função representada",
                    ));
                }
            }
        }
        let components = self
            .components
            .values()
            .filter(|c| c.part_id() == Some(after.id()))
            .map(|component| {
                Ok((
                    component.clone(),
                    component
                        .clone()
                        .with_catalog(after.manufacturer().as_str(), after.code().as_str())?
                        .with_properties(after.properties().clone()),
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let delta = PartUpdateDelta {
            before,
            after,
            components,
        };
        self.restore_part_update(&delta, true)?;
        Ok(delta)
    }
    pub fn restore_part_update(&mut self, delta: &PartUpdateDelta, forward: bool) -> Result<()> {
        let (expected, replacement) = if forward {
            (&delta.before, &delta.after)
        } else {
            (&delta.after, &delta.before)
        };
        if self.part(expected.id()) != Some(expected) {
            return Err(DomainError::InvalidRelation("versão da peça mudou"));
        }
        let mut candidate = self.clone();
        for (before, after) in &delta.components {
            let (expected, replacement) = if forward {
                (before, after)
            } else {
                (after, before)
            };
            if candidate.component(expected.id()) != Some(expected) {
                return Err(DomainError::InvalidRelation(
                    "atribuição alterada desde atualização da peça",
                ));
            }
            candidate
                .components
                .insert(replacement.id(), replacement.clone());
        }
        candidate
            .parts
            .insert(replacement.id().clone(), replacement.clone());
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }
    pub fn remove_part_assignment(
        &mut self,
        component_id: ComponentId,
    ) -> Result<PartRemovalDelta> {
        let component = self
            .component(component_id)
            .cloned()
            .ok_or(DomainError::MissingEntity(EntityRef::Component(
                component_id,
            )))?;
        if component.part_id().is_none() {
            return Err(DomainError::InvalidRelation("componente não atribui peça"));
        }
        let id = component.part_id().expect("checked");
        let removed_part = if self
            .components
            .values()
            .filter(|c| c.part_id() == Some(id))
            .count()
            == 1
        {
            self.part(id).cloned()
        } else {
            None
        };
        let delta = PartRemovalDelta {
            component,
            removed_part,
        };
        self.restore_part_removal(&delta, true)?;
        Ok(delta)
    }
    pub fn restore_part_removal(&mut self, delta: &PartRemovalDelta, forward: bool) -> Result<()> {
        let mut candidate = self.clone();
        if forward {
            if candidate.component(delta.component.id()) != Some(&delta.component) {
                return Err(DomainError::InvalidRelation("atribuição mudou"));
            }
            candidate.components.remove(&delta.component.id());
            if let Some(part) = &delta.removed_part {
                candidate.parts.remove(part.id());
            }
        } else {
            if candidate.component(delta.component.id()).is_some() {
                return Err(DomainError::InvalidRelation("atribuição já existe"));
            }
            if let Some(part) = &delta.removed_part {
                candidate.parts.insert(part.id().clone(), part.clone());
            }
            candidate.insert_component(delta.component.clone())?;
        }
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    /// Associa uma peça, criando somente as funções e pontos ausentes.
    pub fn assign_part(
        &mut self,
        device_id: DeviceId,
        part: Part,
        quantity: u32,
    ) -> Result<PartAssignmentDelta> {
        part.validate()?;
        if quantity == 0 {
            return Err(DomainError::InvalidNumber("quantidade da peça"));
        }
        let device = self
            .device(device_id)
            .ok_or(DomainError::MissingEntity(EntityRef::Device(device_id)))?;
        if self
            .part(part.id())
            .is_some_and(|existing| existing != &part)
        {
            return Err(DomainError::InvalidRelation(
                "versão da peça difere da usada no projeto",
            ));
        }
        let component = Component::new(
            self.id,
            device.designation().clone(),
            part.classification().as_str(),
        )?
        .with_catalog(part.manufacturer().as_str(), part.code().as_str())?
        .with_assignment(Some(device_id), None)
        .with_part(part.id().clone(), quantity)?
        .with_properties(part.properties().clone());
        let mut functions = Vec::new();
        let mut points = Vec::new();
        let mut matched = BTreeSet::new();
        let mut has_main = self.main_function(device_id)?.is_some();
        for model in part.functions() {
            let desired: BTreeSet<&str> = model.points().iter().map(Text::as_str).collect();
            let existing = self
                .functions
                .values()
                .filter(|f| {
                    f.device_id() == device_id
                        && f.kind() == model.kind()
                        && !matched.contains(&f.id())
                })
                .filter_map(|f| {
                    let labels: BTreeSet<_> = self
                        .connection_points
                        .values()
                        .filter(|p| p.function_id() == f.id())
                        .map(|p| p.designation().as_str())
                        .collect();
                    labels.is_subset(&desired).then_some((f, labels.len()))
                })
                .max_by_key(|(_, matched_points)| *matched_points)
                .map(|(function, _)| function);
            let function_id = if let Some(existing) = existing {
                matched.insert(existing.id());
                existing.id()
            } else {
                let role = if model.role() == FunctionRole::Main && !has_main {
                    has_main = true;
                    FunctionRole::Main
                } else {
                    FunctionRole::Auxiliary
                };
                let function = Function::new(
                    self.id,
                    device_id,
                    device.designation().clone(),
                    model.kind().clone(),
                )
                .with_role(role);
                let id = function.id();
                matched.insert(id);
                functions.push(function);
                id
            };
            for label in model.points() {
                if !self.connection_points.values().any(|p| {
                    p.function_id() == function_id && p.designation().as_str() == label.as_str()
                }) {
                    points.push(ConnectionPoint::new(
                        self.id,
                        function_id,
                        DesignationPart::new(label.as_str())?,
                    ));
                }
            }
        }
        let delta = PartAssignmentDelta {
            introduced_part: !self.parts.contains_key(part.id()),
            part,
            component,
            functions,
            points,
        };
        self.restore_part_assignment(&delta, true)?;
        Ok(delta)
    }

    pub fn restore_part_assignment(
        &mut self,
        delta: &PartAssignmentDelta,
        forward: bool,
    ) -> Result<()> {
        let mut candidate = self.clone();
        if forward {
            if candidate
                .parts
                .get(delta.part.id())
                .is_some_and(|p| p != &delta.part)
                || candidate.components.contains_key(&delta.component.id())
            {
                return Err(DomainError::InvalidRelation("atribuição de peça alterada"));
            }
            candidate
                .parts
                .entry(delta.part.id().clone())
                .or_insert(delta.part.clone());
            for function in &delta.functions {
                candidate.insert_function(function.clone())?;
            }
            for point in &delta.points {
                candidate.insert_connection_point(point.clone())?;
            }
            candidate.insert_component(delta.component.clone())?;
        } else {
            if candidate.components.get(&delta.component.id()) != Some(&delta.component)
                || delta
                    .functions
                    .iter()
                    .any(|f| candidate.functions.get(&f.id()) != Some(f))
                || delta
                    .points
                    .iter()
                    .any(|p| candidate.connection_points.get(&p.id()) != Some(p))
            {
                return Err(DomainError::InvalidRelation("atribuição de peça alterada"));
            }
            candidate.components.remove(&delta.component.id());
            for point in &delta.points {
                candidate.connection_points.remove(&point.id());
            }
            for function in &delta.functions {
                candidate.functions.remove(&function.id());
            }
            if delta.introduced_part
                && !candidate
                    .components
                    .values()
                    .any(|c| c.part_id() == Some(delta.part.id()))
            {
                candidate.parts.remove(delta.part.id());
            }
        }
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    pub fn bom_rows(&self) -> Vec<BomRow> {
        let mut rows: BTreeMap<(DeviceId, PartId), BomRow> = BTreeMap::new();
        for component in self.components.values() {
            let (Some(device), Some(id)) = (component.device_id(), component.part_id()) else {
                continue;
            };
            let Some(part) = self.part(id) else { continue };
            let row = rows.entry((device, id.clone())).or_insert_with(|| BomRow {
                device,
                device_designation: self
                    .device(device)
                    .map_or_else(|| device.to_string(), |d| d.designation().to_string()),
                part_id: id.clone(),
                manufacturer: part.manufacturer().as_str().into(),
                code: part.code().as_str().into(),
                description: part.description().as_str().into(),
                quantity: 0,
            });
            row.quantity = row.quantity.saturating_add(component.quantity());
        }
        rows.into_values().collect()
    }
}
