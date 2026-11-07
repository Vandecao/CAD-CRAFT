// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Abrupt process termination during editing and atomic recovery replacement.
//! Start with the parent test; its child uses the production autosave services.
// Synthetic test fixtures fail immediately when setup is invalid.
#![allow(clippy::unwrap_used)]

use cad_craft::domain::Project;
use cad_gui::{
    app::CadApp,
    project_file::{self, Recovery},
};
use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const CHILD_DIR: &str = "CADCRAFT_RECOVERY_PROCESS_TEST_DIR";

struct KillOnDrop(Child);
impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Block only after production atomic replacement has written a partial temp
/// file. The parent then kills the process, so no unwinding or Drop runs there.
struct InterruptedSnapshot {
    marker: PathBuf,
    sent: bool,
}
impl Read for InterruptedSnapshot {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if !self.sent {
            self.sent = true;
            let bytes = b"PK\x03\x04incomplete replacement";
            let count = buffer.len().min(bytes.len());
            buffer[..count].copy_from_slice(&bytes[..count]);
            return Ok(count);
        }
        fs::write(&self.marker, b"partial temporary file written")?;
        loop {
            thread::sleep(Duration::from_millis(50));
        }
    }
}

#[test]
fn recovery_crash_child() {
    let Some(dir) = std::env::var_os(CHILD_DIR).map(PathBuf::from) else {
        return;
    };
    let recovery = Recovery::new(dir.join("recovery"));
    let mut app = CadApp::default();
    app.files.recovery = Some(recovery.clone());
    app.files.autosave_interval = Duration::ZERO;
    app.session
        .replace(Project::new("Synthetic crash recovery").unwrap(), false)
        .unwrap();
    app.session.new_document("First interval").unwrap();
    app.autosave();
    assert_eq!(recovery.entries().len(), 1);
    app.session
        .new_document("Latest completed interval")
        .unwrap();
    app.autosave();
    let expected = app.session.project().unwrap().clone();
    project_file::save(&expected, &dir.join("expected.cadcraft")).unwrap();

    // A subsequent validated edit has not reached the next autosave interval.
    app.files.autosave_interval = Duration::from_secs(3600);
    app.session
        .new_document("Pending edit after latest interval")
        .unwrap();
    assert!(app.session.dirty);
    app.autosave();
    assert_eq!(app.session.project().unwrap().documents().count(), 3);
    let entry = recovery.entries().pop().unwrap();
    assert_eq!(recovery.load(&entry).unwrap(), expected);
    let target = recovery
        .dir()
        .join(format!("{}.cadcraft", entry.project_id));
    project_file::write_atomic(
        &target,
        &mut InterruptedSnapshot {
            marker: dir.join("ready"),
            sent: false,
        },
    )
    .unwrap();
    panic!("parent must kill the child during replacement");
}

fn wait_for_partial_write(child: &mut Child, marker: &Path) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while !marker.is_file() {
        assert!(
            child.try_wait().unwrap().is_none(),
            "child exited before reaching the crash point"
        );
        assert!(
            Instant::now() < deadline,
            "child did not reach the crash point within 30 seconds"
        );
        thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn killed_editor_recovers_exactly_the_last_completed_autosave() {
    let dir = tempfile::tempdir().unwrap();
    let mut child = KillOnDrop(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "recovery_crash_child", "--nocapture"])
            .env(CHILD_DIR, dir.path())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap(),
    );
    wait_for_partial_write(&mut child.0, &dir.path().join("ready"));
    child.0.kill().unwrap();
    assert!(!child.0.wait().unwrap().success());

    // Recreate recovery services just as a fresh application process does.
    let recovery = Recovery::new(dir.path().join("recovery"));
    let entries = recovery.entries();
    assert_eq!(
        entries.len(),
        1,
        "partial temporary files must not become recovery entries"
    );
    let restored = recovery.load(&entries[0]).unwrap();
    let expected = project_file::load(&dir.path().join("expected.cadcraft"))
        .unwrap()
        .project;
    assert_eq!(
        restored, expected,
        "every ID and relation from the last interval must survive"
    );
    assert_eq!(restored.documents().count(), 2);
    assert!(
        restored
            .documents()
            .any(|document| document.name().as_str() == "Latest completed interval")
    );
    assert!(
        !restored
            .documents()
            .any(|document| document.name().as_str() == "Pending edit after latest interval")
    );
    assert!(
        fs::read_dir(recovery.dir()).unwrap().any(|entry| entry
            .unwrap()
            .path()
            .extension()
            .is_some_and(|extension| extension == "tmp")),
        "kill must leave evidence of the interrupted atomic write"
    );
}
