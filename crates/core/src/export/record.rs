//! The export record (ADR-015, ADR-024): a human-readable JSON document derived from
//! `video_exports` and its sampling run, for portability and inspection. It is built on
//! demand from SQLite only, written to `.timelapse/exports/<id>.record.json` when asked,
//! and never read back.

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension};
use sha2::{Digest, Sha256};

use super::store::{self, ExportRecord};
use crate::import::open_root;
use crate::sampling::store::{RunSummary, run_summary};
use crate::{Error, Result};

pub const FORMAT: &str = "timelapse-studio/export-record";
pub const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ProjectInfo {
    pub name: String,
    pub source_root: Option<String>,
    pub schema_version: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SequenceInfo {
    pub run: RunSummary,
    /// The run's own fingerprint of the photographs it could choose from.
    pub input_fingerprint: String,
    /// SHA-256 of `index:image id:file hash` per photograph, in sequence order.
    pub sequence_fingerprint: String,
    pub photos: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ExportDocument {
    pub format: &'static str,
    pub format_version: u32,
    /// Always the same sentence: what this file is and is not.
    pub note: &'static str,
    pub project: ProjectInfo,
    pub export: ExportRecord,
    /// The exact FFmpeg arguments (paths of the temporary output included).
    pub argv: Vec<String>,
    pub sequence: SequenceInfo,
}

const NOTE: &str = "Derived from the project database on request; never read back (ADR-015).";

fn db<T>(r: rusqlite::Result<T>) -> Result<T> {
    r.map_err(|_| Error::StoreTransaction)
}

fn unknown() -> Error {
    Error::Export { reason: "unknown-export" }
}

pub fn record(conn: &Connection, id: i64) -> Result<ExportDocument> {
    let export = store::get(conn, id)?.ok_or_else(unknown)?;
    let argv_json: String = db(conn.query_row(
        "SELECT argv_json FROM video_exports WHERE id = ?1",
        [id],
        |r| r.get(0),
    ))?;
    let argv: Vec<String> = serde_json::from_str(&argv_json).unwrap_or_default();
    let project = db(conn
        .query_row(
            "SELECT name, source_root, schema_version FROM projects ORDER BY id LIMIT 1",
            [],
            |r| {
                Ok(ProjectInfo {
                    name: r.get(0)?,
                    source_root: r.get(1)?,
                    schema_version: r.get(2)?,
                })
            },
        )
        .optional())?
    .unwrap_or(ProjectInfo { name: String::new(), source_root: None, schema_version: 0 });
    let run = run_summary(conn, export.sampling_run_id)?
        .ok_or(Error::UnknownRun { id: export.sampling_run_id })?;
    let input_fingerprint: String = db(conn.query_row(
        "SELECT input_fingerprint FROM sampling_runs WHERE id = ?1",
        [run.id],
        |r| r.get(0),
    ))?;
    let mut stmt = db(conn.prepare(
        r#"
            SELECT s.sequence_index, s.image_id, i.file_hash
            FROM sampling_items s JOIN images i ON i.id = s.image_id
            WHERE s.sampling_run_id = ?1 ORDER BY s.sequence_index
        "#,
    ))?;
    let mut h = Sha256::new();
    let mut photos = 0;
    let rows = db(stmt.query_map([run.id], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get::<_, Option<String>>(2)?))
    }))?;
    for row in rows {
        let (index, image, hash) = db(row)?;
        h.update(format!("{index}:{image}:{}\n", hash.unwrap_or_default()).as_bytes());
        photos += 1;
    }
    let sequence_fingerprint = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
    Ok(ExportDocument {
        format: FORMAT,
        format_version: FORMAT_VERSION,
        note: NOTE,
        project,
        export,
        argv,
        sequence: SequenceInfo { run, input_fingerprint, sequence_fingerprint, photos },
    })
}

/// The record as pretty JSON.
pub fn record_json(root: &Path, id: i64) -> Result<String> {
    let conn = open_root(root)?;
    serde_json::to_string_pretty(&record(&conn, id)?).map_err(|_| Error::StoreTransaction)
}

