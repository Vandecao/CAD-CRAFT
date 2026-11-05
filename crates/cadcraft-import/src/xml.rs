// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Árvore XML mínima sobre `quick-xml` para os leitores de moldura.
//!
//! Recusa DOCTYPE com declarações de entidade (sem entidades externas) e
//! entidades desconhecidas; limita profundidade e quantidade de nós.
use super::{ImportResult, fail};
use quick_xml::XmlVersion;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesStart, Event};

const MAX_DEPTH: usize = 64;
const MAX_NODES: usize = 1_000_000;

#[derive(Debug, Clone, Default)]
pub(crate) struct Element {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<Element>,
    pub text: String,
}
impl Element {
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
    pub fn child(&self, name: &str) -> Option<&Element> {
        self.children.iter().find(|c| c.name == name)
    }
    pub fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Element> {
        self.children.iter().filter(move |c| c.name == name)
    }
}

fn element(start: &BytesStart<'_>) -> ImportResult<Element> {
    let name = start.name().as_ref().to_owned();
    let mut attrs = Vec::new();
    for a in start.attributes() {
        let a = a.map_err(|e| fail(format!("atributo XML inválido: {e}")))?;
        let key = a.key.as_ref().to_owned();
        let value = a
            .normalized_value(XmlVersion::Implicit1_0)
            .map_err(|e| fail(format!("valor de atributo XML inválido: {e}")))?
            .into_owned();
        attrs.push((key, value));
    }
    Ok(Element {
        name,
        attrs,
        ..Default::default()
    })
}

/// Lê o documento e devolve o elemento raiz.
pub(crate) fn parse(bytes: &[u8]) -> ImportResult<Element> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| fail("arquivo XML não está em UTF-8"))?
        .trim_start_matches('\u{feff}');
    let mut reader = quick_xml::Reader::from_str(text);
    reader.config_mut().trim_text(false);
    let mut stack: Vec<Element> = Vec::new();
    let mut root: Option<Element> = None;
    let mut nodes = 0usize;
    loop {
        let event = reader
            .read_event()
            .map_err(|e| fail(format!("XML inválido: {e}")))?;
        match event {
            Event::DocType(d) => {
                if d.xml10_content().contains("<!ENTITY") {
                    return Err(fail("XML com declaração de entidade recusado"));
                }
            }
            Event::Start(s) => {
                nodes += 1;
                if stack.len() >= MAX_DEPTH || nodes > MAX_NODES {
                    return Err(fail("XML grande ou profundo demais"));
                }
                stack.push(element(&s)?);
            }
            Event::Empty(s) => {
                nodes += 1;
                if nodes > MAX_NODES {
                    return Err(fail("XML grande demais"));
                }
                let e = element(&s)?;
                match stack.last_mut() {
                    Some(parent) => parent.children.push(e),
                    None if root.is_none() => root = Some(e),
                    None => return Err(fail("XML com mais de uma raiz")),
                }
            }
            Event::End(_) => {
                let e = stack.pop().ok_or_else(|| fail("XML mal formado"))?;
                match stack.last_mut() {
                    Some(parent) => parent.children.push(e),
                    None if root.is_none() => root = Some(e),
                    None => return Err(fail("XML com mais de uma raiz")),
                }
            }
            Event::Text(t) => {
                if let Some(top) = stack.last_mut() {
                    top.text.push_str(&t.xml10_content());
                }
            }
            Event::CData(c) => {
                if let Some(top) = stack.last_mut() {
                    top.text.push_str(&c);
                }
            }
            Event::GeneralRef(r) => {
                let resolved = match r
                    .resolve_char_ref()
                    .map_err(|e| fail(format!("referência XML inválida: {e}")))?
                {
                    Some(c) => c.to_string(),
                    None => {
                        let name: &str = &r;
                        resolve_predefined_entity(name)
                            .ok_or_else(|| fail(format!("entidade XML desconhecida: {name}")))?
                            .to_owned()
                    }
                };
                if let Some(top) = stack.last_mut() {
                    top.text.push_str(&resolved);
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if !stack.is_empty() {
        return Err(fail("XML incompleto"));
    }
    root.ok_or_else(|| fail("XML sem elemento raiz"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_attributes_text_and_refuses_entities() {
        let r =
            parse(br#"<?xml version="1.0"?><a x="1&amp;2&#10;"><b>t&lt;u</b><c/></a>"#).unwrap();
        assert_eq!(r.attr("x"), Some("1&2\n"));
        assert_eq!(r.child("b").unwrap().text, "t<u");
        assert!(r.child("c").is_some());
        let xxe =
            br#"<?xml version="1.0"?><!DOCTYPE a [<!ENTITY e SYSTEM "file:///x">]><a>&e;</a>"#;
        assert!(parse(xxe).is_err());
        assert!(parse(b"<a><b></a>").is_err());
        assert!(parse(b"<a>&zz;</a>").is_err());
    }
}
