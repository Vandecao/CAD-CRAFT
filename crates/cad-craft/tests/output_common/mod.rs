// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapas 21–22: cenário comum dos testes de saída (PDF e DXF).
#![allow(dead_code)]
use cad_craft::domain::*;

pub fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}
pub fn at(x: f64, y: f64) -> Transform2D {
    Transform2D::new(p(x, y), 0.0, false, false).unwrap()
}
pub fn rect(x: f64, y: f64, w: f64, h: f64) -> Rect2D {
    Rect2D::new(p(x, y), Size2D::new(w, h).unwrap()).unwrap()
}

/// Logotipo PNG real (4×2, com transparência).
pub fn logo() -> Vec<u8> {
    let mut image = image::RgbaImage::new(4, 2);
    for (i, px) in image.pixels_mut().enumerate() {
        *px = image::Rgba([200, 30, 30, if i % 2 == 0 { 255 } else { 0 }]);
    }
    let mut bytes = std::io::Cursor::new(vec![]);
    image.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
    bytes.into_inner()
}

/// Bobina com a designação como texto dinâmico.
pub fn coil() -> SymbolDefinition {
    SymbolDefinition::new(
        SymbolDefinitionId::new("teste.bobina").unwrap(),
        1,
        "Bobina",
        "Comando",
        vec![
            SymbolPrimitive::Line {
                start: p(0., -10.),
                end: p(0., -4.),
            },
            SymbolPrimitive::Rectangle {
                rect: rect(-4., -4., 8., 8.),
            },
            SymbolPrimitive::Line {
                start: p(0., 4.),
                end: p(0., 10.),
            },
            SymbolPrimitive::DynamicText {
                origin: p(6., 0.),
                field: SymbolTextField::Designation,
                height: 3.0,
                align: TextAlign::MiddleLeft,
                rotation_degrees: 0.0,
            },
        ],
        vec![
            SymbolPort::new("1", p(0., -10.), 270., "A1", None).unwrap(),
            SymbolPort::new("2", p(0., 10.), 90., "A2", None).unwrap(),
        ],
    )
    .unwrap()
}
pub fn builtin(key: &str) -> SymbolDefinition {
    SymbolLibrary::builtin()
        .get(&SymbolDefinitionId::new(format!("cad-craft.{key}")).unwrap())
        .unwrap()
        .clone()
}
pub fn place(
    project: &mut Project,
    page: PageId,
    definition: SymbolDefinition,
    device: PlacementDevice,
    kind: FunctionKind,
    (x, y): (f64, f64),
) -> PlacementDelta {
    project
        .place_symbol(SymbolPlacement {
            page,
            definition,
            transform: at(x, y),
            device,
            function_kind: kind,
        })
        .unwrap()
}
pub fn new_device(name: &str) -> PlacementDevice {
    PlacementDevice::New {
        designation: Designation::new(name).unwrap(),
        classification: "Contator".into(),
    }
}
pub fn marker(project: &mut Project, page: PageId, name: &str) -> RepresentationId {
    let def = SymbolDefinition::new(
        SymbolDefinitionId::new("teste.interrupcao").unwrap(),
        1,
        "Interrupção",
        "Referências",
        vec![SymbolPrimitive::Line {
            start: p(0., 0.),
            end: p(5., 0.),
        }],
        vec![SymbolPort::new("p", p(0., 0.), 0., "P", None).unwrap()],
    )
    .unwrap();
    let r = Representation::new_symbol(project.id(), page, &def, None, at(200.0, 100.0)).unwrap();
    let id = r.id();
    project
        .edit_marker(MarkerEdit::Insert {
            marker: ElectricalMarker::new(
                id,
                Text::new(name).unwrap(),
                MarkerKind::Interruption,
                None,
            ),
            representation: Box::new(r),
            definition: Box::new(def),
        })
        .unwrap();
    id
}

pub struct Fixture {
    pub project: Project,
    pub pages: [PageId; 3],
    pub coil: RepresentationId,
    pub contact: RepresentationId,
}

