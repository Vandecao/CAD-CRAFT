// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Original, in-memory documentary sample. Bounds are not electrical symbols.
use cad_craft::{domain::*, error::Result};
pub fn project() -> Result<Project> {
    let mut p = Project::new("Demonstração CAD-CRAFT — Estação de bombeamento")?;
    p.set_description("Exemplo documental próprio. Retângulos e âncoras mostram limites de representações; não são símbolos IEC nem esquemas executáveis.");
    p.set_identification(Some(Text::new("DEMO.001")?));
    p.set_metadata("Autor", "CAD-CRAFT")?;
    p.set_metadata("Finalidade", "Exploração da interface da Etapa 5")?;
    let doc = p.insert_document(Document::new(p.id(), "Engenharia elétrica")?)?;
    let aux = p.insert_document(Document::new(p.id(), "Documentação técnica")?)?;
    let mut pages = Vec::new();
    for (i, (name, kind, function, location)) in [
        ("Visão geral", PageKind::Overview, "BOMBEAMENTO", "ESTACAO"),
        ("Alimentação", PageKind::SingleLine, "ENERGIA", "PAINEL_01"),
        (
            "Comando da bomba",
            PageKind::MultiLine,
            "BOMBEAMENTO/CONTROLE",
            "PAINEL_01",
        ),
        (
            "Arranjo do painel",
            PageKind::PanelLayout,
            "BOMBEAMENTO",
            "PAINEL_01",
        ),
        (
            "Reserva",
            PageKind::MultiLine,
            "BOMBEAMENTO/CONTROLE",
            "CAMPO",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let id = p.insert_page(Page::new(
            p.id(),
            doc,
            name,
            (i + 1).to_string(),
            kind,
            Size2D::new(420.0, 297.0)?,
        )?)?;
        p.set_page_structure(id, DocumentStructure::new(path(function)?, path(location)?))?;
        pages.push(id);
    }
    p.reorder_pages(doc, &pages)?;
    p.insert_page(Page::new(
        p.id(),
        aux,
        "Notas da demonstração",
        "1",
        PageKind::TechnicalDocumentation,
        Size2D::new(210.0, 297.0)?,
    )?)?;
    let device = p.insert_device(Device::new(
        p.id(),
        Designation::new("K1")?,
        "Contator de comando",
    )?)?;
    let function = p.insert_function(Function::new(
        p.id(),
        device,
        Designation::new("K1")?,
        FunctionKind::Coil,
    ))?;
    for (page, x, y, angle, label) in [
        (pages[0], 80.0, 180.0, 0.0, "K1 • Limites documentais"),
        (pages[2], 150.0, 120.0, 30.0, "K1 • Outra ocorrência"),
        (pages[2], 455.0, 80.0, 0.0, "Ocorrência fora da folha"),
    ] {
        let rep = Representation::new(
            p.id(),
            page,
            RepresentationTarget::Device(device),
            Transform2D::new(Point2D::new(x, y)?, angle, false, false)?,
        )
        .with_local_bounds(Rect2D::new(
            Point2D::new(0.0, 0.0)?,
            Size2D::new(50.0, 28.0)?,
        )?)?
        .with_visual(VisualProperties {
            label: Some(Text::new(label)?),
            ..Default::default()
        });
        p.insert_representation(rep)?;
    }
    p.insert_representation(
        Representation::new(
            p.id(),
            pages[0],
            RepresentationTarget::Function(function),
            Transform2D::new(Point2D::new(230.0, 120.0)?, 0.0, false, false)?,
        )
        .with_visual(VisualProperties {
            label: Some(Text::new("Âncora da função K1")?),
            ..Default::default()
        }),
    )?;
    p.validate()?;
    Ok(p)
}
pub fn path(text: &str) -> Result<Vec<DesignationPart>> {
    if text.trim().is_empty() {
        Ok(vec![])
    } else {
        text.trim()
            .split('/')
            .map(|s| DesignationPart::new(s.trim()))
            .collect()
    }
}
