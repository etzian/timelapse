//! The files the app made, found by id for "Play" and "Show in Finder" (ADR-027).
//!
//! The UI never passes a path: it names a kind and an id, and the path is the one stored
//! in SQLite (`video_exports.output_path`, `contact_sheets.output_path`; an export's
//! record lies beside its video). The path is canonicalised and refused unless it is an
//! existing regular file inside `<root>/Timelapse exports/` (ADR-026), so a stored path
//! that was edited, a `..` escape or a symlink out of the folder opens nothing. Reading
//! only: nothing is created, written or moved.

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::OptionalExtension;

use crate::import::{default_db_dir, open_root, store};
use crate::layout::exports_dir;
use crate::{Error, Result};

/// What the id names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputKind {
    /// A video (`video_exports`).
    Export,
    /// A contact sheet PDF (`contact_sheets`).
    ContactSheet,
    /// The saved record of a video (`<video name>.record.json`, beside it).
    ExportRecord,
}

fn refused(reason: &'static str) -> Error {
    Error::Output { reason }
}

/// The stored path of `kind` `id`, as recorded (not yet checked).
fn stored_path(root: &Path, kind: OutputKind, id: i64) -> Result<PathBuf> {
    if !store::db_exists(default_db_dir(root).as_path()) {
        return Err(refused("unknown"));
    }
    let conn = open_root(root)?;
    let table = match kind {
        OutputKind::Export | OutputKind::ExportRecord => "video_exports",
        OutputKind::ContactSheet => "contact_sheets",
    };
    let stored: Option<String> = conn
        .query_row(&format!("SELECT output_path FROM {table} WHERE id = ?1"), [id], |r| r.get(0))
        .optional()
        .map_err(|_| Error::StoreTransaction)?;
    let stored = stored.ok_or_else(|| refused("unknown"))?;
    Ok(match kind {
        OutputKind::ExportRecord => crate::export::record::record_path_for(root, id, &stored),
        _ => PathBuf::from(stored),
    })
}

/// The canonical exports folder of `root`: a real directory (not a symlink) directly in
/// the canonical source folder.
fn canonical_exports(root: &Path) -> Result<PathBuf> {
    let unreadable = || Error::ImportRootUnreadable { path: root.to_string_lossy().into_owned() };
    let root = fs::canonicalize(root).map_err(|_| unreadable())?;
    let dir = exports_dir(&root);
    match fs::symlink_metadata(&dir) {
        Ok(m) if m.file_type().is_dir() => Ok(dir),
        Ok(_) => Err(refused("outside-exports")),
        Err(_) => Err(refused("missing")),
    }
}

/// Checks `path` and returns its canonical form, which is what may be opened: it exists,
/// is a regular file and lies inside the canonical `exports` folder.
fn checked(exports: &Path, path: &Path) -> Result<PathBuf> {
    if !path.is_absolute() {
        return Err(refused("outside-exports"));
    }
    let canonical = fs::canonicalize(path).map_err(|_| refused("missing"))?;
    if !canonical.starts_with(exports) || canonical == exports {
        return Err(refused("outside-exports"));
    }
    let meta = fs::symlink_metadata(&canonical).map_err(|_| refused("missing"))?;
    if !meta.file_type().is_file() {
        return Err(refused("not-a-file"));
    }
    // The opener takes a string: a name that is not valid Unicode is refused rather than
    // approximated (a lossy conversion could name another file).
    if canonical.to_str().is_none() {
        return Err(refused("not-a-file"));
    }
    Ok(canonical)
}

