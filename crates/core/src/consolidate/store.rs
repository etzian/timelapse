//! `consolidation_runs` (schema v6): one row per consolidation, with what it planned. What
//! it did is read from the file-operation journal (`file_operations.job_id`).

use rusqlite::{Connection, OptionalExtension, params};

use super::Config;
use crate::files::now;
use crate::{Error, Result};

fn db<T>(r: rusqlite::Result<T>) -> Result<T> {
    r.map_err(|_| Error::StoreTransaction)
}

/// The plan's counts, stored with the run.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct PlanCounts {
    pub planned: usize,
    pub duplicates: usize,
    pub already_present: usize,
    pub unsafe_names: usize,
    pub bytes: u64,
}

/// A consolidation run as the history shows it.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct RunRecord {
    pub id: i64,
    pub config: Config,
    pub plan: PlanCounts,
    /// `running` · `completed` · `partial` · `cancelled` · `interrupted`.
    pub status: String,
    /// `undone` · `partially-undone`, or `None`.
    pub undo_status: Option<String>,
    /// Photographs placed (completed `copy` / `move`).
    pub placed: usize,
    /// Placements that failed.
    pub failed: usize,
    /// Copies removed / originals moved back by an undo.
    pub undone: usize,
    /// Undo steps that failed (modified or missing copies, photos moved since…).
    pub undo_failed: usize,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub undone_at: Option<i64>,
}

pub fn create(conn: &Connection, cfg: &Config, fingerprint: &str, plan: PlanCounts) -> Result<i64> {
    let json = serde_json::to_string(cfg).map_err(|_| Error::StoreTransaction)?;
    db(conn.execute(
        r#"
            INSERT INTO consolidation_runs (operation, destination, configuration_json,
                sampling_run_id, plan_fingerprint, planned, duplicates, already_present,
                unsafe_names, bytes, status, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'running', ?11)
        "#,
        params![
            cfg.operation.as_str(),
            cfg.destination.to_string_lossy(),
            json,
            cfg.sampling_run,
            fingerprint,
            plan.planned as i64,
            plan.duplicates as i64,
            plan.already_present as i64,
            plan.unsafe_names as i64,
            plan.bytes as i64,
            now()
        ],
    ))?;
    Ok(conn.last_insert_rowid())
}

/// Close a run that is still `running`.
pub fn finish(conn: &Connection, id: i64, status: &str) -> Result<()> {
    db(conn.execute(
        "UPDATE consolidation_runs SET status = ?2, completed_at = ?3
         WHERE id = ?1 AND status = 'running'",
        params![id, status, now()],
    ))
    .map(|_| ())
}

pub fn set_undo(conn: &Connection, id: i64, undo_status: &str) -> Result<()> {
    db(conn.execute(
        "UPDATE consolidation_runs SET undo_status = ?2, undone_at = ?3 WHERE id = ?1",
        params![id, undo_status, now()],
    ))
    .map(|_| ())
}

/// Runs left `running` by a crashed process (call only while none runs here).
pub fn mark_interrupted(conn: &Connection) -> Result<()> {
    db(conn.execute(
        "UPDATE consolidation_runs SET status = 'interrupted', completed_at = ?1
         WHERE status = 'running'",
        [now()],
    ))
    .map(|_| ())
}

const SELECT: &str = r#"
    SELECT r.id, r.configuration_json, r.planned, r.duplicates, r.already_present,
           r.unsafe_names, r.bytes, r.status, r.undo_status, r.created_at, r.completed_at,
           r.undone_at,
           (SELECT count(*) FROM file_operations f WHERE f.job_id = r.id
               AND f.operation_type IN ('copy', 'move') AND f.status = 'completed'),
           (SELECT count(*) FROM file_operations f WHERE f.job_id = r.id
               AND f.operation_type IN ('copy', 'move') AND f.status = 'failed'),
           (SELECT count(*) FROM file_operations f WHERE f.job_id = r.id
               AND f.operation_type IN ('remove-copy', 'move-back') AND f.status = 'completed'),
           (SELECT count(*) FROM file_operations f WHERE f.job_id = r.id
               AND f.operation_type IN ('remove-copy', 'move-back') AND f.status = 'failed')
    FROM consolidation_runs r
"#;

fn record(r: &rusqlite::Row) -> rusqlite::Result<RunRecord> {
    let n = |i: usize| r.get::<_, i64>(i).map(|v| v.max(0) as usize);
    let json: String = r.get(1)?;
    let config = serde_json::from_str(&json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(e))
    })?;
    Ok(RunRecord {
        id: r.get(0)?,
        config,
        plan: PlanCounts {
            planned: n(2)?,
            duplicates: n(3)?,
            already_present: n(4)?,
            unsafe_names: n(5)?,
            bytes: r.get::<_, i64>(6)?.max(0) as u64,
        },
        status: r.get(7)?,
        undo_status: r.get(8)?,
        created_at: r.get(9)?,
        completed_at: r.get(10)?,
        undone_at: r.get(11)?,
        placed: n(12)?,
        failed: n(13)?,
        undone: n(14)?,
        undo_failed: n(15)?,
    })
}

pub fn get(conn: &Connection, id: i64) -> Result<Option<RunRecord>> {
    db(conn.query_row(&format!("{SELECT} WHERE r.id = ?1"), [id], record).optional())
}

/// Every run, newest first.
pub fn list(conn: &Connection) -> Result<Vec<RunRecord>> {
    let mut stmt = db(conn.prepare(&format!("{SELECT} ORDER BY r.id DESC")))?;
    let rows = db(stmt.query_map([], record))?;
    db(rows.collect())
}

/// Images whose placement by `run` has a completed undo step already.
pub fn undone_images(conn: &Connection, run: i64) -> Result<std::collections::HashSet<i64>> {
    let mut stmt = db(conn.prepare(
        "SELECT image_id FROM file_operations
         WHERE job_id = ?1 AND status = 'completed'
           AND operation_type IN ('remove-copy', 'move-back') AND image_id IS NOT NULL",
    ))?;
    let rows = db(stmt.query_map([run], |r| r.get::<_, i64>(0)))?;
    db(rows.collect())
}

/// Where the image is now, as stored (`None` = at its original path).
pub fn current_path(conn: &Connection, image: i64) -> Result<Option<String>> {
    db(conn
        .query_row("SELECT current_path FROM images WHERE id = ?1", [image], |r| r.get(0))
        .optional())
    .map(Option::flatten)
}

pub fn file_hash(conn: &Connection, image: i64) -> Result<Option<String>> {
    db(conn
        .query_row("SELECT file_hash FROM images WHERE id = ?1", [image], |r| r.get(0))
        .optional())
    .map(Option::flatten)
}
