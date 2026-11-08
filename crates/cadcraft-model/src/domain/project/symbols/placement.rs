// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Symbol insertion creating required electrical entities; start with SymbolPlacement. One
//! transaction registers an optional new device, a function, a point per port, the occurrence and
//! port bindings. The delta stores full entities for redo with identical IDs; undo rejects changed
//! entities or new external references.
use super::*;
use std::collections::BTreeSet;

/// Device for the new function: created transactionally or already registered.
#[derive(Clone, Debug, PartialEq)]
pub enum PlacementDevice {
    /// Create a new logical device.
    New {
        /// Documentary designation; does not imply electrical connectivity.
        designation: Designation,
        /// Documentary classification text.
        classification: String,
    },
    /// Use an already registered logical device.
    Existing(DeviceId),
}

/// Inputs for inserting a symbol and its required electrical entities.
#[derive(Clone, Debug, PartialEq)]
pub struct SymbolPlacement {
    /// Page identifier used by this operation or filter.
    pub page: PageId,
    /// Symbol definition registered for this occurrence.
    pub definition: SymbolDefinition,
    /// Validated placement transform in page coordinates.
    pub transform: Transform2D,
    /// Associated logical device.
    pub device: PlacementDevice,
    /// Kind of the associated electrical function.
    pub function_kind: FunctionKind,
}

/// Inputs for placing an occurrence of an existing electrical function.
#[derive(Clone, Debug, PartialEq)]
pub struct ExistingFunctionPlacement {
    /// Page identifier used by this operation or filter.
    pub page: PageId,
    /// Symbol definition registered for this occurrence.
    pub definition: SymbolDefinition,
    /// Validated placement transform in page coordinates.
    pub transform: Transform2D,
    /// Associated electrical function.
    pub function: FunctionId,
    /// Use only after the interface warns that labels do not match.
    pub allow_order_fallback: bool,
}
/// Occurrence, bindings and points changed by an existing-function placement.
#[derive(Clone, Debug, PartialEq)]
pub struct ExistingFunctionDelta {
    representation: Representation,
    definition: SymbolDefinition,
    introduced_definition: bool,
    bindings: Vec<(PortRef, ConnectionPointId)>,
    used_order_fallback: bool,
}
impl ExistingFunctionDelta {
    /// Return the representation identifier.
    pub fn representation_id(&self) -> RepresentationId {
        self.representation.id()
    }
    /// Return IDs of the connection points recorded by this placement.
    pub fn point_ids(&self) -> Vec<ConnectionPointId> {
        self.bindings.iter().map(|(_, id)| *id).collect()
    }
    /// Whether port mapping used order after labels failed to match.
    pub fn used_order_fallback(&self) -> bool {
        self.used_order_fallback
    }
    /// Return the number of contained entries or entity changes recorded by this value.
    pub fn len(&self) -> usize {
        1 + self.bindings.len()
    }
    /// Always false because a placement records at least one occurrence.
    pub fn is_empty(&self) -> bool {
        false
    }
}

/// Electrical entities and occurrence created by symbol placement.
#[derive(Clone, Debug, PartialEq)]
pub struct PlacementDelta {
    device: Option<Device>,
    function: Function,
    points: Vec<ConnectionPoint>,
    representation: Representation,
    definition: SymbolDefinition,
    introduced_definition: bool,
    bindings: Vec<(PortRef, ConnectionPointId)>,
}
impl PlacementDelta {
    /// Return the representation identifier.
    pub fn representation_id(&self) -> RepresentationId {
        self.representation.id()
    }
    /// Return the device identifier.
    pub fn device_id(&self) -> DeviceId {
        self.function.device_id()
    }
    /// Return the function identifier.
    pub fn function_id(&self) -> FunctionId {
        self.function.id()
    }
    /// Whether the transaction registered a new device.
    pub fn created_device(&self) -> bool {
        self.device.is_some()
    }
    /// Return IDs of the connection points recorded by this placement.
    pub fn point_ids(&self) -> Vec<ConnectionPointId> {
        self.points.iter().map(ConnectionPoint::id).collect()
    }
    /// Entities registered by the transaction, for history resource limits.
    pub fn len(&self) -> usize {
        usize::from(self.device.is_some()) + 2 + self.points.len() + self.bindings.len()
    }
    /// Always false because a placement records at least one occurrence.
    pub fn is_empty(&self) -> bool {
        false
    }
}

