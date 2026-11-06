# Source tour

Read [architecture](ARCHITECTURE.md) first. This route follows the neutral model
through storage, display, output, input and the desktop application. File links
below are relative to this document and point at the implementation, not generated
API documentation. Keep [the glossary](GLOSSARY.md) open for domain terms.

## 1. cadcraft-model: the vocabulary and invariants

- [domain/project.rs](../crates/cadcraft-model/src/domain/project.rs): Project owns entity registries; begin at new and validate, then follow the subject submodules. Entity types are grouped in electrical/, physical/, document/, symbols/, checks/ and geometry/; Project operations use matching subject directories.
- [domain/project/symbols/placement.rs](../crates/cadcraft-model/src/domain/project/symbols/placement.rs): follow place_symbol and restore_placement to see a complete reversible edit.
- [domain/project/electrical/connectivity.rs](../crates/cadcraft-model/src/domain/project/electrical/connectivity.rs): explicit connections, are_connected and derived networks; graphics never imply connectivity.

Next try examples 01_new_project and 03_connect below. The domain module
re-exports the value types you encounter. Millimetre comparison helpers live in domain::tolerance; identity and orthogonal-route checks remain exact. Constructors reject invalid values;
Project checks the relationships between entities.

## 2. cadcraft-format: persist only valid models

- [src/lib.rs](../crates/cadcraft-format/src/lib.rs): start at FORMAT, write_project, read_project and migrate; then read the native library package sections.
- [persistence tests](../crates/cad-craft/tests/persistence.rs): synthetic containers, malformed inputs and old-version fixtures. These cross-crate tests live in the facade.

The crate currently has one source module. [Formats](FORMATS.md) maps every
version to a fixture. File locking and atomic replacement belong to the GUI.

## 3. cadcraft-render: describe a sheet

- [display.rs](../crates/cadcraft-render/src/display/mod.rs): DisplayItem, DisplayList and PageDisplayList, then the builder that resolves frames, symbols, wires and references.
- [font.rs](../crates/cadcraft-render/src/font.rs): shared text widths and supported font encoding.

Try 02_place_symbol. Notice that the display list contains exportable geometry,
while selection handles and other interaction aids belong to the viewport.

## 4. cadcraft-export: consume the model and display list

- [pdf.rs](../crates/cadcraft-export/src/pdf/mod.rs): PdfOptions, PdfReport and export_pdf; trace page scope, references, fonts and images.
- [dxf_export.rs](../crates/cadcraft-export/src/dxf_export/mod.rs): per-page DXF R12 and DxfReport.
- [report_export.rs](../crates/cadcraft-export/src/report_export.rs): the same ReportTable becomes CSV or XLSX.

Try 05_export_pdf. Read the result report as well as the bytes; target formats
cannot represent everything in the project.

## 5. cadcraft-import: translate external data with an explicit report

- [lib.rs](../crates/cadcraft-import/src/lib.rs): ImportReport, NoteKind and frame format dispatch; also re-exports the part CSV functions.
- [qet.rs](../crates/cadcraft-import/src/qet/mod.rs): a whole QElectroTech project, embedded symbols, linked occurrences and conductor topology.
- [edz/mod.rs](../crates/cadcraft-import/src/edz/mod.rs): manufacturer archive entry point; follow archive, parts and pxf for bounded decoding and conversion.

Try 06_import_qet. ELMT, titleblock, FN1 and frame DXF readers sit beside qet.rs;
consult [formats](FORMATS.md) for supported subsets and loss reporting. Synthetic
tests are under crates/cad-craft/tests; optional personal samples stay outside Git.

## 6. cadcraft-gui: wire the editor to domain operations

- [app.rs](../crates/cadcraft-gui/src/app/mod.rs): CadApp owns application state and orchestrates frames and command handling.
- [state.rs](../crates/cadcraft-gui/src/state/mod.rs): Session and editing dispatch; follow the history module for undo/redo transactions.
- [viewport.rs](../crates/cadcraft-gui/src/viewport.rs): camera, sheet content and pointer interaction; panels are separate modules.

Read commands.rs when adding an action and project_file.rs for save/recovery.
GUI tests in crates/cadcraft-gui/tests drive egui via AccessKit. The assistant's
network boundary is the assistant/ directory, described below.

## The two remaining crates

cad-craft is the facade used by the examples and GUI:
[lib.rs](../crates/cad-craft/src/lib.rs) gives the stable import paths. Its
[Cargo.toml](../crates/cad-craft/Cargo.toml) shows the re-exported crates; the
[integration tests](../crates/cad-craft/tests/) exercise their combined contracts.

cadcraft-assistant keeps help, queries and proposals transport-free:
[lib.rs](../crates/cadcraft-assistant/src/lib.rs) lists modules,
[tools.rs](../crates/cadcraft-assistant/src/tools.rs) handles project tools, and
[proposal.rs](../crates/cadcraft-assistant/src/proposal.rs) builds validated
proposals. Browser login and requests are implemented only in the GUI's
[assistant directory](../crates/cadcraft-gui/src/assistant/).

## Run the examples

From the repository root:

~~~powershell
cargo run -p cad-craft --example 01_new_project --locked
cargo run -p cad-craft --example 02_place_symbol --locked
cargo run -p cad-craft --example 03_connect --locked
cargo run -p cad-craft --example 04_number --locked
cargo run -p cad-craft --example 05_export_pdf --locked
cargo run -p cad-craft --example 06_import_qet --locked
cargo test -p cad-craft --examples --locked
~~~

Sources are in [crates/cad-craft/examples](../crates/cad-craft/examples/).
Each is under 100 lines, has assertions and a test that runs main. Examples 01
and 05 create a uniquely named directory in the system temporary directory,
verify the saved output, then remove that directory on success. The QET example
uses the synthetic fixture from the integration tests and prints its loss report.
No example needs a GUI, account, network or third-party file. Example 04 applies
numbering and restores the exact prior project, including IDs.

Before changing code, read [CONTRIBUTING](../CONTRIBUTING.md) and AGENTS.md, then
run the full validation command from [architecture](ARCHITECTURE.md). Tests for
source module introductions prevent future source files from losing their entry
point comments; exact-output baselines protect compatibility during reorganization.
