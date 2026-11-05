# R3 — inventário conservador de comparações exatas

Data: 2026-11-05. Este inventário contém todas as 413 linhas com operadores == ou != no modelo, incluindo identidades tipadas, enums, coleções e testes internos. Não representa 413 comparações de f64: é uma lista conservadora para não omitir comparações de pontos/retângulos/vetores que contêm f64. O número aproximado de 69 citado no planejamento anterior não é reproduzido como contagem atual auditada.

Nenhuma dessas comparações foi relaxada. Identidade de ponto/rota, alinhamento ortogonal, limites reconstruídos e ângulos canônicos participam da validação, snapshots e undo/redo exato; trocar igualdade por proximidade alteraria comportamento. Comparações de IDs, enums, tamanhos de coleção e texto não são candidatas a tolerância numérica. As correspondências de ângulos por match em geometry/transform.rs e symbols/symbol/primitives.rs também permanecem exatas.

A única centralização numérica neste bloco foi substituir a constante privada de molduras de 1e-6 por EPS_MM, de mesmo valor. Os predicados e limites de zonas continuam idênticos, cobertos por tests/frames.rs e tests/tolerance.rs. nearly_equal/is_zero têm tolerância absoluta inclusiva, testes de fronteira e rejeição de NaN/infinito; não foram usados para alterar identidade ou topologia.

Para uma futura mudança comportamental, selecionar a regra, escrever teste do contrato pretendido e avaliar explicitamente migração/efeito em saídas. A presença de teste de caminho comum não torna segura a troca de igualdade por tolerância.

