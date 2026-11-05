// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 16: recortar e colar como transações únicas e reversíveis.
//!
//! - Copiar captura instantâneos (o projeto não muda).
//! - Recortar remove na hora as ocorrências e seus vínculos porta→ponto;
//!   dispositivo, função e pontos continuam no cadastro.
//! - Colar: conteúdo livre vira cópia com ID novo; símbolo de função cria um
//!   dispositivo novo com a próxima designação (ou, se pedido, uma função nova
//!   do mesmo dispositivo); item recortado volta com o mesmo ID e vínculos.
//!
//! Rotas, junções e marcadores não entram: a topologia elétrica não pode ser
//! duplicada como desenho isolado.
use super::*;

/// Limite de itens por operação de área de transferência.
pub const MAX_CLIPBOARD_ITEMS: usize = 10_000;

#[derive(Clone, Debug, PartialEq)]
enum ClipSource {
    /// Conteúdo livre ou símbolo sem função: cópia da representação.
    Copy,
    /// Símbolo de função: cadastro necessário para criar outro dispositivo.
    Function {
        definition: Box<SymbolDefinition>,
        function_kind: FunctionKind,
        classification: String,
        prefix: String,
        device: DeviceId,
    },
}

/// Item da área de transferência (instantâneo independente do projeto).
#[derive(Clone, Debug, PartialEq)]
pub struct ClipItem {
    representation: Representation,
    source: ClipSource,
    /// Vínculos removidos pelo recorte; `Some` até a primeira colagem.
    cut: Option<Vec<(PortRef, ConnectionPointId)>>,
}
impl ClipItem {
    pub fn representation(&self) -> &Representation {
        &self.representation
    }
    pub fn is_cut(&self) -> bool {
        self.cut.is_some()
    }
    /// Cria outro dispositivo ao colar (símbolo de função).
    pub fn creates_device(&self) -> bool {
        matches!(self.source, ClipSource::Function { .. })
    }
    /// Após a primeira colagem de um recorte, os itens passam a ser cópias do
    /// que foi colado.
    pub fn as_copy_of(&self, pasted: Representation) -> Self {
        Self {
            representation: pasted,
            source: self.source.clone(),
            cut: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Step {
    Representations(RepresentationDelta),
    Bindings {
        added: Vec<(PortRef, ConnectionPointId)>,
        removed: Vec<(PortRef, ConnectionPointId)>,
    },
    Placement(Box<PlacementDelta>),
    Visual {
        id: RepresentationId,
        before: VisualProperties,
        after: VisualProperties,
    },
}

/// Resultado reversível de recortar ou colar.
#[derive(Clone, Debug, PartialEq)]
pub struct SheetEditDelta {
    project: ProjectId,
    steps: Vec<Step>,
}
impl SheetEditDelta {
    pub fn len(&self) -> usize {
        self.steps
            .iter()
            .map(|s| match s {
                Step::Representations(d) => d.len(),
                Step::Bindings { added, removed } => added.len() + removed.len(),
                Step::Placement(d) => d.len(),
                Step::Visual { .. } => 1,
            })
            .sum()
    }
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

fn clip_error(reason: &'static str) -> DomainError {
    DomainError::InvalidRelation(reason)
}

/// Prefixo da designação (ex.: "K" de "K12").
fn prefix_of(designation: &Designation) -> String {
    let device = designation.device().as_str();
    let prefix = device.trim_end_matches(|c: char| c.is_ascii_digit());
    if prefix.is_empty() {
        "D".to_owned()
    } else {
        prefix.to_owned()
    }
}

impl Project {
    fn symbol_has_route(&self, id: RepresentationId) -> bool {
        self.connection_routes
            .values()
            .any(|r| r.source().representation_id() == id || r.target().representation_id() == id)
    }

    /// Instantâneos da seleção para copiar ou recortar.
    pub fn clipboard_items(&self, ids: &[RepresentationId]) -> Result<Vec<ClipItem>> {
        if ids.is_empty() {
            return Err(clip_error("selecione conteúdo livre ou símbolos"));
        }
        if ids.len() > MAX_CLIPBOARD_ITEMS {
            return Err(clip_error(
                "seleção grande demais para a área de transferência",
            ));
        }
        ids.iter()
            .map(|id| {
                let r = self
                    .representation(*id)
                    .ok_or(DomainError::MissingRepresentation(*id))?;
                if self.connection_route(*id).is_some() {
                    return Err(clip_error(
                        "fios (rotas) não entram na área de transferência",
                    ));
                }
                if self.marker(*id).is_some() {
                    return Err(clip_error(
                        "marcadores elétricos não entram na área de transferência",
                    ));
                }
                let source =
                    match (r.graphic(), r.symbol(), r.target()) {
                        (Some(_), _, _) => ClipSource::Copy,
                        (None, Some(def), RepresentationTarget::Function(f)) => {
                            let function = self
                                .function(f)
                                .ok_or(DomainError::MissingEntity(EntityRef::Function(f)))?;
                            let device = self.device(function.device_id()).ok_or(
                                DomainError::MissingEntity(EntityRef::Device(function.device_id())),
                            )?;
                            ClipSource::Function {
                                definition: Box::new(
                                    self.symbol_definition(def)
                                        .ok_or(clip_error("definição de símbolo ausente"))?
                                        .clone(),
                                ),
                                function_kind: function.kind().clone(),
                                classification: device.classification().as_str().to_owned(),
                                prefix: prefix_of(device.designation()),
                                device: device.id(),
                            }
                        }
                        (None, Some(_), _) => ClipSource::Copy,
                        (None, None, _) => {
                            return Err(clip_error(
                                "representação sem símbolo ou conteúdo livre não admitida",
                            ));
                        }
                    };
                Ok(ClipItem {
                    representation: r.clone(),
                    source,
                    cut: None,
                })
            })
            .collect()
    }

    /// Recorta: remove ocorrências e vínculos numa transação; devolve os itens.
    pub fn cut_representations(
        &mut self,
        ids: &[RepresentationId],
    ) -> Result<(Vec<ClipItem>, SheetEditDelta)> {
        let mut items = self.clipboard_items(ids)?;
        if ids.iter().any(|id| self.symbol_has_route(*id)) {
            return Err(clip_error(
                "símbolo ligado a fios: remova as ligações antes de recortar",
            ));
        }
        let mut removed = Vec::new();
        for item in &mut items {
            let id = item.representation.id();
            let bindings: Vec<_> = self
                .port_bindings
                .iter()
                .filter(|(p, _)| p.representation_id() == id)
                .map(|(p, c)| (p.clone(), *c))
                .collect();
            removed.extend(bindings.iter().cloned());
            item.cut = Some(bindings);
        }
        let mut steps = Vec::new();
        let result = (|| -> Result<()> {
            if !removed.is_empty() {
                let step = Step::Bindings {
                    added: vec![],
                    removed,
                };
                self.apply_step(&step, true)?;
                steps.push(step);
            }
            let delta = self.edit_representations(
                items
                    .iter()
                    .map(|i| RepresentationChange {
                        before: Some(i.representation.clone()),
                        after: None,
                    })
                    .collect(),
            )?;
            steps.push(Step::Representations(delta));
            self.validate()
        })();
        if let Err(error) = result {
            self.rollback(&steps);
            return Err(error);
        }
        Ok((
            items,
            SheetEditDelta {
                project: self.id,
                steps,
            },
        ))
    }

    /// Cola os itens na página, deslocados de (`dx`, `dy`) mm. Devolve o delta
    /// e os IDs das novas ocorrências, na ordem dos itens.
    pub fn paste_items(
        &mut self,
        items: &[ClipItem],
        page: PageId,
        dx: f64,
        dy: f64,
        keep_device: bool,
    ) -> Result<(SheetEditDelta, Vec<Representation>)> {
        self.require(EntityRef::Page(page))?;
        if items.is_empty() {
            return Err(clip_error("área de transferência vazia"));
        }
        if items.len() > MAX_CLIPBOARD_ITEMS {
            return Err(clip_error("área de transferência grande demais"));
        }
        if items
            .iter()
            .any(|i| i.representation.project_id() != self.id)
        {
            return Err(clip_error("itens copiados de outro projeto"));
        }
        let moved = |r: &Representation| -> Result<Transform2D> {
            let t = r.transform();
            Transform2D::new(
                t.translation().translated(dx, dy)?,
                t.rotation_degrees(),
                t.mirror_x(),
                t.mirror_y(),
            )
        };
        let mut steps = Vec::new();
        let mut pasted: Vec<Option<Representation>> = vec![None; items.len()];
        let result = (|| -> Result<()> {
            // 1. Cópias simples e recortes (mesmo ID) num lote de representações.
            let mut changes = Vec::new();
            let mut bindings = Vec::new();
            for (i, item) in items.iter().enumerate() {
                let transform = moved(&item.representation)?;
                let after = match (&item.cut, &item.source) {
                    (Some(b), _) => {
                        bindings.extend(b.iter().cloned());
                        item.representation.relocated(page, transform)?
                    }
                    (None, ClipSource::Copy) => {
                        item.representation.duplicate_on(page, transform)?
                    }
                    (None, ClipSource::Function { .. }) => continue,
                };
                pasted[i] = Some(after.clone());
                changes.push(RepresentationChange {
                    before: None,
                    after: Some(after),
                });
            }
            if !changes.is_empty() {
                steps.push(Step::Representations(self.edit_representations(changes)?));
            }
            if !bindings.is_empty() {
                let step = Step::Bindings {
                    added: bindings,
                    removed: vec![],
                };
                self.apply_step(&step, true)?;
                steps.push(step);
            }
            // 2. Símbolos de função: dispositivo novo (ou função nova).
            for (i, item) in items.iter().enumerate() {
                let (
                    None,
                    ClipSource::Function {
                        definition,
                        function_kind,
                        classification,
                        prefix,
                        device,
                    },
                ) = (&item.cut, &item.source)
                else {
                    continue;
                };
                let device = if keep_device && self.device(*device).is_some() {
                    PlacementDevice::Existing(*device)
                } else {
                    PlacementDevice::New {
                        designation: self.next_device_designation_at(
                            prefix,
                            page,
                            moved(&item.representation)?.translation(),
                        )?,
                        classification: classification.clone(),
                    }
                };
                let delta = self.place_symbol(SymbolPlacement {
                    page,
                    definition: definition.as_ref().clone(),
                    transform: moved(&item.representation)?,
                    device,
                    function_kind: function_kind.clone(),
                })?;
                let id = delta.representation_id();
                steps.push(Step::Placement(Box::new(delta)));
                let created = self
                    .representation(id)
                    .expect("ocorrência recém-criada")
                    .clone();
                let wanted = item.representation.visual().clone();
                if created.visual() != &wanted {
                    let step = Step::Visual {
                        id,
                        before: created.visual().clone(),
                        after: wanted,
                    };
                    self.apply_step(&step, true)?;
                    steps.push(step);
                }
                pasted[i] = self.representation(id).cloned();
            }
            Ok(())
        })();
        if let Err(error) = result {
            self.rollback(&steps);
            return Err(error);
        }
        Ok((
            SheetEditDelta {
                project: self.id,
                steps,
            },
            pasted
                .into_iter()
                .map(|r| r.expect("todo item colado"))
                .collect(),
        ))
    }

    fn apply_step(&mut self, step: &Step, forward: bool) -> Result<()> {
        match step {
            Step::Representations(d) => self.restore_representations(d, forward),
            Step::Placement(d) => self.restore_placement(d, forward),
            Step::Visual { id, before, after } => {
                self.set_representation_visual(*id, if forward { after } else { before }.clone())
            }
            Step::Bindings { added, removed } => {
                let (add, remove) = if forward {
                    (added, removed)
                } else {
                    (removed, added)
                };
                for (port, point) in remove {
                    if self.port_bindings.get(port) != Some(point) {
                        return Err(clip_error("vínculo de porta divergente"));
                    }
                }
                for (port, _) in add {
                    if self.port_bindings.contains_key(port)
                        && !remove.iter().any(|(p, _)| p == port)
                    {
                        return Err(clip_error("porta já vinculada"));
                    }
                }
                for (port, _) in remove {
                    self.port_bindings.remove(port);
                }
                for (i, (port, point)) in add.iter().enumerate() {
                    if let Err(error) = self.check_binding(port, *point) {
                        for (p, _) in &add[..i] {
                            self.port_bindings.remove(p);
                        }
                        for (p, c) in remove {
                            self.port_bindings.insert(p.clone(), *c);
                        }
                        return Err(error);
                    }
                    self.port_bindings.insert(port.clone(), *point);
                }
                Ok(())
            }
        }
    }

    fn rollback(&mut self, steps: &[Step]) {
        for step in steps.iter().rev() {
            let _ = self.apply_step(step, false);
        }
    }

    /// Desfaz (`forward = false`) ou refaz o recorte/colagem.
    pub fn restore_sheet_edit(&mut self, delta: &SheetEditDelta, forward: bool) -> Result<()> {
        self.check_project(delta.project)?;
        let ordered: Vec<&Step> = if forward {
            delta.steps.iter().collect()
        } else {
            delta.steps.iter().rev().collect()
        };
        let mut done: Vec<&Step> = Vec::new();
        for step in ordered {
            if let Err(error) = self.apply_step(step, forward) {
                for s in done.iter().rev() {
                    let _ = self.apply_step(s, !forward);
                }
                return Err(error);
            }
            done.push(step);
        }
        Ok(())
    }
}
