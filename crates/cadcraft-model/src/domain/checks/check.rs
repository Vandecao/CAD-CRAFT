// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Consistency check engine types; start with CHECK_RULES. Pure registry rules have stable codes,
//! configurable severity and justified ignored messages. These checks provide neither normative ERC
//! nor electrical calculations.
use super::*;
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Severity assigned to a consistency-check message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CheckSeverity {
    /// Finding that prevents a consistency check from passing.
    Error,
    /// Finding requiring attention without blocking the project.
    Warning,
    /// Informational finding.
    Info,
}

impl CheckSeverity {
    /// Return the pt-BR display label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Error => "Erro",
            Self::Warning => "Aviso",
            Self::Info => "Informação",
        }
    }
}

/// Available rule with its stable code, title and defaults.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckRule {
    /// Stable rule or catalog code.
    pub code: &'static str,
    /// Human-readable column or rule title.
    pub title: &'static str,
    /// Severity assigned to this rule or finding.
    pub severity: CheckSeverity,
    /// Whether this consistency rule runs.
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

/// Rules in execution order; also listed in ADR-0026.
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
    rule(
        "VER-021",
        "Designação de ponto repetida no dispositivo",
        CheckSeverity::Warning,
    ),
];

/// Look up a consistency rule by its stable code; unknown codes return None.
pub fn check_rule(code: &str) -> Option<&'static CheckRule> {
    CHECK_RULES.iter().find(|r| r.code == code)
}

/// Problem location for navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CheckTarget {
    /// Explicit reference to a page.
    Page(PageId),
    /// Explicit reference to a representation.
    Representation(RepresentationId),
    /// Explicit reference to a device.
    Device(DeviceId),
    /// Explicit reference to a logical function.
    Function(FunctionId),
    /// Explicit reference to a connection.
    Connection(ConnectionId),
    /// Explicit reference to a connection point.
    ConnectionPoint(ConnectionPointId),
    /// Explicit reference to a terminal.
    Terminal(TerminalId),
    /// Explicit reference to a cable.
    Cable(CableId),
    /// Explicit reference to a connector.
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

/// One consistency-check finding with stable identity and navigation targets.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckMessage {
    /// Stable code of the rule that produced the message.
    pub rule: &'static str,
    /// Severity assigned to this rule or finding.
    pub severity: CheckSeverity,
    /// Text displayed to the user.
    pub text: String,
    /// Explicit targets used for navigation and stable finding identity.
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
    /// Stable identity combining rule and targets, used for ignoring and reevaluation.
    pub fn key(&self) -> String {
        let targets: Vec<String> = self.targets.iter().map(|t| t.key()).collect();
        format!("{}|{}", self.rule, targets.join(","))
    }
}

/// Rule override in the project's check scheme.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleSetting {
    /// Whether this consistency rule runs.
    pub enabled: bool,
    /// Severity assigned to this rule or finding.
    pub severity: CheckSeverity,
}

/// Message ignored by the user with a mandatory justification.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IgnoredCheck {
    /// Stable lookup key.
    pub key: String,
    /// User-supplied justification for ignoring the finding.
    pub reason: Text,
}

/// Project check scheme. Rules without overrides use the defaults in [`CHECK_RULES`].
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckScheme {
    /// Rule settings indexed by stable rule code.
    #[serde(default)]
    pub rules: BTreeMap<String, RuleSetting>,
    /// Findings explicitly ignored with a justification.
    #[serde(default)]
    pub ignored: Vec<IgnoredCheck>,
}

impl CheckScheme {
    /// Return the effective setting for a rule, using its default when not overridden.
    pub fn setting(&self, code: &str) -> RuleSetting {
        self.rules.get(code).copied().unwrap_or_else(|| {
            let rule = check_rule(code);
            RuleSetting {
                enabled: rule.is_none_or(|r| r.enabled),
                severity: rule.map_or(CheckSeverity::Warning, |r| r.severity),
            }
        })
    }
    /// Whether the scheme contains an ignored finding with this stable key.
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

/// Scope of a consistency check run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckScope {
    /// The entire project.
    Project,
    /// Explicit reference to a page.
    Page(PageId),
}

/// Results of a consistency-check run, including ignored findings.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CheckReport {
    /// Findings produced by this check run.
    pub messages: Vec<CheckMessage>,
    /// Messages hidden by the user's ignore action.
    pub ignored: usize,
}

impl CheckReport {
    /// Count visible findings with the requested severity.
    pub fn count(&self, severity: CheckSeverity) -> usize {
        self.messages
            .iter()
            .filter(|m| m.severity == severity)
            .count()
    }
}
