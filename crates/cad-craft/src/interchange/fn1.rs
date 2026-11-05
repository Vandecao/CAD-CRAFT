// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Moldura do EPLAN (`.fn1`, XML PXF).
//!
//! O nome dos elementos (`O178`, `O179`…) muda de arquivo para arquivo; o
//! tipo estável é o primeiro número de `A19="tipo/id"`. Códigos conferidos
//! nas molduras do usuário e na prévia SVG embutida pelo próprio EPLAN:
//! 21 moldura, 26 grupo, 30 texto, 31 linha, 34 polilinha, 53 texto
//! especial, 76 configuração, 89 retângulo, 91 imagem, 212 campo de usuário.
use super::build::{Builder, round3};
use super::xml::{self, Element};
use super::{ImportResult, ImportedFrame, check_size, fail};
use crate::domain::{
    FrameElement, FrameTextStyle, LocalizedText, SpecialField, Text, TextAlign, ZoneAxis,
    ZoneLabelStyle, ZoneLayout,
};

/// Espessura usada quando o EPLAN indica "da camada" (`L`).
const DEFAULT_WIDTH: f64 = 0.25;
const DEFAULT_TEXT_HEIGHT: f64 = 2.5;
/// Classe de estrutura do identificador 1001/1003: função (=) e local (+).
const STRUCTURE_FUNCTION: &str = "20132";
const STRUCTURE_LOCATION: &str = "20134";
/// Rótulos de coluna e de linha desenhados pela moldura EPLAN.
const COLUMN_LABEL: u32 = 19005;
const ROW_LABEL: u32 = 19006;

/// Imagens referenciadas (`$(MD_IMG)\logo.jpg`) são pedidas ao chamador,
/// que sabe onde fica a pasta de imagens do EPLAN.
pub type ImageResolver<'a> = dyn FnMut(&str) -> Option<Vec<u8>> + 'a;

fn kind(e: &Element) -> Option<u32> {
    e.attr("A19")?.split('/').next()?.parse().ok()
}
fn num(s: Option<&str>) -> Option<f64> {
    s.and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|v| v.is_finite())
}
fn point(s: Option<&str>) -> Option<(f64, f64)> {
    let (x, y) = s?.split_once('/')?;
    Some((num(Some(x))?, num(Some(y))?))
}
fn list(s: Option<&str>) -> Vec<f64> {
    s.map(|v| v.split(';').filter_map(|p| num(Some(p))).collect())
        .unwrap_or_default()
}
fn width(e: &Element) -> f64 {
    num(e.attr("A845"))
        .filter(|w| *w >= 0.0)
        .unwrap_or(DEFAULT_WIDTH)
}

/// `idioma@texto;…` → texto padrão (pt-BR, neutro, en-US, primeiro) e traduções.
pub(crate) fn localized(raw: &str) -> LocalizedText {
    let mut entries: Vec<(String, String)> = Vec::new();
    for part in raw.split(';') {
        let Some((lang, text)) = part.split_once('@') else {
            continue;
        };
        if text.trim().is_empty() {
            continue;
        }
        entries.push((lang.trim().replace('_', "-"), text.to_owned()));
    }
    let pick = ["pt-BR", "??-??", "en-US"]
        .iter()
        .find_map(|l| entries.iter().position(|(k, _)| k == l))
        .or(if entries.is_empty() { None } else { Some(0) });
    let Some(i) = pick else {
        return LocalizedText::new("");
    };
    let (default_lang, default) = entries[i].clone();
    let mut text = LocalizedText::new(default);
    for (k, v) in entries {
        if k != default_lang && k != "??-??" {
            text.translations.insert(k, v);
        }
    }
    text
}

