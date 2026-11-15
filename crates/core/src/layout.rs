//! Where the app writes inside a source folder (ADR-026).
//!
//! - `<root>/.timelapse/`: the project's own data: database, review thumbnails and
//!   temporary files of running work (`tmp/`). It can be erased at any time (Settings,
//!   Erase all data); only derived data and the project's history are lost.
//! - `<root>/Timelapse exports/`: what the user asked the app to make: videos, their
//!   records and the contact sheets (`contact-sheets/`).
//! - `<root>/Timelapse quarantine/`: the user's rejected originals, kept until restored.
//!
//! The import scan never descends into any of the three. Projects made before ADR-026 kept
//! the exports, contact sheets and quarantine inside `.timelapse/`; [`relocate_legacy`]
//! moves them out (without replacing anything) and rewrites the stored paths, so erasing
//! `.timelapse/` can never delete a photograph or a video.

use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};

use rusqlite::{Connection, params};

use crate::import::project_dir;
use crate::{Error, Result};

/// The visible folder of what the app made, at the top of the source folder.
pub const EXPORTS_DIR: &str = "Timelapse exports";
/// The visible folder of quarantined originals, at the top of the source folder.
pub const QUARANTINE_DIR: &str = "Timelapse quarantine";
const SHEETS_DIR: &str = "contact-sheets";
const TEMP_DIR: &str = "tmp";

pub fn exports_dir(root: &Path) -> PathBuf {
    root.join(EXPORTS_DIR)
}

pub fn sheets_dir(root: &Path) -> PathBuf {
    exports_dir(root).join(SHEETS_DIR)
}

pub fn quarantine_dir(root: &Path) -> PathBuf {
    root.join(QUARANTINE_DIR)
}

/// Temporary files of running work (`.part` files), inside `.timelapse/`.
pub fn temp_dir(root: &Path) -> PathBuf {
    project_dir(root).join(TEMP_DIR)
}

/// The folders at the top of a source folder that hold the app's own files, not the
/// collection. Compared as the volume does (case and normalisation ignored).
pub fn is_app_folder(name: &OsStr) -> bool {
    let key = crate::files::name_key(&name.to_string_lossy());
    [EXPORTS_DIR, QUARANTINE_DIR].iter().any(|d| crate::files::name_key(d) == key)
}

/// Moves `from` into place at `to` without replacing anything: a rename, or (another
/// volume) a copy then removal. Fails if `to` exists.
pub(crate) fn place_new(from: &Path, to: &Path) -> std::io::Result<()> {
    if fs::symlink_metadata(to).is_ok() {
        return Err(std::io::ErrorKind::AlreadyExists.into());
    }
    if fs::rename(from, to).is_ok() {
        return Ok(());
    }
    let mut out = fs::OpenOptions::new().write(true).create_new(true).open(to)?;
    let copied = fs::File::open(from)
        .and_then(|mut i| std::io::copy(&mut i, &mut out))
        .and_then(|_| out.sync_all());
    if let Err(e) = copied {
        drop(out);
        let _ = fs::remove_file(to);
        return Err(e);
    }
    let _ = fs::remove_file(from);
    Ok(())
}

/// The old folders inside `.timelapse/` and where each now lives, as path components
/// relative to the source folder.
const LEGACY: [(&[&str], &[&str]); 3] = [
    (&[".timelapse", "quarantine"], &[QUARANTINE_DIR]),
    (&[".timelapse", "exports"], &[EXPORTS_DIR]),
    (&[".timelapse", "contact-sheets"], &[EXPORTS_DIR, SHEETS_DIR]),
];

fn under(root: &Path, parts: &[&str]) -> PathBuf {
    parts.iter().fold(root.to_path_buf(), |p, c| p.join(c))
}

/// Whether a project made before ADR-026 still has files to move out of `.timelapse/`.
pub(crate) fn has_legacy(root: &Path) -> bool {
    LEGACY.iter().any(|(old, _)| under(root, old).is_dir())
}

