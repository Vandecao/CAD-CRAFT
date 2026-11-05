// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use crate::{
    app::{CadApp, Level},
    commands::CommandId,
    editing::{Edit, GraphicEdit},
    navigation::{kind_label, page_kinds, path_text},
    preferences::PanelId,
    state::{Selection, Session},
    theme,
};
use cad_craft::domain::*;
use eframe::egui::{self, Ui};

#[derive(Default)]
enum Draft {
    #[default]
    Empty,
    Project {
        description: String,
        identification: String,
        meta: Vec<(String, String)>,
    },
    Page {
        id: PageId,
        name: String,
        number: String,
        kind: PageKind,
        width: f64,
        height: f64,
        function: String,
        location: String,
        meta: Vec<(String, String)>,
    },
    Representation {
        id: RepresentationId,
        page: PageId,
        x: f64,
        y: f64,
        angle: f64,
        mx: bool,
        my: bool,
        visible: bool,
        layer: i32,
        label: String,
    },
}
#[derive(Default)]
pub(crate) struct Inspector {
    selection: Selection,
    revision: Option<u64>,
    selection_revision: u64,
    draft: Draft,
    query: String,
    grouped: bool,
    profile: usize,
    multi_ids: Vec<RepresentationId>,
    displacement: [f64; 2],
}
impl Inspector {
    fn sync(&mut self, s: &Session) {
        if self.selection == s.selection
            && self.revision == Some(s.revision)
            && self.selection_revision == s.selection_revision
        {
            return;
        }
        self.displacement = [0.0; 2];
        self.multi_ids = s.graphics.selection.ids().iter().copied().collect();
        self.selection_revision = s.selection_revision;
        self.selection = s.selection;
        self.revision = Some(s.revision);
        self.draft = match (s.project(), s.selection) {
            (Some(p), Selection::Project) => Draft::Project {
                description: p.description().into(),
                identification: p.identification().map(|s| s.as_str()).unwrap_or("").into(),
                meta: metadata_rows(p.metadata(), PROJECT_KEYS),
            },
            (Some(p), Selection::Page(id)) => p
                .page(id)
                .map(|p| Draft::Page {
                    id,
                    name: p.name().as_str().into(),
                    number: p.number().as_str().into(),
                    kind: p.kind().clone(),
                    width: p.size().width(),
                    height: p.size().height(),
                    function: path_text(p.structure().functional_path()),
                    location: path_text(p.structure().location_path()),
                    meta: metadata_rows(p.metadata(), PAGE_KEYS),
                })
                .unwrap_or_default(),
            (Some(p), Selection::Representation(id)) => p
                .representation(id)
                .map(|r| {
                    let t = r.transform();
                    Draft::Representation {
                        id,
                        page: r.page_id(),
                        x: t.translation().x(),
                        y: t.translation().y(),
                        angle: t.rotation_degrees(),
                        mx: t.mirror_x(),
                        my: t.mirror_y(),
                        visible: r.visual().visible,
                        layer: r.visual().layer,
                        label: r
                            .visual()
                            .label
                            .as_ref()
                            .map(Text::as_str)
                            .unwrap_or("")
                            .into(),
                    }
                })
                .unwrap_or_default(),
            _ => Draft::Empty,
        };
    }
}
fn text_row(ui: &mut Ui, query: &str, label: &str, value: &mut String) -> bool {
    if !label.to_lowercase().contains(query) && !value.to_lowercase().contains(query) {
        return false;
    }
    ui.label(label);
    let response = ui.add(egui::TextEdit::singleline(value).desired_width(120.0));
    ui.end_row();
    response.lost_focus() || (response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
}
fn number_row(ui: &mut Ui, query: &str, label: &str, value: &mut f64) -> bool {
    if !label.to_lowercase().contains(query) {
        return false;
    }
    ui.label(label);
    let changed = ui
        .add(egui::DragValue::new(value).speed(0.1).max_decimals(3))
        .changed();
    ui.end_row();
    changed
}
fn read_row(ui: &mut Ui, query: &str, label: &str, value: impl ToString) {
    let value = value.to_string();
    if !label.to_lowercase().contains(query) && !value.to_lowercase().contains(query) {
        return;
    }
    ui.label(egui::RichText::new(label).color(theme::MUTED));
    ui.add(egui::Label::new(value).wrap().selectable(true));
    ui.end_row();
}
fn bool_row(ui: &mut Ui, query: &str, label: &str, value: &mut bool) -> bool {
    if !label.to_lowercase().contains(query) {
        return false;
    }
    ui.label(label);
    let changed = ui.checkbox(value, "").changed();
    ui.end_row();
    changed
}
fn table(ui: &mut Ui, id: &str, show: impl FnOnce(&mut Ui)) {
    egui::Grid::new(id)
        .num_columns(2)
        .striped(true)
        .spacing([7.0, 5.0])
        .max_col_width((ui.available_width() / 2.0).max(45.0))
        .show(ui, show);
}

impl CadApp {
    pub(crate) fn show_properties(&mut self, ui: &mut Ui) {
        if !super::header(
            ui,
            &mut self.preferences,
            PanelId::Properties,
            "Visão geral de propriedades",
        ) {
            return;
        }
        if self.show_connection_properties(ui) {
            return;
        }
        if self.show_reference_properties(ui) {
            return;
        }
        let mut inspector = std::mem::take(&mut self.inspector);
        inspector.sync(&self.session);
        if self.session.graphics.selection.ids().len() > 1 {
            self.show_group_properties(ui, &mut inspector);
            self.inspector = inspector;
            return;
        }
        ui.horizontal(|ui| {
            ui.selectable_value(&mut inspector.grouped, true, "Categorias");
            ui.selectable_value(&mut inspector.grouped, false, "Tabela");
        });
        egui::ComboBox::from_id_salt("property_profile")
            .selected_text(
                [
                    "Todas as propriedades",
                    "Identificação",
                    "Dados documentais",
                    "Entidade elétrica",
                ][inspector.profile],
            )
            .show_ui(ui, |ui| {
                for (i, name) in [
                    "Todas as propriedades",
                    "Identificação",
                    "Dados documentais",
                    "Entidade elétrica",
                ]
                .iter()
                .enumerate()
                {
                    ui.selectable_value(&mut inspector.profile, i, *name);
                }
            });
        ui.add(
            egui::TextEdit::singleline(&mut inspector.query)
                .hint_text("Procurar propriedade")
                .desired_width(f32::INFINITY),
        );
        ui.horizontal(|ui| {
            ui.strong("Propriedade");
            ui.separator();
            ui.strong("Valor");
        });
        ui.separator();
        let query = inspector.query.to_lowercase();
        let mut edits = Vec::new();
        let mut frame_edits: Vec<FrameEdit> = Vec::new();
        let mut error = None;
        let mut related = None;
        let mut drawing_edit = None;
        egui::ScrollArea::vertical().auto_shrink([false,false]).show(ui,|ui| {
            let Some(project)=self.session.project()else{ui.weak("Nenhum projeto aberto.");return;};
            if let Selection::Representation(id)=self.session.selection {drawing_edit=crate::connection_properties::port_controls(ui,project,id);}
            if self.session.selection==Selection::None {ui.weak("Selecione um projeto, documento, página ou representação.");return;}
            let show_identity=inspector.profile==0||inspector.profile==1;
            if show_identity {let content=|ui:&mut Ui| {table(ui,"property_identity",|ui|{
                match self.session.selection {
                    Selection::Project=>{read_row(ui,&query,"Nome",project.name().as_str());read_row(ui,&query,"ProjectId",project.id());},
                    Selection::Document(id)=>{if let Some(doc)=project.document(id){read_row(ui,&query,"Documento",doc.name().as_str());read_row(ui,&query,"DocumentId",id);read_row(ui,&query,"Páginas",project.pages_in_document(id).count());}},
                    Selection::Page(id)=>{read_row(ui,&query,"PageId",id);if let Some(page)=project.page(id){read_row(ui,&query,"Orientação",format!("{:?}",page.orientation()));read_row(ui,&query,"Ordem",page.order().map(|n|n.to_string()).unwrap_or_else(||"Não definida".into()));}},
                    Selection::Representation(id)=>{read_row(ui,&query,"RepresentationId",id);if let Some(rep)=project.representation(id){read_row(ui,&query,"PageId",rep.page_id());read_row(ui,&query,"Identificação",rep.visual().label.as_ref().map(Text::as_str).unwrap_or("Sem rótulo"));read_row(ui,&query,"Alvo",if rep.target()==RepresentationTarget::Unassociated {"Sem associação elétrica".into()}else{format!("{:?}",rep.target())});if let Some(d)=rep.symbol().and_then(|id|project.symbol_definition(id)){symbol_rows(ui,&query,d,rep.transform());}else{read_row(ui,&query,"Geometria",if rep.local_bounds().is_some(){"Limites documentais (não é símbolo)"}else{"Somente âncora conhecida"});}}},
                    Selection::Electrical(target)=>read_row(ui,&query,"Alvo elétrico",format!("{target:?}")),Selection::None=>{}
                }
            });};
                if inspector.grouped {egui::CollapsingHeader::new("Identificação").default_open(true).show(ui,content);}else{content(ui);}
            }
            if inspector.profile==0||inspector.profile==2 {let mut content=|ui:&mut Ui| {table(ui,"property_edits",|ui|{
                match &mut inspector.draft {
                    Draft::Project{description,identification,meta}=>{
                        if text_row(ui,&query,"Descrição",description)&&description!=project.description(){edits.push(Edit::ProjectDescription(description.clone()));}
                        if text_row(ui,&query,"Identificação",identification)&&identification!=project.identification().map(Text::as_str).unwrap_or(""){edits.push(Edit::ProjectIdentification(identification.clone()));}
                        frame_rows(ui,&query,project,None,&mut frame_edits);
                        for (key,value) in meta.iter_mut(){if text_row(ui,&query,key,value)&&project.metadata().get(&Text::new(key.clone()).expect("chave validada")).map(String::as_str).unwrap_or("")!=value.trim(){frame_edits.push(FrameEdit::SetProjectMetadata{key:Text::new(key.clone()).expect("chave validada"),value:Some(value.clone())});}}
                        read_row(ui,&query,"Documentos",project.documents().count());read_row(ui,&query,"Páginas",project.pages().count());read_row(ui,&query,"Representações",project.representations().count());
                    },
                    Draft::Page{id,name,number,kind,width,height,function,location,meta}=>{
                        if let Some(page)=project.page(*id){
                            if text_row(ui,&query,"Nome",name)&&name!=page.name().as_str(){edits.push(Edit::PageName(*id,name.clone()));}
                            if text_row(ui,&query,"Identificação",number)&&number!=page.number().as_str(){edits.push(Edit::PageNumber(*id,number.clone()));}
                            let mut format_changed=false;
                            if "tipo".contains(&query){ui.label("Tipo");egui::ComboBox::from_id_salt("page_kind_property").selected_text(kind_label(kind)).show_ui(ui,|ui|{for candidate in page_kinds(){let label=kind_label(&candidate).to_owned();format_changed|=ui.selectable_value(kind,candidate,label).changed();}});ui.end_row();}
                            if "formato".contains(&query){ui.label("Formato");let current=Size2D::new(*width,*height).ok().and_then(PaperFormat::matching).map(|(f,l)|format!("{} {}",f.label(),if l{"paisagem"}else{"retrato"})).unwrap_or_else(||"Personalizado".into());egui::ComboBox::from_id_salt("page_paper_format").width(100.0).truncate().selected_text(current).show_ui(ui,|ui|{for f in PaperFormat::ALL{for landscape in [true,false]{let s=f.size(landscape);if ui.selectable_label(false,format!("{} {}",f.label(),if landscape{"paisagem"}else{"retrato"})).clicked(){*width=s.width();*height=s.height();format_changed=true;}}}});ui.end_row();}
                            format_changed|=number_row(ui,&query,"Largura (mm)",width);format_changed|=number_row(ui,&query,"Altura (mm)",height);
                            if format_changed{match Size2D::new(*width,*height){Ok(size)=>edits.push(Edit::PageFormat(*id,kind.clone(),size)),Err(e)=>error=Some(e.to_string())}}
                            let f=text_row(ui,&query,"Função (caminho /)",function);let l=text_row(ui,&query,"Localização (caminho /)",location);
                            if (f&&*function!=path_text(page.structure().functional_path()))||(l&&*location!=path_text(page.structure().location_path())) {
                                match crate::demo::path(function).and_then(|f|crate::demo::path(location).map(|l|DocumentStructure::new(f,l))){Ok(structure)=>edits.push(Edit::PageStructure(*id,structure)),Err(e)=>error=Some(e.to_string())}
                            }
                            frame_rows(ui,&query,project,Some(*id),&mut frame_edits);
                            for (key,value) in meta.iter_mut(){if text_row(ui,&query,key,value)&&page.metadata().get(&Text::new(key.clone()).expect("chave validada")).map(String::as_str).unwrap_or("")!=value.trim(){frame_edits.push(FrameEdit::SetPageMetadata{page:*id,key:Text::new(key.clone()).expect("chave validada"),value:Some(value.clone())});}}
                        }
                    },
                    Draft::Representation{id,page,x,y,angle,mx,my,visible,layer,label}=>{
                        number_row(ui,&query,"X (mm)",x);number_row(ui,&query,"Y (mm)",y);number_row(ui,&query,"Ângulo CCW (°)",angle);
                        bool_row(ui,&query,"Espelhar X local",mx);bool_row(ui,&query,"Espelhar Y local",my);
                        if let Some(t)=self.session.graphics.preview(*id){read_row(ui,&query,"Prévia X / Y (mm)",format!("{:.3} / {:.3}",t.translation().x(),t.translation().y()));}
                        if ui.button("Aplicar transformação").on_hover_text("Confirma X/Y, ângulo e espelhos em uma operação reversível; pivô na âncora").clicked(){match Point2D::new(*x,*y).and_then(|p|Transform2D::new(p,*angle,*mx,*my)){Ok(t)=>edits.push(Edit::RepresentationTransform(*id,*page,t)),Err(e)=>error=Some(e.to_string())}}ui.end_row();
                        bool_row(ui,&query,"Visível",visible);
                        if "camada".contains(&query){ui.label("Camada");ui.add(egui::DragValue::new(layer));ui.end_row();}
                        text_row(ui,&query,"Rótulo",label);
                        if ui.button("Aplicar aparência").on_hover_text("Confirma visibilidade, camada e rótulo em uma operação reversível").clicked(){let text=if label.trim().is_empty(){Ok(None)}else{Text::new(label.clone()).map(Some)};match text{Ok(label)=>edits.push(Edit::RepresentationVisual(*id,VisualProperties{visible:*visible,layer:*layer,label})),Err(e)=>error=Some(e.to_string())}}ui.end_row();
                    },Draft::Empty=>{}
                }
            });};if inspector.grouped{egui::CollapsingHeader::new("Dados documentais").default_open(true).show(ui,content);}else{content(ui);}}
            let target=match self.session.selection{Selection::Electrical(t)=>Some(t),Selection::Representation(id)=>project.representation(id).map(Representation::target),_=>None};
            if (inspector.profile==0||inspector.profile==3)&&let Some(target)=target && target!=RepresentationTarget::Unassociated {
                egui::CollapsingHeader::new("Entidade elétrica associada").default_open(true).show(ui,|ui| {
                    table(ui,"target_properties",|ui| {
                        if let RepresentationTarget::Device(id)=target&&let Some(d)=project.device(id){read_row(ui,&query,"Designação",d.designation());read_row(ui,&query,"Classificação",d.classification().as_str());}
                        if let RepresentationTarget::Connection(id)=target&&let Some(c)=project.connections().find(|c|c.id()==id){let(a,b)=c.endpoints();read_row(ui,&query,"Extremo A",a);read_row(ui,&query,"Extremo B",b);}
                        for (k,v) in crate::physical_editor::physical_summary(project,target){read_row(ui,&query,&k,v);}
                        if let Some(props)=target_properties(project,target){for(k,v)in props.known(){read_row(ui,&query,&format!("{k:?}"),v.get());}for(k,v)in props.custom(){read_row(ui,&query,k.as_str(),format!("{v:?}"));}}
                    });
                    if let Ok(reps)=project.representations_for_entity(target){ui.small(format!("{} ocorrência(s)",reps.len()));for r in reps{if let Some(page)=project.page(r.page_id())&&ui.link(format!("{} · {}",page.number().as_str(),page.name().as_str())).clicked(){related=Some((page.id(),r.id()));}}}
                });
            }
            ui.add_space(8.0);ui.weak("Transformação e aparência: confirme em Aplicar. Ctrl+Z desfaz e Ctrl+Y refaz. Edições elétricas e de página encerram o histórico gráfico.");
        });
        self.inspector = inspector;
        self.show_free_graphic_properties(ui);
        if let Some(edit) = drawing_edit {
            self.graphic_edit(GraphicEdit::Drawing(edit));
        }
        for edit in frame_edits {
            self.graphic_edit(GraphicEdit::Frame(edit));
        }
        for edit in edits {
            self.edit(edit);
        }
        if let Some(error) = error {
            self.log(Level::Error, error);
        }
        if let Some((page, rep)) = related {
            self.open_page(page);
            self.session.select(Selection::Representation(rep));
        }
    }
    fn show_group_properties(&mut self, ui: &mut Ui, inspector: &mut Inspector) {
        let ids: Vec<_> = self
            .session
            .graphics
            .selection
            .ids()
            .iter()
            .copied()
            .collect();
        if inspector.multi_ids != ids {
            inspector.multi_ids = ids.clone();
            inspector.displacement = [0.0; 2];
        }
        ui.strong(format!("{} representações selecionadas", ids.len()));
        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Some(p) = self.session.project() {
                let reps: Vec<_> = ids.iter().filter_map(|id| p.representation(*id)).collect();
                let common = |values: Vec<String>| {
                    if values.windows(2).all(|v| v[0] == v[1]) {
                        values.first().cloned().unwrap_or_default()
                    } else {
                        "Valores diferentes".into()
                    }
                };
                table(ui, "multi_properties", |ui| {
                    read_row(
                        ui,
                        "",
                        "Página",
                        self.session
                            .active
                            .map(|id| id.to_string())
                            .unwrap_or_default(),
                    );
                    read_row(
                        ui,
                        "",
                        "X (mm)",
                        common(
                            reps.iter()
                                .map(|r| r.transform().translation().x().to_string())
                                .collect(),
                        ),
                    );
                    read_row(
                        ui,
                        "",
                        "Y (mm)",
                        common(
                            reps.iter()
                                .map(|r| r.transform().translation().y().to_string())
                                .collect(),
                        ),
                    );
                    read_row(
                        ui,
                        "",
                        "Ângulo CCW (°)",
                        common(
                            reps.iter()
                                .map(|r| r.transform().rotation_degrees().to_string())
                                .collect(),
                        ),
                    );
                    read_row(
                        ui,
                        "",
                        "Espelhos locais X / Y",
                        common(
                            reps.iter()
                                .map(|r| {
                                    format!(
                                        "{} / {}",
                                        r.transform().mirror_x(),
                                        r.transform().mirror_y()
                                    )
                                })
                                .collect(),
                        ),
                    );
                    read_row(
                        ui,
                        "",
                        "Visível",
                        common(
                            reps.iter()
                                .map(|r| r.visual().visible.to_string())
                                .collect(),
                        ),
                    );
                    number_row(ui, "", "Deslocar X (mm)", &mut inspector.displacement[0]);
                    number_row(ui, "", "Deslocar Y (mm)", &mut inspector.displacement[1]);
                });
            }
            if ui.button("Mover conjunto").clicked() {
                self.graphic_edit(GraphicEdit::Translate(
                    inspector.displacement[0],
                    inspector.displacement[1],
                ));
                inspector.displacement = [0.0; 2];
            }
            ui.small(
                "Mesmo deslocamento para todos. Rotação e espelho usam a âncora de cada objeto.",
            );
            for id in [
                CommandId::Move,
                CommandId::Rotate90,
                CommandId::Rotate180,
                CommandId::Rotate270,
                CommandId::MirrorHorizontal,
                CommandId::MirrorVertical,
                CommandId::Undo,
                CommandId::Redo,
            ] {
                if self.command_button(ui, id, false, false) {
                    self.execute(id, ui.ctx());
                }
            }
        });
    }
}

