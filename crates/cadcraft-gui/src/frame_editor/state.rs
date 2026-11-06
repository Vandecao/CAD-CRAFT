// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! state support for the frame editor; start at the parent module.

use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FrameTool {
    #[default]
    Select,
    Line,
    Rectangle,
    Circle,
    Arc,
    Text,
    Special,
    Image,
}
impl FrameTool {
    pub const ALL: [Self; 8] = [
        Self::Select,
        Self::Line,
        Self::Rectangle,
        Self::Circle,
        Self::Arc,
        Self::Text,
        Self::Special,
        Self::Image,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Select => "Selecionar",
            Self::Line => "Linha",
            Self::Rectangle => "Retângulo",
            Self::Circle => "Círculo",
            Self::Arc => "Arco",
            Self::Text => "Texto",
            Self::Special => "Texto especial",
            Self::Image => "Imagem",
        }
    }
    /// Number of clicks required to create the element.
    pub(in crate::frame_editor) fn clicks(self) -> usize {
        match self {
            Self::Select => 0,
            Self::Text | Self::Special | Self::Image => 1,
            Self::Line | Self::Rectangle | Self::Circle => 2,
            Self::Arc => 3,
        }
    }
    pub(in crate::frame_editor) fn hint(self) -> &'static str {
        match self {
            Self::Select => {
                "Clique seleciona (Ctrl acumula); arraste move; Delete exclui; Ctrl+D duplica."
            }
            Self::Line => "Clique no início e no fim da linha.",
            Self::Rectangle => "Clique em dois cantos opostos.",
            Self::Circle => "Clique no centro e depois num ponto do contorno.",
            Self::Arc => "Clique no centro, no início e no fim (sentido anti-horário).",
            Self::Text => "Clique para inserir; edite o texto nas propriedades.",
            Self::Special => "Clique para inserir; escolha o campo nas propriedades.",
            Self::Image => "Clique para posicionar a imagem escolhida (40 mm de largura).",
        }
    }
}

pub(in crate::frame_editor) const FIELDS: [&str; 16] = [
    "Nome do projeto",
    "Descrição do projeto",
    "Identificação do projeto",
    "Propriedade do projeto",
    "Documento",
    "Número da página",
    "Descrição da página",
    "Função (=)",
    "Localização (+)",
    "Posição da página",
    "Total de páginas do documento",
    "Total de páginas do projeto",
    "Página anterior",
    "Página seguinte",
    "Propriedade da página",
    "Nome da moldura",
];
pub(in crate::frame_editor) fn field_index(field: &SpecialField) -> (usize, String) {
    use SpecialField as F;
    match field {
        F::ProjectName => (0, String::new()),
        F::ProjectDescription => (1, String::new()),
        F::ProjectIdentification => (2, String::new()),
        F::ProjectProperty(k) => (3, k.as_str().to_owned()),
        F::DocumentName => (4, String::new()),
        F::PageNumber => (5, String::new()),
        F::PageName => (6, String::new()),
        F::PageFunctionalPath => (7, String::new()),
        F::PageLocationPath => (8, String::new()),
        F::PagePosition => (9, String::new()),
        F::PageCountInDocument => (10, String::new()),
        F::PageCountInProject => (11, String::new()),
        F::PreviousPageNumber => (12, String::new()),
        F::NextPageNumber => (13, String::new()),
        F::PageProperty(k) => (14, k.as_str().to_owned()),
        F::FrameName => (15, String::new()),
    }
}
pub(in crate::frame_editor) fn field_from(index: usize, key: &str) -> Result<SpecialField, String> {
    use SpecialField as F;
    let key = || Text::new(key.trim()).map_err(|_| "informe a chave da propriedade".to_owned());
    Ok(match index {
        0 => F::ProjectName,
        1 => F::ProjectDescription,
        2 => F::ProjectIdentification,
        3 => F::ProjectProperty(key()?),
        4 => F::DocumentName,
        5 => F::PageNumber,
        6 => F::PageName,
        7 => F::PageFunctionalPath,
        8 => F::PageLocationPath,
        9 => F::PagePosition,
        10 => F::PageCountInDocument,
        11 => F::PageCountInProject,
        12 => F::PreviousPageNumber,
        13 => F::NextPageNumber,
        14 => F::PageProperty(key()?),
        _ => F::FrameName,
    })
}
pub(in crate::frame_editor) const ALIGNS: [(TextAlign, &str); 9] = [
    (TextAlign::BottomLeft, "Inferior esquerdo"),
    (TextAlign::BottomCenter, "Inferior centro"),
    (TextAlign::BottomRight, "Inferior direito"),
    (TextAlign::MiddleLeft, "Meio esquerdo"),
    (TextAlign::Center, "Centro"),
    (TextAlign::MiddleRight, "Meio direito"),
    (TextAlign::TopLeft, "Superior esquerdo"),
    (TextAlign::TopCenter, "Superior centro"),
    (TextAlign::TopRight, "Superior direito"),
];

