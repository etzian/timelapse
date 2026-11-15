//! Sampling inputs, runs and their ordered selections (schema v4).

use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};

use super::select::{self, ALGORITHM_VERSION, Frame, Method};
use crate::import::metadata::resolver::normalize_unix_ms;
use crate::{Error, Result};

/// Unmatched targets kept in a run record (the count is always exact).
const UNMATCHED_KEPT: usize = 1000;
/// Items a preview sends to the UI; the day counts always cover the whole selection.
pub const PREVIEW_ITEMS: usize = 200;
/// Largest page of a saved run's items.
pub const MAX_PAGE: usize = 1000;

fn db<T>(r: rusqlite::Result<T>) -> Result<T> {
    r.map_err(|_| Error::StoreTransaction)
}

/// A sampling configuration: the method plus which photographs it may choose from.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub method: Method,
    /// Only photographs with `from <= normalized_datetime < to` (normalised format or a
    /// prefix of it, e.g. `2024-01-15`).
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
    /// Include photographs flagged for review (rejected and quarantined never are).
    #[serde(default = "yes")]
    pub include_flagged: bool,
}

fn yes() -> bool {
    true
}

/// Photographs left out of the input, by reason.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Excluded {
    pub rejected: usize,
    pub quarantined: usize,
    pub flagged: usize,
    pub no_timestamp: usize,
    pub out_of_period: usize,
}

struct Input {
    id: i64,
    relative_path: String,
    normalized: String,
    epoch_ms: i64,
}

/// The input of a run: eligible photographs, chronological, plus what was excluded.
fn inputs(conn: &Connection, cfg: &Config) -> Result<(Vec<Input>, Excluded)> {
    let mut stmt = db(conn.prepare(
        r#"
            SELECT i.id, i.relative_path, i.normalized_datetime,
                   CAST(round(unixepoch(i.normalized_datetime, 'subsec') * 1000) AS INTEGER),
                   s.status
            FROM images i LEFT JOIN image_status s ON s.image_id = i.id
            ORDER BY i.normalized_datetime IS NULL, i.normalized_datetime, i.original_path
        "#,
    ))?;
    let rows = db(stmt.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, Option<i64>>(3)?,
            r.get::<_, Option<String>>(4)?,
        ))
    }))?;
    let mut ex = Excluded::default();
    let mut out = Vec::new();
    for row in rows {
        let (id, relative_path, normalized, epoch_ms, status) = db(row)?;
        match status.as_deref() {
            Some("rejected") => ex.rejected += 1,
            Some("quarantined") => ex.quarantined += 1,
            Some("flagged") if !cfg.include_flagged => ex.flagged += 1,
            _ => match (normalized, epoch_ms) {
                (Some(n), Some(ms)) => {
                    let after_from = cfg.from.as_deref().is_none_or(|f| n.as_str() >= f);
                    let before_to = cfg.to.as_deref().is_none_or(|t| n.as_str() < t);
                    if after_from && before_to {
                        out.push(Input { id, relative_path, normalized: n, epoch_ms: ms });
                    } else {
                        ex.out_of_period += 1;
                    }
                }
                _ => ex.no_timestamp += 1,
            },
        }
    }
    Ok((out, ex))
}