pub fn record_path(root: &Path, id: i64) -> PathBuf {
    super::exports_dir(root).join(format!("{id}.record.json"))
}

/// Writes the record next to the exports (replacing an earlier record of the same export:
/// it is derived data) and returns its path.
pub fn write_record(root: &Path, id: i64) -> Result<PathBuf> {
    let json = record_json(root, id)?;
    let dir = super::exports_dir(root);
    let write_failed = || Error::Export { reason: "record-write-failed" };
    fs::create_dir_all(&dir).map_err(|_| write_failed())?;
    let path = record_path(root, id);
    let part = dir.join(format!("{id}.record.json.part"));
    fs::write(&part, json.as_bytes()).map_err(|_| write_failed())?;
    // `rename` replaces an earlier record on every platform (MoveFileEx on Windows).
    fs::rename(&part, &path).map_err(|_| {
        let _ = fs::remove_file(&part);
        write_failed()
    })?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::metadata::fixture::TempDir;
    use crate::import::{backfill_for_root, import_folder};
    use crate::sampling::{Config, Method, create_run_for_root};

    fn project() -> (TempDir, i64) {
        let dir = TempDir::new("p7-record");
        for (i, name) in ["a.jpg", "b.jpg", "c.jpg"].iter().enumerate() {
            let p = dir.path().join(name);
            image::RgbImage::from_pixel(8, 8, image::Rgb([i as u8 * 50, 0, 0])).save(&p).unwrap();
        }
        import_folder(dir.path()).unwrap();
        backfill_for_root(dir.path()).unwrap();
        let run = create_run_for_root(
            dir.path(),
            &Config {
                method: Method::EveryNth { n: 1, offset: 0 },
                from: None,
                to: None,
                include_flagged: true,
            },
        )
        .unwrap();
        open_root(dir.path())
            .unwrap()
            .execute(
                r#"INSERT INTO video_exports (sampling_run_id, settings_json, ffmpeg_version, argv_json,
                       output_path, status, input_photos, output_frames, duration_seconds, created_at)
                   VALUES (?1, '{"codec":"av1","fps":30,"speed":1,"width":1920,"height":1080,"quality":"high"}',
                           '7.1', '["-y","-f","rawvideo"]', 'out.mp4', 'completed', 3, 3, 0.1, 5)"#,
                [run.id],
            )
            .unwrap();
        (dir, run.id)
    }

    #[test]
    fn the_record_is_derived_from_sqlite_and_written_on_demand() {
        let (dir, run) = project();
        let conn = open_root(dir.path()).unwrap();
        let id: i64 =
            conn.query_row("SELECT max(id) FROM video_exports", [], |r| r.get(0)).unwrap();
        let doc = record(&conn, id).unwrap();
        assert_eq!((doc.format, doc.format_version), (FORMAT, 1));
        assert_eq!(doc.export.id, id);
        assert_eq!(doc.argv, ["-y", "-f", "rawvideo"]);
        assert_eq!((doc.sequence.run.id, doc.sequence.photos), (run, 3));
        assert_eq!(doc.sequence.sequence_fingerprint.len(), 64);
        assert_eq!(doc.project.schema_version, crate::import::store::SCHEMA_VERSION);

        // Same state, same document; a different sequence, a different fingerprint.
        assert_eq!(record_json(dir.path(), id).unwrap(), record_json(dir.path(), id).unwrap());
        conn.execute("DELETE FROM sampling_items WHERE sequence_index = 1", []).unwrap();
        assert_ne!(
            record(&conn, id).unwrap().sequence.sequence_fingerprint,
            doc.sequence.sequence_fingerprint
        );

        let path = write_record(dir.path(), id).unwrap();
        assert_eq!(path, record_path(dir.path(), id));
        let written: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(written["export"]["id"], id);
        // Written again, it replaces itself.
        assert_eq!(write_record(dir.path(), id).unwrap(), path);

        assert!(matches!(record(&conn, 999), Err(Error::Export { reason: "unknown-export" })));
    }
}