/// Alinhamento EPLAN (grade 3 × 4): 1–3 topo, 4–6 meio, 7–9 linha de base,
/// 10–12 base; em cada grupo esquerda, centro, direita.
fn align(code: Option<f64>) -> TextAlign {
    use TextAlign::*;
    match code.map(|c| c as i64) {
        Some(1) => TopLeft,
        Some(2) => TopCenter,
        Some(3) => TopRight,
        Some(4) => MiddleLeft,
        Some(5) => Center,
        Some(6) => MiddleRight,
        Some(8) | Some(11) => BottomCenter,
        Some(9) | Some(12) => BottomRight,
        _ => BottomLeft,
    }
}
fn style(e: &Element) -> FrameTextStyle {
    let format = e.children.iter().find(|c| c.attr("A1939").is_some());
    let height = format
        .and_then(|f| num(f.attr("A1939")))
        .filter(|h| *h > 0.0)
        .unwrap_or(DEFAULT_TEXT_HEIGHT);
    let mut s = FrameTextStyle::new(height, align(format.and_then(|f| num(f.attr("A1949")))));
    s.rotation_degrees = round3(
        format
            .and_then(|f| num(f.attr("A1941")))
            .unwrap_or(0.0)
            .to_degrees(),
    );
    s
}

fn key(k: &str) -> Text {
    Text::new(k).expect("chave fixa")
}
/// Número de propriedade EPLAN → campo especial do CAD-CRAFT. Nomes conferidos
/// na documentação da API instalada com o EPLAN; propriedades sem equivalente
/// direto viram metadados com chave legível ou `EPLAN.<número>`.
pub fn eplan_property(id: u32, structure: Option<&str>) -> (SpecialField, bool) {
    use SpecialField::*;
    let project = |k: &str| ProjectProperty(key(k));
    let page = |k: &str| PageProperty(key(k));
    let field = match id {
        10000 => ProjectName,
        10011 => ProjectDescription,
        10013 => ProjectIdentification,
        10300 => PageCountInProject,
        11000 | 11042 => PageNumber,
        11011 => PageName,
        11012 | 25020 => PagePosition,
        11016 => FrameName,
        11040 => NextPageNumber,
        11041 => PreviousPageNumber,
        1001 | 1003 => match structure {
            Some(STRUCTURE_FUNCTION) => PageFunctionalPath,
            Some(STRUCTURE_LOCATION) => PageLocationPath,
            _ => page(&format!("EPLAN.{id}")),
        },
        10014 => project("Comissão"),
        10015 => project("Empresa"),
        10016 => project("Endereço da empresa 1"),
        10017 => project("Endereço da empresa 2"),
        10020 => project("Autor"),
        10021 => project("Data de criação"),
        10151 => project("Projeto original"),
        10160 => project("Aprovado por"),
        10180 => project("Código do cliente"),
        10230 => project("Autor (sigla)"),
        11015 => page("Formulário"),
        11022 => page("Desenhista"),
        11024 => page("Data"),
        11030 => page("Número do desenho"),
        11036 => page("Identificador do objeto"),
        11043 => page("Maior número da estrutura"),
        11059 => page("Observações"),
        11070 => page("Revisão - autor"),
        11071 => page("Revisão - índice"),
        11072 => page("Revisão - descrição"),
        11074 => page("Revisão - data"),
        11080 => page("Aprovado por"),
        11085 => page("Em substituição de"),
        11086 => page("Substituído por"),
        1002 => page("Descrição da estrutura"),
        1012 => page("Descrição da estrutura superior"),
        1520 | 1540 => page("Tipo de documento"),
        1130 => page("Descrição da função"),
        1230 => page("Descrição do local"),
        10025 => project("Responsável"),
        10035 => project("Local"),
        10155 => project("Revisão do projeto"),
        10162 => project("Verificado por"),
        11004 => page("Páginas por estrutura"),
        11017 => page("Tipo de página"),
        11020 => page("Criado por"),
        11021 => page("Data de criação"),
        11048 => page("Escala"),
        11081 => page("Aprovado em"),
        11082 => page("Verificado por"),
        11083 => page("Verificado em"),
        11901 => page("Campo suplementar"),
        n if (10000..=10999).contains(&n) => return (project(&format!("EPLAN.{id}")), false),
        _ => return (page(&format!("EPLAN.{id}")), false),
    };
    (field, true)
}

struct Labels {
    columns: Vec<(f64, f64)>,
    rows: Vec<(f64, f64)>,
}

