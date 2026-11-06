// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! DWG conversion through a user-installed external converter, normally ODA File Converter (proprietary Open Design Alliance software, never distributed or downloaded here). Start with convert: write R12 DXF to a temporary directory, invoke the converter with separate arguments and a timeout, copy output to the destination and clean up temporary directories.
use crate::error::{GuiError, GuiResult};
use cad_craft::dxf_export::DxfFile;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Available output versions: display label and ODA File Converter argument.
pub const DWG_VERSIONS: [(&str, &str); 4] = [
    ("2000", "ACAD2000"),
    ("2007", "ACAD2007"),
    ("2013", "ACAD2013"),
    ("2018", "ACAD2018"),
];

/// Default converter timeout.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);

/// Why the converter cannot be used; None means ready.
pub fn converter_problem(path: &str) -> Option<&'static str> {
    let path = path.trim();
    if path.is_empty() {
        Some("Configure o conversor DWG (ODA File Converter) em Configurações.")
    } else if !Path::new(path).is_file() {
        Some("O conversor DWG configurado não foi encontrado.")
    } else {
        None
    }
}

/// ODA File Converter arguments: input, output, version, type, recursive (0), audit (1) and filter.
pub fn oda_arguments(input: &Path, output: &Path, version: &str) -> Vec<OsString> {
    vec![
        input.as_os_str().to_owned(),
        output.as_os_str().to_owned(),
        version.into(),
        "DWG".into(),
        "0".into(),
        "1".into(),
        "*.DXF".into(),
    ]
}

/// Run the program until completion or timeout; kill it on timeout.
pub fn run_with_timeout(program: &Path, args: &[OsString], timeout: Duration) -> GuiResult<()> {
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW avoids flashing a console window.
        command.creation_flags(0x0800_0000);
    }
    let mut child = command
        .spawn()
        .map_err(|e| GuiError::context("não foi possível iniciar o conversor", e))?;
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(status)) => {
                return Err(GuiError::Converter(format!(
                    "o conversor terminou com erro ({status})"
                )));
            }
            Ok(None) if start.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "o conversor excedeu o tempo limite de {} s e foi encerrado",
                    timeout.as_secs()
                )
                .into());
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(25)),
            Err(e) => return Err(GuiError::context("falha ao acompanhar o conversor", e)),
        }
    }
}

/// Temporary directory removed on scope exit, including errors.
struct TempDir(PathBuf);
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Convert DXF files, copy .dwg files to destination and return the count.
pub fn convert_to_dwg(
    program: &Path,
    files: &[DxfFile],
    destination: &Path,
    version: &str,
    timeout: Duration,
) -> GuiResult<usize> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let root = TempDir(std::env::temp_dir().join(format!("cad-craft-dwg-{stamp}")));
    let (input, output) = (root.0.join("dxf"), root.0.join("dwg"));
    for dir in [&input, &output] {
        std::fs::create_dir_all(dir).map_err(|e| GuiError::context("pasta temporária", e))?;
    }
    for file in files {
        std::fs::write(input.join(&file.name), &file.bytes)
            .map_err(|e| GuiError::context(format!("não foi possível gravar {}", file.name), e))?;
    }
    run_with_timeout(program, &oda_arguments(&input, &output, version), timeout)?;
    let mut copied = 0;
    for entry in std::fs::read_dir(&output).map_err(GuiError::from)? {
        let path = entry.map_err(GuiError::from)?.path();
        if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("dwg"))
            && let Some(name) = path.file_name()
        {
            std::fs::copy(&path, destination.join(name)).map_err(|e| {
                GuiError::context(format!("não foi possível copiar {}", path.display()), e)
            })?;
            copied += 1;
        }
    }
    if copied == 0 {
        return Err("o conversor não gerou arquivos DWG".into());
    }
    Ok(copied)
}
