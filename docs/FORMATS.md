# File formats and interoperability

Start at [cadcraft-format/src/lib.rs](../crates/cadcraft-format/src/lib.rs) for
native packages and [cadcraft-import/src/lib.rs](../crates/cadcraft-import/src/lib.rs)
for ImportReport. The core processes readers, writers or byte slices; the GUI
handles dialogs, file locks, atomic replacement, recovery and library folders.

## Native project: .cadcraft

The current writer uses **format 10, min_reader 10**. Version 0 is the old technical
JSON snapshot. Versions 1–10 use a ZIP with the uncompressed mimetype entry first,
manifest.json (format, min_reader, writer) and project.json. The MIME type is
application/x-cad-craft-project+zip. Image bytes introduced in version 3 are stored
under assets/; references remain in the project data.

read_project detects ZIP versus legacy JSON; read_project_from_slice is the
in-memory counterpart. write_project validates the complete project before
writing. The reader does not extract archive paths to disk: unsafe names,
duplicate entries, missing required content, excessive sizes and invalid models
are rejected. Limits include 4,096 entries, 1 GiB total unpacked size, a 64 KiB
manifest and a 256 MiB legacy JSON input limit. These are safety ceilings, not
performance guarantees for projects of that size.

### Version chain

The migrate function advances every vN to vN+1 before validated model decoding.
Most transitions introduce optional fields with serde defaults, rather than
rewriting existing values. The fixtures below are under
[crates/cad-craft/tests/fixtures](../crates/cad-craft/tests/fixtures/).

| Version | Introduced content / migration from previous version | Fixture |
|---|---|---|
| 0 | Technical JSON snapshot, before the native container | etapa12-v0-snapshot.json |
| 1 | ZIP envelope; v0 must be a JSON object and retains its project content | etapa12-v1.cadcraft |
| 2 | Frames, page frame choices and reference format, with defaults | etapa13-v2.cadcraft |
| 3 | Images, image frame elements and watermark, optional in old projects | etapa14-v3.cadcraft |
| 4 | Free page text and graphics on representations, optional | etapa16-v4.cadcraft |
| 5 | Symbol styles, ellipses and dynamic text, optional | etapa17-v5.cadcraft |
| 6 | Parts and assignments, optional | etapa18-v6.cadcraft |
| 7 | Numbering scheme and labels, defaulted for older entities | etapa20-v7.cadcraft |
| 8 | Optional report definition on pages | etapa23-v8.cadcraft |
| 9 | Optional check scheme; legacy connector mating records without owned connections are completed during model loading | etapa24-v9.cadcraft |
| 10 | Zone divisions generated from the zones (`ZoneLayout::generated_marks`); fixed division lines of older frames that coincide with zone ends are converted on load (ADR-0033) | etapa30-v10.cadcraft |

A loaded result exposes source_format, migrated(), is_legacy_snapshot() and
read_only. Legacy snapshots need Save As to become native containers. An older
native project is migrated on load and saved in the current format. A future
format whose min_reader exceeds this reader is rejected. If the manifest permits
this reader and the payload can be decoded, a newer format is opened read-only
to prevent destructive overwrites; strict project decoding may still reject
unknown fields. Do not assume every future file can be opened.

No project-format version changes in the R2 documentation block. Undo history,
GUI preferences and assistant credentials are not project-format content.

## Native reusable libraries

The library formats use their own manifest version in a ZIP, with mimetype first: `.cadsym`, `.cadmacro` and `.cadpart` are **format 1**; `.cadframe` is **format 2** since Etapa 30 (version 1 frames have their fixed division lines converted on load, as in project format 10). They are not project-format files. Readers apply the
container safety checks and validate decoded data. See the corresponding
read_*_package / write_*_package APIs in cad_craft::persistence.

| Extension | Payload | MIME suffix after application/x-cad-craft- | Scope |
|---|---|---|---|
| .cadsym | symbol.json | symbol+zip | One SymbolDefinition: primitives, ports, styles and dynamic text fields |
| .cadmacro | macro.json | macro+zip | One MacroDefinition, with captured entities and supported embedded resources; insertion remaps identities |
| .cadpart | part.json | part+zip | One Part, including technical properties and function templates |
| .cadframe | frame.json plus assets/ image bytes | frame+zip | FramePackage: template and referenced images |

Native packages are the interchange for CAD-CRAFT libraries. They do not contain
a vendor library or promise that external-format export can reproduce all fields.

## External readers

Conversion targets the neutral model. Whole-project QET import creates a new
Project. Frame and symbol imports produce reusable definitions/packages; EDZ
produces parts and macros, and CSV produces catalog parts. Callers decide whether
to apply the result, resolve conflicts and store it in project/library state.

