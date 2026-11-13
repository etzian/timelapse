//! SQLite adapter: one database per project directory (ADR-007). Keeps the
//! `projects` and `images` tables current, applies migrations once, upserts an
//! import batch in a single transaction (idempotent by original path), and
//! surfaces integrity warnings for UI.
//!
//! Rust-side errors are mapped to `crate::Error::DbMessage { .. }`. The UI
//! renders them through Fluent so no prose is hardcoded here.

use rusqlite::{Connection, params};
use std::fs;
use std::path::Path;

use super::scan::Candidate;
use crate::Error;

const SCHEMA_VERSION: i64 = 1;

fn db_path(db_dir: &Path) -> std::path::PathBuf {
    db_dir.join("timelapse.db")
}

/// Open (creating if needed) the project database at `db_dir/timelapse.db` and run
/// any outstanding migrations. `name`, `source_root` and `project_path` describe
/// the project row — they are upserted on every open so a rename of the source
/// tree is reflected without a migration.
pub fn open(
    db_dir: &Path,
    name: &str,
    project_path: &Path,
    source_root: Option<&Path>,
) -> Result<Connection, Error> {
    fs::create_dir_all(db_dir).map_err(|_| db_err(db_dir))?;
    let conn = Connection::open(db_path(db_dir)).map_err(|_| db_err(db_dir))?;
    conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
        .map_err(|_| db_err(db_dir))?;

    // Migrations: every schema change is a versioned step, never an in-place edit.
    // (docs/DATABASE_SCHEMA.md §6.) This is `CREATE TABLE IF NOT EXISTS` rather than a
    // migration table because the current schema fits entirely in one statement group
    // and there is no existing data to migrate. When the first real migration lands we
    // will introduce the `schema_migrations` mechanism here (open question carried
    // forward, not implemented now).
    conn.execute_batch(
        r#"
            CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                project_path TEXT NOT NULL UNIQUE,
                source_root TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                schema_version INTEGER NOT NULL,
                settings_json TEXT
            );

            CREATE TABLE IF NOT EXISTS images (
                id INTEGER PRIMARY KEY,
                original_path TEXT NOT NULL UNIQUE COLLATE NOCASE,
                relative_path TEXT NOT NULL,
                filename TEXT NOT NULL,
                extension TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                modified_at INTEGER,
                file_hash TEXT,
                width INTEGER,
                height INTEGER,
                orientation INTEGER,
                camera_make TEXT,
                camera_model TEXT,
                exif_datetime_original TEXT,
                exif_create_date TEXT,
                filename_datetime TEXT,
                filesystem_datetime TEXT,
                normalized_datetime TEXT,
                timestamp_source TEXT,
                timestamp_confidence TEXT,
                gps_latitude REAL,
                gps_longitude REAL,
                import_status TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_images_hash ON images (file_hash);
            CREATE INDEX IF NOT EXISTS idx_images_relative ON images (relative_path);
        "#,
    )
    .map_err(|_| db_err(db_dir))?;

    let now = now_unix();
    let _ = conn
        .execute(
            r#"
                INSERT INTO projects (id, name, project_path, source_root, created_at, updated_at, schema_version, settings_json)
                VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, NULL)
                ON CONFLICT(project_path) DO UPDATE SET name = excluded.name,
                    source_root = excluded.source_root,
                    updated_at = excluded.updated_at,
                    schema_version = max(projects.schema_version, excluded.schema_version)
            "#,
            params![name, project_path.to_string_lossy(), source_root.map(|p| p.to_string_lossy().into_owned()), now, now, SCHEMA_VERSION],
        )
        .map_err(|_| db_err(db_dir))?;
    Ok(conn)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UpsertCounts {
    pub total: usize,
    pub new_rows: usize,
    /// Rows already present that were left untouched (same path).
    pub unchanged: usize,
}