fn placement_error() -> DomainError {
    DomainError::InvalidRelation("inserção com cadastro elétrico não corresponde ao documento")
}

impl Project {
    /// Place an existing function symbol and its port bindings in one reversible operation.
    pub fn place_existing_function(
        &mut self,
        placement: ExistingFunctionPlacement,
    ) -> Result<ExistingFunctionDelta> {
        self.require(EntityRef::Page(placement.page))?;
        let function = self
            .function(placement.function)
            .ok_or(DomainError::MissingEntity(EntityRef::Function(
                placement.function,
            )))?;
        if !self.function_references(function.id())?.is_empty() {
            return Err(DomainError::EntityInUse(EntityRef::Function(function.id())));
        }
        placement.definition.validate()?;
        let mut points: Vec<&ConnectionPoint> = self
            .connection_points
            .values()
            .filter(|p| p.function_id() == function.id())
            .collect();
        points.sort_by_key(|p| (p.designation().as_str().to_owned(), p.id()));
        if points.len() != placement.definition.ports().len() || points.is_empty() {
            return Err(DomainError::InvalidRelation(
                "quantidade de portas e pontos incompatível",
            ));
        }
        let by_label: Option<Vec<ConnectionPointId>> = placement
            .definition
            .ports()
            .iter()
            .map(|port| {
                let mut matching = points
                    .iter()
                    .filter(|p| p.designation().as_str() == port.identification());
                let first = matching.next()?;
                matching.next().is_none().then_some(first.id())
            })
            .collect();
        let (ids, fallback) = if let Some(ids) = by_label {
            if ids.iter().collect::<BTreeSet<_>>().len() == ids.len() {
                (ids, false)
            } else if placement.allow_order_fallback {
                (points.iter().map(|p| p.id()).collect(), true)
            } else {
                return Err(DomainError::InvalidRelation(
                    "rótulos de portas não correspondem aos pontos",
                ));
            }
        } else if placement.allow_order_fallback {
            (points.iter().map(|p| p.id()).collect(), true)
        } else {
            return Err(DomainError::InvalidRelation(
                "rótulos de portas não correspondem aos pontos",
            ));
        };
        let representation = Representation::new_symbol(
            self.id,
            placement.page,
            &placement.definition,
            Some(RepresentationTarget::Function(function.id())),
            placement.transform,
        )?;
        let bindings = placement
            .definition
            .ports()
            .iter()
            .zip(ids)
            .map(|(port, id)| Ok((PortRef::new(representation.id(), port.id())?, id)))
            .collect::<Result<Vec<_>>>()?;
        let delta = ExistingFunctionDelta {
            introduced_definition: self.symbol_definition(placement.definition.id()).is_none(),
            representation,
            definition: placement.definition,
            bindings,
            used_order_fallback: fallback,
        };
        self.restore_existing_function_placement(&delta, true)?;
        Ok(delta)
    }

    /// Replay the recorded existing function placement change; forward reapplies it and false restores
    /// the prior state. Reject a mismatched expected state.
    pub fn restore_existing_function_placement(
        &mut self,
        delta: &ExistingFunctionDelta,
        forward: bool,
    ) -> Result<()> {
        let mut candidate = self.clone();
        if forward {
            candidate.insert_symbol_representation(
                delta.representation.clone(),
                delta.definition.clone(),
            )?;
            for (port, point) in &delta.bindings {
                if candidate
                    .port_bindings
                    .insert(port.clone(), *point)
                    .is_some()
                {
                    return Err(DomainError::InvalidRelation("porta já vinculada"));
                }
            }
        } else {
            if candidate.representation(delta.representation.id()) != Some(&delta.representation)
                || delta
                    .bindings
                    .iter()
                    .any(|(port, point)| candidate.port_bindings.get(port) != Some(point))
            {
                return Err(DomainError::InvalidRelation("inserção de função alterada"));
            }
            for (port, _) in &delta.bindings {
                candidate.port_bindings.remove(port);
            }
            candidate.remove_representation(delta.representation.id())?;
            if delta.introduced_definition {
                candidate.remove_unused_symbol_definition(delta.definition.id())?;
            }
        }
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }
    /// First free device designation `{prefix}{n}` with n at least one.
    pub fn next_device_designation(&self, prefix: &str) -> Result<Designation> {
        let prefix = prefix.trim();
        DesignationPart::new(format!("{prefix}1"))?;
        let used: BTreeSet<String> = self
            .devices()
            .filter(|d| d.designation().functional_path().is_empty())
            .filter(|d| d.designation().location_path().is_empty())
            .map(|d| d.designation().device().as_str().to_owned())
            .collect();
        let n = (1..)
            .find(|n| !used.contains(&format!("{prefix}{n}")))
            .expect("unbounded search");
        Designation::new(format!("{prefix}{n}"))
    }