| Input | Entry point under cad_craft::interchange | Supported content | Known losses and approximations |
|---|---|---|---|
| QElectroTech .qet (XML) | qet::import | Sheets, embedded ELMT definitions, frames/title blocks, device/function occurrences, master/slave relationships, conductor routes and locked wire numbers, sheet reports and free text | Nonembedded title blocks use a simple fallback; unsupported free shapes/images, unconvertible definitions, unresolved terminals/references and unsupported dynamic fields are reported. Coordinates are scaled from pixels and Y is flipped. |
| QElectroTech .elmt (XML) | elmt::import | Symbol geometry, styles, named ports and supported dynamic texts | Unsupported elements, invalid terminals and unsupported text fields are reported. Rounded rectangles become straight corners; elliptical arcs may become circular; arrow ends, some colors and fixed text rotations are approximated; duplicate port names are renamed. |
| QElectroTech .titleblock (XML) | titleblock::import | Grid-based labels, variables and supported logos, placed on a caller-selected sheet with border/zones | Unsupported/invalid logos and cells outside the grid are reported; variables without a model equivalent remain reported or approximated. Pixel sizes use a configurable scale. |
| EPLAN .fn1 (XML PXF) | fn1::import | Sheet frame geometry, zones, texts, known special property fields and supported embedded images | Unknown object types, user fields and objects with missing geometry are ignored with notes; unsupported styles or field mappings can be approximated. This is a frame reader, not an EPLAN project reader. |
| EPLAN .edz (ZIP or 7z) | edz::import | Manufacturer part data and supported 2D PXF macros, inline geometry or supported fallback symbols, mapped functions and ports | Embedded ESymLib definitions still need a demonstrative sample; 3D-only variants, additional variants, unmapped styles/dynamic text, unsupported PXF links and uncertain multiport positions are reported. Required missing references can reject an archive. It does not import a full EPLAN project. |
| ASCII .dxf | dxf::import | Frame LINE, LWPOLYLINE/POLYLINE including bulges, CIRCLE, ARC, simple TEXT/MTEXT, attributes and expanded INSERT blocks; known text markers become special fields | 3D/mesh geometry and unsupported entities are reported; block nesting is bounded, missing blocks are reported. Units/options and field mappings must be checked. It is not a general drawing-to-electrical-project importer. |
| Part catalog .csv | import_parts_csv | Supported catalog columns and part function templates | CsvPartReport returns accepted parts and rejected rows; it is a row-oriented report, not ImportReport. CSV is a catalog subset, not a project or full library backup. |

Compressed QET .qetz and full EPLAN project formats (.elk, .zw1, .epj) are not
implemented. DWG input is not an importer. No import silently grants electrical
meaning to arbitrary geometry.

### ImportReport and conversion decisions

ImportReport stores (ImportNote, count) pairs, grouping equal notes. NoteKind is
Ignored, Approximated or Info; count(kind) totals occurrences and lines() gives a
human-readable report. Successful import can therefore still be incomplete.
Read the report before accepting a conversion. Malformed or unsafe input can
instead return ImportError; importers must never panic on external data.

The lists above describe supported subsets, not exhaustive vendor-format
coverage. The notes emitted for the actual input are the authoritative conversion
evidence. Frame DXF export also returns ImportReport, although it is a writer.
Part CSV, page DXF and PDF have distinct report types described here or below.

## Writers and target limitations

| Output | Entry point | Written content and limitations |
|---|---|---|
| Native packages | persistence::write_project / write_*_package | Validated model/library content in the current native version; use these for editable CAD-CRAFT data |
| Frame DXF | interchange::dxf::export | R12 frame geometry in mm, special-field markers and CAD-CRAFT zone metadata in comments; images cannot be represented and are reported in ImportReport |
| Page DXF | dxf_export::export_dxf_pages | One R12 AC1009 file per page, from the display list; geometry is exploded, categorized by layer, fills use SOLID, images become marked boxes. DxfReport counts pages/entities, escaped characters and replaced images. It does not preserve electrical entity identities. |
| PDF | pdf::export_pdf | Vector multipage sheets with document/page bookmarks, cross-reference links, JPEG/PNG images and optional print paper/fit settings; PdfReport counts missing images, links omitted outside scope and characters replaced by ? because Helvetica WinAnsi cannot encode them |
| Report CSV | report_export::export_csv | ReportTable columns and rows, UTF-8 BOM, semicolon-separated; a derived table, without editable project topology or sheet geometry |
| Report XLSX | report_export::export_xlsx | ReportTable written with typed numbers and text, formatted headings; a derived workbook, without project relations or graphical content |
| Catalog / BOM CSV | interchange::export_parts_csv / export_bom_csv | Catalog or derived bill-of-materials rows; same CSV subset caveat as above |
| DWG (GUI only) | GUI dwg.rs | External ODA File Converter installed/configured by the user converts exported DXF. No built-in DWG encoder, network dependency or test-time converter requirement. |

PDF and page DXF share exportable content from PageDisplayList. The screen adds
interactive aids separately. Printing uses PDF generation and the operating
system's print workflow. Report exports return errors for failed output; they do
not use ImportReport. An exported sheet or list is not a lossless project backup.

## Synthetic examples and compatibility checks

Run cargo test -p cad-craft --examples --locked for all six
[examples](../crates/cad-craft/examples/). The last example imports
[entry.qet](../crates/cad-craft/tests/fixtures/synthetic/entry.qet), created for
this repository; its unsupported rectangle intentionally produces a loss note.
All automated inputs are synthetic. See [optional sample tests](TESTES_COM_AMOSTRAS.md)
for local files supplied by the user, which must stay outside the repository.

[persistence.rs](../crates/cad-craft/tests/persistence.rs) checks old-version
fixtures and defensive parsing. [baseline_outputs.rs](../crates/cad-craft/tests/baseline_outputs.rs)
pins exact native/PDF/DXF/display-list outputs of fixed synthetic projects.
Every format bump must include the previous-version migration and a fixture;
intentional baseline changes require an explanation and explicit regeneration.
