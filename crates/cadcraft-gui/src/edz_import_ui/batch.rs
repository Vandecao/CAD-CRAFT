// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! EDZ batch planning and atomic persistence; start with save_packages.
use super::*;

pub(super) fn save_packages(
    packages: &mut [EdzPackage],
    part_folder: &str,
    macro_folder: &str,
    cancel: &AtomicBool,
    sender: Option<&mpsc::Sender<Message>>,
) -> GuiResult<()> {
    // Plan and serialize the entire batch before writing any file.
    let mut inventories = BTreeMap::new();
    for package in packages.iter() {
        for artifact in &package.artifacts {
            let folder = artifact.data.folder(part_folder, macro_folder).trim();
            if folder.is_empty() {
                return Err(
                    "Configure as pastas das bibliotecas de peças e macros antes de confirmar."
                        .into(),
                );
            }
            if artifact.conflict && artifact.choice.is_none() {
                return Err("Resolva cada conflito: substituir, manter as duas ou pular.".into());
            }
            let key = (PathBuf::from(folder), artifact.data.extension());
            if let std::collections::btree_map::Entry::Vacant(entry) =
                inventories.entry(key.clone())
            {
                entry.insert(inventory(&key.0, key.1)?);
            }
        }
    }
    let mut writes: BTreeMap<PathBuf, (usize, Vec<u8>)> = BTreeMap::new();
    let mut counts = vec![0; packages.len()];
    let mut macro_ids = BTreeMap::new();
    let total = packages.iter().map(|p| p.artifacts.len()).sum::<usize>() * 2;
    let mut completed = 0;
    // Macros first so kept/renamed IDs propagate to imported parts.
    for macro_pass in [true, false] {
        for (index, package) in packages.iter().enumerate() {
            for artifact in &package.artifacts {
                if matches!(artifact.data, Data::Macro(_)) != macro_pass {
                    continue;
                }
                if cancel.load(Ordering::Relaxed) {
                    return Err(
                        "Cancelado antes da gravação; nenhuma biblioteca foi alterada.".into(),
                    );
                }
                if artifact.choice == Some(ConflictChoice::Skip) {
                    if macro_pass {
                        let key = (
                            PathBuf::from(artifact.data.folder(part_folder, macro_folder).trim()),
                            artifact.data.extension(),
                        );
                        if let Some(existing) = inventories
                            .get(&key)
                            .and_then(|list| list.iter().find(|e| e.id == artifact.data.id()))
                        {
                            macro_ids.insert(
                                (index, artifact.data.id().to_owned()),
                                existing.id.clone(),
                            );
                        }
                    }
                    continue;
                }
                let mut data = artifact.data.clone();
                let folder = PathBuf::from(data.folder(part_folder, macro_folder).trim());
                let ext = data.extension();
                let list = inventories
                    .get_mut(&(folder.clone(), ext))
                    .expect("inventoried");
                let candidate = folder.join(format!("{}.{}", stem(data.id()), ext));
                let matches: Vec<usize> = list
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| {
                        e.id == data.id()
                            || e.name.eq_ignore_ascii_case(data.name())
                            || same_path(&e.path, &candidate)
                    })
                    .map(|(i, _)| i)
                    .collect();
                let mut path = candidate;
                if !matches.is_empty() || path.exists() || writes.contains_key(&path) {
                    match artifact.choice {
                        Some(ConflictChoice::Replace) => {
                            if matches.len() > 1 {
                                return Err(format!(
                                    "Conflito ambíguo de ID/nome/caminho para {}; escolha manter as duas ou pular.",
                                    data.name()
                                ).into());
                            }
                            if let Some(&i) = matches.first() {
                                path = list[i].path.clone();
                                data = data.rename(&list[i].id, data.name())?;
                                list.remove(i);
                            }
                        }
                        Some(ConflictChoice::KeepBoth) => {
                            for n in 2..=50_001 {
                                let suffix = format!("-{n}");
                                let mut id = data.id().to_owned();
                                while id.len() + suffix.len() > 160 {
                                    id.pop();
                                }
                                id.push_str(&suffix);
                                let name = format!("{} ({n})", data.name());
                                let next = folder.join(format!("{}.{}", stem(&id), ext));
                                if !next.exists()
                                    && !writes.contains_key(&next)
                                    && !list.iter().any(|e| {
                                        e.id == id
                                            || e.name.eq_ignore_ascii_case(&name)
                                            || same_path(&e.path, &next)
                                    })
                                {
                                    data = data.rename(&id, &name)?;
                                    path = next;
                                    break;
                                }
                            }
                            if path.exists() || writes.contains_key(&path) {
                                return Err("Não foi possível gerar um nome livre.".into());
                            }
                        }
                        _ => {
                            return Err(format!(
                                "Novo conflito em {}; analise novamente.",
                                data.name()
                            )
                            .into());
                        }
                    }
                }
                if macro_pass {
                    macro_ids.insert((index, artifact.data.id().to_owned()), data.id().to_owned());
                }
                if let Data::Part(part) = &mut data {
                    let mut properties = part.properties().clone();
                    let ids = properties
                        .custom()
                        .iter()
                        .find(|(k, _)| k.as_str() == "CADCRAFT.MacroIds")
                        .and_then(|(_, v)| {
                            if let PropertyValue::Text(s) = v {
                                Some(s.clone())
                            } else {
                                None
                            }
                        });
                    if let Some(ids) = ids {
                        let mapped = ids
                            .split(';')
                            .filter_map(|id| macro_ids.get(&(index, id.to_owned())).cloned())
                            .collect::<Vec<_>>()
                            .join(";");
                        properties.set_custom("CADCRAFT.MacroIds", PropertyValue::Text(mapped))?;
                        **part = part.as_ref().clone().with_properties(properties);
                    }
                }
                let bytes = data.bytes()?;
                list.push(Existing {
                    id: data.id().into(),
                    name: data.name().into(),
                    path: path.clone(),
                    data,
                });
                if let Some((previous, _)) = writes.insert(path, (index, bytes)) {
                    counts[previous] -= 1;
                }
                counts[index] += 1;
                completed += 1;
                if let Some(sender) = sender {
                    let _ = sender.send(Message::Progress(completed, total));
                }
            }
        }
    }
    let mut saved = vec![Vec::new(); packages.len()];
    let mut error = None;
    for (path, (index, bytes)) in writes {
        if cancel.load(Ordering::Relaxed) {
            error = Some(GuiError::from("Cancelado durante a gravação"));
            break;
        }
        let result = std::fs::create_dir_all(path.parent().expect("library folder"))
            .and_then(|()| crate::project_file::write_atomic(&path, &mut bytes.as_slice()));
        if let Err(e) = result {
            error = Some(GuiError::at_path(&path, e));
            break;
        }
        saved[index].push(path.display().to_string());
        completed += 1;
        if let Some(sender) = sender {
            let _ = sender.send(Message::Progress(completed, total));
        }
    }
    for (index, (package, count)) in packages.iter_mut().zip(counts).enumerate() {
        if let Some(e) = &error {
            package.outcome = format!(
                "Erro: {e}; {} arquivo(s) gravado(s) deste pacote: {}",
                saved[index].len(),
                saved[index].join("; ")
            );
            continue;
        }
        package.outcome = match &package.preview {
            Err(e) => format!("Erro: {e}"),
            Ok(_) if count == 0 => "Ignorado".into(),
            Ok(p)
                if p.report.count(NoteKind::Ignored) + p.report.count(NoteKind::Approximated)
                    > 0
                    || count < package.artifacts.len() =>
            {
                "Convertido parcialmente".into()
            }
            Ok(_) => "Importado".into(),
        };
    }

    if let Some(error) = error {
        Err(error)
    } else {
        Ok(())
    }
}