    /// Validate and register everything atomically; generate new IDs here.
    pub fn place_symbol(&mut self, placement: SymbolPlacement) -> Result<PlacementDelta> {
        self.place_symbol_with_point_labels(placement, &[])
    }

    /// Like [`Project::place_symbol`], naming the new points with `labels`
    /// (one per port, in port order) instead of the port identifications;
    /// an empty slice keeps the identifications.
    pub fn place_symbol_with_point_labels(
        &mut self,
        placement: SymbolPlacement,
        labels: &[String],
    ) -> Result<PlacementDelta> {
        let SymbolPlacement {
            page,
            definition,
            transform,
            device,
            function_kind,
        } = placement;
        self.require(EntityRef::Page(page))?;
        definition.validate()?;
        if definition.ports().is_empty() {
            return Err(DomainError::InvalidRelation(
                "símbolo sem portas não gera pontos de conexão",
            ));
        }
        let (device, device_id, designation, role) = match device {
            PlacementDevice::New {
                designation,
                classification,
            } => {
                let device = Device::new(self.id, designation.clone(), classification)?;
                let id = device.id();
                (Some(device), id, designation, FunctionRole::Main)
            }
            PlacementDevice::Existing(id) => {
                let existing = self
                    .device(id)
                    .ok_or(DomainError::MissingEntity(EntityRef::Device(id)))?;
                let role = if self.main_function(id)?.is_some() {
                    FunctionRole::Auxiliary
                } else {
                    FunctionRole::Main
                };
                (None, id, existing.designation().clone(), role)
            }
        };
        let function =
            Function::new(self.id, device_id, designation, function_kind).with_role(role);
        let mut used = BTreeSet::new();
        let mut points = Vec::with_capacity(definition.ports().len());
        let mut bindings = Vec::with_capacity(definition.ports().len());
        let representation = Representation::new_symbol(
            self.id,
            page,
            &definition,
            Some(RepresentationTarget::Function(function.id())),
            transform,
        )?;
        if !labels.is_empty() && labels.len() != definition.ports().len() {
            return Err(DomainError::InvalidRelation(
                "informe um ponto para cada porta do símbolo",
            ));
        }
        for (index, port) in definition.ports().iter().enumerate() {
            let wanted = labels
                .get(index)
                .map_or(port.identification(), |l| l.trim());
            let label = if used.insert(wanted.to_owned()) {
                wanted
            } else {
                port.id()
            };
            let point = ConnectionPoint::new(self.id, function.id(), DesignationPart::new(label)?);
            bindings.push((PortRef::new(representation.id(), port.id())?, point.id()));
            points.push(point);
        }
        let delta = PlacementDelta {
            device,
            function,
            points,
            introduced_definition: self.symbol_definition(definition.id()).is_none(),
            representation,
            definition,
            bindings,
        };
        self.apply_placement(&delta)?;
        Ok(delta)
    }

    /// Replay the recorded placement change; forward reapplies it and false restores the prior state.
    /// Reject a mismatched expected state.
    pub fn restore_placement(&mut self, delta: &PlacementDelta, forward: bool) -> Result<()> {
        if forward {
            if self.representation(delta.representation_id()).is_some()
                || self.function(delta.function_id()).is_some()
            {
                return Err(placement_error());
            }
            return self.apply_placement(delta);
        }
        self.check_placement_state(delta)?;
        let snapshot = self.take_placement(delta)?;
        // Any external reference to removed entities invalidates the aggregate.
        if let Err(error) = self.validate() {
            self.put_back(snapshot);
            return Err(error);
        }
        Ok(())
    }