impl CadApp {
    fn show_free_graphic_properties(&mut self, ui: &mut Ui) {
        let Some(id) = (match self.session.selection {
            Selection::Representation(id) => Some(id),
            _ => None,
        }) else {
            self.free_edit = None;
            return;
        };
        let Some(graphic) = self
            .session
            .project()
            .and_then(|p| p.representation(id))
            .and_then(Representation::graphic)
            .cloned()
        else {
            self.free_edit = None;
            return;
        };
        if self.free_edit_revision != self.session.revision
            || self.free_edit.as_ref().is_none_or(|draft| draft.0 != id)
        {
            self.free_edit = Some((
                id,
                graphic.primitive().clone(),
                graphic.stroke_mm(),
                graphic.alignment(),
            ));
            self.free_edit_revision = self.session.revision;
        }
        let mut apply = false;
        let mut style_change: Option<PrimitiveStyle> = None;
        ui.collapsing("Conteúdo livre", |ui| {
            // Etapa 17: padrão do traço e preenchimento (aplicação imediata).
            let mut style = graphic.style();
            egui::ComboBox::from_id_salt("free_graphic_dash")
                .selected_text(style.dash.label())
                .show_ui(ui, |ui| {
                    for dash in LineDash::ALL {
                        ui.selectable_value(&mut style.dash, dash, dash.label());
                    }
                });
            if graphic.primitive().is_closed() {
                ui.checkbox(&mut style.fill, "Preenchimento");
            }
            if style != graphic.style() {
                style_change = Some(style);
            }
            if let Some((_, primitive, stroke, alignment)) = &mut self.free_edit {
                ui.horizontal(|ui| {
                    ui.label("Traço (mm)");
                    ui.add(egui::DragValue::new(stroke).range(0.01..=10.0).speed(0.01));
                });
                match primitive {
                    SymbolPrimitive::Text { text, height, .. } => {
                        ui.label("Texto fixo Unicode");
                        ui.add(egui::TextEdit::singleline(text));
                        ui.horizontal(|ui| {
                            ui.label("Altura (mm)");
                            ui.add(egui::DragValue::new(height).range(0.1..=100.0).speed(0.1));
                        });
                        egui::ComboBox::from_id_salt("edit_free_align")
                            .selected_text(format!("{:?}", alignment))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(alignment, TextAlignment::Left, "Esquerda");
                                ui.selectable_value(alignment, TextAlignment::Center, "Centro");
                                ui.selectable_value(alignment, TextAlignment::Right, "Direita");
                            });
                    }
                    SymbolPrimitive::Line { end, .. } => {
                        let mut x = end.x();
                        let mut y = end.y();
                        ui.horizontal(|ui| {
                            ui.label("Extremo local X/Y (mm)");
                            ui.add(egui::DragValue::new(&mut x).speed(0.1));
                            ui.add(egui::DragValue::new(&mut y).speed(0.1));
                        });
                        if let Ok(p) = Point2D::new(x, y) {
                            *end = p;
                        }
                    }
                    SymbolPrimitive::Rectangle { rect } => {
                        let mut w = rect.size().width();
                        let mut h = rect.size().height();
                        ui.horizontal(|ui| {
                            ui.label("Largura/altura (mm)");
                            ui.add(egui::DragValue::new(&mut w).range(0.1..=10000.0).speed(0.1));
                            ui.add(egui::DragValue::new(&mut h).range(0.1..=10000.0).speed(0.1));
                        });
                        if let Ok(size) = Size2D::new(w, h)
                            && let Ok(next) = Rect2D::new(rect.origin(), size)
                        {
                            *rect = next;
                        }
                    }
                    SymbolPrimitive::Circle { radius, .. } => {
                        ui.horizontal(|ui| {
                            ui.label("Raio (mm)");
                            ui.add(egui::DragValue::new(radius).range(0.1..=10000.0).speed(0.1));
                        });
                    }
                    SymbolPrimitive::Arc {
                        radius,
                        start_degrees,
                        sweep_degrees,
                        ..
                    } => {
                        ui.horizontal(|ui| {
                            ui.label("Raio (mm)");
                            ui.add(egui::DragValue::new(radius).range(0.1..=10000.0).speed(0.1));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Início/varredura (°)");
                            ui.add(egui::DragValue::new(start_degrees).speed(1.0));
                            ui.add(
                                egui::DragValue::new(sweep_degrees)
                                    .range(-360.0..=360.0)
                                    .speed(1.0),
                            );
                        });
                    }
                    _ => {}
                }
                apply = ui.button("Aplicar conteúdo livre").clicked();
            }
        });
        if let Some(style) = style_change
            && let Err(e) = graphic
                .clone()
                .with_style(style)
                .and_then(|g| self.session.edit_free_graphic(id, g))
        {
            self.log(Level::Error, e.to_string());
        }
        if apply && let Some((_, primitive, stroke, alignment)) = &self.free_edit {
            match PageGraphic::new(primitive.clone(), *stroke, *alignment)
                .and_then(|g| g.with_style(graphic.style()))
                .and_then(|g| self.session.edit_free_graphic(id, g))
            {
                Ok(_) => {}
                Err(e) => self.log(Level::Error, e.to_string()),
            }
        }
    }
}
fn symbol_rows(ui: &mut Ui, query: &str, d: &SymbolDefinition, t: Transform2D) {
    read_row(ui, query, "Símbolo", d.name());
    read_row(ui, query, "Categoria", d.category());
    read_row(ui, query, "Definição", d.id());
    read_row(ui, query, "Versão da definição", d.version());
    read_row(ui, query, "Geometria", "Vetorial em milímetros");
    read_row(ui, query, "Pontos de conexão", d.ports().len());
    for port in d.ports() {
        if let Ok(world) = port.transformed(t) {
            read_row(
                ui,
                query,
                &format!("Ponto {} · {}", port.id(), port.identification()),
                format!(
                    "X {:.2} / Y {:.2} mm · {:.0}°{}",
                    world.position().x(),
                    world.position().y(),
                    world.orientation_degrees(),
                    port.role().map(|r| format!(" · {r}")).unwrap_or_default()
                ),
            );
        }
    }
}
fn target_properties(p: &Project, target: RepresentationTarget) -> Option<&TechnicalProperties> {
    match target {
        RepresentationTarget::Unassociated => None,
        RepresentationTarget::Device(id) => p.device(id).map(Device::properties),
        RepresentationTarget::Function(id) => p.function(id).map(Function::properties),
        RepresentationTarget::Component(id) => p.component(id).map(Component::properties),
        RepresentationTarget::Terminal(id) => p.terminal(id).map(Terminal::properties),
        RepresentationTarget::ConnectionPoint(id) => {
            p.connection_point(id).map(ConnectionPoint::properties)
        }
        RepresentationTarget::Connection(_) => None,
        RepresentationTarget::Conductor(id) => p
            .conductors()
            .find(|e| e.id() == id)
            .map(Conductor::properties),
        RepresentationTarget::Cable(id) => p.cables().find(|e| e.id() == id).map(Cable::properties),
        RepresentationTarget::CableCore(id) => p
            .cable_cores()
            .find(|e| e.id() == id)
            .map(CableCore::properties),
        RepresentationTarget::TerminalStrip(id) => p
            .terminal_strips()
            .find(|e| e.id() == id)
            .map(TerminalStrip::properties),
    }
}