fn walk(e: &Element, b: &mut Builder, labels: &mut Labels, resolve: &mut ImageResolver<'_>) {
    for c in &e.children {
        let Some(k) = kind(c) else {
            continue; // subobjetos de formato e propriedades
        };
        match k {
            26 => walk(c, b, labels, resolve),
            30 => match (point(c.attr("A1019")), c.attr("A1039")) {
                (Some(at), Some(raw)) => b.text(at, localized(raw), style(c)),
                _ => b.report.ignored("texto EPLAN sem posição ou conteúdo"),
            },
            31 => match (point(c.attr("A1079")), point(c.attr("A1081"))) {
                (Some(a), Some(z)) => b.line(a, z, width(c)),
                _ => b.report.ignored("linha EPLAN sem pontos"),
            },
            34 => {
                let points: Vec<(f64, f64)> = c
                    .attr("A1259")
                    .unwrap_or("")
                    .split(';')
                    .filter_map(|p| point(Some(p)))
                    .collect();
                let bulges = list(c.attr("A1261"));
                if points.len() < 2 {
                    b.report.ignored("polilinha EPLAN com menos de dois pontos");
                    continue;
                }
                let w = width(c);
                for (i, pair) in points.windows(2).enumerate() {
                    b.segment(pair[0], pair[1], bulges.get(i).copied().unwrap_or(0.0), w);
                }
                if c.attr("A1263") == Some("1") && points.first() != points.last() {
                    let last = bulges.get(points.len() - 1).copied().unwrap_or(0.0);
                    b.segment(points[points.len() - 1], points[0], last, w);
                }
            }
            53 => {
                let (Some(at), Some(id)) = (
                    point(c.attr("A1019")),
                    num(c.attr("A1559")).map(|v| v as u32),
                ) else {
                    b.report
                        .ignored("texto especial EPLAN sem posição ou propriedade");
                    continue;
                };
                if id == COLUMN_LABEL {
                    labels.columns.push(at);
                    continue;
                }
                if id == ROW_LABEL {
                    labels.rows.push(at);
                    continue;
                }
                let (field, known) = eplan_property(id, c.attr("A1563"));
                if !known {
                    b.report.approximated(format!(
                        "propriedade EPLAN {id} sem equivalente: mantida como '{}'",
                        field.label()
                    ));
                }
                let index = num(c.attr("A1561")).map(|v| v as u32);
                b.special(at, field, index, style(c));
            }
            89 => match (point(c.attr("A3319")), point(c.attr("A3321"))) {
                (Some(a), Some(z)) => b.rect(a, z, width(c)),
                _ => b.report.ignored("retângulo EPLAN sem cantos"),
            },
            91 => {
                let path = c.attr("A3359").unwrap_or("").to_owned();
                match (point(c.attr("A3319")), point(c.attr("A3321"))) {
                    (Some(a), Some(z)) => match resolve(&path) {
                        Some(bytes) => b.image(a, z, bytes, &path),
                        None => b
                            .report
                            .ignored(format!("imagem externa '{path}' não encontrada")),
                    },
                    _ => b.report.ignored("imagem EPLAN sem retângulo"),
                }
            }
            76 => b
                .report
                .ignored("configuração de exibição EPLAN (sem desenho)"),
            212 => b.report.ignored("campo de usuário EPLAN"),
            other => b.report.ignored(format!("objeto EPLAN tipo {other}")),
        }
    }
}