/// The canonical path of the file `kind` `id` of `root`, or a refusal
/// (`output` with reason `unknown`, `missing`, `not-a-file` or `outside-exports`).
pub fn resolve(root: &Path, kind: OutputKind, id: i64) -> Result<PathBuf> {
    let exports = canonical_exports(root)?;
    let path = stored_path(root, kind, id)?;
    checked(&exports, &path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::metadata::fixture::TempDir;

    /// A project with one export (`clip.mp4`) and one contact sheet (`sheet.pdf`) recorded at
    /// the given paths; returns the folder.
    fn project(video: &Path, sheet: &Path) -> TempDir {
        let dir = TempDir::new("p10-outputs");
        let root = dir.path();
        fs::write(root.join("a.jpg"), b"photo").unwrap();
        crate::import::import_folder(root).unwrap();
        let conn = open_root(root).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = OFF").unwrap();
        conn.execute(
            "INSERT INTO video_exports (id, sampling_run_id, settings_json, argv_json, output_path,
                 status, input_photos, output_frames, duration_seconds, created_at)
             VALUES (1, 1, '{}', '[]', ?1, 'completed', 1, 1, 0.1, 5)",
            [video.to_string_lossy()],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO contact_sheets (id, source_json, layout_json, labels_json, output_path,
                 photos, pages, status, created_at)
             VALUES (1, '{}', '{}', '{}', ?1, 1, 1, 'completed', 5)",
            [sheet.to_string_lossy()],
        )
        .unwrap();
        dir
    }

    fn reason(r: Result<PathBuf>) -> String {
        let e = r.unwrap_err();
        format!("{}:{}", e.kind(), e.args()["reason"].as_str().unwrap_or(""))
    }

    fn set_output(root: &Path, table: &str, path: &Path) {
        open_root(root)
            .unwrap()
            .execute(
                &format!("UPDATE {table} SET output_path = ?1 WHERE id = 1"),
                [path.to_string_lossy()],
            )
            .unwrap();
    }

    #[test]
    fn files_inside_the_exports_folder_resolve_to_their_canonical_path() {
        let dir = project(Path::new("/nowhere/clip.mp4"), Path::new("/nowhere/1.pdf"));
        let root = dir.path();
        let exports = exports_dir(root);
        let sheets = crate::layout::sheets_dir(root);
        fs::create_dir_all(&sheets).unwrap();
        set_output(root, "video_exports", &exports.join("clip.mp4"));
        set_output(root, "contact_sheets", &sheets.join("1.pdf"));
        fs::write(exports.join("clip.mp4"), b"video").unwrap();
        fs::write(sheets.join("1.pdf"), b"pdf").unwrap();
        fs::write(exports.join("clip.record.json"), b"{}").unwrap();

        let canon = fs::canonicalize(&exports).unwrap();
        assert_eq!(resolve(root, OutputKind::Export, 1).unwrap(), canon.join("clip.mp4"));
        assert_eq!(
            resolve(root, OutputKind::ContactSheet, 1).unwrap(),
            canon.join("contact-sheets").join("1.pdf")
        );
        assert_eq!(
            resolve(root, OutputKind::ExportRecord, 1).unwrap(),
            canon.join("clip.record.json")
        );
        // Kinds cross IPC in kebab case.
        let kind: OutputKind = serde_json::from_str("\"export-record\"").unwrap();
        assert_eq!(kind, OutputKind::ExportRecord);
        assert!(serde_json::from_str::<OutputKind>("\"photo\"").is_err());
    }

    #[test]
    fn an_unknown_id_or_a_folder_never_imported_is_refused() {
        let probe = TempDir::new("p10-probe");
        let dir = project(&probe.path().join("x.mp4"), &probe.path().join("x.pdf"));
        fs::create_dir_all(exports_dir(dir.path())).unwrap();
        assert_eq!(reason(resolve(dir.path(), OutputKind::Export, 2)), "output:unknown");
        assert_eq!(reason(resolve(dir.path(), OutputKind::ContactSheet, 7)), "output:unknown");
        assert_eq!(reason(resolve(dir.path(), OutputKind::ExportRecord, 2)), "output:unknown");

        // Never imported: refused, and no `.timelapse/` is created by asking.
        let bare = TempDir::new("p10-bare");
        fs::create_dir_all(exports_dir(bare.path())).unwrap();
        assert_eq!(reason(resolve(bare.path(), OutputKind::Export, 1)), "output:unknown");
        assert!(!crate::import::project_dir(bare.path()).exists());
        // A folder that is gone is reported as unreachable.
        let err = resolve(&bare.path().join("gone"), OutputKind::Export, 1).unwrap_err();
        assert_eq!(err.kind(), "import-root-unreadable");
    }

    #[test]
    fn a_missing_file_or_a_folder_is_refused() {
        let probe = TempDir::new("p10-probe");
        let dir = project(&probe.path().join("x.mp4"), &probe.path().join("x.pdf"));
        let root = dir.path();
        let exports = exports_dir(root);
        // No exports folder at all.
        set_output(root, "video_exports", &exports.join("clip.mp4"));
        assert_eq!(reason(resolve(root, OutputKind::Export, 1)), "output:missing");
        // The folder exists, the video does not (and so its record does not either).
        fs::create_dir_all(&exports).unwrap();
        assert_eq!(reason(resolve(root, OutputKind::Export, 1)), "output:missing");
        assert_eq!(reason(resolve(root, OutputKind::ExportRecord, 1)), "output:missing");
        // A directory where the file should be.
        fs::create_dir_all(exports.join("clip.mp4")).unwrap();
        assert_eq!(reason(resolve(root, OutputKind::Export, 1)), "output:not-a-file");
        // The exports folder itself.
        set_output(root, "video_exports", &exports);
        assert_eq!(reason(resolve(root, OutputKind::Export, 1)), "output:outside-exports");
    }

    #[test]
    fn a_path_outside_the_exports_folder_or_escaping_it_is_refused() {
        let outside = TempDir::new("p10-outside");
        let elsewhere = outside.path().join("elsewhere.mp4");
        fs::write(&elsewhere, b"video").unwrap();
        let dir = project(&elsewhere, &outside.path().join("sheet.pdf"));
        fs::write(outside.path().join("sheet.pdf"), b"pdf").unwrap();
        let root = dir.path();
        let exports = exports_dir(root);
        fs::create_dir_all(&exports).unwrap();
        // Existing files outside the folder, whatever the kind.
        assert_eq!(reason(resolve(root, OutputKind::Export, 1)), "output:outside-exports");
        assert_eq!(reason(resolve(root, OutputKind::ContactSheet, 1)), "output:outside-exports");
        // A photograph of the collection, in the source folder itself.
        set_output(root, "video_exports", &root.join("a.jpg"));
        assert_eq!(reason(resolve(root, OutputKind::Export, 1)), "output:outside-exports");
        // A `..` escape that starts inside the folder.
        set_output(root, "video_exports", &exports.join("..").join("a.jpg"));
        assert_eq!(reason(resolve(root, OutputKind::Export, 1)), "output:outside-exports");
        // A relative path is never resolved against the working directory.
        set_output(root, "video_exports", Path::new("Timelapse exports/clip.mp4"));
        assert_eq!(reason(resolve(root, OutputKind::Export, 1)), "output:outside-exports");
        // The project database, next door in `.timelapse/`.
        let db = fs::read_dir(default_db_dir(root))
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .find(|p| p.is_file())
            .unwrap();
        set_output(root, "video_exports", &db);
        assert_eq!(reason(resolve(root, OutputKind::Export, 1)), "output:outside-exports");
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_pointing_out_of_the_exports_folder_is_refused() {
        use std::os::unix::fs::symlink;
        let outside = TempDir::new("p10-outside");
        let target = outside.path().join("secret.txt");
        fs::write(&target, b"not ours").unwrap();
        let dir = project(&outside.path().join("x.mp4"), &outside.path().join("x.pdf"));
        let root = dir.path();
        let exports = exports_dir(root);
        fs::create_dir_all(crate::layout::sheets_dir(root)).unwrap();

        // A file link inside the folder, pointing out.
        symlink(&target, exports.join("clip.mp4")).unwrap();
        set_output(root, "video_exports", &exports.join("clip.mp4"));
        assert_eq!(reason(resolve(root, OutputKind::Export, 1)), "output:outside-exports");
        // The record beside it, a link out as well.
        symlink(&target, exports.join("clip.record.json")).unwrap();
        assert_eq!(reason(resolve(root, OutputKind::ExportRecord, 1)), "output:outside-exports");
        // A directory link inside the folder, pointing out.
        symlink(outside.path(), exports.join("linked")).unwrap();
        set_output(root, "contact_sheets", &exports.join("linked").join("secret.txt"));
        assert_eq!(reason(resolve(root, OutputKind::ContactSheet, 1)), "output:outside-exports");
        // A link inside pointing to a file inside is the file itself.
        fs::write(exports.join("real.mp4"), b"video").unwrap();
        fs::remove_file(exports.join("clip.mp4")).unwrap();
        symlink(exports.join("real.mp4"), exports.join("clip.mp4")).unwrap();
        assert_eq!(
            resolve(root, OutputKind::Export, 1).unwrap(),
            fs::canonicalize(exports.join("real.mp4")).unwrap()
        );

        // The exports folder itself replaced by a link to elsewhere: nothing is opened.
        let other = TempDir::new("p10-root");
        let dir2 = project(&other.path().join("x.mp4"), &other.path().join("x.pdf"));
        let root2 = dir2.path();
        symlink(outside.path(), exports_dir(root2)).unwrap();
        set_output(root2, "video_exports", &exports_dir(root2).join("secret.txt"));
        assert_eq!(reason(resolve(root2, OutputKind::Export, 1)), "output:outside-exports");
    }
}
