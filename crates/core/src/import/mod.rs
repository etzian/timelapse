//! Public facade for Phase 1: import & inventory.
//!
//! The Tauri shell is a thin adapter over this module; everything heavy (walking,
//! hashing, SQLite persistence, integrity checks) lives here so `crates/core`
//! stays free of any `tauri` dependency (CLAUDE.md "core invariants").
//!
//! Every public function takes only a source `root` and knows where its own
//! database is (via [`default_db_dir`]) — the shell does not have to think about paths.

use std::path::{Path, PathBuf};

mod dedup;
mod scan;
pub mod store;

use crate::{Error, Result};

pub use scan::{Candidate, SUPPORTED_EXTENSIONS as IMAGE_EXTENSIONS};

/// Standard location of the project's database: `<root>/.timelapse/db`. Keeping it inside
/// the source tree means moving the folder moves its data too, and no global app-state
/// dir is required (ADR-011). The only public function with a `db_dir` parameter —
/// internal callers in this crate use `default_db_dir`, so the shell never has to.
pub fn default_db_dir(root: &Path) -> PathBuf {
    root.join(".timelapse").join("db")
}

/// Display name for a source root when no explicit name is given: the folder's own name,
/// e.g. `C:\Photos\2024` → `2024`. Phase 2 will introduce an explicit project-file model
/// where this is renamed by the user; in Phase 1 the path is the identity (ADR-007).
fn derive_name(root: &Path) -> String {
    root.file_name()
        .and_then(|s| s.to_str())
        .map(String::from)
        .unwrap_or_else(|| root.to_string_lossy().into_owned())
}

/// Everything the UI needs to render the import report and inventory.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportReport {
    pub scanned_count: usize,
    pub hash_computed: usize,
    /// Number of files that are content duplicates (appear 2+ times in this tree).
    pub duplicates_found: usize,
    /// Distinct duplicate *groups:* one entry per distinct content appearing 2+ times.
    pub duplicate_groups: Vec<Vec<String>>,
    pub new_rows: usize,
    pub unchanged: usize,
    /// Files present on disk that could not be hashed (permissions / I/O). They are still
    /// imported without a hash so they can be inspected.
    pub skipped_unreadable: usize,
}

/// Phase 1 "import": scan `root`, SHA-256 the supported files, deduplicate on byte equality
/// and upsert into the project's SQLite database (idempotent by original path). Returns
/// a report. The connection is dropped when this function returns — the shell holds no state.
pub fn import_folder(root: &Path) -> Result<ImportReport> {
    let name = derive_name(root);
    let conn = store::open(default_db_dir(root).as_path(), &name, root, Some(root))?;

    let candidates = scan::scan(root)?;
    let n = candidates.len();

    let mut hashes: Vec<Option<String>> = Vec::with_capacity(n);
    let mut skipped_unreadable = 0usize;
    for c in &candidates {
        match dedup::hash_file(&c.path) {
            Ok(h) => hashes.push(Some(h)),
            Err(Error::ImageUnreadable { .. }) => {
                hashes.push(None);
                skipped_unreadable += 1;
            }
            Err(e) => return Err(e),
        }
    }

    let groups = dedup::duplicate_groups(&candidates, &hashes);
    let mut dup_flag = vec![false; n];
    for g in &groups {
        for i in &g.indices {
            if let Some(slot) = dup_flag.get_mut(*i) {
                *slot = true;
            }
        }
    }

    let counts = store::upsert_images(&conn, &candidates, &hashes, &dup_flag)?;

    let duplicate_groups: Vec<Vec<String>> = groups
        .iter()
        .map(|g| {
            g.indices
                .iter()
                .map(|i| candidates[*i].relative_path.to_string_lossy().into_owned())
                .collect()
        })
        .collect();

    Ok(ImportReport {
        scanned_count: n,
        hash_computed: hashes.iter().filter(|h| h.is_some()).count(),
        duplicates_found: dup_flag.iter().filter(|v| **v).count(),
        duplicate_groups,
        new_rows: counts.new_rows,
        unchanged: counts.unchanged,
        skipped_unreadable,
    })
}

/// List the inventory for `root`. Opens (and migrates) the project database before reading.
pub fn list_for_root(root: &Path) -> Result<Vec<store::ImageRow>> {
    let name = derive_name(root);
    let conn = store::open(default_db_dir(root).as_path(), &name, root, Some(root))?;
    store::list_images(&conn)
}

/// Integrity snapshot for `root`: missing files + duplicate groups. Read-only against the DB.
pub fn integrity_for_root(root: &Path) -> Result<store::IntegrityResult> {
    let name = derive_name(root);
    let conn = store::open(default_db_dir(root).as_path(), &name, root, Some(root))?;
    store::integrity_check(&conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ts-{tag}-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&d);
        d
    }

    #[test]
    fn empty_folder_is_a_valid_import() {
        let root = tmp("empty");
        let report = import_folder(&root).unwrap();
        assert_eq!(report.scanned_count, 0);
        assert_eq!(report.new_rows, 0);
    }

    #[test]
    fn identical_files_form_one_duplicate_group() {
        let root = tmp("dup");
        std::fs::write(root.join("a.jpg"), b"same").unwrap();
        std::fs::write(root.join("b.jpg"), b"same").unwrap();

        let report = import_folder(&root).unwrap();
        assert_eq!(report.scanned_count, 2);
        assert_eq!(report.duplicates_found, 2);
        assert_eq!(report.duplicate_groups.len(), 1);
        assert_eq!(report.duplicate_groups[0].len(), 2);
    }

    #[test]
    fn unsupported_extensions_are_ignored() {
        let root = tmp("skip");
        std::fs::write(root.join("readme.txt"), b"hi").unwrap();
        std::fs::write(root.join("photo.jpg"), b"jpg").unwrap();

        let report = import_folder(&root).unwrap();
        assert_eq!(report.scanned_count, 1, "only .jpg should be imported");
    }

    #[test]
    fn list_for_root_returns_the_rows_that_were_imported() {
        let root = tmp("list");
        std::fs::write(root.join("photo.jpg"), b"jpg").unwrap();
        import_folder(&root).unwrap();

        let rows = list_for_root(&root).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].relative_path, "photo.jpg");
    }

    #[test]
    fn integrity_reports_a_missing_file_when_the_on_disk_copy_is_deleted() {
        let root = tmp("integ-missing");
        std::fs::write(root.join("gone.jpg"), b"jpg").unwrap();
        import_folder(&root).unwrap();
        // Simulate the user deleting a file after import.
        std::fs::remove_file(root.join("gone.jpg")).unwrap();

        let res = integrity_for_root(&root).unwrap();
        assert_eq!(res.missing_files.len(), 1);
    }
}