| Arquivo e linha (estado R3) | Expressão preservada |
|---|---|
| crates/cadcraft-model/src/document_index.rs:19 | if representation.target() != RepresentationTarget::Unassociated { |
| crates/cadcraft-model/src/domain/checks/check.rs:110 | CHECK_RULES.iter().find(&#124;r&#124; r.code == code) |
| crates/cadcraft-model/src/domain/checks/check.rs:211 | self.ignored.iter().any(&#124;i&#124; i.key == key) |
| crates/cadcraft-model/src/domain/checks/check.rs:245 | .filter(&#124;m&#124; m.severity == severity) |
| crates/cadcraft-model/src/domain/checks/report.rs:114 | self.before == self.after |
| crates/cadcraft-model/src/domain/document/asset.rs:48 | if w == 0 &#124;&#124; h == 0 &#124;&#124; w > MAX_IMAGE_SIDE &#124;&#124; h > MAX_IMAGE_SIDE { |
| crates/cadcraft-model/src/domain/document/asset.rs:61 | if b.len() < 24 &#124;&#124; &b[12..16] != b"IHDR" { |
| crates/cadcraft-model/src/domain/document/asset.rs:69 | if b[i] != 0xFF { |
| crates/cadcraft-model/src/domain/document/asset.rs:73 | if marker == 0xFF { |
| crates/cadcraft-model/src/domain/document/asset.rs:81 | if marker == 0xD9 &#124;&#124; marker == 0xDA { |
| crates/cadcraft-model/src/domain/document/asset.rs:127 | if d.width_px == 0 |
| crates/cadcraft-model/src/domain/document/asset.rs:128 | &#124;&#124; d.height_px == 0 |
| crates/cadcraft-model/src/domain/document/asset.rs:131 | &#124;&#124; d.byte_len == 0 |
| crates/cadcraft-model/src/domain/document/asset.rs:182 | self.bytes().is_some_and(&#124;b&#124; b == bytes) |
| crates/cadcraft-model/src/domain/document/asset.rs:192 | if format != self.format |
| crates/cadcraft-model/src/domain/document/asset.rs:193 | &#124;&#124; w != self.width_px |
| crates/cadcraft-model/src/domain/document/asset.rs:194 | &#124;&#124; h != self.height_px |
| crates/cadcraft-model/src/domain/document/asset.rs:195 | &#124;&#124; bytes.len() as u64 != self.byte_len |
| crates/cadcraft-model/src/domain/document/frame/mod.rs:86 | let last = i + 1 == self.sizes.len(); |
| crates/cadcraft-model/src/domain/document/frame/mod.rs:298 | Self::Line { start, end, width } => start != end && width_ok(width), |
| crates/cadcraft-model/src/domain/document/frame/mod.rs:316 | && *sweep_degrees != 0.0 |
| crates/cadcraft-model/src/domain/document/page_graphic.rs:54 | if !matches!(primitive, SymbolPrimitive::Text { .. }) && alignment != TextAlignment::Left { |
| crates/cadcraft-model/src/domain/electrical/component.rs:30 | *value == 1 |
| crates/cadcraft-model/src/domain/electrical/component.rs:90 | if quantity == 0 { |
| crates/cadcraft-model/src/domain/electrical/connection.rs:81 | if data.source == data.target { |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:96 | if d.source == d.target { |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:102 | if intent.points()? != routing_base { |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:113 | if !d.auto_clearance && expected != d.points { |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:187 | if points != self.points { |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:223 | if points.first() != self.points.first() &#124;&#124; points.last() != self.points.last() { |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:226 | self.auto_clearance = points != self.points; |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:234 | if intent.points()? != self.points { |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:250 | if da == db { |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:268 | let base = if da == db { |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:336 | if anchors.len() < 2 &#124;&#124; anchors.len() > 4096 &#124;&#124; horizontal_first.len() + 1 != anchors.len() |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:369 | if horizontal_first.len() != anchors.len().saturating_sub(1) { |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:375 | if a.x() != b.x() && a.y() != b.y() { |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:388 | if a == old[0] && b == old[old.len() - 1] { |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:392 | if old.len() == 2 { |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:393 | if a.x() != b.x() && a.y() != b.y() { |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:394 | points.push(if old[0].y() == old[1].y() { |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:402 | points.push(if old[0].y() == first.y() { |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:409 | points.push(if last.y() == old[old.len() - 1].y() { |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:423 | if old.len() >= 4 && a != old[0] && points.len() >= 4 { |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:425 | if points[2] == first_authored && collinear_axis(points[1], points[2], points[3]) { |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:429 | if old.len() >= 4 && b != old[old.len() - 1] && points.len() >= 4 { |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:432 | if points[n - 3] == last_authored |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:442 | (a.x() == b.x() && b.x() == c.x()) &#124;&#124; (a.y() == b.y() && b.y() == c.y()) |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:445 | (a.x() == b.x() && p.x() == a.x() && p.y() >= a.y().min(b.y()) && p.y() <= a.y().max(b.y())) |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:446 | &#124;&#124; (a.y() == b.y() |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:447 | && p.y() == a.y() |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:456 | .any(&#124;s&#124; s[0] == s[1] &#124;&#124; (s[0].x() != s[1].x() && s[0].y() != s[1].y())) |
| crates/cadcraft-model/src/domain/electrical/connection_drawing.rs:573 | self.len() == 0 |
| crates/cadcraft-model/src/domain/electrical/designation.rs:48 | .all(&#124;c&#124; c.is_alphanumeric() &#124;&#124; c == '_' &#124;&#124; c == '.') |
| crates/cadcraft-model/src/domain/electrical/identification.rs:89 | self.len() == 0 |
| crates/cadcraft-model/src/domain/electrical/marker.rs:156 | self.len() == 0 |
| crates/cadcraft-model/src/domain/geometry/transform.rs:12 | if angle == 360.0 { 0.0 } else { angle } |
| crates/cadcraft-model/src/domain/geometry/transform.rs:102 | let reflects = outer.mirror_x != outer.mirror_y; |
| crates/cadcraft-model/src/domain/geometry/transform.rs:109 | self.mirror_x != outer.mirror_x, |
| crates/cadcraft-model/src/domain/geometry/transform.rs:110 | self.mirror_y != outer.mirror_y, |
| crates/cadcraft-model/src/domain/geometry/transform.rs:122 | self.rotation_degrees == 90.0 &#124;&#124; self.rotation_degrees == 270.0 |
| crates/cadcraft-model/src/domain/ids.rs:23 | if value.get_version_num() != 7 &#124;&#124; value.get_variant() != Variant::RFC4122 { |
| crates/cadcraft-model/src/domain/physical/conductor.rs:93 | if source == target { |
| crates/cadcraft-model/src/domain/physical/connector.rs:218 | self.first == connector &#124;&#124; self.second == connector |
| crates/cadcraft-model/src/domain/physical/connector.rs:223 | if p.first == contact { |
| crates/cadcraft-model/src/domain/physical/connector.rs:225 | } else if p.second == contact { |
| crates/cadcraft-model/src/domain/physical/part.rs:16 | if value.trim() != value |
| crates/cadcraft-model/src/domain/physical/part.rs:76 | &#124;&#124; points.iter().collect::<BTreeSet<_>>().len() != points.len() |
| crates/cadcraft-model/src/domain/physical/part.rs:167 | .any(&#124;earlier&#124; earlier.kind == function.kind && earlier.points == function.points) |
| crates/cadcraft-model/src/domain/physical/terminal.rs:79 | *self == Self::Generic |
| crates/cadcraft-model/src/domain/physical/terminal.rs:237 | self.levels.iter().find(&#124;l&#124; l.id() == id) |
| crates/cadcraft-model/src/domain/physical/terminal.rs:268 | if !self.levels.is_empty() && self.kind != TerminalKind::MultiLevel { |
| crates/cadcraft-model/src/domain/physical/terminal.rs:271 | if self.disconnect_state.is_some() != (self.kind == TerminalKind::Disconnect) { |
| crates/cadcraft-model/src/domain/physical/terminal.rs:277 | if self.positions.len() < 2 &#124;&#124; self.sides.len() != self.positions.len() { |
| crates/cadcraft-model/src/domain/physical/terminal.rs:281 | if sides.len() != 2 { |
| crates/cadcraft-model/src/domain/physical/terminal.rs:315 | if seen.len() != self.positions.len() |
| crates/cadcraft-model/src/domain/project/checks/operations.rs:18 | self.before == self.after |
| crates/cadcraft-model/src/domain/project/checks/operations.rs:45 | if &self.check_scheme != expected { |
| crates/cadcraft-model/src/domain/project/checks/operations.rs:67 | .is_some_and(&#124;(p, _)&#124; p == page) |
| crates/cadcraft-model/src/domain/project/checks/operations.rs:101 | CheckTarget::Device(id) => first(&&#124;r&#124; self.device_for_target(r.target()) == Some(id)), |
| crates/cadcraft-model/src/domain/project/checks/operations.rs:103 | first(&&#124;r&#124; self.function_for_target(r.target()) == Some(id)) |
| crates/cadcraft-model/src/domain/project/checks/operations.rs:108 | .filter(&#124;r&#124; r.connection_id() == id) |
| crates/cadcraft-model/src/domain/project/checks/operations.rs:115 | .filter(&#124;(_, point)&#124; **point == id) |
| crates/cadcraft-model/src/domain/project/checks/operations.rs:121 | .filter(&#124;p&#124; p.terminal_id() == Some(id)) |
| crates/cadcraft-model/src/domain/project/checks/operations.rs:202 | let has_main = functions.iter().any(&#124;f&#124; f.role() == FunctionRole::Main); |
| crates/cadcraft-model/src/domain/project/checks/operations.rs:205 | .filter(&#124;f&#124; f.role() == FunctionRole::Auxiliary) |
| crates/cadcraft-model/src/domain/project/checks/operations.rs:368 | if terminal.disconnect_state() == Some(DisconnectState::Open) { |
| crates/cadcraft-model/src/domain/project/checks/reports.rs:17 | filter.document.is_none_or(&#124;id&#124; id == document) |
| crates/cadcraft-model/src/domain/project/checks/reports.rs:27 | if needed == count { |
| crates/cadcraft-model/src/domain/project/checks/reports.rs:47 | if frame.sheet() != size { |
| crates/cadcraft-model/src/domain/project/checks/reports.rs:69 | let self_in_index = request.kind == ReportKind::PageIndex |
| crates/cadcraft-model/src/domain/project/checks/reports.rs:118 | .filter(&#124;p&#124; p.report().is_some_and(&#124;r&#124; r.group == group)) |
| crates/cadcraft-model/src/domain/project/checks/reports.rs:126 | let self_in_index = def.kind == ReportKind::PageIndex |
| crates/cadcraft-model/src/domain/project/checks/reports.rs:134 | Ok(pages.len() != needed) |
| crates/cadcraft-model/src/domain/project/checks/reports.rs:141 | .filter(&#124;p&#124; p.report().is_some_and(&#124;r&#124; r.group == group)) |
| crates/cadcraft-model/src/domain/project/checks/reports.rs:154 | let self_in_index = def.kind == ReportKind::PageIndex |
| crates/cadcraft-model/src/domain/project/checks/reports.rs:241 | if candidate.pages.get(&page.id()) != Some(page) { |
| crates/cadcraft-model/src/domain/project/checks/reports.rs:268 | filter.document.is_none_or(&#124;id&#124; page.document_id() == id) |
| crates/cadcraft-model/src/domain/project/checks/reports.rs:269 | && filter.page.is_none_or(&#124;id&#124; page.id() == id) |
| crates/cadcraft-model/src/domain/project/checks/reports.rs:276 | .all(&#124;(p, f)&#124; p.as_str() == f) |
| crates/cadcraft-model/src/domain/project/checks/reports.rs:283 | .all(&#124;(p, f)&#124; p.as_str() == f) |
| crates/cadcraft-model/src/domain/project/checks/reports.rs:362 | .filter(&#124;b&#124; b.device == device.id()) |
| crates/cadcraft-model/src/domain/project/checks/reports.rs:552 | let (here, other) = if a.terminal_id() == t.terminal { |
| crates/cadcraft-model/src/domain/project/checks/reports.rs:619 | for core in physical.cores.iter().filter(&#124;core&#124; core.cable == c.cable) { |
| crates/cadcraft-model/src/domain/project/checks/reports.rs:634 | .filter(&#124;core&#124; core.cable == c.cable) |
| crates/cadcraft-model/src/domain/project/document/clipboard.rs:120 | .any(&#124;r&#124; r.source().representation_id() == id &#124;&#124; r.target().representation_id() == id) |
| crates/cadcraft-model/src/domain/project/document/clipboard.rs:203 | .filter(&#124;(p, _)&#124; p.representation_id() == id) |
| crates/cadcraft-model/src/domain/project/document/clipboard.rs:263 | .any(&#124;i&#124; i.representation.project_id() != self.id) |
| crates/cadcraft-model/src/domain/project/document/clipboard.rs:352 | if created.visual() != &wanted { |
| crates/cadcraft-model/src/domain/project/document/clipboard.rs:395 | if self.port_bindings.get(port) != Some(point) { |
| crates/cadcraft-model/src/domain/project/document/clipboard.rs:401 | && !remove.iter().any(&#124;(p, _)&#124; p == port) |
| crates/cadcraft-model/src/domain/project/document/documentation.rs:52 | if expected != given &#124;&#124; given.len() != order.len() &#124;&#124; u32::try_from(order.len()).is_err() { |
| crates/cadcraft-model/src/domain/project/document/documentation.rs:93 | .any(&#124;p&#124; p.id() != id && p.report().is_some_and(&#124;r&#124; r.filter.page == Some(id))) |
| crates/cadcraft-model/src/domain/project/document/documentation.rs:106 | .filter(&#124;p&#124; Some(p.id()) != skip_page) |
| crates/cadcraft-model/src/domain/project/document/documentation.rs:122 | .any(&#124;props&#124; props.references().any(&#124;r&#124; r == target)) |
| crates/cadcraft-model/src/domain/project/document/document_edit.rs:33 | matches!(self, Self::Order { before, after, .. } if before == after) |
| crates/cadcraft-model/src/domain/project/document/document_edit.rs:96 | if self.document(document.id()) != Some(document) { |
| crates/cadcraft-model/src/domain/project/document/document_edit.rs:103 | let insert = forward == matches!(delta, DocumentDelta::PageInserted(_)); |
| crates/cadcraft-model/src/domain/project/document/document_edit.rs:107 | if self.page(page.id()) != Some(page) { |
| crates/cadcraft-model/src/domain/project/document/document_edit.rs:123 | if &self.page_orders(*document) != expected { |
| crates/cadcraft-model/src/domain/project/document/document_queries.rs:42 | .filter(&#124;r&#124; r.id() != id) |
| crates/cadcraft-model/src/domain/project/document/frames.rs:9 | *format == ReferenceFormat::default() |
| crates/cadcraft-model/src/domain/project/document/frames.rs:86 | Self::Frame { before, after, .. } => before == after, |
| crates/cadcraft-model/src/domain/project/document/frames.rs:88 | before == after |
| crates/cadcraft-model/src/domain/project/document/frames.rs:90 | Self::ReferenceFormat { before, after } => before == after, |
| crates/cadcraft-model/src/domain/project/document/frames.rs:92 | &#124; Self::PageMetadata { before, after, .. } => before == after, |
| crates/cadcraft-model/src/domain/project/document/frames.rs:93 | Self::Asset { before, after, .. } => before == after, |
| crates/cadcraft-model/src/domain/project/document/frames.rs:158 | let position = ordered.iter().position(&#124;o&#124; o.id() == page); |
| crates/cadcraft-model/src/domain/project/document/frames.rs:197 | self.default_frame == Some(id) &#124;&#124; self.pages.values().any(&#124;p&#124; p.frame() == Some(id)) |
| crates/cadcraft-model/src/domain/project/document/frames.rs:210 | if *id != frame.id() { |
| crates/cadcraft-model/src/domain/project/document/frames.rs:217 | if *id != asset.id() { |
| crates/cadcraft-model/src/domain/project/document/frames.rs:235 | self.frames.values().any(&#124;f&#124; f.assets().any(&#124;a&#124; a == id)) |
| crates/cadcraft-model/src/domain/project/document/frames.rs:366 | if self.frames.get(id) != expected.as_deref() { |
| crates/cadcraft-model/src/domain/project/document/frames.rs:384 | if self.default_frame != *expected { |
| crates/cadcraft-model/src/domain/project/document/frames.rs:398 | if p.frame != *expected { |
| crates/cadcraft-model/src/domain/project/document/frames.rs:405 | if self.reference_format != *expected { |
| crates/cadcraft-model/src/domain/project/document/frames.rs:412 | if self.metadata.get(key) != expected.as_ref() { |
| crates/cadcraft-model/src/domain/project/document/frames.rs:425 | if p.metadata.get(key) != expected.as_ref() { |
| crates/cadcraft-model/src/domain/project/document/frames.rs:432 | if self.assets.get(id) != expected.as_deref() { |
| crates/cadcraft-model/src/domain/project/document/frames.rs:484 | .find(&#124;(k, _)&#124; k.as_str() == wanted) |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:26 | if before == after { |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:39 | if map.get(&c.key) != expected.as_ref() { |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:80 | .find(&#124;p&#124; p.id() == port.port_id()) |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:128 | RepresentationTarget::ConnectionPoint(id) => id == point.id(), |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:129 | RepresentationTarget::Function(id) => id == point.function_id(), |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:130 | RepresentationTarget::Device(id) => id == function.device_id(), |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:131 | RepresentationTarget::Terminal(id) => Some(id) == point.terminal_id(), |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:134 | .is_some_and(&#124;c&#124; c.function_id() == Some(point.function_id())) |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:138 | .is_some_and(&#124;t&#124; t.owner() == TerminalOwner::Component(id)) |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:160 | if expected != route.points() { |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:169 | if edge.endpoints() != (source.min(target), source.max(target)) &#124;&#124; source == target { |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:177 | != route.page_id() |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:182 | if route.points().first() != Some(&self.port_position(route.source())?) |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:183 | &#124;&#124; route.points().last() != Some(&self.port_position(route.target())?) |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:188 | if r.page_id() != route.page_id() |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:189 | &#124;&#124; r.target() != RepresentationTarget::Connection(route.connection_id()) |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:191 | &#124;&#124; r.transform() != identity()? |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:192 | &#124;&#124; r.local_bounds() != Some(route_bounds(route.points())?) |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:201 | let coordinate_key = &#124;v: f64&#124; if v == 0.0 { 0 } else { v.to_bits() }; |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:211 | if unique.len() < 2 &#124;&#124; unique.len() != j.routes().len() { |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:217 | if r.page_id() != j.page_id() &#124;&#124; !r.contains(j.position()) { |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:231 | .any(&#124;r&#124; r.connection_id() == *id) |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:415 | if routes.len() < 2 &#124;&#124; routes.windows(2).any(&#124;w&#124; w[0] == w[1]) { |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:421 | .any(&#124;j&#124; j.page_id() == page && j.position() == position) |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:431 | if r.page_id() != page &#124;&#124; !r.contains(position) { |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:449 | .any(&#124;c&#124; c.endpoints() == (root.min(endpoint), root.max(endpoint))) |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:543 | .any(&#124;p&#124; p.representation_id() == id) |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:552 | .any(&#124;r&#124; r.connection_id() == id) |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:606 | if self.connection_routes.contains_key(&id) && old.transform() != t { |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:649 | if da == db { |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:675 | .all(&#124;id&#124; translations.get(id) == Some(&offset)) |
| crates/cadcraft-model/src/domain/project/electrical/connection_drawing.rs:698 | if overrides.len() != transforms.len() { |
| crates/cadcraft-model/src/domain/project/electrical/function_edit.rs:60 | self.len() == 0 |
| crates/cadcraft-model/src/domain/project/electrical/function_edit.rs:99 | if before != after { |
| crates/cadcraft-model/src/domain/project/electrical/function_edit.rs:114 | .any(&#124;r&#124; self.function_for_target(r.target()) == Some(function)) |
| crates/cadcraft-model/src/domain/project/electrical/function_edit.rs:121 | .filter(&#124;p&#124; p.function_id() == function) |
| crates/cadcraft-model/src/domain/project/electrical/function_edit.rs:141 | .any(&#124;p&#124; p.function_id() == function && p.designation() == &designation) |
| crates/cadcraft-model/src/domain/project/electrical/function_edit.rs:160 | p.id() != point |
| crates/cadcraft-model/src/domain/project/electrical/function_edit.rs:161 | && p.function_id() == before.function_id() |
| crates/cadcraft-model/src/domain/project/electrical/function_edit.rs:162 | && p.designation() == &designation |
| crates/cadcraft-model/src/domain/project/electrical/function_edit.rs:170 | if before != after { |
| crates/cadcraft-model/src/domain/project/electrical/function_edit.rs:185 | &#124;&#124; self.port_bindings.values().any(&#124;id&#124; *id == point) |
| crates/cadcraft-model/src/domain/project/electrical/function_edit.rs:188 | a == point &#124;&#124; b == point |
| crates/cadcraft-model/src/domain/project/electrical/function_edit.rs:208 | if before != after { |
| crates/cadcraft-model/src/domain/project/electrical/function_edit.rs:224 | if before != after { |
| crates/cadcraft-model/src/domain/project/electrical/function_edit.rs:254 | if candidate.functions.get(&id) != expected.as_ref() { |
| crates/cadcraft-model/src/domain/project/electrical/function_edit.rs:279 | if candidate.connection_points.get(&id) != expected.as_ref() { |
| crates/cadcraft-model/src/domain/project/electrical/identification.rs:11 | (before != after).then_some(IdentificationChange { before, after }) |
| crates/cadcraft-model/src/domain/project/electrical/identification.rs:23 | .any(&#124;d&#124; d.id() != device.id() && d.designation() == device.designation()) |
| crates/cadcraft-model/src/domain/project/electrical/identification.rs:44 | if f.role() == FunctionRole::Main && !main.insert(f.device_id()) { |
| crates/cadcraft-model/src/domain/project/electrical/identification.rs:54 | .find(&#124;f&#124; f.role() == FunctionRole::Main)) |
| crates/cadcraft-model/src/domain/project/electrical/identification.rs:66 | .filter(&#124;p&#124; p.terminal_id() == Some(id)) |
| crates/cadcraft-model/src/domain/project/electrical/identification.rs:69 | if ids.len() == 1 { |
| crates/cadcraft-model/src/domain/project/electrical/identification.rs:97 | .filter(&#124;r&#124; self.device_for_target(r.target()) == Some(device)) |
| crates/cadcraft-model/src/domain/project/electrical/identification.rs:111 | .filter(&#124;r&#124; r.function == Some(function)) |
| crates/cadcraft-model/src/domain/project/electrical/identification.rs:126 | .filter(&#124;p&#124; p.function_id() == function.id()) |
| crates/cadcraft-model/src/domain/project/electrical/identification.rs:152 | .filter(&#124;cp&#124; cp.function_id() == f.id()) |
| crates/cadcraft-model/src/domain/project/electrical/identification.rs:201 | if before.designation != after.designation { |
| crates/cadcraft-model/src/domain/project/electrical/identification.rs:216 | if before != after { |
| crates/cadcraft-model/src/domain/project/electrical/identification.rs:221 | .filter(&#124;d&#124; d.id() != device && d.designation() == before.designation()) |
| crates/cadcraft-model/src/domain/project/electrical/identification.rs:271 | if f.device_id() != device { |
| crates/cadcraft-model/src/domain/project/electrical/identification.rs:277 | if function == Some(before.id()) { |
| crates/cadcraft-model/src/domain/project/electrical/identification.rs:279 | } else if before.role() == FunctionRole::Main { |
| crates/cadcraft-model/src/domain/project/electrical/identification.rs:297 | if before != after && self.drawing_references_representation(representation) { |
| crates/cadcraft-model/src/domain/project/electrical/identification.rs:340 | if $map.get(&expected.id()) != Some(expected) { |
| crates/cadcraft-model/src/domain/project/electrical/identification.rs:361 | if from.designation() != to.designation() { |
| crates/cadcraft-model/src/domain/project/electrical/identification.rs:364 | .filter(&#124;d&#124; d.id() != to.id() && d.designation() == to.designation()) |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:16 | if before != after { |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:27 | != if forward { |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:78 | if d.ports().len() != 1 |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:79 | &#124;&#124; d.ports()[0].id() != "p" |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:80 | &#124;&#124; d.ports()[0].position() != Point2D::new(0., 0.)? |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:104 | let a = points[if g.mode == ReferenceMode::Star { |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:110 | if a != b { |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:124 | != m.point().map_or( |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:128 | &#124;&#124; r.visual().label.as_ref() != Some(m.name()) |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:129 | &#124;&#124; self.port_binding(&self.marker_port(m.id())?) != m.point() |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:144 | .is_none_or(&#124;m&#124; *m.kind() != MarkerKind::Interruption) |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:160 | if actual != expected { |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:177 | other.id() != id |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:193 | let index = g.members.iter().position(&#124;m&#124; *m == id).ok_or(invalid())?; |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:195 | ReferenceMode::Star if index == 0 => g.members[1..].to_vec(), |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:231 | ) => x != y, |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:234 | if a.name() != b.name() &#124;&#124; kinds { |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:257 | &#124;&#124; representation.id() != id |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:258 | &#124;&#124; representation.symbol() != Some(definition.id()) |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:263 | if old != definition.as_ref() { |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:295 | if old.point() != point |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:342 | .any(&#124;other&#124; other.id() != id && other.symbol() == r.symbol()) |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:359 | r.source().representation_id() == id &#124;&#124; r.target().representation_id() == id |
| crates/cadcraft-model/src/domain/project/electrical/markers.rs:382 | g.mode == mode && g.members == members && g.continuity == continuity |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:31 | self.len() == 0 |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:44 | self.before == self.after |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:139 | Some(base) if number == self.numbering_scheme.start => base.clone(), |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:144 | if self.devices().all(&#124;d&#124; d.designation() != &designation) { |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:158 | if start == 0 { |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:193 | if before.number() != &row.before { |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:200 | if before != after { |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:221 | if candidate.pages.get(&expected.id()) != Some(expected) { |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:238 | if scheme.start == 0 &#124;&#124; scheme.step == 0 { |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:280 | if self.connection(expected.id()) != Some(expected) { |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:307 | position_order(order, (rank, point), (old.0, old.1)) == std::cmp::Ordering::Less |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:351 | &#124;&#124; (!named.is_empty() && !locked.is_empty() && named != locked)) |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:405 | let candidate = if *n == scheme.start { |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:458 | if before.number() != row.before.as_ref() |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:459 | &#124;&#124; (before.number_locked() && before.number() != row.after.as_ref()) |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:465 | if before != after { |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:490 | if candidate.connections.get(&expected.id()) != Some(expected) { |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:531 | .is_some_and(&#124;p&#124; p.document_id() == id) |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:565 | before_in_order((rank, point), (old.0, old.1)) == std::cmp::Ordering::Less |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:597 | .filter(&#124;_&#124; scheme.device_format == DeviceNumberFormat::PageColumn) |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:610 | let name = if *next == scheme.start { |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:682 | if before.designation != row.before |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:685 | && row.after != row.before) |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:693 | if before != after { |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:700 | if before != after { |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:710 | .find(&#124;c&#124; c.before.id() == device.id()) |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:735 | if candidate.devices.get(&expected.id()) != Some(expected) { |
| crates/cadcraft-model/src/domain/project/electrical/numbering.rs:748 | if candidate.functions.get(&expected.id()) != Some(expected) { |
| crates/cadcraft-model/src/domain/project/electrical/representations.rs:47 | if change.before.as_ref().is_some_and(&#124;r&#124; r.id() != id) |
| crates/cadcraft-model/src/domain/project/electrical/representations.rs:48 | &#124;&#124; change.after.as_ref().is_some_and(&#124;r&#124; r.id() != id) |
| crates/cadcraft-model/src/domain/project/electrical/representations.rs:52 | if self.representation(id) != change.before.as_ref() { |
| crates/cadcraft-model/src/domain/project/electrical/representations.rs:71 | .filter(&#124;c&#124; c.before != c.after) |
| crates/cadcraft-model/src/domain/project/electrical/representations.rs:191 | .is_some_and(&#124;m&#124; visual.label.as_ref() != Some(m.name())) |
| crates/cadcraft-model/src/domain/project/electrical/representations.rs:259 | &#124;&#124; representation.target() != RepresentationTarget::Unassociated |
| crates/cadcraft-model/src/domain/project/electrical/representations.rs:272 | None if representation.target() == RepresentationTarget::Unassociated |
| crates/cadcraft-model/src/domain/project/electrical/route_clearance.rs:41 | if cross == 0.0 |
| crates/cadcraft-model/src/domain/project/electrical/route_clearance.rs:49 | if (a.y() > p.y()) != (b.y() > p.y()) |
| crates/cadcraft-model/src/domain/project/electrical/route_clearance.rs:66 | let horizontal = a.y() == b.y(); |
| crates/cadcraft-model/src/domain/project/electrical/route_clearance.rs:71 | let t = if horizontal && p.y() != q.y() { |
| crates/cadcraft-model/src/domain/project/electrical/route_clearance.rs:73 | if (0.0..=1.0).contains(&u) && a.x() != b.x() { |
| crates/cadcraft-model/src/domain/project/electrical/route_clearance.rs:78 | } else if !horizontal && p.x() != q.x() { |
| crates/cadcraft-model/src/domain/project/electrical/route_clearance.rs:80 | if (0.0..=1.0).contains(&u) && a.y() != b.y() { |
| crates/cadcraft-model/src/domain/project/electrical/route_clearance.rs:157 | if (a.x() == b.x() && b.x() == c.x()) &#124;&#124; (a.y() == b.y() && b.y() == c.y()) { |
| crates/cadcraft-model/src/domain/project/electrical/route_clearance.rs:171 | let x = if degrees == 0.0 { |
| crates/cadcraft-model/src/domain/project/electrical/route_clearance.rs:188 | let y = if degrees == 90.0 { |
| crates/cadcraft-model/src/domain/project/electrical/route_clearance.rs:226 | let candidate = if a.y() == b.y() { |
| crates/cadcraft-model/src/domain/project/electrical/route_clearance.rs:303 | .find(&#124;p&#124; p.id() == port.port_id()) |
| crates/cadcraft-model/src/domain/project/electrical/route_clearance.rs:331 | if rep.page_id() != route.page_id() |
| crates/cadcraft-model/src/domain/project/electrical/route_clearance.rs:332 | &#124;&#124; rep.id() == route.source().representation_id() |
| crates/cadcraft-model/src/domain/project/electrical/route_clearance.rs:333 | &#124;&#124; rep.id() == route.target().representation_id() |
| crates/cadcraft-model/src/domain/project/electrical/sheet_references.rs:124 | for (id, _, role) in occurrences.iter().filter(&#124;o&#124; o.1 == page) { |
| crates/cadcraft-model/src/domain/project/electrical/sheet_references.rs:127 | .filter(&#124;o&#124; o.0 != *id && o.2 == FunctionRole::Main) |
| crates/cadcraft-model/src/domain/project/electrical/sheet_references.rs:129 | let targets: Vec<_> = if *role != FunctionRole::Main && !main.is_empty() { |
| crates/cadcraft-model/src/domain/project/electrical/sheet_references.rs:132 | occurrences.iter().filter(&#124;o&#124; o.0 != *id).collect() |
| crates/cadcraft-model/src/domain/project/electrical/sheet_references.rs:171 | if function.role() != FunctionRole::Main { |
| crates/cadcraft-model/src/domain/project/electrical/sheet_references.rs:217 | .filter(&#124;r&#124; r.page_id() == page) |
| crates/cadcraft-model/src/domain/project/integrity.rs:9 | EntityRef::Project(id) => self.id == id, |
| crates/cadcraft-model/src/domain/project/integrity.rs:32 | if found == self.id { |
| crates/cadcraft-model/src/domain/project/integrity.rs:76 | if function.role() == FunctionRole::Main |
| crates/cadcraft-model/src/domain/project/integrity.rs:79 | .any(&#124;other&#124; other.id() != function.id() && other.role() == FunctionRole::Main) |
| crates/cadcraft-model/src/domain/project/integrity.rs:88 | if component.quantity() == 0 { |
| crates/cadcraft-model/src/domain/project/integrity.rs:105 | if component.device_id() != Some(function.device_id()) { |
| crates/cadcraft-model/src/domain/project/integrity.rs:149 | TerminalOwner::Device(device) if device == function.device_id() => {} |
| crates/cadcraft-model/src/domain/project/integrity.rs:154 | if component.device_id() != Some(function.device_id()) |
| crates/cadcraft-model/src/domain/project/integrity.rs:157 | .is_some_and(&#124;id&#124; id != function.id()) |
| crates/cadcraft-model/src/domain/project/integrity.rs:175 | if self.numbering_scheme.start == 0 &#124;&#124; self.numbering_scheme.step == 0 { |
| crates/cadcraft-model/src/domain/project/integrity.rs:224 | if self.build_graph()? != self.graph { |
| crates/cadcraft-model/src/domain/project/integrity.rs:229 | if self.build_physical_index()? != self.physical_index { |
| crates/cadcraft-model/src/domain/project/integrity.rs:232 | if self.build_document_index()? != self.document_index { |
| crates/cadcraft-model/src/domain/project/physical/connectors.rs:56 | .filter(&#124;p&#124; p.terminal_id() == Some(terminal)) |
| crates/cadcraft-model/src/domain/project/physical/connectors.rs:101 | .is_some_and(&#124;d&#124; component.device_id() != Some(d)) |
| crates/cadcraft-model/src/domain/project/physical/connectors.rs:118 | if contact.kind() != TerminalKind::Generic &#124;&#124; !contact.levels().is_empty() { |
| crates/cadcraft-model/src/domain/project/physical/connectors.rs:124 | && self.terminal_owner_device(contact) != Some(device) |
| crates/cadcraft-model/src/domain/project/physical/connectors.rs:136 | if mating.first() == mating.second() { |
| crates/cadcraft-model/src/domain/project/physical/connectors.rs:180 | if owned.len() != expected.len() { |
| crates/cadcraft-model/src/domain/project/physical/connectors.rs:189 | if connection.endpoints() != (a.min(b), a.max(b)) { |
| crates/cadcraft-model/src/domain/project/physical/connectors.rs:207 | .filter(&#124;p&#124; p.terminal_id() == Some(contact)) |
| crates/cadcraft-model/src/domain/project/physical/connectors.rs:248 | if let Some(i) = previous.iter().position(&#124;c&#124; c.endpoints() == endpoints) { |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:61 | if candidate.part(part.id()) != Some(&part) { |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:101 | if before == after { |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:107 | .filter(&#124;c&#124; c.part_id() == Some(after.id())) |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:116 | .any(&#124;new&#124; new.kind() == model.kind() && new.points() == model.points()); |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:123 | .filter(&#124;f&#124; f.device_id() == device && f.kind() == model.kind()) |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:128 | .filter(&#124;p&#124; p.function_id() == f.id()) |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:138 | .any(&#124;r&#124; r.target() == RepresentationTarget::Function(f.id())) |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:150 | .filter(&#124;c&#124; c.part_id() == Some(after.id())) |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:175 | if self.part(expected.id()) != Some(expected) { |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:185 | if candidate.component(expected.id()) != Some(expected) { |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:218 | .filter(&#124;c&#124; c.part_id() == Some(id)) |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:220 | == 1 |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:236 | if candidate.component(delta.component.id()) != Some(&delta.component) { |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:265 | if quantity == 0 { |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:273 | .is_some_and(&#124;existing&#124; existing != &part) |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:298 | f.device_id() == device_id |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:299 | && f.kind() == model.kind() |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:306 | .filter(&#124;p&#124; p.function_id() == f.id()) |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:317 | let role = if model.role() == FunctionRole::Main && !has_main { |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:337 | p.function_id() == function_id && p.designation().as_str() == label.as_str() |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:368 | .is_some_and(&#124;p&#124; p != &delta.part) |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:385 | if candidate.components.get(&delta.component.id()) != Some(&delta.component) |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:389 | .any(&#124;f&#124; candidate.functions.get(&f.id()) != Some(f)) |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:393 | .any(&#124;p&#124; candidate.connection_points.get(&p.id()) != Some(p)) |
| crates/cadcraft-model/src/domain/project/physical/parts.rs:408 | .any(&#124;c&#124; c.part_id() == Some(delta.part.id())) |
| crates/cadcraft-model/src/domain/project/physical/physical_edit.rs:87 | self.len() == 0 |
| crates/cadcraft-model/src/domain/project/physical/physical_edit.rs:102 | if map.get(&change.key) != expected.as_ref() { |
| crates/cadcraft-model/src/domain/project/physical/physical_edit.rs:133 | (before != after).then_some(PhysicalChange { key, before, after }) |
| crates/cadcraft-model/src/domain/project/physical/physical_edit.rs:371 | if before.strip_id() != value.strip_id() |
| crates/cadcraft-model/src/domain/project/physical/physical_edit.rs:372 | &#124;&#124; before.project_id() != value.project_id() |
| crates/cadcraft-model/src/domain/project/physical/physical_integrity.rs:38 | .is_some_and(&#124;t&#124; t.kind() == TerminalKind::ProtectiveEarth), |
| crates/cadcraft-model/src/domain/project/physical/physical_integrity.rs:41 | if earth[0] != earth[1] { |
| crates/cadcraft-model/src/domain/project/physical/physical_integrity.rs:83 | .is_some_and(&#124;t&#124; t.kind() == TerminalKind::ProtectiveEarth); |
| crates/cadcraft-model/src/domain/project/physical/physical_integrity.rs:103 | if a == b { |
| crates/cadcraft-model/src/domain/project/physical/physical_integrity.rs:130 | let target = if source == a { |
| crates/cadcraft-model/src/domain/project/physical/physical_integrity.rs:132 | } else if source == b { |
| crates/cadcraft-model/src/domain/project/physical/physical_integrity.rs:148 | if point.terminal_id() != Some(end.terminal_id()) { |
| crates/cadcraft-model/src/domain/project/physical/physical_integrity.rs:194 | .is_some_and(&#124;area&#124; area != conductor.cross_section()) |
| crates/cadcraft-model/src/domain/project/physical/physical_integrity.rs:195 | &#124;&#124; matches!((core.marking().color_value(), conductor.marking().color_value()), (Some(a), Some(b)) if a != b) |
| crates/cadcraft-model/src/domain/project/physical/physical_lists.rs:226 | if b.terminal_id() != a.terminal_id() { |
| crates/cadcraft-model/src/domain/project/physical/physical_lists.rs:440 | if m.first() == c.id() { |
| crates/cadcraft-model/src/domain/project/physical/physical_lists.rs:514 | .filter(&#124;p&#124; p.terminal_id() == Some(terminal)) |
| crates/cadcraft-model/src/domain/project/physical/physical_queries.rs:73 | TerminalOwner::Device(id) => id == device, |
| crates/cadcraft-model/src/domain/project/physical/physical_queries.rs:76 | .is_some_and(&#124;c&#124; c.device_id() == Some(device)), |
| crates/cadcraft-model/src/domain/project/snapshot.rs:157 | if project.parts.len() != part_count { |
| crates/cadcraft-model/src/domain/project/symbols/macros/capture.rs:20 | if *page.get_or_insert(r.page_id()) != r.page_id() { |
| crates/cadcraft-model/src/domain/project/symbols/macros/capture.rs:34 | .filter(&#124;r&#124; r.page_id() == page) |
| crates/cadcraft-model/src/domain/project/symbols/macros/capture.rs:120 | if r.page_id() != page { |
| crates/cadcraft-model/src/domain/project/symbols/macros/capture.rs:134 | if j.page_id() != page { |
| crates/cadcraft-model/src/domain/project/symbols/macros/capture.rs:138 | if inside == j.routes().len() { |
| crates/cadcraft-model/src/domain/project/symbols/macros/capture.rs:244 | EntityRef::Component(id) => content.components.iter().any(&#124;c&#124; c.id() == *id), |
| crates/cadcraft-model/src/domain/project/symbols/macros/insertion.rs:197 | Some(existing) if existing != &d => report.kept_symbols.push(d.id().clone()), |
| crates/cadcraft-model/src/domain/project/symbols/macros/insertion.rs:204 | Some(existing) if existing != &p => report.kept_parts.push(p.id().clone()), |
| crates/cadcraft-model/src/domain/project/symbols/macros/insertion.rs:317 | && self.numbering_scheme.device_format == DeviceNumberFormat::PageColumn |
| crates/cadcraft-model/src/domain/project/symbols/macros/insertion.rs:330 | Some(base) if n == self.numbering_scheme.start => base.clone(), |
| crates/cadcraft-model/src/domain/project/symbols/macros/insertion.rs:385 | map.get(&k) == Some(&v) |
| crates/cadcraft-model/src/domain/project/symbols/macros/mod.rs:36 | *self == Self::default() |
| crates/cadcraft-model/src/domain/project/symbols/macros/mod.rs:136 | self.len() == 0 |
| crates/cadcraft-model/src/domain/project/symbols/operations.rs:27 | && previous != &definition |
| crates/cadcraft-model/src/domain/project/symbols/operations.rs:48 | .any(&#124;r&#124; r.symbol() == Some(id)) |
| crates/cadcraft-model/src/domain/project/symbols/operations.rs:61 | if r.symbol() != Some(d.id()) &#124;&#124; r.local_bounds() != Some(d.bounds()) { |
| crates/cadcraft-model/src/domain/project/symbols/operations.rs:117 | .filter(&#124;r&#124; r.symbol() == Some(id)) |
| crates/cadcraft-model/src/domain/project/symbols/operations.rs:129 | if new == old { |
| crates/cadcraft-model/src/domain/project/symbols/operations.rs:138 | .is_some_and(&#124;r&#124; r.symbol() == Some(new.id())) |
| crates/cadcraft-model/src/domain/project/symbols/operations.rs:148 | .is_some_and(&#124;r&#124; r.symbol() == Some(new.id())) |
| crates/cadcraft-model/src/domain/project/symbols/operations.rs:161 | .filter(&#124;r&#124; r.symbol() == Some(new.id()) && !new.orientations().allows(r.transform())) |
| crates/cadcraft-model/src/domain/project/symbols/operations.rs:223 | if self.symbol_definition(expected.id()) != Some(expected) { |
| crates/cadcraft-model/src/domain/project/symbols/operations.rs:241 | Some(existing) if existing == &definition => Ok(false), |
| crates/cadcraft-model/src/domain/project/symbols/placement.rs:123 | .filter(&#124;p&#124; p.function_id() == function.id()) |
| crates/cadcraft-model/src/domain/project/symbols/placement.rs:126 | if points.len() != placement.definition.ports().len() &#124;&#124; points.is_empty() { |
| crates/cadcraft-model/src/domain/project/symbols/placement.rs:138 | .filter(&#124;p&#124; p.designation().as_str() == port.identification()); |
| crates/cadcraft-model/src/domain/project/symbols/placement.rs:144 | if ids.iter().collect::<BTreeSet<_>>().len() == ids.len() { |
| crates/cadcraft-model/src/domain/project/symbols/placement.rs:206 | if candidate.representation(delta.representation.id()) != Some(&delta.representation) |
| crates/cadcraft-model/src/domain/project/symbols/placement.rs:210 | .any(&#124;(port, point)&#124; candidate.port_bindings.get(port) != Some(point)) |
| crates/cadcraft-model/src/domain/project/symbols/placement.rs:365 | if self.port_bindings.get(port) == Some(point) { |
| crates/cadcraft-model/src/domain/project/symbols/placement.rs:370 | if self.representations.get(&id) == Some(&delta.representation) { |
| crates/cadcraft-model/src/domain/project/symbols/placement.rs:377 | if self.connection_points.get(&point.id()) == Some(point) { |
| crates/cadcraft-model/src/domain/project/symbols/placement.rs:381 | if self.functions.get(&delta.function_id()) == Some(&delta.function) { |
| crates/cadcraft-model/src/domain/project/symbols/placement.rs:385 | && self.devices.get(&device.id()) == Some(device) |
| crates/cadcraft-model/src/domain/project/symbols/placement.rs:395 | .is_none_or(&#124;d&#124; self.device(d.id()) == Some(d)) |
| crates/cadcraft-model/src/domain/project/symbols/placement.rs:396 | && self.function(delta.function_id()) == Some(&delta.function) |
| crates/cadcraft-model/src/domain/project/symbols/placement.rs:400 | .all(&#124;p&#124; self.connection_point(p.id()) == Some(p)) |
| crates/cadcraft-model/src/domain/project/symbols/placement.rs:401 | && self.representation(delta.representation_id()) == Some(&delta.representation) |
| crates/cadcraft-model/src/domain/project/symbols/placement.rs:402 | && self.symbol_definition(delta.definition.id()) == Some(&delta.definition) |
| crates/cadcraft-model/src/domain/project/symbols/placement.rs:406 | .all(&#124;(port, point)&#124; self.port_bindings.get(port) == Some(point)); |
| crates/cadcraft-model/src/domain/project/symbols/placement.rs:408 | port.representation_id() == delta.representation_id() |
| crates/cadcraft-model/src/domain/project/symbols/placement.rs:409 | && !delta.bindings.iter().any(&#124;(p, _)&#124; p == port) |
| crates/cadcraft-model/src/domain/project/symbols/placement.rs:429 | .any(&#124;r&#124; r.symbol() == Some(delta.definition.id())); |
| crates/cadcraft-model/src/domain/project.rs:322 | .filter(move &#124;page&#124; page.document_id() == document) |
| crates/cadcraft-model/src/domain/project.rs:327 | .filter(move &#124;function&#124; function.device_id() == device) |
| crates/cadcraft-model/src/domain/project.rs:433 | scheme == &CheckScheme::default() |
| crates/cadcraft-model/src/domain/properties.rs:54 | if unique.len() != v.choices.len() &#124;&#124; !unique.contains(&v.selected) { |
| crates/cadcraft-model/src/domain/symbols/macro_def.rs:24 | if value.trim() != value |
| crates/cadcraft-model/src/domain/symbols/macro_def.rs:130 | self.definitions.iter().find(&#124;d&#124; d.id() == id) |
| crates/cadcraft-model/src/domain/symbols/macro_def.rs:309 | if self.version == 0 { |
| crates/cadcraft-model/src/domain/symbols/symbol/mod.rs:182 | *d == LineDash::Solid |
| crates/cadcraft-model/src/domain/symbols/symbol/mod.rs:186 | *self == Self::default() |
| crates/cadcraft-model/src/domain/symbols/symbol/mod.rs:427 | if version == 0 &#124;&#124; primitives.is_empty() { |
| crates/cadcraft-model/src/domain/symbols/symbol/mod.rs:561 | if reconstructed.bounds != self.bounds { |
| crates/cadcraft-model/src/domain/symbols/symbol/primitives.rs:49 | if start == end { |
| crates/cadcraft-model/src/domain/symbols/symbol/primitives.rs:56 | &#124;&#124; points.windows(2).any(&#124;p&#124; p[0] == p[1]) |
| crates/cadcraft-model/src/domain/symbols/symbol/primitives.rs:87 | &#124;&#124; *sweep_degrees == 0. |
| crates/cadcraft-model/src/domain/symbols/symbol_library.rs:179 | let ports = if key == "protective-earth" { |
| crates/cadcraft-model/src/graph.rs:58 | if point == target { |
