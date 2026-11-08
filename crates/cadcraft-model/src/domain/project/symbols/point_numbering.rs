// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! IEC 60947 point numbers suggested for a new function of an existing
//! device; start with Project::suggested_point_labels.
//!
//! The suggestion only proposes labels: the insertion centre shows them for
//! editing and the placement records whatever the user confirms.
use super::*;
use std::collections::BTreeSet;

/// Contact family read from the symbol's port identifications.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Family {
    Coil,
    NormallyOpen,
    NormallyClosed,
    MainContact,
}

/// Unit digit of a two-digit point number such as "13" or "22".
fn unit(label: &str) -> Option<u32> {
    let mut chars = label.chars();
    let (tens, units) = (chars.next()?, chars.next()?);
    (chars.next().is_none() && tens.is_ascii_digit() && tens != '0')
        .then(|| units.to_digit(10))
        .flatten()
}

fn family(definition: &SymbolDefinition) -> Option<Family> {
    let ids: Vec<&str> = definition
        .ports()
        .iter()
        .map(SymbolPort::identification)
        .collect();
    if ids.len() != 2 {
        return None;
    }
    let kind = definition.symbol_function_kind();
    if kind == SymbolFunctionKind::Coil || ids.iter().any(|i| i.starts_with('A')) {
        return Some(Family::Coil);
    }
    if kind == SymbolFunctionKind::MainContact {
        return Some(Family::MainContact);
    }
    let units: Option<BTreeSet<u32>> = ids.iter().map(|i| unit(i)).collect();
    match units?.into_iter().collect::<Vec<_>>().as_slice() {
        [3, 4] | [7, 8] => Some(Family::NormallyOpen),
        [1, 2] | [5, 6] => Some(Family::NormallyClosed),
        _ => None,
    }
}

/// Letters at the start of a device name: "KT" for "KT1", "F" for "F1".
fn device_prefix(name: &str) -> String {
    name.chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect()
}

impl Project {
    /// Point labels for a new function of `device` drawn with `definition`,
    /// one per port in port order: the next free IEC 60947 numbers (coil
    /// A1-A2; NO 13-14, 23-24…; NC 21-22, 31-32…; thermal relay F 95-96 and
    /// 97-98; timer KT 15-16 and 15-18; main poles 1-2, 3-4, 5-6). Symbols the
    /// numbering does not recognise keep their port identifications.
    pub fn suggested_point_labels(
        &self,
        device: DeviceId,
        definition: &SymbolDefinition,
    ) -> Vec<String> {
        let keep: Vec<String> = definition
            .ports()
            .iter()
            .map(|p| p.identification().to_owned())
            .collect();
        let (Some(family), Some(d)) = (family(definition), self.device(device)) else {
            return keep;
        };
        let functions: BTreeSet<FunctionId> = self
            .functions()
            .filter(|f| f.device_id() == device)
            .map(Function::id)
            .collect();
        let used: BTreeSet<String> = self
            .connection_points()
            .filter(|p| functions.contains(&p.function_id()))
            .map(|p| p.designation().as_str().to_owned())
            .collect();
        let free = |a: &str, b: &str| !used.contains(a) && !used.contains(b) && a != b;
        let prefix = device_prefix(d.designation().device().as_str());
        let pair = match family {
            Family::Coil => ["A1", "B1", "C1"]
                .iter()
                .map(|a| (a.to_string(), a.replace('1', "2")))
                .find(|(a, b)| free(a, b)),
            Family::MainContact => (1..=11)
                .step_by(2)
                .map(|n| (n.to_string(), (n + 1).to_string()))
                .find(|(a, b)| free(a, b)),
            Family::NormallyOpen | Family::NormallyClosed => {
                let open = family == Family::NormallyOpen;
                let special = match (prefix.as_str(), open) {
                    ("F", false) => Some(("95", "96")),
                    ("F", true) => Some(("97", "98")),
                    ("KT", false) => Some(("15", "16")),
                    ("KT", true) => Some(("15", "18")),
                    _ => None,
                }
                .filter(|(a, b)| free(a, b))
                .map(|(a, b)| (a.to_owned(), b.to_owned()));
                // Sequence digit: the first tens digit no point of the device uses.
                let sequence = || {
                    let taken: BTreeSet<char> = used
                        .iter()
                        .filter(|l| unit(l).is_some())
                        .filter_map(|l| l.chars().next())
                        .collect();
                    ('1'..='9').find(|c| !taken.contains(c)).map(|c| {
                        let (a, b) = if open { ('3', '4') } else { ('1', '2') };
                        (format!("{c}{a}"), format!("{c}{b}"))
                    })
                };
                special.or_else(sequence)
            }
        };
        let Some((first, second)) = pair else {
            return keep;
        };
        // The port whose identification is the lower number (13, 21, A1)
        // receives the first label.
        let lower_first = keep[0] <= keep[1];
        if lower_first {
            vec![first, second]
        } else {
            vec![second, first]
        }
    }
}