/// Upsert a batch of candidates into the current project's `images` table.
/// Idempotent: re-running on the same root produces zero new rows — `ON CONFLICT
/// DO NOTHING` leaves existing rows untouched, and re-import is therefore a no-op
/// for known paths (ADR-006 originals never change).
///
/// `hashes[i]` is the SHA-256 of `candidates[i]`; `None` means "unknown / hashing failed".
pub fn upsert_images(
    conn: &Connection,
    candidates: &[Candidate],
    hashes: &[Option<String>],
    dup_flags: &[bool],
) -> Result<UpsertCounts, Error> {
    debug_assert_eq!(candidates.len(), hashes.len());
    debug_assert_eq!(candidates.len(), dup_flags.len());
    let now = now_unix();
    let tx = conn.unchecked_transaction().map_err(|_| Error::StoreTransaction)?;

    let mut new_rows = 0usize;
    let mut unchanged = 0usize;

    for i in 0..candidates.len() {
        let c = &candidates[i];
        let status = if dup_flags[i] { "duplicate-suspect" } else { "imported" };
        let n = tx
            .execute(
                r#"
                    INSERT INTO images (original_path, relative_path, filename, extension, file_size,
                                        modified_at, file_hash, import_status, created_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                    ON CONFLICT(original_path) DO NOTHING
                "#,
                params![
                    c.path.to_string_lossy(),
                    c.relative_path.to_string_lossy(),
                    c.filename,
                    c.extension,
                    c.file_size,
                    c.modified_at_unix,
                    hashes[i].as_deref(),
                    status,
                    now
                ],
            )
            .map_err(|_| Error::StoreTransaction)?;
        if n > 0 {
            new_rows += 1;
        } else {
            unchanged += 1;
        }
    }

    tx.commit().map_err(|_| Error::StoreTransaction)?;
    Ok(UpsertCounts { total: candidates.len(), new_rows, unchanged })
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ImageRow {
    pub id: i64,
    pub original_path: String,
    pub relative_path: String,
    pub filename: String,
    pub extension: String,
    pub file_size: i64,
    pub modified_at: Option<i64>,
    pub import_status: String,
}

pub fn list_images(conn: &Connection) -> Result<Vec<ImageRow>, Error> {
    let mut stmt = conn
        .prepare("SELECT id, original_path, relative_path, filename, extension, file_size, modified_at, import_status FROM images ORDER BY relative_path")
        .map_err(|_| Error::StoreTransaction)?;
    let rows = stmt
        .query_map([], |r| {
            Ok(ImageRow {
                id: r.get(0)?,
                original_path: r.get(1)?,
                relative_path: r.get(2)?,
                filename: r.get(3)?,
                extension: r.get(4)?,
                file_size: r.get(5)?,
                modified_at: r.get(6)?,
                import_status: r.get::<_, String>(7)?,
            })
        })
        .map_err(|_| Error::StoreTransaction)?;
    rows.collect::<Result<_, _>>().map_err(|_| Error::StoreTransaction)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct IntegrityResult {
    pub missing_files: Vec<String>,
    /// Hash groups with two or more images, each item is the list of paths that share it.
    pub duplicates: Vec<Vec<String>>,
    pub total_images: usize,
}

/// Cheaply check the inventory against the filesystem and surface warnings to the
/// UI without touching the database (ADR-011 + ADR-006).
pub fn integrity_check(conn: &Connection) -> Result<IntegrityResult, Error> {
    let total = conn
        .query_row("SELECT COUNT(*) FROM images", params![], |r| r.get::<_, i64>(0))
        .map_err(|_| Error::StoreTransaction)? as usize;

    // Missing files: rows whose original_path no longer exists on disk.
    let mut stmt =
        conn.prepare("SELECT original_path FROM images").map_err(|_| Error::StoreTransaction)?;
    let paths: Vec<String> = {
        let it =
            stmt.query_map([], |r| r.get::<_, String>(0)).map_err(|_| Error::StoreTransaction)?;
        it.flatten().collect()
    };
    let missing_files = paths.iter().filter(|p| !Path::new(p).exists()).cloned().collect();

    // Duplicate content groups: rows sharing a non-NULL file_hash, 2+.
    let mut stmt2 = conn
        .prepare("SELECT original_path, file_hash FROM images WHERE file_hash IS NOT NULL")
        .map_err(|_| Error::StoreTransaction)?;
    let mut dup_map: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    {
        let it = stmt2
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|_| Error::StoreTransaction)?;
        for row in it.flatten() {
            dup_map.entry(row.1).or_default().push(row.0);
        }
    }
    let duplicates = dup_map.into_values().filter(|v| v.len() >= 2).collect();

    Ok(IntegrityResult { missing_files, duplicates, total_images: total })
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn db_err(dir: &Path) -> Error {
    Error::DbMessage { path: dir.to_string_lossy().into_owned() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("ts-store-{tag}-{}", std::process::id()));
        let _ = fs::create_dir_all(&d);
        d
    }

    fn cand(path: &std::path::Path, relative: &str) -> Candidate {
        Candidate {
            path: path.to_path_buf(),
            relative_path: relative.into(),
            filename: "x.jpg".into(),
            extension: "jpg".into(),
            file_size: 10,
            modified_at_unix: 0,
        }
    }

    #[test]
    fn upsert_is_idempotent_per_original_path() {
        let root = tmp_dir("idem");
        let root_img = root.join("source.jpg");
        fs::write(&root_img, b"jpg").unwrap();
        let src = root.join("src");
        fs::create_dir_all(&src).unwrap();

        let conn = open(&root, "p1", &src, Some(&root)).unwrap();
        let cands = vec![cand(&root_img, "source.jpg")];
        let flags = vec![false];
        let hashes = vec![Some("deadbeef".into())];

        let first = super::upsert_images(&conn, &cands, &hashes, &flags).unwrap();
        assert_eq!(first.new_rows, 1);

        let second = super::upsert_images(&conn, &cands, &hashes, &flags).unwrap();
        assert_eq!(second.unchanged, 1, "re-importing the same path must not add rows");
    }

    #[test]
    fn integrity_marks_missing_files_and_grouping_duplicates() {
        let root = tmp_dir("integ");
        fs::create_dir_all(&root).unwrap();

        // Two rows sharing a hash (a.jpg + b.jpg, present on disk) and one
        // row ('gone.jpg') whose file is absent on purpose.
        let a_path = root.join("a.jpg");
        let b_path = root.join("b.jpg");
        fs::write(&a_path, b"dup").unwrap();
        fs::write(&b_path, b"dup").unwrap();
        let gone_path = root.join("gone.jpg"); // intentionally not created

        let conn = open(&root, "p2", &root, Some(&root)).unwrap();
        let _ = conn.execute(
            "INSERT OR IGNORE INTO images (original_path, relative_path, filename, extension, file_size, import_status, created_at)
             VALUES (?1, 'gone.jpg', 'gone.jpg', 'jpg', 4, 'imported', 0),
                    (?2, 'a.jpg', 'a.jpg', 'jpg', 8, 'imported', 0),
                    (?3, 'b.jpg', 'b.jpg', 'jpg', 8, 'duplicate-suspect', 0)",
            params![gone_path.to_string_lossy().into_owned(), a_path.to_string_lossy().into_owned(), b_path.to_string_lossy().into_owned()],
        )
        .unwrap();
        let _ = conn
            .execute(
                "UPDATE images SET file_hash = 'abc' WHERE original_path IN (?1, ?2)",
                params![
                    a_path.to_string_lossy().into_owned(),
                    b_path.to_string_lossy().into_owned()
                ],
            )
            .unwrap();

        let res = super::integrity_check(&conn).unwrap();
        assert_eq!(res.missing_files.len(), 1);
        assert_eq!(res.duplicates.len(), 1);
        assert_eq!(res.duplicates[0].len(), 2);
    }
}