/// SHA-256 of the inputs (`id:normalized_datetime` per line, in order).
fn fingerprint(inputs: &[Input]) -> String {
    let mut h = Sha256::new();
    for i in inputs {
        h.update(format!("{}:{}\n", i.id, i.normalized).as_bytes());
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// One selected photograph, in output order.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SampledItem {
    pub sequence_index: usize,
    pub image_id: i64,
    pub relative_path: String,
    pub target_datetime: Option<String>,
    pub actual_datetime: String,
    pub epoch_ms: i64,
}

/// Everything the preview shows; also what a run records.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SamplingPreview {
    pub config: Config,
    pub algorithm_version: i64,
    pub input_count: usize,
    pub input_fingerprint: String,
    pub excluded: Excluded,
    pub target_count: usize,
    pub selected_count: usize,
    pub unmatched_count: usize,
    /// At most the first 1000 unmatched targets, normalised.
    pub unmatched_targets: Vec<String>,
    /// `1 - selected / input` (0 when there is no input).
    pub reduction: f64,
    /// Mean time between consecutive selected photographs.
    pub effective_interval_seconds: Option<f64>,
    /// Selected photographs per UTC day, over the whole selection.
    pub day_counts: Vec<DayCount>,
    /// The first [`PREVIEW_ITEMS`] selected photographs (all of them inside the store).
    pub items: Vec<SampledItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DayCount {
    /// `YYYY-MM-DD` (UTC).
    pub day: String,
    pub count: usize,
}

fn day_counts(items: &[SampledItem]) -> Vec<DayCount> {
    let mut out: Vec<DayCount> = Vec::new();
    for i in items {
        let day = &i.actual_datetime[..10.min(i.actual_datetime.len())];
        match out.last_mut() {
            Some(last) if last.day == day => last.count += 1,
            _ => out.push(DayCount { day: day.to_owned(), count: 1 }),
        }
    }
    out
}

/// Run `cfg` against the current inputs without writing anything. Only the first
/// [`PREVIEW_ITEMS`] items are returned; counts and day counts cover everything.
pub fn preview(conn: &Connection, cfg: &Config) -> Result<SamplingPreview> {
    let mut p = compute(conn, cfg)?;
    p.items.truncate(PREVIEW_ITEMS);
    Ok(p)
}

/// The full result, every item included.
fn compute(conn: &Connection, cfg: &Config) -> Result<SamplingPreview> {
    let (inputs, excluded) = inputs(conn, cfg)?;
    let frames: Vec<Frame> =
        inputs.iter().map(|i| Frame { image_id: i.id, at_ms: i.epoch_ms }).collect();
    let selection = select::select(&frames, &cfg.method)
        .map_err(|e| Error::InvalidSampling { reason: e.as_str() })?;
    let by_id: std::collections::HashMap<i64, &Input> = inputs.iter().map(|i| (i.id, i)).collect();
    let items: Vec<SampledItem> = selection
        .picks
        .iter()
        .enumerate()
        .map(|(k, p)| {
            let input = by_id[&p.image_id];
            SampledItem {
                sequence_index: k,
                image_id: p.image_id,
                relative_path: input.relative_path.clone(),
                target_datetime: p.target_ms.and_then(normalize_unix_ms),
                actual_datetime: input.normalized.clone(),
                epoch_ms: p.actual_ms,
            }
        })
        .collect();
    let n = items.len();
    let effective = (n >= 2)
        .then(|| (items[n - 1].epoch_ms - items[0].epoch_ms) as f64 / 1000.0 / (n - 1) as f64);
    Ok(SamplingPreview {
        config: cfg.clone(),
        algorithm_version: ALGORITHM_VERSION,
        input_count: inputs.len(),
        input_fingerprint: fingerprint(&inputs),
        excluded,
        target_count: selection.target_count,
        selected_count: n,
        unmatched_count: selection.unmatched_targets.len(),
        unmatched_targets: selection
            .unmatched_targets
            .iter()
            .take(UNMATCHED_KEPT)
            .filter_map(|ms| normalize_unix_ms(*ms))
            .collect(),
        reduction: if inputs.is_empty() { 0.0 } else { 1.0 - n as f64 / inputs.len() as f64 },
        effective_interval_seconds: effective,
        day_counts: day_counts(&items),
        items,
    })
}

/// A saved run, without its items.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct RunSummary {
    pub id: i64,
    pub method: String,
    pub config: Config,
    pub algorithm_version: i64,
    pub input_count: usize,
    pub excluded: Excluded,
    pub target_count: usize,
    pub selected_count: usize,
    pub unmatched_count: usize,
    pub created_at: i64,
}

