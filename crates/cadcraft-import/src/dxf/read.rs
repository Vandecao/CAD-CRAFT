// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Reads ASCII DXF sections, blocks and entities; start at `parse`.
use super::*;

#[derive(Debug, Clone, Default)]
pub(super) struct Entity {
    pub(super) kind: String,
    pub(super) groups: Vec<(i32, String)>,
    /// Vértices de `POLYLINE` e atributos de `INSERT`.
    pub(super) children: Vec<Entity>,
}
impl Entity {
    pub(super) fn get(&self, code: i32) -> Option<&str> {
        self.groups
            .iter()
            .find(|(c, _)| *c == code)
            .map(|(_, v)| v.as_str())
    }
    pub(super) fn num(&self, code: i32) -> Option<f64> {
        self.get(code)
            .and_then(|v| v.trim().parse::<f64>().ok())
            .filter(|v| v.is_finite())
    }
    pub(super) fn num_or(&self, code: i32, default: f64) -> f64 {
        self.num(code).unwrap_or(default)
    }
    pub(super) fn int(&self, code: i32) -> i64 {
        self.get(code)
            .and_then(|v| v.trim().parse::<i64>().ok())
            .unwrap_or(0)
    }
}

pub(super) struct Block {
    pub(super) base: (f64, f64),
    pub(super) entities: Vec<Entity>,
}

fn decode(bytes: &[u8]) -> String {
    cadcraft_model::dxf::decode(bytes)
}
fn pairs(text: &str) -> ImportResult<Vec<(i32, String)>> {
    cadcraft_model::dxf::parse_groups(text).map_err(|e| fail(e.to_string()))
}

pub(super) struct Parsed {
    pub(super) header: BTreeMap<String, Vec<(i32, String)>>,
    pub(super) blocks: BTreeMap<String, Block>,
    pub(super) entities: Vec<Entity>,
    pub(super) meta: Option<String>,
}

pub(super) fn parse(bytes: &[u8]) -> ImportResult<Parsed> {
    check_size(bytes)?;
    let text = decode(bytes);
    let pairs = pairs(&text)?;
    if !pairs.iter().any(|(c, v)| *c == 0 && v == "SECTION") {
        return Err(fail("o arquivo não é um DXF ASCII"));
    }
    if !pairs.iter().any(|(c, v)| *c == 0 && v == "EOF") {
        return Err(fail("DXF truncado (sem EOF)"));
    }
    let mut parsed = Parsed {
        header: BTreeMap::new(),
        blocks: BTreeMap::new(),
        entities: Vec::new(),
        meta: None,
    };
    let mut i = 0;
    while i < pairs.len() {
        let (code, value) = &pairs[i];
        if *code == 999 && value.starts_with(META_PREFIX) {
            parsed.meta = Some(value[META_PREFIX.len()..].to_owned());
        }
        if *code == 0 && value == "SECTION" && i + 1 < pairs.len() && pairs[i + 1].0 == 2 {
            let name = pairs[i + 1].1.clone();
            let end = (i + 2..pairs.len())
                .find(|&j| pairs[j].0 == 0 && pairs[j].1 == "ENDSEC")
                .ok_or_else(|| fail(format!("DXF truncado (seção {name} sem ENDSEC)")))?;
            let body = &pairs[i + 2..end];
            match name.as_str() {
                "HEADER" => {
                    let mut current: Option<String> = None;
                    for (c, v) in body {
                        if *c == 9 {
                            current = Some(v.clone());
                        } else if let Some(k) = &current {
                            parsed
                                .header
                                .entry(k.clone())
                                .or_default()
                                .push((*c, v.clone()));
                        }
                    }
                }
                "BLOCKS" => {
                    for block in entities(body)? {
                        if block.kind == "BLOCK" {
                            let name = block.get(2).unwrap_or("").to_owned();
                            parsed.blocks.insert(
                                name,
                                Block {
                                    base: (block.num_or(10, 0.0), block.num_or(20, 0.0)),
                                    entities: block.children,
                                },
                            );
                        }
                    }
                }
                "ENTITIES" => parsed.entities = entities(body)?,
                _ => {}
            }
            i = end;
        }
        i += 1;
    }
    Ok(parsed)
}

/// Agrupa pares em entidades; `POLYLINE`/`INSERT` recebem os filhos até
/// `SEQEND`, `BLOCK` recebe as entidades até `ENDBLK`.
fn entities(body: &[(i32, String)]) -> ImportResult<Vec<Entity>> {
    let mut flat: Vec<Entity> = Vec::new();
    for (c, v) in body {
        if *c == 0 {
            flat.push(Entity {
                kind: v.clone(),
                ..Default::default()
            });
        } else if let Some(e) = flat.last_mut() {
            e.groups.push((*c, v.clone()));
        }
    }
    let mut out: Vec<Entity> = Vec::new();
    let mut block: Option<Entity> = None;
    let mut open: Option<Entity> = None;
    for e in flat {
        let target = |block: &mut Option<Entity>, out: &mut Vec<Entity>, e: Entity| match block {
            Some(b) => b.children.push(e),
            None => out.push(e),
        };
        match e.kind.as_str() {
            "BLOCK" => block = Some(e),
            "ENDBLK" => {
                if let Some(p) = open.take() {
                    target(&mut block, &mut out, p);
                }
                if let Some(b) = block.take() {
                    out.push(b);
                }
            }
            "VERTEX" | "ATTRIB" if open.is_some() => {
                open.as_mut().expect("aberto").children.push(e);
            }
            "SEQEND" => {
                if let Some(p) = open.take() {
                    target(&mut block, &mut out, p);
                }
            }
            _ => {
                if let Some(p) = open.take() {
                    target(&mut block, &mut out, p);
                }
                let has_children = e.kind == "POLYLINE" || (e.kind == "INSERT" && e.int(66) == 1);
                if has_children {
                    open = Some(e);
                } else {
                    target(&mut block, &mut out, e);
                }
            }
        }
    }
    if let Some(p) = open {
        out.push(p);
    }
    Ok(out)
}
