// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Terminal symbols placed as strip terminals; start with
//! Project::place_strip_terminal.
//!
//! Inserting a terminal symbol with a strip name (X1) uses or creates that
//! strip and its device -X1, adds the next terminal (X1:1, X1:2…) and binds
//! every port of the symbol to the terminal's single connection point, so
//! wires on both sides belong to the same network. One delta undoes all of
//! it.
use super::*;

/// Inputs for placing a terminal symbol on a strip.
#[derive(Clone, Debug, PartialEq)]
pub struct StripTerminalPlacement {
    /// Page receiving the occurrence.
    pub page: PageId,
    /// Terminal symbol definition.
    pub definition: SymbolDefinition,
    /// Placement transform in page coordinates.
    pub transform: Transform2D,
    /// Strip designation (X1): an existing strip with it, or a new one.
    pub strip: DesignationPart,
}

/// Everything a strip-terminal placement created or changed.
#[derive(Clone, Debug, PartialEq)]
pub struct StripTerminalDelta {
    device: Option<Device>,
    strip_before: Option<TerminalStrip>,
    strip_after: TerminalStrip,
    terminal: Terminal,
    function: Function,
    point: ConnectionPoint,
    representation: Representation,
    definition: SymbolDefinition,
    introduced_definition: bool,
    bindings: Vec<PortRef>,
}
impl StripTerminalDelta {
    /// Return the representation identifier.
    pub fn representation_id(&self) -> RepresentationId {
        self.representation.id()
    }
    /// Return the terminal created.
    pub fn terminal_id(&self) -> TerminalId {
        self.terminal.id()
    }
    /// Return the strip that received the terminal.
    pub fn strip_id(&self) -> TerminalStripId {
        self.strip_after.id()
    }
    /// Entities recorded, for history limits.
    pub fn len(&self) -> usize {
        6 + usize::from(self.device.is_some()) + self.bindings.len()
    }
    /// Always false because the placement records at least one occurrence.
    pub fn is_empty(&self) -> bool {
        false
    }
}

fn changed() -> DomainError {
    DomainError::InvalidRelation("inserção de borne alterada")
}

impl Project {
    /// Next terminal number of the strip named `strip` ("1" for a new strip).
    pub fn next_strip_terminal(&self, strip: &DesignationPart) -> String {
        let used: Vec<u64> = self
            .terminal_strips()
            .filter(|s| s.designation() == strip)
            .flat_map(|s| s.terminals().to_vec())
            .filter_map(|t| self.terminal(t))
            .filter_map(|t| t.designation().as_str().parse().ok())
            .collect();
        (used.iter().max().copied().unwrap_or(0) + 1).to_string()
    }

