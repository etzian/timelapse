//! `video_exports` (schema v5): one row per export attempt.

use std::ffi::OsString;
use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

use super::ffmpeg::{FfmpegInfo, Probed};
use super::settings::{ExportSettings, duration_seconds};
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

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ExportRecord {
    pub id: i64,
    pub sampling_run_id: i64,
    pub settings: ExportSettings,
    pub ffmpeg_version: Option<String>,
    pub output_path: String,
    /// `running` · `completed` · `failed` · `cancelled`.
    pub status: String,
    pub input_photos: usize,
    pub output_frames: usize,
    pub duration_seconds: f64,
    pub file_size: Option<u64>,
    pub probe: Option<Probed>,
    /// Stable kind (`error-export` selector) when not completed.
    pub failure: Option<String>,
    /// FFmpeg's last stderr lines, for the technical-detail disclosure.
    pub stderr_tail: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

#[allow(clippy::too_many_arguments)]
pub fn begin(
    conn: &Connection,
    run: i64,
    s: &ExportSettings,
    info: &FfmpegInfo,
    args: &[OsString],
    output: &Path,
    photos: usize,
    frames: usize,
) -> Result<i64> {
    let argv: Vec<String> = args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
    db(conn.execute(
        r#"
            INSERT INTO video_exports (sampling_run_id, settings_json, ffmpeg_version, argv_json,
                output_path, status, input_photos, output_frames, duration_seconds, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, 'running', ?6, ?7, ?8, ?9)
        "#,
        params![
            run,
            serde_json::to_string(s).map_err(|_| Error::StoreTransaction)?,
            info.version,
            serde_json::to_string(&argv).map_err(|_| Error::StoreTransaction)?,
            output.to_string_lossy(),
            photos as i64,
            frames as i64,
            duration_seconds(photos, s.fps, s.speed),
            now()
        ],
    ))?;
    Ok(conn.last_insert_rowid())
}

pub fn complete(conn: &Connection, id: i64, probed: &Probed, size: u64) -> Result<()> {
    db(conn.execute(
        "UPDATE video_exports SET status = 'completed', probe_json = ?2, file_size = ?3, completed_at = ?4
         WHERE id = ?1 AND status = 'running'",
        params![id, serde_json::to_string(probed).map_err(|_| Error::StoreTransaction)?, size as i64, now()],
    ))?;
    Ok(())
}

pub fn failed(conn: &Connection, id: i64, reason: &str, stderr_tail: &str) -> Result<()> {
    let status = if reason == "cancelled" { "cancelled" } else { "failed" };
    db(conn.execute(
        "UPDATE video_exports SET status = ?2, failure = ?3, stderr_tail = ?4, completed_at = ?5
         WHERE id = ?1 AND status = 'running'",
        params![id, status, reason, (!stderr_tail.is_empty()).then_some(stderr_tail), now()],
    ))?;
    Ok(())
}

/// Rows left `running` by a process that exited mid-export (call only when idle).
pub fn mark_interrupted(conn: &Connection) -> Result<()> {
    db(conn.execute(
        "UPDATE video_exports SET status = 'failed', failure = 'interrupted', completed_at = ?1 WHERE status = 'running'",
        [now()],
    ))?;
    Ok(())
}

const COLUMNS: &str = "id, sampling_run_id, settings_json, ffmpeg_version, output_path, status, input_photos,
    output_frames, duration_seconds, file_size, probe_json, failure, stderr_tail, created_at, completed_at";

fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<ExportRecord> {
    let json_err = |i: usize, e: serde_json::Error| {
        rusqlite::Error::FromSqlConversionFailure(i, rusqlite::types::Type::Text, Box::new(e))
    };
    let settings: String = r.get(2)?;
    let probe: Option<String> = r.get(10)?;
    Ok(ExportRecord {
        id: r.get(0)?,
        sampling_run_id: r.get(1)?,
        settings: serde_json::from_str(&settings).map_err(|e| json_err(2, e))?,
        ffmpeg_version: r.get(3)?,
        output_path: r.get(4)?,
        status: r.get(5)?,
        input_photos: r.get::<_, i64>(6)? as usize,
        output_frames: r.get::<_, i64>(7)? as usize,
        duration_seconds: r.get(8)?,
        file_size: r.get::<_, Option<i64>>(9)?.map(|v| v as u64),
        probe: probe.map(|p| serde_json::from_str(&p)).transpose().map_err(|e| json_err(10, e))?,
        failure: r.get(11)?,
        stderr_tail: r.get(12)?,
        created_at: r.get(13)?,
        completed_at: r.get(14)?,
    })
}

pub fn get(conn: &Connection, id: i64) -> Result<Option<ExportRecord>> {
    db(conn
        .query_row(&format!("SELECT {COLUMNS} FROM video_exports WHERE id = ?1"), [id], row)
        .optional())
}

pub fn list(conn: &Connection) -> Result<Vec<ExportRecord>> {
    let mut stmt =
        db(conn.prepare(&format!("SELECT {COLUMNS} FROM video_exports ORDER BY id DESC")))?;
    let rows = db(stmt.query_map([], row))?;
    db(rows.collect())
}
