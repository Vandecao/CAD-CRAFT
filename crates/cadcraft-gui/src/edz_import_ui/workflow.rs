// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! EDZ analysis, confirmation and panel workflow; start with CadApp::start_edz_analysis.
use super::*;

impl CadApp {
    pub(crate) fn start_edz_analysis(&mut self) {
        let mut paths = self.edz.paths.clone();
        paths.sort();
        paths.dedup();
        if paths.is_empty() || paths.len() > MAX_BATCH {
            self.edz.status = "Escolha entre 1 e 1.000 pacotes.".into();
            return;
        }
        let (sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        self.edz.total = paths.len();
        self.edz.completed = 0;
        self.edz.packages.clear();
        self.edz.prepared = false;
        self.edz.confirmed = false;
        self.edz.status = "Analisando…".into();
        self.edz.job = Some(Job { receiver, cancel });
        std::thread::spawn(move || {
            for path in paths {
                if flag.load(Ordering::Relaxed) {
                    break;
                }
                let preview = read_package(&path).map_err(|error| error.to_string());
                if flag.load(Ordering::Relaxed) {
                    break;
                }
                if sender
                    .send(Message::Package(Box::new(EdzPackage {
                        path,
                        preview,
                        outcome: String::new(),
                        artifacts: Vec::new(),
                    })))
                    .is_err()
                {
                    return;
                }
            }
            let _ = sender.send(Message::Finished);
        });
    }
    pub(crate) fn poll_edz(&mut self, ctx: &Context) {
        let Some(job) = &self.edz.job else {
            return;
        };
        let mut done = false;
        loop {
            match job.receiver.try_recv() {
                Ok(Message::Package(p)) => self.edz.packages.push(*p),
                Ok(Message::Finished) => {
                    done = true;
                    break;
                }
                Ok(Message::Progress(n, total)) => {
                    self.edz.completed = n;
                    self.edz.total = total;
                }
                Ok(Message::Prepared(packages, result)) => {
                    let cancelled = job.cancel.load(Ordering::Relaxed);
                    self.edz.packages = packages;
                    self.edz.job = None;
                    self.edz.prepared = result.is_ok() && !cancelled;
                    self.edz.confirmed = cancelled;
                    self.edz.status = if cancelled {
                        "Cancelado; nenhuma biblioteca foi alterada.".into()
                    } else {
                        match result {
                            Ok(()) => {
                                format!("Análise concluída: {} pacote(s).", self.edz.packages.len())
                            }
                            Err(e) => format!("Erro: {e}"),
                        }
                    };
                    return;
                }
                Ok(Message::Saved(packages, result, catalogs)) => {
                    self.edz.packages = packages;
                    self.edz.job = None;
                    self.edz.confirmed = true;
                    self.edz.status = match result {
                        Ok(()) => "Importação concluída.".into(),
                        Err(e) => format!("Erro: {e}"),
                    };
                    match catalogs {
                        Ok((parts, macros, part_folder, macro_folder)) => {
                            if let Some(parts) = parts
                                && self.preferences.part_library == part_folder
                            {
                                self.part_editor.install_catalog(parts, &part_folder);
                            }
                            if let Some(macros) = macros
                                && self.preferences.macro_library == macro_folder
                            {
                                self.macro_library.install_catalog(macros, &macro_folder);
                            }
                        }
                        Err(e) => self.edz.status.push_str(&format!(" Biblioteca: {e}")),
                    }
                    return;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.edz.status = "Erro: análise interrompida.".into();
                    self.edz.job = None;
                    return;
                }
                Err(mpsc::TryRecvError::Empty) => break,
            }
        }
        if done {
            let cancelled = job.cancel.load(Ordering::Relaxed);
            self.edz.job = None;
            self.edz.status = if cancelled {
                "Cancelado; nenhuma biblioteca foi alterada.".into()
            } else {
                format!("Análise concluída: {} pacote(s).", self.edz.packages.len())
            };
            if cancelled {
                self.edz.confirmed = true;
            } else {
                self.start_edz_preparation(ctx);
            }
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(40));
        }
    }
    #[cfg(test)]
    pub(super) fn prepare_edz(&mut self) -> GuiResult<()> {
        prepare_packages(
            &mut self.edz.packages,
            &self.preferences.part_library,
            &self.preferences.macro_library,
            &AtomicBool::new(false),
        )?;
        self.edz.prepared = true;
        Ok(())
    }
    fn start_edz_preparation(&mut self, ctx: &Context) {
        let mut packages = std::mem::take(&mut self.edz.packages);
        let part_folder = self.preferences.part_library.clone();
        let macro_folder = self.preferences.macro_library.clone();
        let (sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        self.edz.status = "Verificando conflitos nas bibliotecas…".into();
        self.edz.job = Some(Job { receiver, cancel });
        std::thread::spawn(move || {
            let result = prepare_packages(&mut packages, &part_folder, &macro_folder, &flag);
            let _ = sender.send(Message::Prepared(packages, result));
        });
        ctx.request_repaint();
    }
    fn start_edz_persistence(&mut self) {
        let mut packages = std::mem::take(&mut self.edz.packages);
        let part_folder = self.preferences.part_library.clone();
        let macro_folder = self.preferences.macro_library.clone();
        let (sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        self.edz.total = packages.iter().map(|p| p.artifacts.len()).sum::<usize>() * 2;
        self.edz.completed = 0;
        self.edz.status = "Preparando e gravando bibliotecas…".into();
        self.edz.job = Some(Job { receiver, cancel });
        std::thread::spawn(move || {
            let result = save_packages(
                &mut packages,
                &part_folder,
                &macro_folder,
                &flag,
                Some(&sender),
            );
            if let Err(e) = &result {
                for package in &mut packages {
                    if package.outcome.is_empty() {
                        package.outcome =
                            format!("Erro: {e}; nenhum arquivo deste pacote foi gravado.");
                    }
                }
            }
            let catalogs = load_catalogs(&part_folder, &macro_folder);
            let _ = sender.send(Message::Saved(packages, result, catalogs));
        });
    }
    #[cfg(test)]
    pub(super) fn persist_edz(&mut self) -> GuiResult<()> {
        save_packages(
            &mut self.edz.packages,
            &self.preferences.part_library,
            &self.preferences.macro_library,
            &AtomicBool::new(false),
            None,
        )?;
        self.part_editor.reload(&self.preferences)?;
        self.macro_library.reload(&self.preferences)?;
        Ok(())
    }
    pub(crate) fn show_edz(&mut self, ctx: &Context) {
        if !self.edz.open {
            return;
        }
        let mut open = true;
        egui::Window::new("Importar .edz")
            .open(&mut open)
            .default_size([760., 620.])
            .show(ctx, |ui| {
                let busy = self.edz.busy();
                ui.add_enabled_ui(!busy, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Escolher arquivos .edz…").clicked() {
                            let paths = self.files.picker.open_edz();
                            if !paths.is_empty() {
                                self.edz.paths = paths;
                                self.edz.prepared = false;
                                self.edz.packages.clear();
                            }
                        }
                        if ui.button("Escolher pasta…").clicked()
                            && let Some(dir) = self
                                .files
                                .picker
                                .pick_folder("Importar pasta de pacotes .edz")
                        {
                            match std::fs::read_dir(dir) {
                                Ok(entries) => {
                                    self.edz.paths = entries
                                        .filter_map(Result::ok)
                                        .map(|e| e.path())
                                        .filter(|p| is_edz(p))
                                        .take(MAX_BATCH + 1)
                                        .collect();
                                    self.edz.prepared = false;
                                    self.edz.packages.clear();
                                }
                                Err(e) => self.edz.status = format!("Erro: {e}"),
                            }
                        }
                        if ui
                            .add_enabled(
                                !self.edz.paths.is_empty(),
                                egui::Button::new("Analisar pacotes"),
                            )
                            .clicked()
                        {
                            self.start_edz_analysis();
                        }
                    });
                });
                ui.label(format!(
                    "{} arquivo(s) selecionado(s)",
                    self.edz.paths.len()
                ));
                if busy {
                    let completed = self.edz.completed.max(self.edz.packages.len());
                    ui.add(
                        egui::ProgressBar::new(completed as f32 / self.edz.total.max(1) as f32)
                            .text(format!("{} / {}", completed, self.edz.total)),
                    );
                    if ui.button(crate::strings::CANCEL_OPERATION).clicked() {
                        self.edz.cancel();
                    }
                }
                ui.label(&self.edz.status);
                let unresolved = self
                    .edz
                    .packages
                    .iter()
                    .flat_map(|p| &p.artifacts)
                    .any(|a| a.conflict && a.choice.is_none());
                if ui
                    .add_enabled(
                        self.edz.prepared && !busy && !self.edz.confirmed && !unresolved,
                        egui::Button::new("Confirmar importação nas bibliotecas"),
                    )
                    .clicked()
                {
                    self.start_edz_persistence();
                }
                egui::ScrollArea::vertical()
                    .id_salt("edz_packages")
                    .show(ui, |ui| {
                        for (index, package) in self.edz.packages.iter_mut().enumerate() {
                            ui.push_id(index, |ui| {
                                ui.separator();
                                ui.strong(
                                    package
                                        .path
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy(),
                                );
                                if !package.outcome.is_empty() {
                                    ui.label(&package.outcome);
                                }
                                match &package.preview {
                                    Err(e) => {
                                        ui.label(format!("Erro: {e}"));
                                    }
                                    Ok(p) => {
                                        let symbols: usize = p
                                            .macros
                                            .iter()
                                            .map(|m| m.content().definitions().len())
                                            .sum();
                                        ui.label(format!(
                                            "{:?} · {} peça(s), {} macro(s), {} símbolo(s)",
                                            p.container,
                                            p.parts.len(),
                                            p.macros.len(),
                                            symbols
                                        ));
                                        for part in &p.parts {
                                            ui.label(format!(
                                                "Peça: {} · {}",
                                                part.code().as_str(),
                                                part.manufacturer().as_str()
                                            ));
                                        }
                                        for m in &p.macros {
                                            ui.label(format!("Macro: {}", m.name().as_str()));
                                        }
                                        for (n, a) in package
                                            .artifacts
                                            .iter_mut()
                                            .enumerate()
                                            .filter(|(_, a)| a.conflict)
                                        {
                                            ui.push_id(n, |ui| {
                                                ui.label(format!(
                                                    "Conflito: {} ({})",
                                                    a.data.name(),
                                                    a.data.id()
                                                ));
                                                ui.add_enabled_ui(!self.edz.confirmed, |ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.radio_value(
                                                            &mut a.choice,
                                                            Some(ConflictChoice::Replace),
                                                            "Substituir",
                                                        );
                                                        ui.radio_value(
                                                            &mut a.choice,
                                                            Some(ConflictChoice::KeepBoth),
                                                            "Manter as duas",
                                                        );
                                                        ui.radio_value(
                                                            &mut a.choice,
                                                            Some(ConflictChoice::Skip),
                                                            "Pular",
                                                        );
                                                    });
                                                });
                                            });
                                        }
                                        for line in p.report.lines() {
                                            ui.label(line);
                                        }
                                        if !p.unknown_codes.is_empty() {
                                            ui.label(format!(
                                                "Códigos desconhecidos: {}",
                                                p.unknown_codes
                                                    .iter()
                                                    .cloned()
                                                    .collect::<Vec<_>>()
                                                    .join(", ")
                                            ));
                                        }
                                    }
                                }
                            });
                        }
                    });
            });
        self.edz.open = open;
    }
}
