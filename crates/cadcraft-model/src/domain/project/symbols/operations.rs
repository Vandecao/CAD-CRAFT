// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Project symbol definitions and updates; start with update_symbol_definition.
use super::*;
impl Project {
    pub fn symbol_definition(&self, id: &SymbolDefinitionId) -> Option<&SymbolDefinition> {
        self.symbol_definitions.get(id)
    }
    pub fn symbol_definitions(&self) -> impl Iterator<Item = &SymbolDefinition> {
        self.symbol_definitions.values()
    }
    /// Validação completa precede tanto o registro da definição como a ocorrência.
    pub fn insert_symbol_representation(
        &mut self,
        representation: Representation,
        definition: SymbolDefinition,
    ) -> Result<RepresentationId> {
        let id = representation.id();
        if self.representations.contains_key(&id) {
            return Err(DomainError::DuplicateRepresentation(id));
        }
        self.check_project(representation.project_id())?;
        self.require(EntityRef::Page(representation.page_id()))?;
        self.require_representation_target(representation.target())?;
        self.check_representation_definition(&representation, &definition)?;
        if let Some(previous) = self.symbol_definition(definition.id())
            && previous != &definition
        {
            return Err(DomainError::InvalidRelation(
                "definição registrada é imutável",
            ));
        }
        self.symbol_definitions
            .entry(definition.id().clone())
            .or_insert(definition);
        self.document_index.insert(&representation);
        self.representations.insert(id, representation);
        Ok(id)
    }
    /// Usada pelo histórico após remover uma ocorrência introduzida na transação.
    pub fn remove_unused_symbol_definition(
        &mut self,
        id: &SymbolDefinitionId,
    ) -> Result<Option<SymbolDefinition>> {
        if self
            .representations
            .values()
            .any(|r| r.symbol() == Some(id))
        {
            return Err(DomainError::InvalidRelation(
                "definição de símbolo ainda utilizada",
            ));
        }
        Ok(self.symbol_definitions.remove(id))
    }
    pub(in crate::domain::project) fn check_representation_definition(
        &self,
        r: &Representation,
        d: &SymbolDefinition,
    ) -> Result<()> {
        if r.symbol() != Some(d.id()) || r.local_bounds() != Some(d.bounds()) {
            return Err(DomainError::InvalidRelation(
                "referência ou limites de símbolo inconsistentes",
            ));
        }
        if !d.orientations().allows(r.transform()) {
            return Err(DomainError::InvalidRelation(
                "orientação de símbolo não permitida",
            ));
        }
        r.bounds()?;
        Ok(())
    }
    pub(in crate::domain::project) fn check_symbol_transform(
        &self,
        r: &Representation,
        t: Transform2D,
    ) -> Result<()> {
        if let Some(id) = r.symbol() {
            let d = self
                .symbol_definition(id)
                .ok_or(DomainError::InvalidRelation("definição de símbolo ausente"))?;
            if !d.orientations().allows(t) {
                return Err(DomainError::InvalidRelation(
                    "orientação de símbolo não permitida",
                ));
            }
        }
        Ok(())
    }
}

/// Etapa 17: troca reversível da definição de um símbolo por outra versão.
#[derive(Clone, Debug, PartialEq)]
pub struct SymbolUpdateDelta {
    project: ProjectId,
    before: SymbolDefinition,
    after: SymbolDefinition,
}
impl SymbolUpdateDelta {
    pub fn definition_id(&self) -> &SymbolDefinitionId {
        self.after.id()
    }
    pub fn before(&self) -> &SymbolDefinition {
        &self.before
    }
    pub fn after(&self) -> &SymbolDefinition {
        &self.after
    }
}

impl Project {
    /// Ocorrências que usam a definição.
    pub fn symbol_occurrences(&self, id: &SymbolDefinitionId) -> Vec<RepresentationId> {
        self.representations
            .values()
            .filter(|r| r.symbol() == Some(id))
            .map(Representation::id)
            .collect()
    }

