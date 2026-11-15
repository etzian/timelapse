//! The files the app made, found by id for "Play" and "Show in Finder" (ADR-027).
//!
//! The UI never passes a path: it names a kind and an id, and the path is the one stored
//! in SQLite (`video_exports.output_path`, `contact_sheets.output_path`; an export's
//! record lies beside its video). The path is canonicalised and refused unless it is an
//! existing regular file inside `<root>/Timelapse exports/` (ADR-026) with the extension
//! the app writes for that kind, so a stored path that was edited, a `..` escape, a
//! symlink out of the folder or a planted program opens nothing.
//!
//! Reading only: the database is opened with [`store::open_read_only`], so nothing is
//! created, migrated, settled or moved. A project whose database still needs its migration,
//! or whose files still sit in the pre-ADR-026 layout, answers `unknown` until a command
//! that opens it for work has brought it up to date.

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::OptionalExtension;

use crate::import::{default_db_dir, store};
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

impl OutputKind {
    /// The extensions (lowercase) of the files the app writes for this kind: an export is
    /// always an MP4 (`export::run`), a contact sheet a PDF, a record JSON.
    fn extensions(self) -> &'static [&'static str] {
        match self {
            Self::Export => &["mp4"],
            Self::ContactSheet => &["pdf"],
            Self::ExportRecord => &["json"],
        }
    }
}

fn refused(reason: &'static str) -> Error {
    Error::Output { reason }
}

/// The stored path of `kind` `id`, as recorded (not yet checked).
fn stored_path(root: &Path, kind: OutputKind, id: i64) -> Result<PathBuf> {
    if crate::layout::has_legacy(root) {
        return Err(refused("unknown"));
    }
    let conn = match store::open_read_only(default_db_dir(root).as_path())? {
        store::ReadOnly::Ready(conn) => conn,
        store::ReadOnly::Missing | store::ReadOnly::Outdated => return Err(refused("unknown")),
    };
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
/// the canonical source folder, itself canonicalised so the comparison uses the same
/// spelling as the file's canonical path.
fn canonical_exports(root: &Path) -> Result<PathBuf> {
    let unreadable = || Error::ImportRootUnreadable { path: root.to_string_lossy().into_owned() };
    let root = fs::canonicalize(root).map_err(|_| unreadable())?;
    let dir = exports_dir(&root);
    match fs::symlink_metadata(&dir) {
        Ok(m) if m.file_type().is_dir() => fs::canonicalize(&dir).map_err(|_| refused("missing")),
        Ok(_) => Err(refused("outside-exports")),
        Err(_) => Err(refused("missing")),
    }
}

/// Checks `path` and returns its canonical form, which is what may be opened: it exists,
/// is a regular file inside the canonical `exports` folder, has a name the opener can take
/// and an extension the app writes for `kind`.
fn checked(exports: &Path, kind: OutputKind, path: &Path) -> Result<String> {
    if !path.is_absolute() {
        return Err(refused("outside-exports"));
    }
    let canonical = fs::canonicalize(path).map_err(|_| refused("missing"))?;
    // A strict prefix: the folder itself is a directory and could never pass anyway.
    if !canonical.starts_with(exports) {
        return Err(refused("outside-exports"));
    }
    let meta = fs::symlink_metadata(&canonical).map_err(|_| refused("missing"))?;
    if !meta.file_type().is_file() {
        return Err(refused("not-a-file"));
    }
    let ext = canonical.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase);
    if !ext.is_some_and(|e| kind.extensions().contains(&e.as_str())) {
        return Err(refused("unsupported-type"));
    }
    // The opener takes a string: a name that is not valid Unicode is refused rather than
    // approximated (a lossy conversion could name another file).
    canonical.into_os_string().into_string().map_err(|_| refused("unsupported-name"))
}

