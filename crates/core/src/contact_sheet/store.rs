//! `contact_sheets` (schema v7) and the queries that turn a source into photographs, from
//! SQLite only (ADR-024).

use std::path::PathBuf;

use rusqlite::{Connection, OptionalExtension, params};

use super::layout::Layout;
use super::pdf::Labels;
use crate::{Error, Result};

fn db<T>(r: rusqlite::Result<T>) -> Result<T> {
    r.map_err(|_| Error::StoreTransaction)
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// What a sheet shows (PRD §5.11).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum Source {
    /// Every imported photograph; unresolved times last.
    All,
    /// `from <= normalized_datetime < to`, both in the normalised format or a prefix of it.
    Period {
        from: String,
        to: String,
    },
    Flagged,
    Quarantined,
    /// A saved sampling selection, in its sequence order.
    Sampling {
        run: i64,
    },
    /// The selection an export encoded, in its sequence order.
    Export {
        export: i64,
    },
}

/// One photograph of a sheet, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub image_id: i64,
    pub relative_path: String,
    pub normalized_datetime: Option<String>,
    /// Its location of record.
    pub location: PathBuf,
    pub hash: Option<String>,
}

const CHRONOLOGICAL: &str = r#"
    SELECT i.id, i.relative_path, i.normalized_datetime,
           COALESCE(i.current_path, i.original_path), i.file_hash
    FROM images i LEFT JOIN image_status s ON s.image_id = i.id
    WHERE (?1 IS NULL OR s.status = ?1)
      AND (?2 IS NULL OR i.normalized_datetime >= ?2)
      AND (?3 IS NULL OR (i.normalized_datetime IS NOT NULL AND i.normalized_datetime < ?3))
    ORDER BY i.normalized_datetime IS NULL, i.normalized_datetime, i.original_path
"#;

const SEQUENCE: &str = r#"
    SELECT i.id, i.relative_path, i.normalized_datetime,
           COALESCE(i.current_path, i.original_path), i.file_hash
    FROM sampling_items s JOIN images i ON i.id = s.image_id
    WHERE s.sampling_run_id = ?1
    ORDER BY s.sequence_index
"#;

fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Item> {
    Ok(Item {
        image_id: r.get(0)?,
        relative_path: r.get(1)?,
        normalized_datetime: r.get(2)?,
        location: PathBuf::from(r.get::<_, String>(3)?),
        hash: r.get(4)?,
    })
}

fn sampling_run_exists(conn: &Connection, run: i64) -> Result<bool> {
    db(conn
        .query_row("SELECT 1 FROM sampling_runs WHERE id = ?1", [run], |_| Ok(()))
        .optional()
        .map(|o| o.is_some()))
}

/// The photographs of `source`, in sheet order.
pub fn items(conn: &Connection, source: &Source) -> Result<Vec<Item>> {
    let chrono =
        |status: Option<&str>, from: Option<&str>, to: Option<&str>| -> Result<Vec<Item>> {
            let mut stmt = db(conn.prepare(CHRONOLOGICAL))?;
            let rows = db(stmt.query_map(params![status, from, to], row))?;
            db(rows.collect())
        };
    let sequence = |run: i64| -> Result<Vec<Item>> {
        if !sampling_run_exists(conn, run)? {
            return Err(Error::UnknownRun { id: run });
        }
        let mut stmt = db(conn.prepare(SEQUENCE))?;
        let rows = db(stmt.query_map([run], row))?;
        db(rows.collect())
    };
    match source {
        Source::All => chrono(None, None, None),
        Source::Period { from, to } => {
            if from.is_empty() || to.is_empty() || from >= to {
                return Err(Error::ContactSheet { reason: "invalid-period" });
            }
            chrono(None, Some(from), Some(to))
        }
        Source::Flagged => chrono(Some("flagged"), None, None),
        Source::Quarantined => chrono(Some("quarantined"), None, None),
        Source::Sampling { run } => sequence(*run),
        Source::Export { export } => {
            let run: Option<i64> = db(conn
                .query_row(
                    "SELECT sampling_run_id FROM video_exports WHERE id = ?1",
                    [export],
                    |r| r.get(0),
                )
                .optional())?;
            sequence(run.ok_or(Error::ContactSheet { reason: "unknown-export" })?)
        }
    }
}