/// Selected-element form with values in millimetres and degrees.
#[derive(Clone, Debug, Default)]
pub(in crate::frame_editor) struct ElementForm {
    pub(in crate::frame_editor) nums: [f64; 6],
    pub(in crate::frame_editor) text: String,
    pub(in crate::frame_editor) translations: String,
    pub(in crate::frame_editor) field: usize,
    pub(in crate::frame_editor) key: String,
    pub(in crate::frame_editor) index: u32,
    pub(in crate::frame_editor) style: Option<FrameTextStyle>,
    pub(in crate::frame_editor) keep_aspect: bool,
}
/// Frame form.
#[derive(Clone, Debug, Default)]
pub(in crate::frame_editor) struct FrameForm {
    pub(in crate::frame_editor) name: String,
    pub(in crate::frame_editor) description: String,
    pub(in crate::frame_editor) sheet: [f64; 2],
    pub(in crate::frame_editor) usable: [f64; 4],
    pub(in crate::frame_editor) axes: [AxisForm; 2],
}
#[derive(Clone, Debug, Default)]
pub(in crate::frame_editor) struct AxisForm {
    pub(in crate::frame_editor) count: usize,
    pub(in crate::frame_editor) size: f64,
    pub(in crate::frame_editor) custom: String,
    pub(in crate::frame_editor) letters: bool,
    pub(in crate::frame_editor) start: u32,
    pub(in crate::frame_editor) labels_start: bool,
    pub(in crate::frame_editor) labels_end: bool,
}
impl AxisForm {
    pub(in crate::frame_editor) fn from(axis: &ZoneAxis) -> Self {
        let uniform = axis.sizes.windows(2).all(|w| (w[0] - w[1]).abs() < 1e-9);
        let (letters, start) = match axis.style {
            ZoneLabelStyle::Numbers { start } => (false, start),
            ZoneLabelStyle::Letters { start } => (true, start),
        };
        Self {
            count: axis.sizes.len(),
            size: axis.sizes.first().copied().unwrap_or(40.0),
            custom: if uniform {
                String::new()
            } else {
                axis.sizes
                    .iter()
                    .map(|s| format!("{s}"))
                    .collect::<Vec<_>>()
                    .join("; ")
            },
            letters,
            start,
            labels_start: axis.labels_start,
            labels_end: axis.labels_end,
        }
    }
    pub(in crate::frame_editor) fn build(&self) -> Result<ZoneAxis, String> {
        let sizes = if self.custom.trim().is_empty() {
            vec![self.size; self.count]
        } else {
            self.custom
                .split(';')
                .map(|s| s.trim().replace(',', ".").parse::<f64>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| "tamanhos individuais: números separados por ;".to_owned())?
        };
        Ok(ZoneAxis {
            sizes,
            style: if self.letters {
                ZoneLabelStyle::Letters { start: self.start }
            } else {
                ZoneLabelStyle::Numbers { start: self.start }
            },
            labels_start: self.labels_start,
            labels_end: self.labels_end,
        })
    }
}

#[derive(Default)]
pub struct FrameEditor {
    /// Open frame; None means the editor is closed.
    pub frame: Option<FrameTemplateId>,
    pub camera: Camera,
    pub needs_fit: bool,
    pub tool: FrameTool,
    pub selection: BTreeSet<usize>,
    /// Drawing area of the latest frame, for tests and automation.
    pub viewport: Option<Rect>,
    pub(in crate::frame_editor) pending: Vec<Point2D>,
    pub(in crate::frame_editor) drag: Option<(Point2D, Point2D)>,
    pub(in crate::frame_editor) pending_image: Option<ImageAsset>,
    pub(in crate::frame_editor) element_form: ElementForm,
    pub(in crate::frame_editor) frame_form: FrameForm,
    pub(in crate::frame_editor) form_key: Option<(FrameTemplateId, u32, Vec<usize>)>,
}
impl FrameEditor {
    pub fn is_open(&self) -> bool {
        self.frame.is_some()
    }
    /// Cancel the tool and pending points; return true if anything was canceled.
    pub fn cancel(&mut self) -> bool {
        let busy =
            !self.pending.is_empty() || self.drag.is_some() || self.tool != FrameTool::Select;
        self.pending.clear();
        self.drag = None;
        self.pending_image = None;
        self.tool = FrameTool::Select;
        busy
    }
}
