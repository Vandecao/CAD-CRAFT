// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Project file actions and recovery; start with CadApp::perform.
use super::*;

impl CadApp {
    pub(crate) fn request(&mut self, action: PendingAction, ctx: &Context) {
        if self.session.dirty {
            self.dialogs_state.dialog = Some(Dialog::Confirm(action));
        } else {
            self.perform(action, ctx);
        }
    }
    pub(crate) fn perform(&mut self, action: PendingAction, ctx: &Context) {
        // Opening a file can stop at the lock question without replacing the session.
        if let PendingAction::Open(path) = &action {
            match self.open_project_file(path, LockMode::Normal) {
                Ok(true) => self.after_session_change(),
                Ok(false) => {}
                Err(e) => self.log(Level::Error, e),
            }
            return;
        }
        // Confirmed discard of the current session also drops its recovery copy
        // (unless that copy is exactly what is being restored).
        let current = self.session.project().map(|p| p.id().to_string());
        if !matches!(&action, PendingAction::Recover(e) if Some(&e.project_id) == current.as_ref())
        {
            self.discard_recovery();
        }
        let mut start_checks = false;
        let result = match action {
            PendingAction::Open(_) => unreachable!("handled above"),
            PendingAction::Recover(entry) => self.restore_recovery(&entry),
            PendingAction::DemoConnections => crate::connection_demo::project()
                .and_then(|p| {
                    let page = p.pages().next().map(Page::id);
                    self.session.replace(p, true)?;
                    if let Some(page) = page {
                        self.session.open_page(page)?;
                    }
                    Ok(())
                })
                .map_err(|e| e.to_string()),
            PendingAction::Demo => crate::demo::project()
                .and_then(|p| self.session.replace(p, true))
                .map_err(|e| e.to_string()),
            PendingAction::New(name) => Project::new(name)
                .and_then(crate::state::with_factory_frames)
                .and_then(|p| self.session.replace(p, false))
                .map_err(|e| e.to_string()),
            PendingAction::Import(json) => self.session.import_snapshot(&json),
            PendingAction::ImportQet {
                project,
                open_checks,
            } => {
                start_checks = open_checks;
                self.install_imported(*project).map_err(String::from)
            }
            PendingAction::Close => {
                self.session.close();
                Ok(())
            }
            PendingAction::Exit => {
                self.chrome.exit_confirmed = true;
                ctx.send_viewport_cmd(ViewportCommand::Close);
                Ok(())
            }
        };
        match result {
            Ok(()) => {
                self.after_session_change();
                if start_checks {
                    self.start_check(false);
                }
            }
            Err(e) => self.log(Level::Error, e),
        }
    }
    fn after_session_change(&mut self) {
        self.dialogs_state.backstage = None;
        self.navigator = NavigatorState::default();
        self.panels_state.inspector = Inspector::default();
        self.panels_state.insertion = InsertionState::default();
        self.files.last_autosave = None;
        self.log(Level::Info, "Sessão atualizada.");
    }

    // Project files.

