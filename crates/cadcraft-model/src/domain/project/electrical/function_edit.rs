// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Reversible function and connection point edits; start with FunctionPointEdit.
use super::*;

/// Function or connection-point creation, removal or property edit.
#[derive(Clone, Debug)]
pub enum FunctionPointEdit {
    /// Create function.
    CreateFunction {
        /// Associated logical device.
        device: DeviceId,
        /// Documentary designation; does not imply electrical connectivity.
        designation: Designation,
        /// Declared category of this record or operation.
        kind: FunctionKind,
        /// Explicitly declared functional role.
        role: FunctionRole,
    },
    /// Update function.
    UpdateFunction {
        /// Associated electrical function.
        function: FunctionId,
        /// Documentary designation; does not imply electrical connectivity.
        designation: Designation,
        /// Declared category of this record or operation.
        kind: FunctionKind,
        /// Explicitly declared functional role.
        role: FunctionRole,
    },
    /// Remove function.
    RemoveFunction {
        /// Associated electrical function.
        function: FunctionId,
    },
    /// Create point.
    CreatePoint {
        /// Associated electrical function.
        function: FunctionId,
        /// Documentary designation; does not imply electrical connectivity.
        designation: DesignationPart,
    },
    /// Rename point.
    RenamePoint {
        /// Explicit functional connection point.
        point: ConnectionPointId,
        /// Documentary designation; does not imply electrical connectivity.
        designation: DesignationPart,
    },
    /// Remove point.
    RemovePoint {
        /// Explicit functional connection point.
        point: ConnectionPointId,
    },
    /// Set function properties.
    SetFunctionProperties {
        /// Associated electrical function.
        function: FunctionId,
        /// Typed technical properties.
        properties: TechnicalProperties,
    },
    /// Set point properties.
    SetPointProperties {
        /// Explicit functional connection point.
        point: ConnectionPointId,
        /// Typed technical properties.
        properties: TechnicalProperties,
    },
}

