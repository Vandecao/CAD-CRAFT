// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Technical JSON snapshot I/O; start with the bounded reader and writer functions.
use std::{
    io::{self, Read},
    path::Path,
};
pub(crate) fn write_from(path: &Path, source: &mut impl Read, overwrite: bool) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    // Prepare beside the destination, on the same filesystem. Dropping the
    // temporary file on any preparation failure preserves the previous snapshot.
    let mut file = tempfile::Builder::new()
        .prefix(".cad-craft-")
        .suffix(".tmp")
        .tempfile_in(parent)?;
    io::copy(source, &mut file)?;
    file.as_file().sync_all()?;
    if overwrite {
        file.persist(path).map_err(|error| error.error)?;
    } else {
        file.persist_noclobber(path).map_err(|error| error.error)?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Broken {
        sent: bool,
    }
    impl Read for Broken {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.sent {
                Err(io::Error::other("injected I/O failure"))
            } else {
                self.sent = true;
                buf[0] = b'{';
                Ok(1)
            }
        }
    }
    #[test]
    fn failed_snapshot_preparation_preserves_previous_file_and_cleans_temporary() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("snapshot.json");
        std::fs::write(&path, "previous complete snapshot").unwrap();
        assert!(write_from(&path, &mut Broken { sent: false }, true).is_err());
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "previous complete snapshot"
        );
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
    #[test]
    fn snapshot_replacement_requires_explicit_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("snapshot.json");
        std::fs::write(&path, "old").unwrap();
        assert!(write_from(&path, &mut "new".as_bytes(), false).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "old");
        write_from(&path, &mut "new".as_bytes(), true).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
    }
}