/// Move the files of a pre-ADR-026 project out of `.timelapse/` and point the database at
/// them. Idempotent; never replaces a file (one already at the new place stays, and so does
/// the old one, with its row unchanged). The caller holds the file-operation lock.
pub(crate) fn relocate_legacy(conn: &Connection, root: &Path) -> Result<()> {
    for (old, new) in LEGACY {
        let from = under(root, old);
        if from.is_dir() {
            merge(&from, &under(root, new));
        }
    }
    rewrite_paths(conn)
}

fn merge(from: &Path, to: &Path) {
    let _ = fs::create_dir_all(to);
    let Ok(entries) = fs::read_dir(from) else { return };
    for entry in entries.flatten() {
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        match entry.file_type() {
            Ok(t) if t.is_dir() => merge(&src, &dst),
            // A `.part` is a write an older version never finished: it is not kept.
            Ok(t) if t.is_file() && src.extension() == Some(OsStr::new("part")) => {
                let _ = fs::remove_file(&src);
            }
            Ok(t) if t.is_file() => {
                let _ = place_new(&src, &dst);
            }
            _ => {}
        }
    }
    let _ = fs::remove_dir(from); // only if now empty
}

/// `path` with its legacy folder replaced by the new one, if it lies in one.
fn relocated(path: &str) -> Option<String> {
    let comps: Vec<Component> = Path::new(path).components().collect();
    for (old, new) in LEGACY {
        let Some(at) = comps
            .windows(old.len())
            .position(|w| w.iter().zip(old.iter()).all(|(c, o)| c.as_os_str() == OsStr::new(o)))
        else {
            continue;
        };
        let mut p = PathBuf::new();
        comps[..at].iter().for_each(|c| p.push(c));
        new.iter().for_each(|c| p.push(c));
        comps[at + old.len()..].iter().for_each(|c| p.push(c));
        return Some(p.to_string_lossy().into_owned());
    }
    None
}

/// Stored paths into a legacy folder follow the file when it was moved (the new file is
/// there, or the old one is gone).
fn rewrite_paths(conn: &Connection) -> Result<()> {
    const COLUMNS: [(&str, &str); 5] = [
        ("images", "current_path"),
        ("file_operations", "source_path"),
        ("file_operations", "destination_path"),
        ("video_exports", "output_path"),
        ("contact_sheets", "output_path"),
    ];
    let db = |_| Error::StoreTransaction;
    let tx = conn.unchecked_transaction().map_err(db)?;
    for (table, column) in COLUMNS {
        let rows: Vec<(i64, String)> = {
            let mut stmt = tx
                .prepare(&format!(
                    "SELECT id, {column} FROM {table} WHERE {column} LIKE '%.timelapse%'"
                ))
                .map_err(db)?;
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).map_err(db)?;
            rows.collect::<std::result::Result<_, _>>().map_err(db)?
        };
        for (id, old) in rows {
            let Some(new) = relocated(&old) else { continue };
            if Path::new(&new).exists() || !Path::new(&old).exists() {
                tx.execute(
                    &format!("UPDATE {table} SET {column} = ?1 WHERE id = ?2"),
                    params![new, id],
                )
                .map_err(db)?;
            }
        }
    }
    tx.commit().map_err(db)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_paths_are_rewritten_component_by_component() {
        let q = Path::new("/p").join(".timelapse").join("quarantine").join("a").join("b.jpg");
        let want = Path::new("/p").join(QUARANTINE_DIR).join("a").join("b.jpg");
        assert_eq!(relocated(&q.to_string_lossy()).map(PathBuf::from), Some(want));
        let s = Path::new("/p").join(".timelapse").join("contact-sheets").join("3.pdf");
        let want = Path::new("/p").join(EXPORTS_DIR).join("contact-sheets").join("3.pdf");
        assert_eq!(relocated(&s.to_string_lossy()).map(PathBuf::from), Some(want));
        assert_eq!(
            relocated(&Path::new("/p").join(".timelapse").join("db").to_string_lossy()),
            None
        );
        assert_eq!(relocated("/p/a.jpg"), None);
    }

    #[test]
    fn app_folders_are_recognised_whatever_the_case() {
        assert!(is_app_folder(OsStr::new("Timelapse exports")));
        assert!(is_app_folder(OsStr::new("timelapse QUARANTINE")));
        assert!(!is_app_folder(OsStr::new("exports")));
    }
}