/// Compute and persist a run (configuration, inputs fingerprint, ordered items) in one
/// transaction. Returns the stored summary.
pub fn create_run(conn: &Connection, cfg: &Config) -> Result<RunSummary> {
    let p = compute(conn, cfg)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let tx = db(conn.unchecked_transaction())?;
    db(tx.execute(
        r#"
            INSERT INTO sampling_runs (method, configuration_json, algorithm_version, input_count,
                input_fingerprint, excluded_json, target_count, selected_count, unmatched_count,
                unmatched_json, status, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'completed', ?11)
        "#,
        params![
            cfg.method.name(),
            json(&p.config)?,
            p.algorithm_version,
            p.input_count as i64,
            p.input_fingerprint,
            json(&p.excluded)?,
            p.target_count as i64,
            p.selected_count as i64,
            p.unmatched_count as i64,
            json(&p.unmatched_targets)?,
            now
        ],
    ))?;
    let run = tx.last_insert_rowid();
    {
        let mut stmt = db(tx.prepare(
            "INSERT INTO sampling_items (sampling_run_id, image_id, target_datetime, actual_datetime, sequence_index)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        ))?;
        for i in &p.items {
            db(stmt.execute(params![
                run,
                i.image_id,
                i.target_datetime,
                i.actual_datetime,
                i.sequence_index as i64
            ]))?;
        }
    }
    db(tx.commit())?;
    run_summary(conn, run)?.ok_or(Error::StoreTransaction)
}

fn json<T: serde::Serialize>(v: &T) -> Result<String> {
    serde_json::to_string(v).map_err(|_| Error::StoreTransaction)
}

const SUMMARY_COLUMNS: &str =
    "id, method, configuration_json, algorithm_version, input_count, excluded_json,
    target_count, selected_count, unmatched_count, created_at";

fn from_json<T: serde::de::DeserializeOwned>(
    r: &rusqlite::Row<'_>,
    idx: usize,
) -> rusqlite::Result<T> {
    let text: String = r.get(idx)?;
    serde_json::from_str(&text).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(idx, rusqlite::types::Type::Text, Box::new(e))
    })
}

fn summary(r: &rusqlite::Row<'_>) -> rusqlite::Result<RunSummary> {
    Ok(RunSummary {
        id: r.get(0)?,
        method: r.get(1)?,
        config: from_json(r, 2)?,
        algorithm_version: r.get(3)?,
        input_count: r.get::<_, i64>(4)? as usize,
        excluded: from_json(r, 5)?,
        target_count: r.get::<_, i64>(6)? as usize,
        selected_count: r.get::<_, i64>(7)? as usize,
        unmatched_count: r.get::<_, i64>(8)? as usize,
        created_at: r.get(9)?,
    })
}

/// SHA-256 of `index:image id:file hash` per photograph of saved run `run`, in sequence
/// order, and the number of photographs. Identifies exactly which bytes a run shows, so a
/// later step (an export, a deflicker or stabilization run) can record what it used.
pub fn sequence_fingerprint(conn: &Connection, run: i64) -> Result<(String, usize)> {
    let mut stmt = db(conn.prepare(
        r#"
            SELECT s.sequence_index, s.image_id, i.file_hash
            FROM sampling_items s JOIN images i ON i.id = s.image_id
            WHERE s.sampling_run_id = ?1 ORDER BY s.sequence_index
        "#,
    ))?;
    let mut h = Sha256::new();
    let mut photos = 0;
    let rows = db(stmt.query_map([run], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get::<_, Option<String>>(2)?))
    }))?;
    for row in rows {
        let (index, image, hash) = db(row)?;
        h.update(format!("{index}:{image}:{}\n", hash.unwrap_or_default()).as_bytes());
        photos += 1;
    }
    Ok((h.finalize().iter().map(|b| format!("{b:02x}")).collect(), photos))
}

pub fn run_summary(conn: &Connection, id: i64) -> Result<Option<RunSummary>> {
    db(conn
        .query_row(
            &format!("SELECT {SUMMARY_COLUMNS} FROM sampling_runs WHERE id = ?1"),
            [id],
            summary,
        )
        .optional())
}

/// Saved runs, newest first.
pub fn list_runs(conn: &Connection) -> Result<Vec<RunSummary>> {
    let mut stmt =
        db(conn.prepare(&format!("SELECT {SUMMARY_COLUMNS} FROM sampling_runs ORDER BY id DESC")))?;
    let rows = db(stmt.query_map([], summary))?;
    db(rows.collect())
}