#[derive(Clone, Debug, PartialEq)]
struct Change<T> {
    before: Option<T>,
    after: Option<T>,
}
/// Touched functions and points retained for exact replay.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionPointDelta {
    project: ProjectId,
    functions: Vec<Change<Function>>,
    points: Vec<Change<ConnectionPoint>>,
}
impl FunctionPointDelta {
    /// Return the number of contained entries or entity changes recorded by this value.
    pub fn len(&self) -> usize {
        self.functions.len() + self.points.len()
    }
    /// Whether this collection or recorded edit contains no effective entries or changes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Project {
    /// Validate and apply the function point operation, returning the recorded reversible change.
    pub fn edit_function_point(&mut self, edit: FunctionPointEdit) -> Result<FunctionPointDelta> {
        let mut delta = FunctionPointDelta {
            project: self.id,
            functions: Vec::new(),
            points: Vec::new(),
        };
        match edit {
            FunctionPointEdit::CreateFunction {
                device,
                designation,
                kind,
                role,
            } => {
                self.require(EntityRef::Device(device))?;
                let function = Function::new(self.id, device, designation, kind).with_role(role);
                delta.functions.push(Change {
                    before: None,
                    after: Some(function),
                });
            }
            FunctionPointEdit::UpdateFunction {
                function,
                designation,
                kind,
                role,
            } => {
                let before = self
                    .function(function)
                    .cloned()
                    .ok_or(DomainError::MissingEntity(EntityRef::Function(function)))?;
                let mut after = before.clone();
                after.designation = designation;
                after.kind = kind;
                after.role = role;
                if before != after {
                    delta.functions.push(Change {
                        before: Some(before),
                        after: Some(after),
                    });
                }
            }
            FunctionPointEdit::RemoveFunction { function } => {
                let before = self
                    .function(function)
                    .cloned()
                    .ok_or(DomainError::MissingEntity(EntityRef::Function(function)))?;
                if self
                    .representations
                    .values()
                    .any(|r| self.function_for_target(r.target()) == Some(function))
                {
                    return Err(DomainError::EntityInUse(EntityRef::Function(function)));
                }
                for point in self
                    .connection_points
                    .values()
                    .filter(|p| p.function_id() == function)
                {
                    delta.points.push(Change {
                        before: Some(point.clone()),
                        after: None,
                    });
                }
                delta.functions.push(Change {
                    before: Some(before),
                    after: None,
                });
            }
            FunctionPointEdit::CreatePoint {
                function,
                designation,
            } => {
                self.require(EntityRef::Function(function))?;
                if self
                    .connection_points
                    .values()
                    .any(|p| p.function_id() == function && p.designation() == &designation)
                {
                    return Err(DomainError::InvalidRelation(
                        "rótulo de ponto duplicado na função",
                    ));
                }
                delta.points.push(Change {
                    before: None,
                    after: Some(ConnectionPoint::new(self.id, function, designation)),
                });
            }
            FunctionPointEdit::RenamePoint { point, designation } => {
                let before =
                    self.connection_point(point)
                        .cloned()
                        .ok_or(DomainError::MissingEntity(EntityRef::ConnectionPoint(
                            point,
                        )))?;
                if self.connection_points.values().any(|p| {
                    p.id() != point
                        && p.function_id() == before.function_id()
                        && p.designation() == &designation
                }) {
                    return Err(DomainError::InvalidRelation(
                        "rótulo de ponto duplicado na função",
                    ));
                }
                let mut after = before.clone();
                after.designation = designation;
                if before != after {
                    delta.points.push(Change {
                        before: Some(before),
                        after: Some(after),
                    });
                }
            }
            FunctionPointEdit::RemovePoint { point } => {
                let before =
                    self.connection_point(point)
                        .cloned()
                        .ok_or(DomainError::MissingEntity(EntityRef::ConnectionPoint(
                            point,
                        )))?;
                if before.terminal_id().is_some()
                    || self.port_bindings.values().any(|id| *id == point)
                    || self.connections.values().any(|c| {
                        let (a, b) = c.endpoints();
                        a == point || b == point
                    })
                {
                    return Err(DomainError::EntityInUse(EntityRef::ConnectionPoint(point)));
                }
                delta.points.push(Change {
                    before: Some(before),
                    after: None,
                });
            }
            FunctionPointEdit::SetFunctionProperties {
                function,
                properties,
            } => {
                let before = self
                    .function(function)
                    .cloned()
                    .ok_or(DomainError::MissingEntity(EntityRef::Function(function)))?;
                let mut after = before.clone();
                after.properties = properties;
                if before != after {
                    delta.functions.push(Change {
                        before: Some(before),
                        after: Some(after),
                    });
                }
            }
            FunctionPointEdit::SetPointProperties { point, properties } => {
                let before =
                    self.connection_point(point)
                        .cloned()
                        .ok_or(DomainError::MissingEntity(EntityRef::ConnectionPoint(
                            point,
                        )))?;
                let mut after = before.clone();
                after.properties = properties;
                if before != after {
                    delta.points.push(Change {
                        before: Some(before),
                        after: Some(after),
                    });
                }
            }
        }
        self.restore_function_point(&delta, true)?;
        Ok(delta)
    }

    /// Replay the recorded function point change; forward reapplies it and false restores the prior
    /// state. Reject a mismatched expected state.
    pub fn restore_function_point(
        &mut self,
        delta: &FunctionPointDelta,
        forward: bool,
    ) -> Result<()> {
        self.check_project(delta.project)?;
        let mut candidate = self.clone();
        for change in &delta.functions {
            let (expected, replacement) = if forward {
                (&change.before, &change.after)
            } else {
                (&change.after, &change.before)
            };
            let id = expected
                .as_ref()
                .or(replacement.as_ref())
                .expect("change has entity")
                .id();
            if candidate.functions.get(&id) != expected.as_ref() {
                return Err(DomainError::InvalidRelation(
                    "função alterada desde a edição",
                ));
            }
            match replacement {
                Some(value) => {
                    candidate.functions.insert(id, value.clone());
                }
                None => {
                    candidate.functions.remove(&id);
                }
            }
        }
        for change in &delta.points {
            let (expected, replacement) = if forward {
                (&change.before, &change.after)
            } else {
                (&change.after, &change.before)
            };
            let id = expected
                .as_ref()
                .or(replacement.as_ref())
                .expect("change has entity")
                .id();
            if candidate.connection_points.get(&id) != expected.as_ref() {
                return Err(DomainError::InvalidRelation(
                    "ponto alterado desde a edição",
                ));
            }
            match replacement {
                Some(value) => {
                    candidate.connection_points.insert(id, value.clone());
                }
                None => {
                    candidate.connection_points.remove(&id);
                }
            }
        }
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }
}
