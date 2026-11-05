// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::*;

#[derive(Clone, Debug)]
pub enum FunctionPointEdit {
    CreateFunction {
        device: DeviceId,
        designation: Designation,
        kind: FunctionKind,
        role: FunctionRole,
    },
    UpdateFunction {
        function: FunctionId,
        designation: Designation,
        kind: FunctionKind,
        role: FunctionRole,
    },
    RemoveFunction {
        function: FunctionId,
    },
    CreatePoint {
        function: FunctionId,
        designation: DesignationPart,
    },
    RenamePoint {
        point: ConnectionPointId,
        designation: DesignationPart,
    },
    RemovePoint {
        point: ConnectionPointId,
    },
    SetFunctionProperties {
        function: FunctionId,
        properties: TechnicalProperties,
    },
    SetPointProperties {
        point: ConnectionPointId,
        properties: TechnicalProperties,
    },
}

#[derive(Clone, Debug, PartialEq)]
struct Change<T> {
    before: Option<T>,
    after: Option<T>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionPointDelta {
    project: ProjectId,
    functions: Vec<Change<Function>>,
    points: Vec<Change<ConnectionPoint>>,
}
impl FunctionPointDelta {
    pub fn len(&self) -> usize {
        self.functions.len() + self.points.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Project {
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
