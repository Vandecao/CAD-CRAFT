// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 22: DWG por conversor externo instalado pelo usuário (padrão:
//! ODA File Converter, licença proprietária da Open Design Alliance; o
//! CAD-CRAFT só o chama, não o distribui nem o baixa).
//!
//! O DXF R12 é gravado numa pasta temporária; o conversor é chamado com
//! argumentos separados (sem shell) e tempo limite; os `.dwg` gerados vão
//! para o destino; as pastas temporárias são sempre removidas.
use crate::error::{GuiError, GuiResult};
use cad_craft::dxf_export::DxfFile;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Versões de saída oferecidas (rótulo, argumento do ODA File Converter).
pub const DWG_VERSIONS: [(&str, &str); 4] = [
    ("2000", "ACAD2000"),
    ("2007", "ACAD2007"),
    ("2013", "ACAD2013"),
    ("2018", "ACAD2018"),
];

/// Tempo limite padrão do conversor.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);

/// Por que o conversor não pode ser usado; `None` = pronto.
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

/// Argumentos do ODA File Converter: entrada, saída, versão, tipo,
/// recursivo (0), auditoria (1) e filtro.
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

/// Executa o programa e espera até `timeout`; ao estourar, encerra o processo.
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
        // CREATE_NO_WINDOW: sem janela de console piscando.
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

/// Pasta temporária removida ao sair do escopo (também em erro).
struct TempDir(PathBuf);
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Converte os DXF e copia os `.dwg` para `destination`; devolve quantos.
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