pub(super) fn prepare_packages(
    packages: &mut [EdzPackage],
    part_folder: &str,
    macro_folder: &str,
    cancel: &AtomicBool,
) -> GuiResult<()> {
    type Seen = BTreeMap<&'static str, Vec<(String, String, PathBuf)>>;
    let mut seen = Seen::new();
    for (ext, folder) in [
        (persistence::PART_EXTENSION, part_folder),
        (persistence::MACRO_EXTENSION, macro_folder),
    ] {
        if cancel.load(Ordering::Relaxed) {
            return Err("Cancelado antes da gravação.".into());
        }
        let list = if folder.trim().is_empty() {
            Vec::new()
        } else {
            inventory(Path::new(folder), ext)?
        };
        seen.insert(
            ext,
            list.into_iter().map(|e| (e.id, e.name, e.path)).collect(),
        );
    }
    for package in packages.iter_mut() {
        if cancel.load(Ordering::Relaxed) {
            return Err("Cancelado antes da gravação.".into());
        }
        if let Ok(preview) = &package.preview {
            for data in preview
                .macros
                .iter()
                .cloned()
                .map(|m| Data::Macro(Box::new(m)))
                .chain(
                    preview
                        .parts
                        .iter()
                        .cloned()
                        .map(|p| Data::Part(Box::new(p))),
                )
            {
                let list = seen.get_mut(data.extension()).expect("both types");
                let candidate = PathBuf::from(match &data {
                    Data::Part(_) => part_folder,
                    Data::Macro(_) => macro_folder,
                })
                .join(format!("{}.{}", stem(data.id()), data.extension()));
                let conflict = list.iter().any(|(id, name, path)| {
                    id == data.id()
                        || name.eq_ignore_ascii_case(data.name())
                        || same_path(path, &candidate)
                }) || candidate.exists();
                list.push((data.id().into(), data.name().into(), candidate));
                package.artifacts.push(Artifact {
                    data,
                    conflict,
                    choice: None,
                });
            }
        }
    }

    Ok(())
}
