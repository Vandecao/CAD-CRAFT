# Architecture

CAD-CRAFT is an electrical CAD application built around a neutral, validated
Project. Start with [the source tour](TOUR.md), then use [the glossary](GLOSSARY.md)
and [format reference](FORMATS.md) while reading the code. The public APIs are
available through the cad-craft facade; the GUI is a consumer of those APIs.

## Eight crates

The arrows below mean **depends on**, not data flow. Only workspace dependencies
are shown; external libraries are listed in each Cargo.toml.

~~~text
cadcraft-gui (library cad_gui, executable cad-gui)
    -> cad-craft (facade)
        -> cadcraft-model
        -> cadcraft-format    -> cadcraft-model
        -> cadcraft-render    -> cadcraft-model
        -> cadcraft-import    -> cadcraft-model
        -> cadcraft-export    -> cadcraft-model + cadcraft-render
        -> cadcraft-assistant -> cadcraft-model
~~~

| Crate | Responsibility | Facade path |
|---|---|---|
| cadcraft-model | Entities, relations, validation, transactions, connectivity, reports and checks | cad_craft::domain, cad_craft::error |
| cadcraft-format | Native packages, defensive reading and version migrations | cad_craft::persistence |
| cadcraft-render | Page display lists and shared font metrics | cad_craft::display, cad_craft::font |
| cadcraft-export | PDF, page DXF and report CSV/XLSX | cad_craft::pdf, cad_craft::dxf_export, cad_craft::report_export |
| cadcraft-import | External formats translated into model entities, with loss reports | cad_craft::interchange |
| cadcraft-assistant | Local help, project queries, proposals and conversation protocol | cad_craft::assistant |
| cad-craft | Stable re-exports; examples and cross-crate integration tests | Facade itself |
| cadcraft-gui | egui/eframe application, interaction, file lifecycle and assistant transport | Separate cad_gui library |

No core crate depends on the GUI. The assistant has a test-only dependency on
cadcraft-format for fixtures; it is not a production dependency. Methods of
Project live only in cadcraft-model. Other crates add extension traits: for
example, PageDisplayList in cadcraft-render. The facade's domain module re-exports
that trait so a domain wildcard import makes page_display_list available.

## Neutral model and ownership

Project owns typed registries and validates relations between them. Device is a
logical identity, Function is an electrical role within it, and Component is a
physical item that can refer to a catalog Part. Representation is an occurrence
on a Page; it refers to an entity and carries graphics and a transform. A device
can have multiple functions and occurrences without becoming multiple devices.

ConnectionPoint belongs to a function. Connection joins logical points;
Conductor describes physical wiring. A drawn route binds symbol ports to points
and refers to a logical connection. Crossing strokes alone does not connect
points. Networks are derived from explicit connectivity, include isolated
points, and have no persisted identity. Requery them after edits.

Internal geometry is in millimetres, origin at bottom left, X right, Y up.
Screen and foreign-file coordinates are converted at their boundaries. Entity
IDs are typed UUID v7 values where applicable; library identifiers such as
SymbolDefinitionId are validated strings.

## Edit, delta, undo and redo

~~~text
GUI command / assistant proposal
    -> Session dispatch / validated model operation
    -> validate references and preconditions
    -> apply transaction, return delta with before/after state and IDs
    -> GUI history stores the delta
undo -> restore_*(delta, false) -> validate expected current state -> prior state
redo -> restore_*(delta, true)  -> validate expected current state -> edited state
~~~

Read model domain/project/symbols/placement.rs for a compact complete example:
place_symbol creates the device (if needed), function, points, symbol occurrence
and port bindings together. restore_placement replays the saved entities and
preserves their IDs. Numbering similarly separates preview, apply and restore.
Batches must be atomic, and replay must reject a conflicting current state.

Low-level insert_* methods validate relationships and are useful for constructing
a new project, as the examples do. Interactive edits use the reversible APIs and
Session history; direct mutation of serialized JSON is not an editing API.
The GUI owns undo/redo history, dirty state and selection, while the model owns
validation and reversible domain changes. History is not saved in project files.

## Display and output

~~~text
external files -> import -> Project <-> native format
                              |
                              v
                       page display list -> PDF / page DXF
                              |
                       shared sheet content
                              v
                       GUI sheet rendering
~~~

The GUI also draws editing aids and handles input. The display list describes
exportable content: paths, text, images, layers and reference links. It excludes
grid, selection, snap hints and port handles. Read render/src/display.rs and then
export/src/pdf.rs to follow a page into output. ReportTable similarly feeds both
page reports and tabular exports. Output-specific losses have their own reports.

## Network boundary and file lifecycle

All assistant network code is confined to
[crates/cadcraft-gui/src/assistant/](../crates/cadcraft-gui/src/assistant/): browser
OAuth, callback handling and requests. cadcraft-assistant operates on data and
has no transport dependency. Tests use synthetic input and never the network.
Authentication state belongs to the GUI session, not the neutral project format.

The format crate reads and writes bounded containers and validates loaded models.
The GUI owns file locks, atomic saves, recent files and recovery copies. External
importers return an ImportReport describing approximations and losses; malformed
input must return errors, never panic. See [formats](FORMATS.md) before adding an
importer or changing a persisted field.

## Checks before review

Run pwsh -NoProfile -File scripts/validate.ps1 from the repository root. It checks
formatting, all workspace targets, tests, runnable facade examples, Clippy,
PowerShell tests and whitespace. baseline_outputs.rs pins exact fixed-project
outputs. GUI tests drive the real egui application through AccessKit nodes.
Architecture decisions are in [docs/adr](adr/); historical implementation records
are in docs/internal. Local contributor workflow rules are in AGENTS.local.md
when present.