    /// Motivos que impedem trocar a definição registrada por `new` (vazio =
    /// compatível): portas usadas que deixam de existir e orientações.
    pub fn symbol_update_problems(&self, new: &SymbolDefinition) -> Vec<String> {
        let Some(old) = self.symbol_definition(new.id()) else {
            return vec![format!("símbolo '{}' não existe no projeto", new.id())];
        };
        let mut problems = Vec::new();
        if new == old {
            problems.push("o projeto já usa esta versão".to_owned());
            return problems;
        }
        let ports: std::collections::BTreeSet<&str> = new.ports().iter().map(|p| p.id()).collect();
        let mut missing = std::collections::BTreeSet::new();
        for port in self.port_bindings.keys() {
            if self
                .representation(port.representation_id())
                .is_some_and(|r| r.symbol() == Some(new.id()))
                && !ports.contains(port.port_id())
            {
                missing.insert(port.port_id().to_owned());
            }
        }
        for route in self.connection_routes.values() {
            for end in [route.source(), route.target()] {
                if self
                    .representation(end.representation_id())
                    .is_some_and(|r| r.symbol() == Some(new.id()))
                    && !ports.contains(end.port_id())
                {
                    missing.insert(end.port_id().to_owned());
                }
            }
        }
        for port in missing {
            problems.push(format!("porta '{port}' em uso não existe na nova versão"));
        }
        let blocked = self
            .representations
            .values()
            .filter(|r| r.symbol() == Some(new.id()) && !new.orientations().allows(r.transform()))
            .count();
        if blocked > 0 {
            problems.push(format!(
                "{blocked} ocorrência(s) com orientação não permitida na nova versão"
            ));
        }
        problems
    }

    fn swap_symbol_definition(&mut self, definition: SymbolDefinition) {
        let id = definition.id().clone();
        let bounds = definition.bounds();
        let ids = self.symbol_occurrences(&id);
        for rid in ids {
            if let Some(mut r) = self.representations.remove(&rid) {
                self.document_index.remove(&r);
                r.local_bounds = Some(bounds);
                self.document_index.insert(&r);
                self.representations.insert(rid, r);
            }
        }
        self.symbol_definitions.insert(id, definition);
    }

    /// Troca a definição pela nova versão em todas as ocorrências (mesmos IDs
    /// e vínculos). Recusa sem alterar nada se houver incompatibilidade ou se o
    /// projeto resultante for inválido (ex.: fio ligado a porta deslocada).
    pub fn update_symbol_definition(&mut self, new: SymbolDefinition) -> Result<SymbolUpdateDelta> {
        new.validate()?;
        if !self.symbol_update_problems(&new).is_empty() {
            return Err(DomainError::InvalidRelation(
                "nova versão do símbolo incompatível com o projeto",
            ));
        }
        let before = self
            .symbol_definition(new.id())
            .expect("conferido acima")
            .clone();
        self.swap_symbol_definition(new.clone());
        if let Err(error) = self.validate() {
            self.swap_symbol_definition(before);
            return Err(error);
        }
        Ok(SymbolUpdateDelta {
            project: self.id,
            before,
            after: new,
        })
    }

    pub fn restore_symbol_update(
        &mut self,
        delta: &SymbolUpdateDelta,
        forward: bool,
    ) -> Result<()> {
        self.check_project(delta.project)?;
        let (expected, target) = if forward {
            (&delta.before, &delta.after)
        } else {
            (&delta.after, &delta.before)
        };
        if self.symbol_definition(expected.id()) != Some(expected) {
            return Err(DomainError::InvalidRelation(
                "estado do símbolo divergente do histórico",
            ));
        }
        self.swap_symbol_definition(target.clone());
        if let Err(error) = self.validate() {
            self.swap_symbol_definition(expected.clone());
            return Err(error);
        }
        Ok(())
    }

    /// Registra uma definição nova (biblioteca → projeto) sem ocorrências.
    /// Recusa se já existir outra versão com o mesmo ID.
    pub fn add_symbol_definition(&mut self, definition: SymbolDefinition) -> Result<bool> {
        definition.validate()?;
        match self.symbol_definition(definition.id()) {
            Some(existing) if existing == &definition => Ok(false),
            Some(_) => Err(DomainError::InvalidRelation(
                "o projeto já tem outra versão deste símbolo; use Atualizar símbolos",
            )),
            None => {
                self.symbol_definitions
                    .insert(definition.id().clone(), definition);
                Ok(true)
            }
        }
    }
}