/// Etapa 13: chaves de metadados sempre oferecidas (fonte dos textos especiais).
const PROJECT_KEYS: &[&str] = &["Autor", "Cliente", "Desenhista", "Data", "Revisão"];
const PAGE_KEYS: &[&str] = &["Data", "Desenhista", "Revisão"];
fn metadata_rows(metadata: &Metadata, standard: &[&str]) -> Vec<(String, String)> {
    let mut rows: Vec<(String, String)> = standard
        .iter()
        .map(|k| {
            let value = metadata
                .iter()
                .find(|(key, _)| key.as_str() == *k)
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            ((*k).to_owned(), value)
        })
        .collect();
    for (key, value) in metadata {
        if !standard.contains(&key.as_str()) {
            rows.push((key.as_str().to_owned(), value.clone()));
        }
    }
    rows
}
/// Moldura (padrão do projeto ou da página) e formato das referências.
fn frame_rows(
    ui: &mut Ui,
    query: &str,
    project: &Project,
    page: Option<PageId>,
    edits: &mut Vec<FrameEdit>,
) {
    if !"moldura".contains(query) && !"referência".contains(query) {
        return;
    }
    let name = |id: Option<FrameTemplateId>| {
        id.and_then(|id| project.frame(id))
            .map(|f| f.name().as_str().to_owned())
    };
    ui.label("Moldura");
    match page {
        None => {
            let current = project.default_frame();
            egui::ComboBox::from_id_salt("project_default_frame")
                .width(100.0)
                .truncate()
                .selected_text(name(current).unwrap_or_else(|| "Sem moldura".into()))
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(current.is_none(), "Sem moldura")
                        .clicked()
                    {
                        edits.push(FrameEdit::SetDefaultFrame(None));
                    }
                    for f in project.frames() {
                        if ui
                            .selectable_label(current == Some(f.id()), f.name().as_str())
                            .clicked()
                        {
                            edits.push(FrameEdit::SetDefaultFrame(Some(f.id())));
                        }
                    }
                });
            ui.end_row();
            ui.label("Referências");
            let format = project.reference_format();
            let text = |f: ReferenceFormat| match f {
                ReferenceFormat::PageColumn => "/página.coluna",
                ReferenceFormat::PageColumnRow => "/página.colunaLinha",
            };
            egui::ComboBox::from_id_salt("project_reference_format")
                .width(100.0)
                .truncate()
                .selected_text(text(format))
                .show_ui(ui, |ui| {
                    for f in [ReferenceFormat::PageColumn, ReferenceFormat::PageColumnRow] {
                        if ui.selectable_label(format == f, text(f)).clicked() && format != f {
                            edits.push(FrameEdit::SetReferenceFormat(f));
                        }
                    }
                });
            ui.end_row();
        }
        Some(page) => {
            let own = project.page(page).and_then(Page::frame);
            let default = format!(
                "Padrão do projeto ({})",
                name(project.default_frame()).unwrap_or_else(|| "sem moldura".into())
            );
            egui::ComboBox::from_id_salt("page_frame")
                .width(100.0)
                .truncate()
                .selected_text(name(own).unwrap_or_else(|| default.clone()))
                .show_ui(ui, |ui| {
                    if ui.selectable_label(own.is_none(), &default).clicked() && own.is_some() {
                        edits.push(FrameEdit::SetPageFrame { page, frame: None });
                    }
                    for f in project.frames() {
                        if ui
                            .selectable_label(own == Some(f.id()), f.name().as_str())
                            .clicked()
                            && own != Some(f.id())
                        {
                            edits.push(FrameEdit::SetPageFrame {
                                page,
                                frame: Some(f.id()),
                            });
                        }
                    }
                });
            ui.end_row();
            if let (Some(f), Some(p)) = (project.page_frame(page), project.page(page))
                && ((f.sheet().width() - p.size().width()).abs() > 0.5
                    || (f.sheet().height() - p.size().height()).abs() > 0.5)
            {
                ui.label("");
                ui.add(
                    egui::Label::new(
                        egui::RichText::new("Moldura com tamanho diferente da folha")
                            .color(ui.visuals().warn_fg_color),
                    )
                    .wrap(),
                );
                ui.end_row();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn abandoned_group_draft_does_not_reappear_after_selection_or_page_roundtrip() {
        let mut s = Session::default();
        s.replace(crate::demo::project().unwrap(), true).unwrap();
        let page = s
            .project()
            .unwrap()
            .representations()
            .next()
            .unwrap()
            .page_id();
        let ids: Vec<_> = s
            .project()
            .unwrap()
            .representations_on_page(page)
            .unwrap()
            .iter()
            .map(|r| r.id())
            .collect();
        s.open_page(page).unwrap();
        s.select_graphics(ids.clone());
        let mut inspector = Inspector::default();
        inspector.sync(&s);
        inspector.multi_ids = ids.clone();
        inspector.displacement = [12.5, -3.0];
        s.select(Selection::Representation(ids[0]));
        inspector.sync(&s);
        s.select_graphics(ids.clone());
        inspector.sync(&s);
        assert_eq!(inspector.displacement, [0.0, 0.0]);
        inspector.displacement = [8.0, 0.0];
        let next = s
            .project()
            .unwrap()
            .pages()
            .find(|p| p.id() != page)
            .unwrap()
            .id();
        // The inspector can be hidden throughout the page round-trip.
        s.open_page(next).unwrap();
        s.open_page(page).unwrap();
        s.select_graphics(ids);
        inspector.sync(&s);
        assert_eq!(inspector.displacement, [0.0, 0.0]);
        inspector.displacement = [4.0, 0.0];
        s.dispatch_graphic(GraphicEdit::Rotate(90.0)).unwrap();
        inspector.sync(&s);
        assert_eq!(inspector.displacement, [0.0, 0.0]);
    }
}
