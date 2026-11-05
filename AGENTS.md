# Instructions for coding agents — CAD-CRAFT

These are the technical rules of the project, for human contributors and AI coding agents alike. Read [CONTRIBUTING.md](CONTRIBUTING.md) too.

If an `AGENTS.local.md` file exists next to this one, read it first: it holds the local workflow rules of the person running the agent (approvals, Git policy, session hand-off). It is not versioned.

## Architecture

- Rust workspace with dependencies in one direction only:
  - `cadcraft-model`: neutral project model; depends on no other project crate.
  - `cadcraft-format` (native files and migrations), `cadcraft-render` (display list), `cadcraft-import` (other platforms) and `cadcraft-export` (PDF, DXF, lists) depend on the model; `export` also on `render`.
  - `cad-craft`: facade that re-exports the crates above with stable paths (`cad_craft::domain`, `::persistence`, `::interchange`…).
  - `cadcraft-gui`: the egui/eframe application (library `cad_gui`, executable `cad-gui`). No core crate depends on the GUI.
- Methods of `Project` can only be declared in `cadcraft-model`; other crates extend it with traits (for example `PageDisplayList`).
- Internal units are millimetres; origin at the bottom left; X to the right and Y up.
- Typed IDs with UUID v7 where applicable.
- Components, devices and representations are separate; logical connections and physical conductors are separate; networks are derived from connectivity.
- Every edit goes through a validated, reversible change (delta), so undo and redo restore the exact state. The core validates every relation; never bypass its checks.
- File formats are versioned. A new version always ships with a migration from the previous one and a fixture.
- Importers turn external files into the neutral `Project` model and report what was lost or approximated (`ImportReport`). They must never panic on external input.
- Do not create empty modules ahead of the feature that needs them.

## Validation

Before asking for review, run:

```powershell
pwsh -NoProfile -File scripts/validate.ps1
```

It runs `cargo fmt --check`, `cargo check --workspace --all-targets --locked`, `cargo test --workspace --locked`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, the PowerShell tests and `git diff --check`, stopping at the first failure.

## Tests

- Add tests relevant to every change; never remove or weaken tests to hide failures.
- Automated tests use only synthetic files. Real files from other platforms are proprietary or personal: they never enter the repository. Optional tests read them from `CADCRAFT_SAMPLES` (see [docs/TESTES_COM_AMOSTRAS.md](docs/TESTES_COM_AMOSTRAS.md)).
- `crates/cad-craft/tests/baseline_outputs.rs` pins the exact outputs of fixed projects. A reorganization must keep them identical; an intentional output change is regenerated with `CAD_CRAFT_WRITE_BASELINE=1` and explained.
- GUI tests drive the real egui application through AccessKit nodes.

## Code style and language

- Code, file and module names, API comments (`///`, `//!`) and commit messages in English. User interface text in Brazilian Portuguese for now. Older Portuguese comments are converted gradually.
- Every module starts with a `//!` comment saying what it does and where to start reading.
- Prefer `expect("reason")` to `unwrap()` outside tests.
- Keep files small (around 800 lines at most as a goal) and split by subject.

## Licensing and data

- The project is GPL-3.0-or-later. Every `.rs` file starts with the SPDX header (checked by a test); new dependencies must be license-compatible (`deny.toml`).
- Never commit credentials, tokens, `.env` files, backups, build artifacts or third-party files (EPLAN, QElectroTech or manufacturer data).
- Do not copy proprietary code. Knowledge of external formats is used only for interoperability.

## Documentation

- Architecture decisions: `docs/adr/`.
- Stage specs, plans, reports and project memory: `docs/internal/`.