/// The canonical path of the file `kind` `id` of `root`, as a string the opener can take,
/// or a refusal. A folder that cannot be reached is `import-root-unreadable`; otherwise the
/// error is `output` with reason:
/// - `unknown`: no such id, a folder never imported, or a project not yet brought up to
///   date (database migration or pre-ADR-026 layout pending);
/// - `missing`: no exports folder, or no file at the stored path;
/// - `not-a-file`: the stored path is not a regular file;
/// - `outside-exports`: the canonical path is not inside `<root>/Timelapse exports/` (a
///   relative path, a `..` escape, a symlink out, or an exports folder that is a symlink);
/// - `unsupported-type`: not the extension the app writes for `kind` (mp4, pdf, json);
/// - `unsupported-name`: the path is not valid Unicode.
///
/// The shell adds `open-failed` when the system opener itself fails.
pub fn resolve(root: &Path, kind: OutputKind, id: i64) -> Result<String> {
    let exports = canonical_exports(root)?;
    let path = stored_path(root, kind, id)?;
    checked(&exports, kind, &path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::metadata::fixture::TempDir;
    use crate::import::open_root;

    fn resolved(root: &Path, kind: OutputKind, id: i64) -> Result<PathBuf> {
        resolve(root, kind, id).map(PathBuf::from)
    }

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
        assert_eq!(resolved(root, OutputKind::Export, 1).unwrap(), canon.join("clip.mp4"));
        assert_eq!(
            resolved(root, OutputKind::ContactSheet, 1).unwrap(),
            canon.join("contact-sheets").join("1.pdf")
        );
        assert_eq!(
            resolved(root, OutputKind::ExportRecord, 1).unwrap(),
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
        assert_eq!(reason(resolved(dir.path(), OutputKind::Export, 2)), "output:unknown");
        assert_eq!(reason(resolved(dir.path(), OutputKind::ContactSheet, 7)), "output:unknown");
        assert_eq!(reason(resolved(dir.path(), OutputKind::ExportRecord, 2)), "output:unknown");

        // Never imported: refused, and no `.timelapse/` is created by asking.
        let bare = TempDir::new("p10-bare");
        fs::create_dir_all(exports_dir(bare.path())).unwrap();
        assert_eq!(reason(resolved(bare.path(), OutputKind::Export, 1)), "output:unknown");
        assert!(!crate::import::project_dir(bare.path()).exists());
        // A folder that is gone is reported as unreachable.
        let err = resolved(&bare.path().join("gone"), OutputKind::Export, 1).unwrap_err();
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
        assert_eq!(reason(resolved(root, OutputKind::Export, 1)), "output:missing");
        // The folder exists, the video does not (and so its record does not either).
        fs::create_dir_all(&exports).unwrap();
        assert_eq!(reason(resolved(root, OutputKind::Export, 1)), "output:missing");
        assert_eq!(reason(resolved(root, OutputKind::ExportRecord, 1)), "output:missing");
        // A directory where the file should be.
        fs::create_dir_all(exports.join("clip.mp4")).unwrap();
        assert_eq!(reason(resolved(root, OutputKind::Export, 1)), "output:not-a-file");
        // The exports folder itself.
        set_output(root, "video_exports", &exports);
        assert_eq!(reason(resolved(root, OutputKind::Export, 1)), "output:not-a-file");
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
        assert_eq!(reason(resolved(root, OutputKind::Export, 1)), "output:outside-exports");
        assert_eq!(reason(resolved(root, OutputKind::ContactSheet, 1)), "output:outside-exports");
        // A photograph of the collection, in the source folder itself.
        set_output(root, "video_exports", &root.join("a.jpg"));
        assert_eq!(reason(resolved(root, OutputKind::Export, 1)), "output:outside-exports");
        // A `..` escape that starts inside the folder.
        set_output(root, "video_exports", &exports.join("..").join("a.jpg"));
        assert_eq!(reason(resolved(root, OutputKind::Export, 1)), "output:outside-exports");
        // A relative path is never resolved against the working directory.
        set_output(root, "video_exports", Path::new("Timelapse exports/clip.mp4"));
        assert_eq!(reason(resolved(root, OutputKind::Export, 1)), "output:outside-exports");
        // The project database, next door in `.timelapse/`.
        let db = fs::read_dir(default_db_dir(root))
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .find(|p| p.is_file())
            .unwrap();
        set_output(root, "video_exports", &db);
        assert_eq!(reason(resolved(root, OutputKind::Export, 1)), "output:outside-exports");
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
        assert_eq!(reason(resolved(root, OutputKind::Export, 1)), "output:outside-exports");
        // The record beside it, a link out as well.
        symlink(&target, exports.join("clip.record.json")).unwrap();
        assert_eq!(reason(resolved(root, OutputKind::ExportRecord, 1)), "output:outside-exports");
        // A directory link inside the folder, pointing out.
        symlink(outside.path(), exports.join("linked")).unwrap();
        set_output(root, "contact_sheets", &exports.join("linked").join("secret.txt"));
        assert_eq!(reason(resolved(root, OutputKind::ContactSheet, 1)), "output:outside-exports");
        // A link inside pointing to a file inside is the file itself.
        fs::write(exports.join("real.mp4"), b"video").unwrap();
        fs::remove_file(exports.join("clip.mp4")).unwrap();
        symlink(exports.join("real.mp4"), exports.join("clip.mp4")).unwrap();
        assert_eq!(
            resolved(root, OutputKind::Export, 1).unwrap(),
            fs::canonicalize(exports.join("real.mp4")).unwrap()
        );

        // The exports folder itself replaced by a link to elsewhere: nothing is opened.
        let other = TempDir::new("p10-root");
        let dir2 = project(&other.path().join("x.mp4"), &other.path().join("x.pdf"));
        let root2 = dir2.path();
        symlink(outside.path(), exports_dir(root2)).unwrap();
        set_output(root2, "video_exports", &exports_dir(root2).join("secret.txt"));
        assert_eq!(reason(resolved(root2, OutputKind::Export, 1)), "output:outside-exports");
    }

    #[test]
    fn only_the_file_types_the_app_writes_are_opened() {
        let probe = TempDir::new("p10-probe");
        let dir = project(&probe.path().join("x.mp4"), &probe.path().join("x.pdf"));
        let root = dir.path();
        let exports = exports_dir(root);
        fs::create_dir_all(&exports).unwrap();

        // Programs planted in the folder, under every kind.
        fs::write(exports.join("run.command"), b"#!/bin/sh\necho hi\n").unwrap();
        fs::create_dir_all(exports.join("Tool.app/Contents")).unwrap();
        fs::write(exports.join("Tool.app/Contents/Info.plist"), b"<plist/>").unwrap();
        fs::write(exports.join("noext"), b"x").unwrap();
        for planted in ["run.command", "Tool.app/Contents/Info.plist", "noext"] {
            set_output(root, "video_exports", &exports.join(planted));
            set_output(root, "contact_sheets", &exports.join(planted));
            assert_eq!(reason(resolved(root, OutputKind::Export, 1)), "output:unsupported-type");
            assert_eq!(
                reason(resolved(root, OutputKind::ContactSheet, 1)),
                "output:unsupported-type"
            );
        }
        // An `.app` bundle is a directory: refused before its type is looked at.
        set_output(root, "video_exports", &exports.join("Tool.app"));
        assert_eq!(reason(resolved(root, OutputKind::Export, 1)), "output:not-a-file");
        // The right extension for another kind is still refused.
        fs::write(exports.join("sheet.pdf"), b"pdf").unwrap();
        fs::write(exports.join("clip.MP4"), b"video").unwrap();
        set_output(root, "video_exports", &exports.join("sheet.pdf"));
        assert_eq!(reason(resolved(root, OutputKind::Export, 1)), "output:unsupported-type");
        set_output(root, "contact_sheets", &exports.join("clip.MP4"));
        assert_eq!(reason(resolved(root, OutputKind::ContactSheet, 1)), "output:unsupported-type");
        // The record beside the video is JSON and passes.
        set_output(root, "video_exports", &exports.join("clip.MP4"));
        fs::write(exports.join("clip.record.json"), b"{}").unwrap();
        assert!(resolved(root, OutputKind::ExportRecord, 1).is_ok());
        // Allowed types pass, whatever their case.
        let canon = fs::canonicalize(&exports).unwrap();
        assert_eq!(resolved(root, OutputKind::Export, 1).unwrap(), canon.join("clip.MP4"));
        set_output(root, "contact_sheets", &exports.join("sheet.pdf"));
        assert_eq!(resolved(root, OutputKind::ContactSheet, 1).unwrap(), canon.join("sheet.pdf"));
    }

    #[test]
    fn resolving_writes_nothing_and_leaves_a_project_to_bring_up_to_date_alone() {
        let probe = TempDir::new("p10-probe");
        let dir = project(&probe.path().join("x.mp4"), &probe.path().join("x.pdf"));
        let root = dir.path();
        let exports = exports_dir(root);
        fs::create_dir_all(&exports).unwrap();
        fs::write(exports.join("clip.mp4"), b"video").unwrap();
        set_output(root, "video_exports", &exports.join("clip.mp4"));
        let db = default_db_dir(root).join("timelapse.db");
        let bytes = fs::read(&db).unwrap();
        let mtime = fs::metadata(&db).unwrap().modified().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));

        assert!(resolved(root, OutputKind::Export, 1).is_ok());
        assert_eq!(fs::read(&db).unwrap(), bytes);
        assert_eq!(fs::metadata(&db).unwrap().modified().unwrap(), mtime);

        // A pre-ADR-026 layout: nothing is moved, and the answer is `unknown`.
        let legacy = crate::import::project_dir(root).join("contact-sheets");
        fs::create_dir_all(&legacy).unwrap();
        fs::write(legacy.join("1.pdf"), b"pdf").unwrap();
        assert_eq!(reason(resolved(root, OutputKind::Export, 1)), "output:unknown");
        assert!(legacy.join("1.pdf").is_file());
        assert!(!crate::layout::sheets_dir(root).exists());
        fs::remove_dir_all(&legacy).unwrap();

        // A database from an older build: not migrated, `unknown`.
        rusqlite::Connection::open(&db).unwrap().pragma_update(None, "user_version", 1).unwrap();
        let bytes = fs::read(&db).unwrap();
        assert_eq!(reason(resolved(root, OutputKind::Export, 1)), "output:unknown");
        assert_eq!(fs::read(&db).unwrap(), bytes);
    }
}
