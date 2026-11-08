// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! How the functions of a part match the functions a device already has;
//! start with Project::part_assignment_preview.
//!
//! The preview is shown before assigning, and assign_part applies exactly
//! it: a part function either reuses a device function, reuses it after
//! renaming its points to the part's names, or creates a new function.
//! Points keep their IDs when renamed, so wires and port bindings stay.
use super::*;
use std::collections::BTreeSet;

/// What assigning a part does with one of its functions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PartFunctionMatch {
    /// Reuses this device function; missing points of the part are added.
    Existing(FunctionId),
    /// Reuses this device function after renaming its points (old → new).
    Rename {
        /// Device function reused.
        function: FunctionId,
        /// Point renames, each `(point, new label)`.
        points: Vec<(ConnectionPointId, String)>,
    },
    /// No device function fits: a new function is created.
    New,
}

/// Preview row: the part function (by index in `Part::functions`) and its match.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartFunctionPreview {
    /// Index of the function in the part.
    pub index: usize,
    /// Decision for that function.
    pub decision: PartFunctionMatch,
}

/// Sort key comparing point labels by their number ("2" before "13").
fn natural(label: &str) -> (String, u64, String) {
    let letters: String = label.chars().take_while(|c| !c.is_ascii_digit()).collect();
    let rest = &label[letters.len()..];
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    (
        letters,
        digits.parse().unwrap_or(0),
        rest[digits.len()..].to_owned(),
    )
}

impl Project {
    /// Decision for each function of `part` on `device`, in part order:
    /// 1. same kind, points contained in the part's → reuse;
    /// 2. any kind, exactly the part's points → reuse (functions drawn with
    ///    untyped symbols);
    /// 3. same kind, no more points than the part's → rename to the part's
    ///    names, in natural order;
    /// 4. otherwise → new function.
    pub fn part_assignment_preview(
        &self,
        device: DeviceId,
        part: &Part,
    ) -> Vec<PartFunctionPreview> {
        let functions: Vec<&Function> = self
            .functions
            .values()
            .filter(|f| f.device_id() == device)
            .collect();
        let points_of = |f: FunctionId| -> Vec<&ConnectionPoint> {
            let mut points: Vec<_> = self
                .connection_points
                .values()
                .filter(|p| p.function_id() == f)
                .collect();
            points.sort_by_key(|p| natural(p.designation().as_str()));
            points
        };
        let mut used = BTreeSet::new();
        let mut out = Vec::new();
        // Exact and contained matches first, so a rename never takes a
        // function that another part function matches without changes.
        let mut decisions: Vec<Option<PartFunctionMatch>> = vec![None; part.functions().len()];
        for (index, model) in part.functions().iter().enumerate() {
            let desired: BTreeSet<&str> = model.points().iter().map(Text::as_str).collect();
            let labels = |f: &Function| -> BTreeSet<&str> {
                points_of(f.id())
                    .into_iter()
                    .map(|p| p.designation().as_str())
                    .collect()
            };
            let contained = functions
                .iter()
                .filter(|f| f.kind() == model.kind() && !used.contains(&f.id()))
                .map(|f| (f, labels(f)))
                .filter(|(_, l)| l.is_subset(&desired))
                .max_by_key(|(_, l)| l.len())
                .map(|(f, _)| f.id());
            let exact = || {
                functions
                    .iter()
                    .filter(|f| !used.contains(&f.id()))
                    .find(|f| labels(f) == desired)
                    .map(|f| f.id())
            };
            if let Some(id) = contained.or_else(exact) {
                used.insert(id);
                decisions[index] = Some(PartFunctionMatch::Existing(id));
            }
        }
        for (index, model) in part.functions().iter().enumerate() {
            if decisions[index].is_none() {
                let mut desired: Vec<&str> = model.points().iter().map(Text::as_str).collect();
                desired.sort_by_key(|l| natural(l));
                let rename = functions
                    .iter()
                    .filter(|f| f.kind() == model.kind() && !used.contains(&f.id()))
                    .find_map(|f| {
                        let points = points_of(f.id());
                        (!points.is_empty() && points.len() <= desired.len()).then(|| {
                            let renames = points
                                .iter()
                                .zip(&desired)
                                .filter(|(p, l)| p.designation().as_str() != **l)
                                .map(|(p, l)| (p.id(), (*l).to_owned()))
                                .collect();
                            (f.id(), renames)
                        })
                    });
                decisions[index] = Some(match rename {
                    Some((function, points)) => {
                        used.insert(function);
                        PartFunctionMatch::Rename { function, points }
                    }
                    None => PartFunctionMatch::New,
                });
            }
            out.push(PartFunctionPreview {
                index,
                decision: decisions[index].clone().expect("decided above"),
            });
        }
        out
    }
}
