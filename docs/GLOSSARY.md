# Electrical CAD glossary

These terms describe the neutral model, not a particular vendor's database.
Portuguese names match the current Brazilian Portuguese UI. Read
[architecture](ARCHITECTURE.md) for ownership and [the tour](TOUR.md) for source
entry points.

| English | Português | Meaning in CAD-CRAFT |
|---|---|---|
| Project | Projeto | Aggregate that owns entities, validates relations and derives networks and engineering lists. |
| Document | Documento | Ordered group of project pages. |
| Page / sheet | Página / folha | Drawing or report surface with a number, size, metadata and optional frame. Internal coordinates are millimetres with Y up. |
| Component | Componente | Physical item, with optional links to a device and function and an optional catalog part and quantity. It is distinct from both the logical device and a drawing occurrence. |
| Device | Dispositivo | Logical equipment identity, such as a relay. It owns functions and can appear on several sheets without being duplicated. |
| Representation / occurrence | Representação / ocorrência | Graphical occurrence on a page, with its own ID, target, transform and visual settings. A target may be a device, function or other supported entity; free graphics need no electrical identity. |
| Function | Função | Electrical role of a device (coil, contact, etc.), with main/auxiliary role and connection points. Several functions can belong to one device. |
| Symbol | Símbolo | Reusable geometry and named ports in a SymbolDefinition. Placing it creates a representation; electrical placement also registers a function and point bindings. |
| Port | Porta | Named attachment location in a symbol definition. A PortRef identifies a port on one occurrence and can be bound to a logical connection point. |
| Connection point | Ponto de conexão | Logical terminal of a function. Its optional physical terminal association is validated separately. |
| Logical connection | Conexão lógica | Explicit graph edge between two connection points. It can carry a wire number and have a graphical route; it does not itself describe the physical conductor. |
| Route | Rota / traçado | Polyline occurrence of a logical connection. Explicit junctions and port bindings establish topology; mere crossings do not. |
| Conductor | Condutor | Physical wiring entity linked to logical connectivity, with physical endpoints and attributes. It may be assigned a cable core. |
| Network | Rede | Connected set derived from explicit topology, including isolated points. Networks are recalculated, have no persistent ID and do not follow proximity of drawn lines. |
| Terminal | Borne | Physical connection entity with validated ownership and optional strip membership; supports passage, PE, multilevel, distribution and disconnect specializations. Connector contacts also reuse terminal entities. |
| Terminal strip | Régua de bornes | Ordered grouping of terminals. Terminal numbers must be unique within the strip. |
| Bridge | Ponte | Explicit connection between terminals in a strip, with a TerminalBridge record; it participates in connectivity rather than being only decorative geometry. |
| Connector | Conector | Industrial connector made of terminal contacts. Explicit mating creates owned logical connections for paired contacts that both have associated connection points. Connector membership or equal contact names alone create no continuity. |
| Cable | Cabo | Physical cable grouping identified cores, with designation and technical properties. It is distinct from the logical networks those cores carry. |
| Cable core | Veia | Individually identified member of a cable, which can be assigned to a physical conductor. |
| Frame / title block | Moldura / carimbo | Reusable sheet border, zones, fixed and resolved property texts, images and watermark. A project default or page override chooses the frame. The title block is the information area within it. |
| Macro | Macro | Reusable captured selection or page, with graphics and supported electrical data. Insertion remaps IDs and designations through a reversible transaction. |
| Catalog part | Peça | Reusable manufacturer/catalog data and function templates, assigned to components; separate from an instance of equipment in a project. |
| Designation | Designação | Validated structured identifier, including function/location/product aspects as applicable. It is distinct from a free text label or an entity ID. |
| Cross-reference | Referência cruzada | Derived sheet/zone reference between occurrences of related functions and markers; carries navigation targets and can become a PDF link. |
| Potential | Potencial | Named electrical potential expressed by a marker and group. Continuity follows explicit group/topology rules; a text with the same name alone is not a connection. |
| Delta | Delta / alteração reversível | Saved before/after information from a validated edit, sufficient to undo or redo with the same IDs and precondition checks. |
| Display list | Lista de exibição | Exportable page content represented as paths, text, images, layers and reference links; shared drawing rules exclude interactive editing aids. |
| Import report | Relatório de importação | Counted notes about ignored, approximated or informational conversion results. It accompanies successful external imports and must be reviewed before accepting them. |
