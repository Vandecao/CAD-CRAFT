# Contributing to CAD-CRAFT

Thank you for helping. This guide covers how to build, test and submit changes. Contributions in Portuguese are welcome too (issues and pull request discussions); the rules below apply to code and commits.

## Build and test

Windows 10/11 x64 with [Rustup](https://rustup.rs) and the Microsoft C++ Build Tools. The Rust version is pinned in `rust-toolchain.toml`.

```powershell
pwsh -NoProfile -File scripts/validate.ps1
```

The script runs, in order and stopping at the first failure: `cargo fmt --check`, `cargo check --workspace --all-targets --locked`, `cargo test --workspace --locked`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `git diff --check` and the PowerShell tests. The CI runs the same checks plus `cargo deny check`.

Rules for tests:

- Do not remove or weaken tests to make a change pass.
- Automated tests use only synthetic files. Real files from other platforms never go into the repository; see [docs/TESTES_COM_AMOSTRAS.md](docs/TESTES_COM_AMOSTRAS.md) for the optional tests with your own samples.
- `crates/cad-craft/tests/baseline_outputs.rs` records the exact output (`.cadcraft`, PDF, DXF, display list) of fixed projects. If your change alters an output on purpose, regenerate it with `CAD_CRAFT_WRITE_BASELINE=1` and explain why in the pull request.

## Architecture rules

- `crates/cad-craft` is the core and never depends on the GUI.
- Internal units are millimetres; origin at the bottom left; X to the right and Y up.
- Typed IDs (UUID v7). Components, devices and representations are separate; logical connections and physical conductors are separate; networks are derived from connectivity.
- Every edit is a validated, reversible change, so undo and redo stay exact.
- File formats are versioned; a new version always comes with a migration and a fixture.
- `unsafe` code is forbidden in the workspace.
- Importers must never panic on external input: a malformed file becomes an `ImportError` or a note in the import report.

## Language

- Code, file and module names, API comments (`///`, `//!`) and commit messages: **English**.
- User interface text: **Brazilian Portuguese** for now.
- Older comments in Portuguese are being converted gradually; you do not need to convert code you are not changing.

## Licensing and sign-off

CAD-CRAFT is licensed under GPL-3.0-or-later. Every Rust file starts with:

```rust
// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
```

A test checks this header in every `.rs` file. New dependencies must have a license compatible with GPL-3.0 (see `deny.toml`).

Contributions use the [Developer Certificate of Origin](https://developercertificate.org/): sign every commit with `git commit -s`, which adds a `Signed-off-by:` line certifying that you have the right to submit the change under the project license.

## Commits and pull requests

- Small, reviewable pull requests, one subject each.
- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/), for example `feat(import/qet): read folio reports` or `fix(gui): keep selection after undo`.
- Describe what changed, why, and how it was tested.

## Reporting a file that does not import correctly

Open an issue with the source program and version, the file type and the import report. If you can, attach a **minimal** file that reproduces the problem, without proprietary content.

## Security

Do not open public issues for vulnerabilities. See [SECURITY.md](SECURITY.md).