/// Três páginas A3 com moldura de fábrica + logotipo; bobina -K1 e fio
/// numerado na página 1; contato de -K1 na página 2; interrupções 2 ↔ 3.
pub fn fixture(name: &str) -> Fixture {
    let mut project = Project::new(name).unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "Esquema elétrico").unwrap())
        .unwrap();
    let pages = [("Força", "1"), ("Comando", "2"), ("Sinalização", "3")].map(|(n, number)| {
        project
            .insert_page(
                Page::new(
                    project.id(),
                    doc,
                    n,
                    number,
                    PageKind::MultiLine,
                    PaperFormat::A3.size(true),
                )
                .unwrap(),
            )
            .unwrap()
    });
    let frame = FrameTemplate::factory_a3();
    let asset = ImageAsset::from_bytes(logo()).unwrap();
    let mut draft = frame.draft();
    draft.elements.push(FrameElement::Image {
        rect: rect(300.0, 10.0, 40.0, 20.0),
        asset: asset.id(),
    });
    let fid = frame.id();
    project
        .edit_frame(FrameEdit::Batch(vec![
            FrameEdit::AddAsset(asset),
            FrameEdit::UpsertFrame(frame.edited(draft).unwrap()),
            FrameEdit::SetDefaultFrame(Some(fid)),
        ]))
        .unwrap();
    let k1 = place(
        &mut project,
        pages[0],
        coil(),
        new_device("K1"),
        FunctionKind::Coil,
        (130.0, 150.0),
    );
    let k2 = place(
        &mut project,
        pages[0],
        coil(),
        new_device("K2"),
        FunctionKind::Coil,
        (130.0, 100.0),
    );
    let main = project.function_for_target(
        project
            .representation(k1.representation_id())
            .unwrap()
            .target(),
    );
    project
        .edit_device(DeviceEdit::SetMainFunction {
            device: k1.device_id(),
            function: main,
        })
        .unwrap();
    let contact = place(
        &mut project,
        pages[1],
        builtin("contact-no"),
        PlacementDevice::Existing(k1.device_id()),
        FunctionKind::AuxiliaryContact,
        (60.0, 160.0),
    );
    let a = PortRef::new(k1.representation_id(), "1").unwrap();
    let b = PortRef::new(k2.representation_id(), "2").unwrap();
    project
        .edit_connection_drawing(ConnectionDrawingEdit::CreateRoute {
            source: a,
            target: b,
            points: vec![p(130.0, 140.0), p(130.0, 110.0)],
        })
        .unwrap();
    let preview = project.preview_connection_numbering().unwrap();
    project.apply_connection_numbering(&preview).unwrap();
    let first = marker(&mut project, pages[1], "X1");
    let second = marker(&mut project, pages[2], "X1");
    project
        .edit_marker(MarkerEdit::SetGroup {
            id: ReferenceGroupId::new(),
            mode: ReferenceMode::Star,
            members: vec![first, second],
            continuity: false,
        })
        .unwrap();
    Fixture {
        project,
        pages,
        coil: k1.representation_id(),
        contact: contact.representation_id(),
    }
}

/// Projeto de escala: uma página A3 com moldura, 30 bobinas e 30 fios,
/// copiada até `pages` páginas.
pub fn scale_project(pages: usize) -> Project {
    let mut project = Project::new("Escala").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "D").unwrap())
        .unwrap();
    let a3 = FrameTemplate::factory_a3();
    let id = a3.id();
    project.edit_frame(FrameEdit::UpsertFrame(a3)).unwrap();
    project
        .edit_frame(FrameEdit::SetDefaultFrame(Some(id)))
        .unwrap();
    let first = project
        .insert_page(
            Page::new(
                project.id(),
                doc,
                "P",
                "1",
                PageKind::MultiLine,
                PaperFormat::A3.size(true),
            )
            .unwrap(),
        )
        .unwrap();
    let mut reps = vec![];
    for i in 0..30 {
        let x = 30.0 + 12.0 * f64::from(i);
        let delta = place(
            &mut project,
            first,
            coil(),
            new_device(&format!("K{}", i + 1)),
            FunctionKind::Coil,
            (x, 150.0),
        );
        reps.push((delta.representation_id(), x));
    }
    for (i, (rep, x)) in reps.iter().enumerate() {
        let next = reps[(i + 1) % reps.len()];
        project
            .edit_connection_drawing(ConnectionDrawingEdit::CreateRoute {
                source: PortRef::new(*rep, "2").unwrap(),
                target: PortRef::new(next.0, "1").unwrap(),
                points: vec![
                    p(*x, 160.0),
                    p(*x, 170.0 + i as f64),
                    p(next.1, 170.0 + i as f64),
                    p(next.1, 140.0),
                ],
            })
            .unwrap();
    }
    // Copia a página no JSON com IDs novos e valida uma vez ao carregar
    // (inserir macros de página validaria o projeto inteiro a cada vez).
    replicate_page(&project, pages)
}

/// Projeto de uma página → `count` páginas iguais (IDs novos, número da
/// página sequencial). Coleções compartilhadas (documentos, símbolos,
/// molduras, imagens) não são copiadas.
pub fn replicate_page(project: &Project, count: usize) -> Project {
    const COPIED: [&str; 12] = [
        "pages",
        "devices",
        "functions",
        "components",
        "connection_points",
        "connections",
        "representations",
        "port_bindings",
        "connection_routes",
        "connection_junctions",
        "markers",
        "marker_groups",
    ];
    let mut value = serde_json::to_value(project).unwrap();
    let subset: serde_json::Map<String, serde_json::Value> = COPIED
        .iter()
        .filter_map(|k| value.get(*k).map(|v| ((*k).to_owned(), v.clone())))
        .collect();
    let ids: Vec<String> = subset
        .values()
        .filter_map(|v| v.as_array())
        .flatten()
        .filter_map(|o| o.get("id").and_then(|id| id.as_str()).map(str::to_owned))
        .collect();
    let template = serde_json::to_string(&subset).unwrap();
    for n in 2..=count {
        let mut copy = template.clone();
        for id in &ids {
            copy = copy.replace(id.as_str(), &uuid::Uuid::now_v7().to_string());
        }
        let mut copy: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&copy).unwrap();
        copy["pages"][0]["number"] = serde_json::Value::String(n.to_string());
        for (key, items) in copy {
            let target = value[key.as_str()].as_array_mut().unwrap();
            target.extend(items.as_array().unwrap().iter().cloned());
        }
    }
    serde_json::from_value(value).unwrap()
}
