// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::super::{ImportReport, ImportResult, fail, xml::Element};
use crate::domain::*;
use std::collections::BTreeMap;

pub(super) fn descendants<'a>(root: &'a Element, name: &str) -> Vec<&'a Element> {
    let mut result = Vec::new();
    fn visit<'a>(e: &'a Element, n: &str, r: &mut Vec<&'a Element>) {
        if e.name == n {
            r.push(e);
        }
        for c in &e.children {
            visit(c, n, r);
        }
    }
    visit(root, name, &mut result);
    result
}
fn localized(raw: &str) -> String {
    if !raw.contains('@') {
        return raw.trim().to_owned();
    }
    let entries: Vec<_> = raw.split(';').filter_map(|s| s.split_once('@')).collect();
    ["pt_BR", "pt-BR", "en_US", "en-US"]
        .iter()
        .find_map(|lang| {
            entries
                .iter()
                .find(|(l, t)| l == lang && !t.trim().is_empty())
                .map(|(_, t)| t.trim().to_string())
        })
        .or_else(|| {
            entries
                .iter()
                .find(|(_, t)| !t.trim().is_empty())
                .map(|(_, t)| t.trim().to_string())
        })
        .unwrap_or_default()
}
fn field(e: &Element, keys: &[&str], fallback: &str) -> String {
    keys.iter()
        .find_map(|k| e.attr(k).map(localized).filter(|s| !s.is_empty()))
        .unwrap_or_else(|| fallback.to_owned())
}
fn preserve_tree(e: &Element, prefix: &str, props: &mut TechnicalProperties) -> ImportResult<()> {
    for (key, value) in &e.attrs {
        props
            .set_custom(format!("{prefix}{key}"), PropertyValue::Text(value.clone()))
            .map_err(|e| fail(e.to_string()))?;
    }
    for (i, child) in e.children.iter().enumerate() {
        preserve_tree(child, &format!("{prefix}{}[{i}].", child.name), props)?;
    }
    if !e.text.trim().is_empty() {
        props
            .set_custom(format!("{prefix}text"), PropertyValue::Text(e.text.clone()))
            .map_err(|e| fail(e.to_string()))?;
    }
    Ok(())
}
pub(super) fn convert(
    root: &Element,
    manufacturers: &BTreeMap<String, String>,
    terminals: &[Element],
    report: &mut ImportReport,
) -> ImportResult<Vec<Part>> {
    if root.name != "partsmanagement" {
        return Err(fail("part.xml sem partsmanagement"));
    }
    let mut result = Vec::new();
    for e in descendants(root, "part") {
        let code = e
            .attr("P_ARTICLE_PARTNR")
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| fail("peça sem P_ARTICLE_PARTNR"))?;
        let maker = field(e, &["P_ARTICLE_MANUFACTURER"], "Não informado");
        let manufacturer = manufacturers.get(&maker).cloned().unwrap_or(maker);
        let description = field(
            e,
            &[
                "P_ARTICLE_DESCR1",
                "P_ARTICLE_DESCRIPTION",
                "P_ARTICLE_DESCR2",
            ],
            code,
        );
        let classification = field(
            e,
            &[
                "P_ARTICLE_PRODUCTGROUP",
                "P_ARTICLE_PRODUCTSUBGROUP",
                "P_ARTICLE_PRODUCTTOPGROUP",
                "P_ARTICLE_PARTTYPE",
            ],
            "EPLAN",
        );
        let mut props = TechnicalProperties::default();
        preserve_tree(e, "", &mut props)?;
        for (key, _) in &e.attrs {
            if !matches!(
                key.as_str(),
                "P_ARTICLE_PARTNR"
                    | "P_ARTICLE_MANUFACTURER"
                    | "P_ARTICLE_DESCR1"
                    | "P_ARTICLE_DESCR2"
                    | "P_ARTICLE_DESCRIPTION"
                    | "P_ARTICLE_PRODUCTGROUP"
                    | "P_ARTICLE_PRODUCTSUBGROUP"
                    | "P_ARTICLE_PRODUCTTOPGROUP"
                    | "P_ARTICLE_PARTTYPE"
            ) {
                report.info(format!("propriedade preservada com chave original: {key}"));
            }
        }
        let mut functions = Vec::new();
        for f in descendants(e, "functiontemplate") {
            let raw = f.attr("connectionDesignation").unwrap_or("");
            let points: Vec<_> = raw
                .split([';', '\n'])
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            if points.is_empty() {
                report.ignored("functiontemplate sem pontos; atributos originais preservados");
                continue;
            }
            let kind = FunctionKind::Other(
                Text::new(format!(
                    "EPLAN.{}/{}/{}",
                    f.attr("functiondefcategory").unwrap_or("?"),
                    f.attr("functiondefgroup").unwrap_or("?"),
                    f.attr("functiondefid").unwrap_or("?")
                ))
                .map_err(|e| fail(e.to_string()))?,
            );
            // Os códigos numéricos ficam opacos; não inferir bobina/contato.
            let role = match f.attr("mainfunction") {
                Some("1") => FunctionRole::Main,
                Some("0") => FunctionRole::Auxiliary,
                _ => FunctionRole::Unspecified,
            };
            let function = match PartFunction::new(kind, role, points) {
                Ok(function) => function,
                Err(error) => {
                    report.ignored(format!("functiontemplate inválido, preservado: {error}"));
                    continue;
                }
            };
            if functions.iter().any(|earlier: &PartFunction| {
                earlier.kind() == function.kind() && earlier.points() == function.points()
            }) {
                report.info("função repetida em variante, preservada nos atributos originais");
            } else {
                functions.push(function);
            }
        }
        for t in terminals {
            for terminal in descendants(t, "terminal") {
                if terminal.attr("P_PART_TERMINAL_NAME") == Some(code) {
                    preserve_tree(terminal, "EPLAN.connectionpoints.", &mut props)?;
                }
            }
        }
        let clean = |value: &str| value.split_whitespace().collect::<Vec<_>>().join(" ");
        let code_id = if PartId::new(code).is_ok() {
            code.to_owned()
        } else {
            report.approximated("identificador da peça normalizado; código original preservado");
            super::identifier(code)
        };
        if [&manufacturer, code, &description, &classification]
            .iter()
            .any(|s| s.chars().any(char::is_control))
        {
            report.approximated(
                "controles/espaços normalizados nos rótulos da peça; valores originais preservados",
            );
        }
        let mut part = Part::new(
            code_id,
            clean(&manufacturer),
            clean(code),
            clean(&description),
            clean(&classification),
        )
        .map_err(|e| fail(format!("peça inválida: {e}")))?
        .with_properties(props)
        .with_functions(functions)
        .map_err(|e| fail(e.to_string()))?;
        for (lang, desc) in e
            .attr("P_ARTICLE_DESCR1")
            .unwrap_or("")
            .split(';')
            .filter_map(|s| s.split_once('@'))
        {
            if !desc.trim().is_empty() {
                part = part
                    .with_translation(lang.replace('_', "-"), clean(desc))
                    .map_err(|e| fail(e.to_string()))?;
            }
        }
        result.push(part);
    }
    if result.is_empty() {
        report.ignored("part.xml sem peças");
    }
    Ok(result)
}