/// Converte o `.fn1` em moldura. `resolve` recebe o caminho gravado pelo
/// EPLAN (ex.: `$(MD_IMG)\logo.jpg`) e devolve os bytes, se encontrar.
pub fn import(bytes: &[u8], resolve: &mut ImageResolver<'_>) -> ImportResult<ImportedFrame> {
    check_size(bytes)?;
    let root = xml::parse(bytes)?;
    if root.name != "EplanPxfRoot" {
        return Err(fail("o arquivo não é um XML PXF do EPLAN"));
    }
    if root.attr("Type") != Some("Frame") {
        return Err(fail("o arquivo PXF não é uma moldura (Type=Frame)"));
    }
    let frame = root
        .children
        .iter()
        .find(|c| kind(c) == Some(21))
        .ok_or_else(|| fail("moldura EPLAN sem objeto de moldura"))?;
    let mut b = Builder::default();
    for c in root
        .children
        .iter()
        .filter(|c| kind(c).is_some_and(|k| k != 21))
    {
        match kind(c) {
            Some(76) => b
                .report
                .ignored("configuração de exibição EPLAN (sem desenho)"),
            Some(k) => b
                .report
                .ignored(format!("objeto EPLAN tipo {k} fora da moldura")),
            None => {}
        }
    }
    let mut labels = Labels {
        columns: vec![],
        rows: vec![],
    };
    walk(frame, &mut b, &mut labels, resolve);

    let props = frame.child("P39");
    let prop = |k: &str| props.and_then(|p| p.attr(k));
    let sheet = match (num(prop("P12033")), num(prop("P12034"))) {
        (Some(w), Some(h)) if w > 0.0 && h > 0.0 => Some((w, h)),
        _ => None,
    };
    // O PXF guarda desenho em coordenadas relativas ao centro/offset interno.
    // A prévia SVG aplica uma matriz de translação para colocá-lo na folha.
    // A borda externa (quando tem exatamente as dimensões da folha) fornece
    // essa origem sem depender do nome variável dos objetos O<n>.
    if let Some((sw, sh)) = sheet {
        let border = b.elements.iter().find_map(|e| match e {
            FrameElement::Rectangle { rect, .. }
                if (rect.size().width() - sw).abs() <= 0.5
                    && (rect.size().height() - sh).abs() <= 0.5 =>
            {
                Some(rect.origin())
            }
            _ => None,
        });
        if let Some(origin) = border {
            let (dx, dy) = (-origin.x(), -origin.y());
            if dx.abs() > 0.001 || dy.abs() > 0.001 {
                b.elements = b
                    .elements
                    .iter()
                    .map(|e| e.translated(dx, dy))
                    .collect::<crate::error::Result<_>>()
                    .map_err(|e| fail(format!("origem EPLAN inválida: {e}")))?;
                for point in labels.columns.iter_mut().chain(labels.rows.iter_mut()) {
                    point.0 += dx;
                    point.1 += dy;
                }
                b.report
                    .info(format!("origem EPLAN ajustada em ({dx:.3}; {dy:.3}) mm"));
            }
        }
    }
    let description = prop("P18011")
        .map(|d| localized(d).default)
        .unwrap_or_default();
    let columns = list(frame.attr("A539"));
    let rows = list(frame.attr("A541"));
    let origin = point(frame.attr("A543")).unwrap_or((0.0, 0.0));
    let usable = match (columns.is_empty(), rows.is_empty()) {
        (false, false) => Some((origin.0, origin.1, columns.iter().sum(), rows.iter().sum())),
        _ => None,
    };
    let center = usable.map(|(x, y, w, h)| (x + w / 2.0, y + h / 2.0));
    let axis = |sizes: Vec<f64>,
                style: ZoneLabelStyle,
                found: &[(f64, f64)],
                coord: fn(&(f64, f64)) -> f64,
                mid: Option<f64>,
                start_is_high: bool| {
        if sizes.len() < 2 {
            return ZoneAxis::none();
        }
        let mut a = ZoneAxis {
            sizes,
            style,
            labels_start: false,
            labels_end: false,
        };
        match mid {
            Some(m) if !found.is_empty() => {
                let high = found.iter().any(|p| coord(p) > m);
                let low = found.iter().any(|p| coord(p) < m);
                (a.labels_start, a.labels_end) = if start_is_high {
                    (high, low)
                } else {
                    (low, high)
                };
            }
            _ => a.labels_start = true,
        }
        a
    };
    let zones = ZoneLayout {
        // Colunas: início = acima (y alto); linhas: início = à esquerda.
        columns: axis(
            columns,
            ZoneLabelStyle::Numbers { start: 1 },
            &labels.columns,
            |p| p.1,
            center.map(|c| c.1),
            true,
        ),
        rows: axis(
            rows,
            ZoneLabelStyle::Letters { start: 0 },
            &labels.rows,
            |p| p.0,
            center.map(|c| c.0),
            false,
        ),
    };
    if !labels.columns.is_empty() || !labels.rows.is_empty() {
        b.report.info(format!(
            "{} rótulos de coluna/linha do EPLAN substituídos pelos rótulos da divisão",
            labels.columns.len() + labels.rows.len()
        ));
    }
    let name = root
        .attr("Name")
        .or(frame.attr("A1821"))
        .unwrap_or("Moldura EPLAN");
    b.finish(name, &description, sheet, usable, zones, false)
}