/// One generated (or attempted) sheet.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SheetRecord {
    pub id: i64,
    pub source: Source,
    pub layout: Layout,
    pub output_path: String,
    pub photos: usize,
    pub unreadable: usize,
    pub pages: usize,
    pub file_size: Option<u64>,
    /// `running` · `completed` · `failed` · `cancelled` · `interrupted`.
    pub status: String,
    pub failure: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

const COLUMNS: &str = "id, source_json, layout_json, output_path, photos, unreadable, pages, \
                       file_size, status, failure, created_at, completed_at";

fn record(r: &rusqlite::Row<'_>) -> rusqlite::Result<SheetRecord> {
    let source: String = r.get(1)?;
    let layout: String = r.get(2)?;
    Ok(SheetRecord {
        id: r.get(0)?,
        source: serde_json::from_str(&source).unwrap_or(Source::All),
        layout: serde_json::from_str(&layout).unwrap_or_default(),
        output_path: r.get(3)?,
        photos: r.get::<_, i64>(4)? as usize,
        unreadable: r.get::<_, i64>(5)? as usize,
        pages: r.get::<_, i64>(6)? as usize,
        file_size: r.get::<_, Option<i64>>(7)?.map(|s| s as u64),
        status: r.get(8)?,
        failure: r.get(9)?,
        created_at: r.get(10)?,
        completed_at: r.get(11)?,
    })
}

/// A `running` row; the output path is set once the id is known.
pub fn begin(
    conn: &Connection,
    source: &Source,
    layout: &Layout,
    labels: &Labels,
    photos: usize,
    pages: usize,
) -> Result<i64> {
    db(conn.execute(
        r#"
            INSERT INTO contact_sheets (source_json, layout_json, labels_json, output_path,
                                        photos, pages, status, created_at)
            VALUES (?1, ?2, ?3, '', ?4, ?5, 'running', ?6)
        "#,
        params![json(source), json(layout), json(labels), photos as i64, pages as i64, now()],
    ))?;
    Ok(conn.last_insert_rowid())
}

fn json<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_default()
}

pub fn set_output(conn: &Connection, id: i64, path: &str) -> Result<()> {
    db(conn.execute("UPDATE contact_sheets SET output_path = ?2 WHERE id = ?1", params![id, path]))
        .map(drop)
}

pub fn complete(conn: &Connection, id: i64, unreadable: usize, size: u64) -> Result<()> {
    db(conn.execute(
        r#"
            UPDATE contact_sheets SET status = 'completed', unreadable = ?2, file_size = ?3,
                                      completed_at = ?4
            WHERE id = ?1
        "#,
        params![id, unreadable as i64, size as i64, now()],
    ))
    .map(drop)
}

/// `status` is `failed`, `cancelled` or `interrupted`.
pub fn end(conn: &Connection, id: i64, status: &str, failure: Option<&str>) -> Result<()> {
    db(conn.execute(
        "UPDATE contact_sheets SET status = ?2, failure = ?3, completed_at = ?4 WHERE id = ?1",
        params![id, status, failure, now()],
    ))
    .map(drop)
}

/// Rows left `running` by a crash: marked `interrupted`; their ids are returned so the
/// caller can remove their temporary files. Only call while no sheet is being generated.
pub fn mark_interrupted(conn: &Connection) -> Result<Vec<i64>> {
    let mut stmt = db(conn.prepare("SELECT id FROM contact_sheets WHERE status = 'running'"))?;
    let ids: Vec<i64> = db(db(stmt.query_map([], |r| r.get(0)))?.collect())?;
    for &id in &ids {
        end(conn, id, "interrupted", None)?;
    }
    Ok(ids)
}

pub fn get(conn: &Connection, id: i64) -> Result<Option<SheetRecord>> {
    db(conn
        .query_row(&format!("SELECT {COLUMNS} FROM contact_sheets WHERE id = ?1"), [id], record)
        .optional())
}

/// Newest first.
pub fn list(conn: &Connection) -> Result<Vec<SheetRecord>> {
    let mut stmt =
        db(conn.prepare(&format!("SELECT {COLUMNS} FROM contact_sheets ORDER BY id DESC")))?;
    let rows = db(stmt.query_map([], record))?;
    db(rows.collect())
}