    fn apply_placement(&mut self, delta: &PlacementDelta) -> Result<()> {
        if let Some(device) = &delta.device {
            self.insert_device(device.clone())?;
        }
        let result = (|| {
            self.insert_function(delta.function.clone())?;
            for point in &delta.points {
                self.insert_connection_point(point.clone())?;
            }
            self.insert_symbol_representation(
                delta.representation.clone(),
                delta.definition.clone(),
            )?;
            for (port, point) in &delta.bindings {
                if self.port_bindings.contains_key(port) {
                    return Err(placement_error());
                }
                self.check_binding(port, *point)?;
                self.port_bindings.insert(port.clone(), *point);
            }
            Ok(())
        })();
        if let Err(error) = result {
            self.discard_placement(delta);
            return Err(error);
        }
        Ok(())
    }

    /// Tolerant removal used only to roll back partial application.
    fn discard_placement(&mut self, delta: &PlacementDelta) {
        for (port, point) in &delta.bindings {
            if self.port_bindings.get(port) == Some(point) {
                self.port_bindings.remove(port);
            }
        }
        let id = delta.representation_id();
        if self.representations.get(&id) == Some(&delta.representation) {
            let _ = self.remove_representation(id);
            if delta.introduced_definition {
                let _ = self.remove_unused_symbol_definition(delta.definition.id());
            }
        }
        for point in &delta.points {
            if self.connection_points.get(&point.id()) == Some(point) {
                self.connection_points.remove(&point.id());
            }
        }
        if self.functions.get(&delta.function_id()) == Some(&delta.function) {
            self.functions.remove(&delta.function_id());
        }
        if let Some(device) = &delta.device
            && self.devices.get(&device.id()) == Some(device)
        {
            self.devices.remove(&device.id());
        }
    }

    fn check_placement_state(&self, delta: &PlacementDelta) -> Result<()> {
        let same = delta
            .device
            .as_ref()
            .is_none_or(|d| self.device(d.id()) == Some(d))
            && self.function(delta.function_id()) == Some(&delta.function)
            && delta
                .points
                .iter()
                .all(|p| self.connection_point(p.id()) == Some(p))
            && self.representation(delta.representation_id()) == Some(&delta.representation)
            && self.symbol_definition(delta.definition.id()) == Some(&delta.definition)
            && delta
                .bindings
                .iter()
                .all(|(port, point)| self.port_bindings.get(port) == Some(point));
        let extra_bindings = self.port_bindings.keys().any(|port| {
            port.representation_id() == delta.representation_id()
                && !delta.bindings.iter().any(|(p, _)| p == port)
        });
        if !same || extra_bindings {
            return Err(placement_error());
        }
        Ok(())
    }

    fn take_placement(&mut self, delta: &PlacementDelta) -> Result<PlacementSnapshot> {
        for (port, _) in &delta.bindings {
            self.port_bindings.remove(port);
        }
        if let Err(error) = self.remove_representation(delta.representation_id()) {
            for (port, point) in &delta.bindings {
                self.port_bindings.insert(port.clone(), *point);
            }
            return Err(error);
        }
        let shared = self
            .representations()
            .any(|r| r.symbol() == Some(delta.definition.id()));
        let definition = if delta.introduced_definition && !shared {
            self.symbol_definitions.remove(delta.definition.id())
        } else {
            None
        };
        for point in &delta.points {
            self.connection_points.remove(&point.id());
        }
        self.functions.remove(&delta.function_id());
        if let Some(device) = &delta.device {
            self.devices.remove(&device.id());
        }
        Ok(PlacementSnapshot {
            delta: delta.clone(),
            definition,
        })
    }

    fn put_back(&mut self, snapshot: PlacementSnapshot) {
        let PlacementSnapshot { delta, definition } = snapshot;
        if let Some(device) = delta.device {
            self.devices.insert(device.id(), device);
        }
        self.functions.insert(delta.function.id(), delta.function);
        for point in delta.points {
            self.connection_points.insert(point.id(), point);
        }
        if let Some(definition) = definition {
            self.symbol_definitions
                .insert(definition.id().clone(), definition);
        }
        self.document_index.insert(&delta.representation);
        self.representations
            .insert(delta.representation.id(), delta.representation);
        for (port, point) in delta.bindings {
            self.port_bindings.insert(port, point);
        }
    }
}

struct PlacementSnapshot {
    delta: PlacementDelta,
    definition: Option<SymbolDefinition>,
}
