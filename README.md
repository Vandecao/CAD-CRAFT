# CAD-CRAFT

**Open-source electrical CAD for industrial control panels, built to exchange projects between platforms.**

[Português](README.pt-BR.md) · License: [GPL-3.0-or-later](LICENSE) · Status: **alpha**

CAD-CRAFT is a Windows desktop application, written in Rust, for drawing and documenting industrial electrical schematics: devices, connections, terminals, cables, cross references, reports and checks. Its goal is to keep projects from being locked into a single vendor: it reads and writes formats from other electrical engineering platforms into one neutral project model.

> A screenshot of the application will be added here.

## Exchange between platforms

Everything goes into the neutral `Project` model and everything comes out of it. Each import reports what was lost or approximated.

### Import

| Source | Extension | What is imported | Status |
|---|---|---|---|
| QElectroTech | `.qet` | Project: sheets, frames, elements, conductors with wire numbers, sheet references, texts | Implemented |
| QElectroTech | `.qetz` | Compressed project | Planned |
| QElectroTech | `.elmt` | Symbol (element) | Implemented |
| QElectroTech | `.titleblock` | Frame / title block | Implemented |
| EPLAN | `.edz` | Parts and 2D macros from manufacturer packages | Implemented (embedded ESymLib symbol definitions pending a sample) |
| EPLAN | `.fn1` | Frame (sheet form) | Implemented |
| EPLAN | full project (`.elk`, `.zw1`, `.epj`) | Project | **Not yet**, next major exchange milestone |
| Generic CAD | `.dxf` | Frame | Implemented |
| Spreadsheet | `.csv` | Parts catalog | Implemented |

### Export

| Target | Extension |
|---|---|
| Vector PDF, multi-page, with links and bookmarks; printing | `.pdf` |
| DXF R12, one file per page | `.dxf` |
| DWG, through the ODA File Converter installed by the user | `.dwg` |
| Lists and reports | `.csv`, `.xlsx` |
| Frame | `.dxf` |

### Native formats

| Extension | Content |
|---|---|
| `.cadcraft` | Project (ZIP with manifest and JSON; versioned, with migrations from every earlier version) |
| `.cadsym` | Symbol |
| `.cadmacro` | Macro |
| `.cadpart` | Part |
| `.cadframe` | Frame |

*EPLAN is a registered trademark of EPLAN GmbH & Co. KG. QElectroTech is an independent project. CAD-CRAFT is not affiliated with either of them and does not distribute files, libraries or data from these platforms: users open their own files.*

## Features

- Schematic editor with symbols, typed ports and orthogonal connection routes.
- Devices, functions and parts kept apart from their graphical representations; logical connections kept apart from physical conductors; networks derived from connectivity.
- Symbol, part and macro libraries and editors.
- Frames with zones and column-based cross references; frame editor.
- Device, wire and page numbering with locked numbers.
- Terminal strips, bridges, cables, cores and industrial connectors.
- Page reports (index, devices, bill of materials, connections, terminals, cables) and CSV/XLSX lists.
- Consistency checks with navigation to each problem.
- Transactional undo/redo for every edit.

## Build and run

Requirements: Windows 10/11 x64, [Rustup](https://rustup.rs) and the Microsoft C++ Build Tools (Desktop C++ workload with the Windows SDK). The toolchain version is pinned in `rust-toolchain.toml` and installed by Rustup automatically.

```powershell
cargo build --release --locked -p cadcraft-gui
.\target\release\cad-gui.exe
```

Full validation (format, check, tests, Clippy, whitespace and the PowerShell tests):

```powershell
pwsh -NoProfile -File scripts/validate.ps1
```

The core crates have no GUI dependency and also build and test on Linux: `cargo test --workspace --exclude cadcraft-gui`.

## Project status

Alpha. The `.cadcraft` format may still change, always with a migration from earlier versions. Development follows numbered stages with a spec, a plan, an architecture decision record (ADR) and a report for each one.

## Repository layout

Everything goes into the neutral model and everything comes out of it:

```
files from other platforms ──► cadcraft-import ──► cadcraft-model ◄──► cadcraft-format (.cadcraft)
                                                        │
                                                        └──► cadcraft-render ──► screen, cadcraft-export (PDF, DXF)
```

- `crates/cadcraft-model`: neutral project model (entities, connectivity, documents, symbols, numbering, reports, checks).
- `crates/cadcraft-format`: native files and migrations.
- `crates/cadcraft-import`: QElectroTech, EPLAN, DXF and CSV importers.
- `crates/cadcraft-render`: display list shared by the screen, the PDF and the DXF.
- `crates/cadcraft-export`: PDF, DXF and CSV/XLSX exporters.
- `crates/cad-craft`: facade that re-exports the crates above.
- `crates/cadcraft-gui`: desktop application (egui/eframe).
- `docs/adr`: architecture decision records.
- `docs/TESTES_COM_AMOSTRAS.md`: optional tests with your own real files.
- `docs/internal`: specs, plans, reports and project memory of each stage.

## Contributing and license

Contributions are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md), the [code of conduct](CODE_OF_CONDUCT.md) and the [security policy](SECURITY.md).

CAD-CRAFT is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version. See [LICENSE](LICENSE).
