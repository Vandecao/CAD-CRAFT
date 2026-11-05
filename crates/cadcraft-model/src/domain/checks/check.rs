// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 24: verificações de consistência — tipos do motor. Regras puras do
//! cadastro, códigos estáveis, gravidade configurável e mensagens ignoradas
//! com justificativa. Não é ERC normativo nem cálculo elétrico.
use super::*;
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CheckSeverity {
    Error,
    Warning,
    Info,
}

impl CheckSeverity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Error => "Erro",
            Self::Warning => "Aviso",
            Self::Info => "Informação",
        }
    }
}

/// Regra disponível: código estável, título e padrão.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckRule {
    pub code: &'static str,
    pub title: &'static str,
    pub severity: CheckSeverity,
    pub enabled: bool,
}

const fn rule(code: &'static str, title: &'static str, severity: CheckSeverity) -> CheckRule {
    CheckRule {
        code,
        title,
        severity,
        enabled: true,
    }
}

/// Regras desta etapa, na ordem de execução (tabela também no ADR-0026).
pub const CHECK_RULES: &[CheckRule] = &[
    rule(
        "VER-001",
        "Designação de dispositivo repetida",
        CheckSeverity::Error,
    ),
    rule(
        "VER-002",
        "Designação de conector repetida",
        CheckSeverity::Warning,
    ),
    rule(
        "VER-003",
        "Número de página repetido no documento",
        CheckSeverity::Error,
    ),
    rule(
        "VER-004",
        "Dispositivo com auxiliares sem função principal",
        CheckSeverity::Warning,
    ),
    rule(
        "VER-006",
        "Função de peça não representada",
        CheckSeverity::Info,
    ),
    rule(
        "VER-007",
        "Porta de símbolo sem conexão",
        CheckSeverity::Warning,
    ),
    rule(
        "VER-008",
        "Fio sem número com numeração aplicada",
        CheckSeverity::Warning,
    ),
    rule(
        "VER-009",
        "Rede com potenciais nomeados diferentes",
        CheckSeverity::Error,
    ),
    rule(
        "VER-010",
        "Interrupção sem par no grupo",
        CheckSeverity::Warning,
    ),
    rule(
        "VER-011",
        "Marcador de potencial sem ponto",
        CheckSeverity::Warning,
    ),
    rule("VER-014", "Borne de régua sem destino", CheckSeverity::Info),
    rule("VER-016", "Borne seccionável aberto", CheckSeverity::Info),
    rule(
        "VER-017",
        "Contato acoplado sem ponto associado",
        CheckSeverity::Warning,
    ),
    rule("VER-018", "Dispositivo sem peça", CheckSeverity::Info),
    rule("VER-020", "Relatório desatualizado", CheckSeverity::Warning),
];

pub fn check_rule(code: &str) -> Option<&'static CheckRule> {
    CHECK_RULES.iter().find(|r| r.code == code)
}

/// Onde está o problema (para navegar).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CheckTarget {
    Page(PageId),
    Representation(RepresentationId),
    Device(DeviceId),
    Function(FunctionId),
    Connection(ConnectionId),
    ConnectionPoint(ConnectionPointId),
    Terminal(TerminalId),
    Cable(CableId),
    Connector(ConnectorId),
}

impl CheckTarget {
    fn key(self) -> String {
        match self {
            Self::Page(id) => format!("p:{id}"),
            Self::Representation(id) => format!("r:{id}"),
            Self::Device(id) => format!("d:{id}"),
            Self::Function(id) => format!("f:{id}"),
            Self::Connection(id) => format!("c:{id}"),
            Self::ConnectionPoint(id) => format!("cp:{id}"),
            Self::Terminal(id) => format!("t:{id}"),
            Self::Cable(id) => format!("ca:{id}"),
            Self::Connector(id) => format!("x:{id}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CheckMessage {
    pub rule: &'static str,
    pub severity: CheckSeverity,
    pub text: String,
    pub targets: Vec<CheckTarget>,
}

impl CheckMessage {
    pub(in crate::domain) fn new(
        rule: &'static str,
        text: String,
        mut targets: Vec<CheckTarget>,
    ) -> Self {
        targets.sort();
        targets.dedup();
        Self {
            rule,
            severity: check_rule(rule).map_or(CheckSeverity::Warning, |r| r.severity),
            text,
            targets,
        }
    }
    /// Identidade estável (regra + alvos), usada para ignorar e reavaliar.
    pub fn key(&self) -> String {
        let targets: Vec<String> = self.targets.iter().map(|t| t.key()).collect();
        format!("{}|{}", self.rule, targets.join(","))
    }
}

/// Ajuste de uma regra no esquema do projeto.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleSetting {
    pub enabled: bool,
    pub severity: CheckSeverity,
}

/// Mensagem ignorada pelo usuário, com justificativa obrigatória.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IgnoredCheck {
    pub key: String,
    pub reason: Text,
}

/// Esquema de verificação do projeto (Etapa 24, decisão 4). Regras sem
/// ajuste usam o padrão de [`CHECK_RULES`].
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckScheme {
    #[serde(default)]
    pub rules: BTreeMap<String, RuleSetting>,
    #[serde(default)]
    pub ignored: Vec<IgnoredCheck>,
}

impl CheckScheme {
    pub fn setting(&self, code: &str) -> RuleSetting {
        self.rules.get(code).copied().unwrap_or_else(|| {
            let rule = check_rule(code);
            RuleSetting {
                enabled: rule.is_none_or(|r| r.enabled),
                severity: rule.map_or(CheckSeverity::Warning, |r| r.severity),
            }
        })
    }
    pub fn is_ignored(&self, key: &str) -> bool {
        self.ignored.iter().any(|i| i.key == key)
    }
    pub(in crate::domain) fn validate(&self) -> Result<()> {
        if self.rules.keys().any(|c| check_rule(c).is_none()) {
            return Err(DomainError::InvalidRelation(
                "regra de verificação desconhecida",
            ));
        }
        let mut keys = std::collections::BTreeSet::new();
        if self.ignored.iter().any(|i| !keys.insert(i.key.as_str())) {
            return Err(DomainError::InvalidRelation("mensagem ignorada duplicada"));
        }
        Ok(())
    }
}

/// Alcance da verificação.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckScope {
    Project,
    Page(PageId),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CheckReport {
    pub messages: Vec<CheckMessage>,
    /// Mensagens escondidas por "Ignorar".
    pub ignored: usize,
}

impl CheckReport {
    pub fn count(&self, severity: CheckSeverity) -> usize {
        self.messages
            .iter()
            .filter(|m| m.severity == severity)
            .count()
    }
}