    /// Ok(false) means a lock decision is pending; the session has not changed.
    pub(crate) fn open_project_file(
        &mut self,
        path: &std::path::Path,
        mode: LockMode,
    ) -> Result<bool, String> {
        use crate::project_file::{FileLock, LockError, ProjectFile, load};
        let loaded = load(path)?;
        let legacy = loaded.is_legacy_snapshot();
        // Reopening the file this session already holds reuses its own lock.
        let own = self
            .session
            .file
            .as_mut()
            .filter(|f| f.path == path && f.locked())
            .and_then(|f| f.take_lock());
        let lock = if legacy || mode == LockMode::ReadOnly {
            None
        } else if own.is_some() {
            own
        } else if mode == LockMode::TakeOver {
            Some(FileLock::take_over(path).map_err(|e| e.to_string())?)
        } else {
            match FileLock::acquire(path) {
                Ok(lock) => Some(lock),
                Err(LockError::Held(owner)) => {
                    self.dialogs_state.dialog = Some(Dialog::LockConflict {
                        path: path.to_path_buf(),
                        owner,
                        take_over: false,
                    });
                    return Ok(false);
                }
                Err(e) => return Err(e.to_string()),
            }
        };
        let read_only = !legacy && (loaded.read_only || lock.is_none());
        let source = loaded.source_format;
        // The user already confirmed discarding the previous session.
        self.discard_recovery();
        self.session
            .replace(loaded.project, false)
            .map_err(|e| e.to_string())?;
        crate::project_file::push_recent(&mut self.files.recent, path);
        if legacy {
            // Migrated content must be written with "Salvar como" into a .cadcraft.
            self.session.dirty = true;
            self.log(
                Level::Warning,
                "Snapshot técnico JSON (formato v0) migrado. Use Salvar para gravar como .cadcraft.",
            );
        } else {
            self.session.file = Some(ProjectFile::new(
                path.to_path_buf(),
                read_only,
                source,
                lock,
            ));
            if read_only {
                self.log(
                    Level::Warning,
                    "Projeto aberto somente leitura. Use Salvar como para gravar uma cópia.",
                );
            }
        }
        self.after_session_change();
        Ok(true)
    }
    pub(crate) fn open_project_dialog(&mut self, ctx: &Context) {
        if let Some(path) = self.files.picker.open_project() {
            self.request(PendingAction::Open(path), ctx);
        }
    }
    /// Save the associated file; use Save As if no writable file is attached.
    pub fn save_project(&mut self) -> bool {
        match &self.session.file {
            Some(file) if !file.read_only => {
                let path = file.path.clone();
                self.save_to(&path)
            }
            _ => self.save_project_as(),
        }
    }
    pub fn save_project_as(&mut self) -> bool {
        let Some(name) = self.session.project().map(|p| p.name().as_str().to_owned()) else {
            return false;
        };
        let Some(path) = self.files.picker.save_project(&name) else {
            return false;
        };
        self.save_to(&crate::project_file::with_extension(path))
    }
    fn save_to(&mut self, path: &std::path::Path) -> bool {
        use crate::project_file::{FileLock, ProjectFile, save};
        let same = self.session.file.as_ref().is_some_and(|f| f.path == path);
        let reuse = same && self.session.file.as_ref().is_some_and(|f| f.locked());
        let new_lock = if reuse {
            None
        } else {
            match FileLock::acquire(path) {
                Ok(lock) => Some(lock),
                Err(e) => {
                    self.log(Level::Error, format!("{}: {e}", path.display()));
                    return false;
                }
            }
        };
        let Some(project) = self.session.project() else {
            return false;
        };
        let id = project.id().to_string();
        if let Err(e) = save(project, path) {
            // The previous file and lock stay as they were.
            self.log(Level::Error, e);
            return false;
        }
        let lock = match new_lock {
            Some(lock) => Some(lock),
            None => self.session.file.as_mut().and_then(|f| f.take_lock()),
        };
        self.session.file = Some(ProjectFile::new(
            path.to_path_buf(),
            false,
            cad_craft::persistence::FORMAT,
            lock,
        ));
        self.session.dirty = false;
        if let Some(recovery) = &self.files.recovery {
            recovery.discard(&id);
        }
        crate::project_file::push_recent(&mut self.files.recent, path);
        self.log(Level::Info, format!("Projeto salvo em {}.", path.display()));
        true
    }
    /// Periodically save a recovery copy while unsaved changes exist.
    pub fn autosave(&mut self) {
        let interval = self.files.autosave_interval;
        let due = self
            .files
            .last_autosave
            .is_none_or(|t| t.elapsed() >= interval);
        // Confirmed exit already discarded the recovery copy; final frames must not recreate it.
        if !self.session.dirty || !due || self.chrome.exit_confirmed {
            return;
        }
        let (Some(recovery), Some(project)) = (&self.files.recovery, self.session.project()) else {
            return;
        };
        let origin = self.session.file.as_ref().map(|f| f.path.clone());
        self.files.last_autosave = Some(std::time::Instant::now());
        if let Err(e) = recovery.store(project, origin.as_deref()) {
            self.log(Level::Warning, format!("Cópia de recuperação falhou: {e}"));
        }
    }
    fn discard_recovery(&mut self) {
        if let (Some(recovery), Some(project)) = (&self.files.recovery, self.session.project()) {
            recovery.discard(&project.id().to_string());
        }
    }
    fn restore_recovery(
        &mut self,
        entry: &crate::project_file::RecoveryEntry,
    ) -> Result<(), String> {
        use crate::project_file::{FileLock, ProjectFile};
        let recovery = self
            .files
            .recovery
            .clone()
            .ok_or("Recuperação indisponível nesta sessão")?;
        let project = recovery.load(entry)?;
        self.session
            .replace(project, false)
            .map_err(|e| e.to_string())?;
        // The recovered copy stays until the project is saved or discarded.
        self.session.dirty = true;
        if let Some(origin) = entry.origin.as_ref().filter(|p| p.is_file()) {
            let lock = FileLock::acquire(origin).ok();
            let read_only = lock.is_none();
            self.session.file = Some(ProjectFile::new(
                origin.clone(),
                read_only,
                cad_craft::persistence::FORMAT,
                lock,
            ));
        }
        Ok(())
    }
    pub(crate) fn recovery_entries(&self) -> Vec<crate::project_file::RecoveryEntry> {
        self.files
            .recovery
            .as_ref()
            .map(|r| r.entries())
            .unwrap_or_default()
    }
}
