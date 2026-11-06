// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! input support for the frame editor; start at the parent module.

use super::*;

impl CadApp {
    pub fn open_frame_editor(&mut self, id: FrameTemplateId) {
        self.symbol_editor = crate::symbol_editor::SymbolEditor::default();
        self.frame_editor = FrameEditor {
            frame: Some(id),
            needs_fit: true,
            ..Default::default()
        };
        self.dialogs_state.backstage = None;
    }
    pub fn close_frame_editor(&mut self) {
        self.frame_editor = FrameEditor::default();
    }
    pub(in crate::frame_editor) fn editor_frame(&self) -> Option<FrameTemplate> {
        let id = self.frame_editor.frame?;
        self.session.project()?.frame(id).cloned()
    }
    /// Confirma o rascunho como nova versão; `extra` vem antes (ex.: imagem).
    pub fn commit_frame_draft(&mut self, draft: FrameDraft, extra: Vec<FrameEdit>) -> bool {
        let Some(frame) = self.editor_frame() else {
            return false;
        };
        match frame.edited(draft) {
            Ok(edited) => {
                let version = edited.version();
                let mut edits = extra;
                edits.push(FrameEdit::UpsertFrame(edited));
                let edit = if edits.len() == 1 {
                    edits.pop().expect("one edit")
                } else {
                    FrameEdit::Batch(edits)
                };
                self.graphic_edit(GraphicEdit::Frame(edit));
                self.editor_frame().is_some_and(|f| f.version() == version)
            }
            Err(e) => {
                self.log(Level::Error, e.to_string());
                false
            }
        }
    }
    pub fn set_frame_tool(&mut self, tool: FrameTool) {
        self.frame_editor.pending.clear();
        self.frame_editor.drag = None;
        self.frame_editor.tool = tool;
        if tool == FrameTool::Image {
            self.frame_editor.pending_image = None;
            match self.files.picker.pick_image() {
                Some(path) => match load_image_file(&path) {
                    Ok(asset) => self.frame_editor.pending_image = Some(asset),
                    Err(e) => {
                        self.log(Level::Error, format!("{}: {e}", path.display()));
                        self.frame_editor.tool = FrameTool::Select;
                    }
                },
                None => self.frame_editor.tool = FrameTool::Select,
            }
        }
    }
    pub(in crate::frame_editor) fn editor_delete_selection(&mut self) {
        let Some(frame) = self.editor_frame() else {
            return;
        };
        if self.frame_editor.selection.is_empty() {
            return;
        }
        let mut draft = frame.draft();
        let selected = std::mem::take(&mut self.frame_editor.selection);
        draft.elements = draft
            .elements
            .into_iter()
            .enumerate()
            .filter(|(i, _)| !selected.contains(i))
            .map(|(_, e)| e)
            .collect();
        self.commit_frame_draft(draft, vec![]);
    }
    pub(in crate::frame_editor) fn editor_transform_selection(
        &mut self,
        duplicate: bool,
        dx: f64,
        dy: f64,
        align: bool,
    ) {
        let Some(frame) = self.editor_frame() else {
            return;
        };
        let mut draft = frame.draft();
        let mut added = BTreeSet::new();
        let grid = self.preferences.grid_mm;
        for &i in &self.frame_editor.selection {
            let Some(e) = draft.elements.get(i).cloned() else {
                continue;
            };
            let (mx, my) = if align {
                let a = e.anchor();
                let s = snap(a, grid, true);
                (s.x() - a.x(), s.y() - a.y())
            } else {
                (dx, dy)
            };
            let Ok(moved) = e.translated(mx, my) else {
                continue;
            };
            if duplicate {
                added.insert(draft.elements.len());
                draft.elements.push(moved);
            } else {
                draft.elements[i] = moved;
            }
        }
        if self.commit_frame_draft(draft, vec![]) && duplicate {
            self.frame_editor.selection = added;
        }
    }
}
impl CadApp {
    pub(in crate::frame_editor) fn hit(&self, frame: &FrameTemplate, p: Point2D) -> Option<usize> {
        let project = self.session.project()?;
        let tolerance = (6.0 / self.frame_editor.camera.scale()).max(0.5);
        frame
            .elements()
            .iter()
            .enumerate()
            .map(|(i, e)| (i, element_distance(project, e, p)))
            .filter(|(_, d)| *d <= tolerance)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)
    }

    /// Elemento da ferramenta atual com os pontos dados (prévia aceita menos).
    pub(in crate::frame_editor) fn build_tool_element(
        &self,
        points: &[Point2D],
        preview: bool,
    ) -> Option<FrameElement> {
        let tool = self.frame_editor.tool;
        if points.is_empty() || (!preview && points.len() < tool.clicks()) {
            return None;
        }
        let a = points[0];
        let b = *points.get(1).unwrap_or(&a);
        Some(match tool {
            FrameTool::Select => return None,
            FrameTool::Line if points.len() >= 2 && a != b => FrameElement::Line {
                start: a,
                end: b,
                width: 0.25,
            },
            FrameTool::Rectangle if points.len() >= 2 => {
                let (x0, y0) = (a.x().min(b.x()), a.y().min(b.y()));
                let (w, h) = ((a.x() - b.x()).abs(), (a.y() - b.y()).abs());
                FrameElement::Rectangle {
                    rect: Rect2D::new(pt(x0, y0)?, Size2D::new(w, h).ok()?).ok()?,
                    width: 0.35,
                }
            }
            FrameTool::Circle if points.len() >= 2 => FrameElement::Circle {
                center: a,
                radius: a.distance_to(b).ok()?,
                width: 0.25,
            },
            FrameTool::Arc if points.len() >= 3 => {
                let c = points[2];
                let angle = |p: Point2D| (p.y() - a.y()).atan2(p.x() - a.x()).to_degrees();
                let start = angle(b);
                let mut sweep = angle(c) - start;
                if sweep <= 0.0 {
                    sweep += 360.0;
                }
                FrameElement::Arc {
                    center: a,
                    radius: a.distance_to(b).ok()?,
                    start_degrees: start,
                    sweep_degrees: sweep,
                    width: 0.25,
                }
            }
            FrameTool::Text => FrameElement::Text {
                origin: a,
                text: LocalizedText::new("Texto"),
                style: FrameTextStyle::new(3.5, TextAlign::BottomLeft),
            },
            FrameTool::Special => FrameElement::Special {
                origin: a,
                field: SpecialField::ProjectName,
                index: None,
                style: FrameTextStyle::new(3.0, TextAlign::BottomLeft),
            },
            FrameTool::Image => {
                let asset = self.frame_editor.pending_image.as_ref()?;
                let w = 40.0;
                FrameElement::Image {
                    rect: Rect2D::new(a, Size2D::new(w, w / asset.aspect()).ok()?).ok()?,
                    asset: asset.id(),
                }
            }
            _ => return None,
        })
    }

    pub(in crate::frame_editor) fn place_tool_element(&mut self, points: &[Point2D]) {
        let Some(frame) = self.editor_frame() else {
            return;
        };
        let Some(mut element) = self.build_tool_element(points, false) else {
            self.log(
                Level::Warning,
                "Elemento degenerado; tente pontos diferentes.",
            );
            return;
        };
        let mut extra = vec![];
        if let FrameElement::Image { asset, .. } = &mut element
            && let Some(image) = self.frame_editor.pending_image.clone()
        {
            let bytes = image.bytes().unwrap_or_default();
            match self.session.project().and_then(|p| {
                p.assets()
                    .find(|a| a.same_content(bytes))
                    .map(ImageAsset::id)
            }) {
                // Imagem igual já está no projeto: reaproveita.
                Some(existing) => *asset = existing,
                None => extra.push(FrameEdit::AddAsset(image)),
            }
        }
        let mut draft = frame.draft();
        draft.elements.push(element);
        let index = draft.elements.len() - 1;
        if self.commit_frame_draft(draft, extra) {
            self.frame_editor.selection = BTreeSet::from([index]);
            if self.frame_editor.tool == FrameTool::Image {
                // A mesma imagem pode ser colocada de novo; o projeto já a tem.
                if let Some(id) = self.session.project().and_then(|p| {
                    let bytes = self.frame_editor.pending_image.as_ref()?.bytes()?;
                    p.assets()
                        .find(|a| a.same_content(bytes))
                        .map(ImageAsset::id)
                }) && let Some(img) = &mut self.frame_editor.pending_image
                {
                    *img = img.clone().with_id(id);
                }
            }
        }
    }

    // -------------------------------------------------------- propriedades
}
impl CadApp {
    pub(crate) fn frame_command(&mut self, id: crate::commands::CommandId) {
        use crate::commands::CommandId as C;
        let Some(project) = self.session.project() else {
            return;
        };
        match id {
            C::FrameEditor => {
                let target = self
                    .session
                    .active
                    .and_then(|page| project.page_frame(page))
                    .map(FrameTemplate::id)
                    .or(project.default_frame())
                    .or(project.frames().next().map(FrameTemplate::id));
                match target {
                    Some(id) => self.open_frame_editor(id),
                    None => self.log(Level::Warning, "Projeto sem molduras: use Nova moldura."),
                }
            }
            C::FrameNew => {
                let size = self
                    .session
                    .active
                    .and_then(|p| project.page(p))
                    .map(Page::size)
                    .unwrap_or_else(|| PaperFormat::A3.size(true));
                let name = format!("Moldura {}", project.frames().count() + 1);
                match FrameTemplate::blank(name, size) {
                    Ok(frame) => {
                        let id = frame.id();
                        self.graphic_edit(GraphicEdit::Frame(FrameEdit::UpsertFrame(frame)));
                        if self
                            .session
                            .project()
                            .is_some_and(|p| p.frame(id).is_some())
                        {
                            self.open_frame_editor(id);
                        }
                    }
                    Err(e) => self.log(Level::Error, e.to_string()),
                }
            }
            C::FrameDuplicate => {
                let Some(frame) = self.editor_frame() else {
                    self.log(Level::Warning, "Abra uma moldura no editor para duplicar.");
                    return;
                };
                match frame.duplicate(format!("{} (cópia)", frame.name().as_str())) {
                    Ok(copy) => {
                        let id = copy.id();
                        self.graphic_edit(GraphicEdit::Frame(FrameEdit::UpsertFrame(copy)));
                        self.open_frame_editor(id);
                    }
                    Err(e) => self.log(Level::Error, e.to_string()),
                }
            }
            C::FrameDelete => {
                let Some(id) = self.frame_editor.frame else {
                    self.log(Level::Warning, "Abra uma moldura no editor para excluir.");
                    return;
                };
                self.graphic_edit(GraphicEdit::Frame(FrameEdit::RemoveFrame(id)));
                if self
                    .session
                    .project()
                    .is_some_and(|p| p.frame(id).is_none())
                {
                    self.close_frame_editor();
                }
            }
            C::FrameExport => {
                let Some(id) = self.frame_editor.frame else {
                    self.log(Level::Warning, "Abra uma moldura no editor para exportar.");
                    return;
                };
                let result = project
                    .frame_package(id)
                    .map_err(GuiError::from)
                    .and_then(|pkg| {
                        cad_craft::persistence::write_frame_package(&pkg)
                            .map(|b| (b, pkg.frame.name().as_str().to_owned()))
                            .map_err(GuiError::from)
                    });
                match result {
                    Ok((bytes, name)) => {
                        let library = self.frame_library();
                        if let Some(path) = self.files.picker.save_frame(&name, library.as_deref())
                        {
                            let path = with_frame_extension(path);
                            match crate::project_file::write_atomic(&path, &mut bytes.as_slice()) {
                                Ok(()) => self.log(
                                    Level::Info,
                                    format!("Moldura exportada em {}.", path.display()),
                                ),
                                Err(e) => {
                                    self.log(Level::Error, format!("{}: {e}", path.display()))
                                }
                            }
                        }
                    }
                    Err(e) => self.log(Level::Error, e.to_string()),
                }
            }
            C::FrameImport => {
                let library = self.frame_library();
                let Some(path) = self.files.picker.open_frame(library.as_deref()) else {
                    return;
                };
                let result = read_frame_file(&path).and_then(|pkg| {
                    // As imagens do pacote também precisam decodificar.
                    for a in &pkg.assets {
                        frame_view::decode_image(a.bytes().unwrap_or_default())?;
                    }
                    project.frame_import_edits(&pkg).map_err(GuiError::from)
                });
                match result {
                    Ok((edits, id)) => {
                        self.graphic_edit(GraphicEdit::Frame(FrameEdit::Batch(edits)));
                        if self
                            .session
                            .project()
                            .is_some_and(|p| p.frame(id).is_some())
                        {
                            self.log(
                                Level::Info,
                                format!("Moldura importada de {}.", path.display()),
                            );
                            self.open_frame_editor(id);
                        }
                    }
                    Err(e) => self.log(Level::Error, format!("{}: {e}", path.display())),
                }
            }
            C::FrameImportExternal => {
                let library = self.frame_library();
                let Some(path) = self.files.picker.open_external_frame(library.as_deref()) else {
                    return;
                };
                let result = read_external_frame(
                    &path,
                    self.session
                        .active
                        .and_then(|id| project.page(id))
                        .map(Page::size),
                    None,
                    None,
                );
                match result {
                    Ok(imported) => {
                        self.dialogs_state.dialog = Some(crate::dialogs::Dialog::ExternalFrame {
                            path,
                            imported: Box::new(imported),
                            page: self.session.active,
                            dxf_unit_mm: None,
                            dxf_sheet: None,
                            preview_dirty: false,
                        })
                    }
                    Err(e) => self.log(Level::Error, format!("{}: {e}", path.display())),
                }
            }
            C::FrameExportDxf => {
                let Some(frame) = self.editor_frame() else {
                    self.log(
                        Level::Warning,
                        "Abra uma moldura no editor para exportar DXF.",
                    );
                    return;
                };
                let (bytes, report) = cad_craft::interchange::dxf::export(&frame);
                let library = self.frame_library();
                if let Some(path) = self
                    .files
                    .picker
                    .save_dxf(frame.name().as_str(), library.as_deref())
                {
                    let path = if path
                        .extension()
                        .is_some_and(|e| e.eq_ignore_ascii_case("dxf"))
                    {
                        path
                    } else {
                        path.with_extension("dxf")
                    };
                    match crate::project_file::write_atomic(&path, &mut bytes.as_slice()) {
                        Ok(()) => {
                            self.log(Level::Info, format!("DXF exportado em {}.", path.display()));
                            for line in report.lines() {
                                self.log(Level::Warning, line);
                            }
                        }
                        Err(e) => self.log(Level::Error, format!("{}: {e}", path.display())),
                    }
                }
            }
            _ => {}
        }
    }
    pub(in crate::frame_editor) fn frame_library(&self) -> Option<std::path::PathBuf> {
        let dir = self.preferences.frame_library.trim();
        (!dir.is_empty()).then(|| std::path::PathBuf::from(dir))
    }
}