/// The ordered selection of a saved run.
pub fn run_items(conn: &Connection, run: i64) -> Result<Vec<SampledItem>> {
    if run_summary(conn, run)?.is_none() {
        return Err(Error::UnknownRun { id: run });
    }
    let mut stmt = db(conn.prepare(
        r#"
            SELECT s.sequence_index, s.image_id, i.relative_path, s.target_datetime, s.actual_datetime,
                   CAST(round(unixepoch(s.actual_datetime, 'subsec') * 1000) AS INTEGER)
            FROM sampling_items s JOIN images i ON i.id = s.image_id
            WHERE s.sampling_run_id = ?1 ORDER BY s.sequence_index
        "#,
    ))?;
    let rows = db(stmt.query_map([run], |r| {
        Ok(SampledItem {
            sequence_index: r.get::<_, i64>(0)? as usize,
            image_id: r.get(1)?,
            relative_path: r.get(2)?,
            target_datetime: r.get(3)?,
            actual_datetime: r.get(4)?,
            epoch_ms: r.get(5)?,
        })
    }))?;
    db(rows.collect())
}

/// One page of a saved run's sequence, plus its day counts.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RunPage {
    pub run_id: i64,
    pub total: usize,
    pub offset: usize,
    pub items: Vec<SampledItem>,
    pub day_counts: Vec<DayCount>,
}

pub fn run_page(conn: &Connection, run: i64, offset: usize, limit: usize) -> Result<RunPage> {
    let summary = run_summary(conn, run)?.ok_or(Error::UnknownRun { id: run })?;
    let limit = limit.clamp(1, MAX_PAGE);
    let mut stmt = db(conn.prepare(
        r#"
            SELECT s.sequence_index, s.image_id, i.relative_path, s.target_datetime, s.actual_datetime,
                   CAST(round(unixepoch(s.actual_datetime, 'subsec') * 1000) AS INTEGER)
            FROM sampling_items s JOIN images i ON i.id = s.image_id
            WHERE s.sampling_run_id = ?1 ORDER BY s.sequence_index LIMIT ?2 OFFSET ?3
        "#,
    ))?;
    let rows = db(stmt.query_map(params![run, limit as i64, offset as i64], |r| {
        Ok(SampledItem {
            sequence_index: r.get::<_, i64>(0)? as usize,
            image_id: r.get(1)?,
            relative_path: r.get(2)?,
            target_datetime: r.get(3)?,
            actual_datetime: r.get(4)?,
            epoch_ms: r.get(5)?,
        })
    }))?;
    let items: Vec<SampledItem> = db(rows.collect())?;
    let mut days = db(conn.prepare(
        "SELECT substr(actual_datetime, 1, 10) AS day, COUNT(*) FROM sampling_items
         WHERE sampling_run_id = ?1 GROUP BY day ORDER BY day",
    ))?;
    let day_rows = db(days.query_map([run], |r| {
        Ok(DayCount { day: r.get(0)?, count: r.get::<_, i64>(1)? as usize })
    }))?;
    Ok(RunPage {
        run_id: run,
        total: summary.selected_count,
        offset,
        day_counts: db(day_rows.collect())?,
        items,
    })
}

/// Whether a saved run still describes the collection.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct VerifyReport {
    pub run_id: i64,
    /// The eligible inputs are exactly those the run was made from.
    pub inputs_unchanged: bool,
    pub algorithm_unchanged: bool,
    /// Re-running the stored configuration now gives the stored sequence.
    pub selection_identical: bool,
}

pub fn verify_run(conn: &Connection, run: i64) -> Result<VerifyReport> {
    let (cfg_json, version, stored_fp): (String, i64, String) = db(conn
        .query_row(
            "SELECT configuration_json, algorithm_version, input_fingerprint FROM sampling_runs WHERE id = ?1",
            [run],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional())?
    .ok_or(Error::UnknownRun { id: run })?;
    let cfg: Config = serde_json::from_str(&cfg_json).map_err(|_| Error::StoreTransaction)?;
    let now = compute(conn, &cfg)?;
    let stored = run_items(conn, run)?;
    let same = |a: &SampledItem, b: &SampledItem| {
        a.image_id == b.image_id
            && a.actual_datetime == b.actual_datetime
            && a.target_datetime == b.target_datetime
    };
    Ok(VerifyReport {
        run_id: run,
        inputs_unchanged: now.input_fingerprint == stored_fp,
        algorithm_unchanged: version == ALGORITHM_VERSION,
        selection_identical: now.items.len() == stored.len()
            && now.items.iter().zip(&stored).all(|(a, b)| same(a, b)),
    })
}