    /// Place a terminal symbol as the next terminal of a strip, creating the
    /// strip and its device when needed, in one reversible operation.
    pub fn place_strip_terminal(
        &mut self,
        placement: StripTerminalPlacement,
    ) -> Result<StripTerminalDelta> {
        let StripTerminalPlacement {
            page,
            definition,
            transform,
            strip,
        } = placement;
        self.require(EntityRef::Page(page))?;
        definition.validate()?;
        if definition.ports().is_empty() {
            return Err(DomainError::InvalidRelation(
                "símbolo de borne sem portas não gera ponto de conexão",
            ));
        }
        let strip_before = self
            .terminal_strips()
            .find(|s| s.designation() == &strip)
            .cloned();
        // The strip device shares the strip name (-X1).
        let name = Designation::new(strip.as_str())?;
        let existing_device = self
            .devices()
            .find(|d| d.designation() == &name)
            .map(Device::id);
        let device = match existing_device {
            Some(_) => None,
            None => Some(Device::new(self.id, name.clone(), "Régua de bornes")?),
        };
        let device_id = existing_device
            .or(device.as_ref().map(Device::id))
            .expect("device found or created");
        let number = DesignationPart::new(self.next_strip_terminal(&strip))?;
        let terminal = Terminal::new(self.id, TerminalOwner::Device(device_id), number.clone());
        let base = strip_before
            .clone()
            .unwrap_or_else(|| TerminalStrip::new(self.id, strip.clone()));
        let mut order = base.terminals().to_vec();
        order.push(terminal.id());
        let strip_after = base.with_terminals(order);
        let function = Function::new(
            self.id,
            device_id,
            name,
            FunctionKind::Other(Text::new("Borne")?),
        )
        // Terminals are neither main nor auxiliary functions (no VER-004).
        .with_role(FunctionRole::Unspecified);
        let point =
            ConnectionPoint::new(self.id, function.id(), number).with_terminal(terminal.id());
        let representation = Representation::new_symbol(
            self.id,
            page,
            &definition,
            Some(RepresentationTarget::Terminal(terminal.id())),
            transform,
        )?;
        let bindings = definition
            .ports()
            .iter()
            .map(|port| PortRef::new(representation.id(), port.id()))
            .collect::<Result<Vec<_>>>()?;
        let delta = StripTerminalDelta {
            device,
            strip_before,
            strip_after,
            terminal,
            function,
            point,
            introduced_definition: self.symbol_definition(definition.id()).is_none(),
            representation,
            definition,
            bindings,
        };
        self.restore_strip_terminal(&delta, true)?;
        Ok(delta)
    }

    /// Replay the recorded strip-terminal placement; forward reapplies it and
    /// false restores the prior state. Reject a mismatched expected state.
    pub fn restore_strip_terminal(
        &mut self,
        delta: &StripTerminalDelta,
        forward: bool,
    ) -> Result<()> {
        let mut c = self.clone();
        if forward {
            if c.terminal_strip(delta.strip_after.id()) != delta.strip_before.as_ref()
                || c.representation(delta.representation.id()).is_some()
            {
                return Err(changed());
            }
            if let Some(device) = &delta.device {
                c.insert_device(device.clone())?;
            }
            c.edit_physical(PhysicalEdit::Batch(vec![
                PhysicalEdit::UpsertTerminal(delta.terminal.clone()),
                PhysicalEdit::UpsertStrip(delta.strip_after.clone()),
            ]))?;
            c.insert_function(delta.function.clone())?;
            c.insert_connection_point(delta.point.clone())?;
            c.insert_symbol_representation(delta.representation.clone(), delta.definition.clone())?;
            for port in &delta.bindings {
                c.check_binding(port, delta.point.id())?;
                c.port_bindings.insert(port.clone(), delta.point.id());
            }
        } else {
            if c.representation(delta.representation.id()) != Some(&delta.representation)
                || c.terminal_strip(delta.strip_after.id()) != Some(&delta.strip_after)
                || c.connection_point(delta.point.id()) != Some(&delta.point)
                || delta
                    .bindings
                    .iter()
                    .any(|port| c.port_bindings.get(port) != Some(&delta.point.id()))
            {
                return Err(changed());
            }
            for port in &delta.bindings {
                c.port_bindings.remove(port);
            }
            c.remove_representation(delta.representation.id())?;
            if delta.introduced_definition {
                c.remove_unused_symbol_definition(delta.definition.id())?;
            }
            c.connection_points.remove(&delta.point.id());
            c.functions.remove(&delta.function.id());
            let strip = match &delta.strip_before {
                Some(before) => PhysicalEdit::UpsertStrip(before.clone()),
                None => PhysicalEdit::DeleteStrip(delta.strip_after.id()),
            };
            c.edit_physical(PhysicalEdit::Batch(vec![
                strip,
                PhysicalEdit::DeleteTerminal(delta.terminal.id()),
            ]))?;
            if let Some(device) = &delta.device {
                c.devices.remove(&device.id());
            }
        }
        c.validate()?;
        *self = c;
        Ok(())
    }
}
