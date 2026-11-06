// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Session-only bounded history of graphical deltas. No Project snapshots.
use cad_craft::{
    domain::*,
    error::{DomainError, Result},
};
use std::collections::VecDeque;

#[derive(Debug)]
enum Transaction {
    Physical(Box<PhysicalDelta>),
    Device(Box<DeviceDelta>),
    NumberDevices(Box<DeviceNumberDelta>),
    NumberConnections(Box<ConnectionNumberingDelta>),
    NumberPages(Box<PageNumberDelta>),
    Report(Box<ReportDelta>),
    CheckScheme(Box<CheckSchemeDelta>),
    ConnectionNumber(Box<ConnectionNumberDelta>),
    Marker(Box<MarkerDelta>),
    Drawing(ConnectionDrawingDelta),
    Representations(RepresentationDelta),
    Sheet(SheetEditDelta),
    /// Macro insertion or paste with wires.
    Macro(Box<MacroInsertDelta>),
    Symbols(Vec<SymbolUpdateDelta>),
    Visual {
        id: RepresentationId,
        before: VisualProperties,
        after: VisualProperties,
    },
    Insertion {
        representation: Box<Representation>,
        definition: Box<SymbolDefinition>,
        introduced_definition: bool,
    },
    Placement(Box<PlacementDelta>),
    ExistingFunction(Box<ExistingFunctionDelta>),
    PartAssignment(Box<PartAssignmentDelta>),
    PartRemoval(Box<PartRemovalDelta>),
    PartUpdate(Box<PartUpdateDelta>),
    PartUpdates(Vec<PartUpdateDelta>),
    FunctionPoint(Box<FunctionPointDelta>),
    Document(Box<DocumentDelta>),
    Frame(Box<FrameDelta>),
}
impl Transaction {
    fn len(&self) -> usize {
        match self {
            Self::Physical(d) => d.len(),
            Self::Device(d) => d.len(),
            Self::NumberDevices(d) => d.len(),
            Self::NumberConnections(d) => d.len(),
            Self::NumberPages(d) => d.len(),
            Self::Report(d) => d.len(),
            Self::CheckScheme(_) => 1,
            Self::ConnectionNumber(_) => 1,
            Self::Marker(d) => d.len(),
            Self::Drawing(d) => d.len(),
            Self::Representations(d) => d.len(),
            Self::Sheet(d) => d.len(),
            Self::Macro(d) => d.len(),
            Self::Symbols(d) => d.len(),
            Self::Placement(d) => d.len(),
            Self::ExistingFunction(d) => d.len(),
            Self::PartAssignment(d) => d.len(),
            Self::PartRemoval(_) | Self::PartUpdate(_) => 1,
            Self::PartUpdates(deltas) => deltas.len(),
            Self::FunctionPoint(d) => d.len(),
            Self::Document(d) => d.len(),
            Self::Frame(_) => 1,
            Self::Visual { .. } | Self::Insertion { .. } => 1,
        }
    }
}
#[derive(Default, Debug)]
pub struct History {
    undo: VecDeque<Transaction>,
    redo: Vec<Transaction>,
}
const MAX_TRANSACTIONS: usize = 100;
const MAX_DELTAS: usize = 50_000;
impl History {
    pub(crate) fn apply_report_delta(
        &mut self,
        p: &mut Project,
        delta: ReportDelta,
    ) -> Result<bool> {
        if delta.is_empty() {
            return Ok(false);
        }
        p.restore_report(&delta, true)?;
        self.record(Transaction::Report(Box::new(delta)));
        Ok(true)
    }
    pub(crate) fn generate_report(
        &mut self,
        p: &mut Project,
        request: &ReportRequest,
    ) -> Result<bool> {
        let delta = p.generate_report(request)?;
        if delta.is_empty() {
            return Ok(false);
        }
        self.record(Transaction::Report(Box::new(delta)));
        Ok(true)
    }
    pub(crate) fn update_reports(&mut self, p: &mut Project, groups: &[ReportId]) -> Result<bool> {
        let delta = p.update_reports(groups)?;
        if delta.is_empty() {
            return Ok(false);
        }
        self.record(Transaction::Report(Box::new(delta)));
        Ok(true)
    }
    pub(crate) fn number_devices(
        &mut self,
        p: &mut Project,
        preview: &DeviceNumberPreview,
    ) -> Result<bool> {
        if preview.rows.len() > MAX_DELTAS {
            return Err(DomainError::InvalidRelation(
                crate::strings::HISTORY_CAPACITY_EXCEEDED,
            ));
        }
        let delta = p.apply_device_numbering(preview)?;
        if delta.is_empty() {
            return Ok(false);
        }
        self.record(Transaction::NumberDevices(Box::new(delta)));
        Ok(true)
    }
    pub(crate) fn number_connections(
        &mut self,
        p: &mut Project,
        preview: &ConnectionNumberPreview,
    ) -> Result<bool> {
        if preview.rows.len() > MAX_DELTAS {
            return Err(DomainError::InvalidRelation(
                crate::strings::HISTORY_CAPACITY_EXCEEDED,
            ));
        }
        let delta = p.apply_connection_numbering(preview)?;
        if delta.is_empty() {
            return Ok(false);
        }
        self.record(Transaction::NumberConnections(Box::new(delta)));
        Ok(true)
    }
    /// Edit a wire number and lock manually in one transaction.
    pub(crate) fn connection_number(
        &mut self,
        p: &mut Project,
        id: ConnectionId,
        number: Option<Text>,
        locked: bool,
    ) -> Result<bool> {
        let delta = p.set_connection_number(id, number, locked)?;
        if delta.is_empty() {
            return Ok(false);
        }
        self.record(Transaction::ConnectionNumber(Box::new(delta)));
        Ok(true)
    }
    pub(crate) fn number_pages(
        &mut self,
        p: &mut Project,
        preview: &PageNumberPreview,
    ) -> Result<bool> {
        if preview.rows.len() > MAX_DELTAS {
            return Err(DomainError::InvalidRelation(
                crate::strings::HISTORY_CAPACITY_EXCEEDED,
            ));
        }
        let delta = p.apply_page_numbering(preview)?;
        if delta.is_empty() {
            return Ok(false);
        }
        self.record(Transaction::NumberPages(Box::new(delta)));
        Ok(true)
    }
    /// Validation rules and exclusions in one transaction.
    pub(crate) fn check_scheme(&mut self, p: &mut Project, scheme: CheckScheme) -> Result<bool> {
        let delta = p.set_check_scheme(scheme)?;
        if delta.is_empty() {
            return Ok(false);
        }
        self.record(Transaction::CheckScheme(Box::new(delta)));
        Ok(true)
    }
    pub(crate) fn physical(&mut self, p: &mut Project, edit: PhysicalEdit) -> Result<bool> {
        let delta = p.edit_physical(edit)?;
        if delta.is_empty() {
            return Ok(false);
        }
        if delta.len() > MAX_DELTAS {
            p.restore_physical(&delta, false)?;
            return Err(DomainError::InvalidRelation(
                crate::strings::HISTORY_CAPACITY_EXCEEDED,
            ));
        }
        self.record(Transaction::Physical(Box::new(delta)));
        Ok(true)
    }
    pub(crate) fn device(&mut self, p: &mut Project, edit: DeviceEdit) -> Result<bool> {
        let delta = p.edit_device(edit)?;
        if delta.is_empty() {
            return Ok(false);
        }
        if delta.len() > MAX_DELTAS {
            p.restore_device_edit(&delta, false)?;
            return Err(DomainError::InvalidRelation(
                crate::strings::HISTORY_CAPACITY_EXCEEDED,
            ));
        }
        self.record(Transaction::Device(Box::new(delta)));
        Ok(true)
    }
    pub(crate) fn marker(&mut self, p: &mut Project, edit: MarkerEdit) -> Result<bool> {
        let delta = p.edit_marker(edit)?;
        if delta.is_empty() {
            return Ok(false);
        }
        if delta.len() > MAX_DELTAS {
            p.restore_marker(&delta, false)?;
            return Err(DomainError::InvalidRelation(
                crate::strings::HISTORY_CAPACITY_EXCEEDED,
            ));
        }
        self.record(Transaction::Marker(Box::new(delta)));
        Ok(true)
    }
    /// Insertion with electrical registration in a single transaction.
    pub(crate) fn place(
        &mut self,
        p: &mut Project,
        placement: SymbolPlacement,
    ) -> Result<RepresentationId> {
        let delta = p.place_symbol(placement)?;
        let id = delta.representation_id();
        self.record(Transaction::Placement(Box::new(delta)));
        Ok(id)
    }
    pub(crate) fn place_existing_function(
        &mut self,
        p: &mut Project,
        placement: ExistingFunctionPlacement,
    ) -> Result<RepresentationId> {
        let delta = p.place_existing_function(placement)?;
        let id = delta.representation_id();
        self.record(Transaction::ExistingFunction(Box::new(delta)));
        Ok(id)
    }
    pub(crate) fn assign_part(
        &mut self,
        p: &mut Project,
        device: DeviceId,
        part: Part,
        quantity: u32,
    ) -> Result<bool> {
        let delta = p.assign_part(device, part, quantity)?;
        self.record(Transaction::PartAssignment(Box::new(delta)));
        Ok(true)
    }
    pub(crate) fn remove_part(&mut self, p: &mut Project, component: ComponentId) -> Result<bool> {
        let delta = p.remove_part_assignment(component)?;
        self.record(Transaction::PartRemoval(Box::new(delta)));
        Ok(true)
    }
    pub(crate) fn update_part(&mut self, p: &mut Project, part: Part) -> Result<bool> {
        let delta = p.update_part(part)?;
        self.record(Transaction::PartUpdate(Box::new(delta)));
        Ok(true)
    }
    pub(crate) fn update_parts(&mut self, p: &mut Project, parts: Vec<Part>) -> Result<bool> {
        let deltas = p.update_parts(parts)?;
        if deltas.is_empty() {
            return Ok(false);
        }
        self.record(Transaction::PartUpdates(deltas));
        Ok(true)
    }
    pub(crate) fn function_point(
        &mut self,
        p: &mut Project,
        edit: FunctionPointEdit,
    ) -> Result<bool> {
        let delta = p.edit_function_point(edit)?;
        if delta.is_empty() {
            return Ok(false);
        }
        self.record(Transaction::FunctionPoint(Box::new(delta)));
        Ok(true)
    }
    /// Documents and pages share the same history.
    pub(crate) fn document(&mut self, p: &mut Project, edit: DocumentEdit) -> Result<bool> {
        let delta = p.edit_document(edit)?;
        if delta.is_empty() {
            return Ok(false);
        }
        self.record(Transaction::Document(Box::new(delta)));
        Ok(true)
    }
    /// Frames, assignments, reference format and metadata.
    pub(crate) fn frame(&mut self, p: &mut Project, edit: FrameEdit) -> Result<bool> {
        let delta = p.edit_frame(edit)?;
        if delta.is_empty() {
            return Ok(false);
        }
        self.record(Transaction::Frame(Box::new(delta)));
        Ok(true)
    }
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
    pub(crate) fn commit(
        &mut self,
        p: &mut Project,
        batch: Vec<(RepresentationId, Transform2D)>,
    ) -> Result<bool> {
        if batch.len() > MAX_DELTAS {
            return Err(DomainError::InvalidRelation(
                "seleção excede a capacidade do histórico (50.000 objetos)",
            ));
        }
        let mut ids = std::collections::BTreeSet::new();
        for (id, _) in &batch {
            if !ids.insert(*id) {
                return Err(DomainError::DuplicateRepresentation(*id));
            }
        }
        // A selected route follows its selected endpoint symbols. Its immutable
        // identity transform must not be translated a second time by a group drag.
        let batch = batch
            .into_iter()
            .filter(|(id, _)| {
                p.connection_route(*id).is_none_or(|r| {
                    !ids.contains(&r.source().representation_id())
                        || !ids.contains(&r.target().representation_id())
                })
            })
            .collect();
        self.drawing(p, ConnectionDrawingEdit::Transform(batch))
    }
    pub(crate) fn drawing(&mut self, p: &mut Project, edit: ConnectionDrawingEdit) -> Result<bool> {
        let delta = p.edit_connection_drawing(edit)?;
        if delta.is_empty() {
            return Ok(false);
        }
        // The bounded history counts all affected routes/junctions, too.
        if delta.len() > MAX_DELTAS {
            p.restore_connection_drawing(&delta, false)?;
            return Err(DomainError::InvalidRelation(
                crate::strings::HISTORY_CAPACITY_EXCEEDED,
            ));
        }
        self.record(Transaction::Drawing(delta));
        Ok(true)
    }
    pub(crate) fn representations(
        &mut self,
        p: &mut Project,
        changes: Vec<RepresentationChange>,
    ) -> Result<bool> {
        if changes.len() > MAX_DELTAS {
            return Err(DomainError::InvalidRelation(
                crate::strings::HISTORY_CAPACITY_EXCEEDED,
            ));
        }
        let delta = p.edit_representations(changes)?;
        if delta.is_empty() {
            return Ok(false);
        }
        self.record(Transaction::Representations(delta));
        Ok(true)
    }
    /// Update project symbols atomically in one undo step.
    pub(crate) fn symbol_updates(
        &mut self,
        p: &mut Project,
        definitions: Vec<SymbolDefinition>,
    ) -> Result<bool> {
        let mut deltas: Vec<SymbolUpdateDelta> = Vec::new();
        for definition in definitions {
            match p.update_symbol_definition(definition) {
                Ok(d) => deltas.push(d),
                Err(e) => {
                    for d in deltas.iter().rev() {
                        let _ = p.restore_symbol_update(d, false);
                    }
                    return Err(e);
                }
            }
        }
        if deltas.is_empty() {
            return Ok(false);
        }
        self.record(Transaction::Symbols(deltas));
        Ok(true)
    }
    /// Cut or paste already applied by Core.
    pub(crate) fn sheet(&mut self, delta: SheetEditDelta) {
        if !delta.is_empty() {
            self.record(Transaction::Sheet(delta));
        }
    }
    /// Macro insertion already applied by Core.
    pub(crate) fn macro_insert(&mut self, delta: MacroInsertDelta) {
        if !delta.is_empty() {
            self.record(Transaction::Macro(Box::new(delta)));
        }
    }
    fn record(&mut self, transaction: Transaction) {
        self.redo.clear();
        self.undo.push_back(transaction);
        while self.undo.len() > MAX_TRANSACTIONS
            || self.undo.iter().map(Transaction::len).sum::<usize>() > MAX_DELTAS
        {
            self.undo.pop_front();
        }
    }
    pub(crate) fn visual(
        &mut self,
        p: &mut Project,
        id: RepresentationId,
        after: VisualProperties,
    ) -> Result<bool> {
        let before = p
            .representation(id)
            .ok_or(DomainError::MissingRepresentation(id))?
            .visual()
            .clone();
        if before == after {
            return Ok(false);
        }
        p.set_representation_visual(id, after.clone())?;
        self.record(Transaction::Visual { id, before, after });
        Ok(true)
    }
    pub(crate) fn insert_symbol(
        &mut self,
        p: &mut Project,
        definition: &SymbolDefinition,
        page: PageId,
        target: Option<RepresentationTarget>,
        transform: Transform2D,
    ) -> Result<RepresentationId> {
        let representation =
            Representation::new_symbol(p.id(), page, definition, target, transform)?;
        let introduced_definition = p.symbol_definition(definition.id()).is_none();
        let id = p.insert_symbol_representation(representation.clone(), definition.clone())?;
        self.record(Transaction::Insertion {
            representation: Box::new(representation),
            definition: Box::new(definition.clone()),
            introduced_definition,
        });
        Ok(id)
    }
    pub(crate) fn undo(&mut self, p: &mut Project) -> Result<bool> {
        let Some(deltas) = self.undo.back() else {
            return Ok(false);
        };
        Self::restore(p, deltas, false)?;
        self.redo.push(self.undo.pop_back().expect("checked"));
        Ok(true)
    }
    pub(crate) fn redo(&mut self, p: &mut Project) -> Result<bool> {
        let Some(deltas) = self.redo.last() else {
            return Ok(false);
        };
        Self::restore(p, deltas, true)?;
        self.undo.push_back(self.redo.pop().expect("checked"));
        Ok(true)
    }
    fn restore(p: &mut Project, transaction: &Transaction, forward: bool) -> Result<()> {
        match transaction {
            Transaction::Physical(delta) => p.restore_physical(delta, forward),
            Transaction::Device(delta) => p.restore_device_edit(delta, forward),
            Transaction::NumberDevices(delta) => p.restore_device_numbering(delta, forward),
            Transaction::NumberConnections(delta) => p.restore_connection_numbering(delta, forward),
            Transaction::NumberPages(delta) => p.restore_page_numbering(delta, forward),
            Transaction::Report(delta) => p.restore_report(delta, forward),
            Transaction::CheckScheme(delta) => p.restore_check_scheme(delta, forward),
            Transaction::ConnectionNumber(delta) => p.restore_connection_number(delta, forward),
            Transaction::Marker(delta) => p.restore_marker(delta, forward),
            Transaction::Insertion {
                representation,
                definition,
                introduced_definition,
            } => {
                if forward {
                    p.insert_symbol_representation(
                        (**representation).clone(),
                        (**definition).clone(),
                    )
                    .map(|_| ())
                } else {
                    if p.representation(representation.id()) != Some(representation.as_ref()) {
                        return Err(DomainError::InvalidRelation(
                            "histórico de inserção não corresponde ao documento atual",
                        ));
                    }
                    // Definition removal is optional when shared; validate its presence first.
                    if p.symbol_definition(definition.id()) != Some(definition.as_ref()) {
                        return Err(DomainError::InvalidRelation(
                            "definição do histórico não corresponde ao documento atual",
                        ));
                    }
                    let shared = p.representations().any(|r| {
                        r.id() != representation.id() && r.symbol() == Some(definition.id())
                    });
                    p.remove_representation(representation.id())?;
                    if *introduced_definition && !shared {
                        p.remove_unused_symbol_definition(definition.id())?;
                    }
                    Ok(())
                }
            }
            Transaction::Visual { id, before, after } => {
                let current = p
                    .representation(*id)
                    .ok_or(DomainError::MissingRepresentation(*id))?
                    .visual();
                if current != if forward { before } else { after } {
                    return Err(DomainError::InvalidRelation(
                        "histórico visual não corresponde ao documento atual",
                    ));
                }
                p.set_representation_visual(*id, if forward { after } else { before }.clone())
            }
            Transaction::Drawing(delta) => p.restore_connection_drawing(delta, forward),
            Transaction::Representations(delta) => p.restore_representations(delta, forward),
            Transaction::Sheet(delta) => p.restore_sheet_edit(delta, forward),
            Transaction::Macro(delta) => p.restore_macro_insert(delta, forward),
            Transaction::Symbols(deltas) => {
                let ordered: Vec<&SymbolUpdateDelta> = if forward {
                    deltas.iter().collect()
                } else {
                    deltas.iter().rev().collect()
                };
                let mut done: Vec<&SymbolUpdateDelta> = Vec::new();
                for d in ordered {
                    if let Err(e) = p.restore_symbol_update(d, forward) {
                        for undo in done.iter().rev() {
                            let _ = p.restore_symbol_update(undo, !forward);
                        }
                        return Err(e);
                    }
                    done.push(d);
                }
                Ok(())
            }
            Transaction::Placement(delta) => p.restore_placement(delta, forward),
            Transaction::ExistingFunction(delta) => {
                p.restore_existing_function_placement(delta, forward)
            }
            Transaction::PartAssignment(delta) => p.restore_part_assignment(delta, forward),
            Transaction::PartRemoval(delta) => p.restore_part_removal(delta, forward),
            Transaction::PartUpdate(delta) => p.restore_part_update(delta, forward),
            Transaction::PartUpdates(deltas) => p.restore_part_updates(deltas, forward),
            Transaction::FunctionPoint(delta) => p.restore_function_point(delta, forward),
            Transaction::Document(delta) => p.restore_document_edit(delta, forward),
            Transaction::Frame(delta) => p.restore_frame_edit(delta, forward),
        }
    }
}
