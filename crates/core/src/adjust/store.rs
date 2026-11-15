//! `flicker_runs` / `flicker_frames` and `stabilization_runs` / `stabilization_frames`
//! (schema v8, ADR-025). A run is written in one transaction, complete, or not at all.

use rusqlite::{Connection, OptionalExtension, params};

use super::deflicker::{ChannelMeans, DeflickerConfig, FrameGains};
use super::stabilize::{FrameShift, StabilizeConfig};
use crate::{Error, Result};

/// Bumped whenever an algorithm's output for the same input changes.
pub const DEFLICKER_VERSION: i64 = 1;
pub const STABILIZE_VERSION: i64 = 1;

fn db<T>(r: rusqlite::Result<T>) -> Result<T> {
    r.map_err(|_| Error::StoreTransaction)
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn json<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_default()
}

/// One frame of a run: its place in the selection and its photograph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameRef {
    pub sequence_index: usize,
    pub image_id: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FlickerResults {
    pub frames: usize,
    /// Frames that could be measured (the others keep a gain of 1).
    pub measured: usize,
    /// Largest and mean exposure correction, in stops (absolute).
    pub max_stops: f64,
    pub mean_stops: f64,
    /// Largest colour correction between channels of one frame, in stops.
    pub max_colour_stops: f64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct FlickerRun {
    pub id: i64,
    pub sampling_run_id: i64,
    pub config: DeflickerConfig,
    pub algorithm_version: i64,
    pub sequence_fingerprint: String,
    pub results: FlickerResults,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StabilizationResults {
    pub frames: usize,
    /// Frame pairs whose motion was measured with confidence.
    pub measured: usize,
    /// Largest and mean correction, as fractions of the frame.
    pub max_shift: f64,
    pub mean_shift: f64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct StabilizationRun {
    pub id: i64,
    pub sampling_run_id: i64,
    pub config: StabilizeConfig,
    pub algorithm_version: i64,
    pub sequence_fingerprint: String,
    /// Crop per side used for every frame (fraction of the frame).
    pub crop: f64,
    pub results: StabilizationResults,
    pub created_at: i64,
}

const LN2: f64 = std::f64::consts::LN_2;

fn flicker_results(gains: &[FrameGains]) -> FlickerResults {
    let exposure: Vec<f64> = gains
        .iter()
        .map(|g| {
            // The exposure part is the gain on luminance.
            (0.2126 * g.gains[0] + 0.7152 * g.gains[1] + 0.0722 * g.gains[2]).ln().abs() / LN2
        })
        .collect();
    let colour = gains
        .iter()
        .map(|g| {
            let (lo, hi) =
                g.gains.iter().fold((f64::MAX, f64::MIN), |(lo, hi), v| (lo.min(*v), hi.max(*v)));
            (hi / lo).ln() / LN2
        })
        .fold(0.0f64, f64::max);
    FlickerResults {
        frames: gains.len(),
        measured: gains.iter().filter(|g| g.luma.is_some()).count(),
        max_stops: exposure.iter().copied().fold(0.0, f64::max),
        mean_stops: if exposure.is_empty() {
            0.0
        } else {
            exposure.iter().sum::<f64>() / exposure.len() as f64
        },
        max_colour_stops: colour,
    }
}

/// Store a complete deflicker run.
pub fn insert_flicker(
    conn: &Connection,
    sampling_run: i64,
    config: &DeflickerConfig,
    fingerprint: &str,
    frames: &[FrameRef],
    means: &[Option<ChannelMeans>],
    gains: &[FrameGains],
) -> Result<FlickerRun> {
    let results = flicker_results(gains);
    let tx = db(conn.unchecked_transaction())?;
    let created_at = now();
    db(tx.execute(
        r#"
            INSERT INTO flicker_runs (sampling_run_id, configuration_json, algorithm_version,
                                      sequence_fingerprint, status, results_json, created_at)
            VALUES (?1, ?2, ?3, ?4, 'completed', ?5, ?6)
        "#,
        params![
            sampling_run,
            json(config),
            DEFLICKER_VERSION,
            fingerprint,
            json(&results),
            created_at
        ],
    ))?;
    let id = tx.last_insert_rowid();
    {
        let mut stmt = db(tx.prepare(
            r#"
                INSERT INTO flicker_frames (run_id, sequence_index, image_id, mean_r, mean_g, mean_b,
                                            trend, gain_r, gain_g, gain_b)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
        ))?;
        for ((f, m), g) in frames.iter().zip(means).zip(gains) {
            db(stmt.execute(params![
                id,
                f.sequence_index as i64,
                f.image_id,
                m.map(|m| m.r),
                m.map(|m| m.g),
                m.map(|m| m.b),
                g.trend,
                g.gains[0],
                g.gains[1],
                g.gains[2]
            ]))?;
        }
    }
    db(tx.commit())?;
    Ok(FlickerRun {
        id,
        sampling_run_id: sampling_run,
        config: *config,
        algorithm_version: DEFLICKER_VERSION,
        sequence_fingerprint: fingerprint.to_string(),
        results,
        created_at,
    })
}

/// Store a complete stabilization run.
pub fn insert_stabilization(
    conn: &Connection,
    sampling_run: i64,
    config: &StabilizeConfig,
    fingerprint: &str,
    frames: &[FrameRef],
    shifts: &[FrameShift],
    crop: f64,
) -> Result<StabilizationRun> {
    let magnitudes: Vec<f64> =
        shifts.iter().map(|s| s.correction.0.abs().max(s.correction.1.abs())).collect();
    let results = StabilizationResults {
        frames: shifts.len(),
        measured: shifts.iter().filter(|s| s.measured.is_some()).count(),
        max_shift: magnitudes.iter().copied().fold(0.0, f64::max),
        mean_shift: if magnitudes.is_empty() {
            0.0
        } else {
            magnitudes.iter().sum::<f64>() / magnitudes.len() as f64
        },
    };
    let tx = db(conn.unchecked_transaction())?;
    let created_at = now();
    db(tx.execute(
        r#"
            INSERT INTO stabilization_runs (sampling_run_id, configuration_json, algorithm_version,
                                            sequence_fingerprint, crop, status, results_json, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, 'completed', ?6, ?7)
        "#,
        params![sampling_run, json(config), STABILIZE_VERSION, fingerprint, crop, json(&results), created_at],
    ))?;
    let id = tx.last_insert_rowid();
    {
        let mut stmt = db(tx.prepare(
            r#"
                INSERT INTO stabilization_frames (run_id, sequence_index, image_id, measured_x,
                    measured_y, confidence, path_x, path_y, smooth_x, smooth_y, shift_x, shift_y)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
        ))?;
        for (f, s) in frames.iter().zip(shifts) {
            db(stmt.execute(params![
                id,
                f.sequence_index as i64,
                f.image_id,
                s.measured.map(|m| m.0),
                s.measured.map(|m| m.1),
                s.confidence,
                s.path.0,
                s.path.1,
                s.smoothed.0,
                s.smoothed.1,
                s.correction.0,
                s.correction.1
            ]))?;
        }
    }
    db(tx.commit())?;
    Ok(StabilizationRun {
        id,
        sampling_run_id: sampling_run,
        config: *config,
        algorithm_version: STABILIZE_VERSION,
        sequence_fingerprint: fingerprint.to_string(),
        crop,
        results,
        created_at,
    })
}

fn flicker_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<FlickerRun> {
    let config: String = r.get(2)?;
    let results: String = r.get(5)?;
    Ok(FlickerRun {
        id: r.get(0)?,
        sampling_run_id: r.get(1)?,
        config: serde_json::from_str(&config).unwrap_or_default(),
        algorithm_version: r.get(3)?,
        sequence_fingerprint: r.get(4)?,
        results: serde_json::from_str(&results).unwrap_or(FlickerResults {
            frames: 0,
            measured: 0,
            max_stops: 0.0,
            mean_stops: 0.0,
            max_colour_stops: 0.0,
        }),
        created_at: r.get(6)?,
    })
}

fn stabilization_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<StabilizationRun> {
    let config: String = r.get(2)?;
    let results: String = r.get(6)?;
    Ok(StabilizationRun {
        id: r.get(0)?,
        sampling_run_id: r.get(1)?,
        config: serde_json::from_str(&config).unwrap_or_default(),
        algorithm_version: r.get(3)?,
        sequence_fingerprint: r.get(4)?,
        crop: r.get(5)?,
        results: serde_json::from_str(&results).unwrap_or(StabilizationResults {
            frames: 0,
            measured: 0,
            max_shift: 0.0,
            mean_shift: 0.0,
        }),
        created_at: r.get(7)?,
    })
}

const FLICKER_COLUMNS: &str = "id, sampling_run_id, configuration_json, algorithm_version, \
                               sequence_fingerprint, results_json, created_at";
const STABILIZATION_COLUMNS: &str = "id, sampling_run_id, configuration_json, algorithm_version, \
                                     sequence_fingerprint, crop, results_json, created_at";

pub fn flicker_run(conn: &Connection, id: i64) -> Result<Option<FlickerRun>> {
    db(conn
        .query_row(
            &format!("SELECT {FLICKER_COLUMNS} FROM flicker_runs WHERE id = ?1"),
            [id],
            flicker_row,
        )
        .optional())
}

pub fn stabilization_run(conn: &Connection, id: i64) -> Result<Option<StabilizationRun>> {
    db(conn
        .query_row(
            &format!("SELECT {STABILIZATION_COLUMNS} FROM stabilization_runs WHERE id = ?1"),
            [id],
            stabilization_row,
        )
        .optional())
}

/// Newest first.
pub fn flicker_runs(conn: &Connection) -> Result<Vec<FlickerRun>> {
    let mut stmt =
        db(conn.prepare(&format!("SELECT {FLICKER_COLUMNS} FROM flicker_runs ORDER BY id DESC")))?;
    let rows = db(stmt.query_map([], flicker_row))?;
    db(rows.collect())
}

pub fn stabilization_runs(conn: &Connection) -> Result<Vec<StabilizationRun>> {
    let mut stmt = db(conn.prepare(&format!(
        "SELECT {STABILIZATION_COLUMNS} FROM stabilization_runs ORDER BY id DESC"
    )))?;
    let rows = db(stmt.query_map([], stabilization_row))?;
    db(rows.collect())
}

/// One stored deflicker frame, for charts and previews.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct FlickerFrame {
    pub sequence_index: usize,
    pub image_id: i64,
    pub means: Option<ChannelMeans>,
    pub trend: Option<f64>,
    pub gains: [f64; 3],
}

/// Every frame of deflicker run `run`, in sequence order.
pub fn flicker_frames(conn: &Connection, run: i64) -> Result<Vec<FlickerFrame>> {
    let mut stmt = db(conn.prepare(
        r#"
            SELECT sequence_index, image_id, mean_r, mean_g, mean_b, trend, gain_r, gain_g, gain_b
            FROM flicker_frames WHERE run_id = ?1 ORDER BY sequence_index
        "#,
    ))?;
    let rows = db(stmt.query_map([run], |r| {
        let (mr, mg, mb): (Option<f64>, Option<f64>, Option<f64>) =
            (r.get(2)?, r.get(3)?, r.get(4)?);
        Ok(FlickerFrame {
            sequence_index: r.get::<_, i64>(0)? as usize,
            image_id: r.get(1)?,
            means: match (mr, mg, mb) {
                (Some(r), Some(g), Some(b)) => Some(ChannelMeans { r, g, b }),
                _ => None,
            },
            trend: r.get(5)?,
            gains: [r.get(6)?, r.get(7)?, r.get(8)?],
        })
    }))?;
    db(rows.collect())
}

/// One stored stabilization frame.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct StabilizationFrame {
    pub sequence_index: usize,
    pub image_id: i64,
    pub measured: Option<(f64, f64)>,
    pub confidence: f64,
    pub path: (f64, f64),
    pub smoothed: (f64, f64),
    pub shift: (f64, f64),
}

pub fn stabilization_frames(conn: &Connection, run: i64) -> Result<Vec<StabilizationFrame>> {
    let mut stmt = db(conn.prepare(
        r#"
            SELECT sequence_index, image_id, measured_x, measured_y, confidence, path_x, path_y,
                   smooth_x, smooth_y, shift_x, shift_y
            FROM stabilization_frames WHERE run_id = ?1 ORDER BY sequence_index
        "#,
    ))?;
    let rows = db(stmt.query_map([run], |r| {
        let (mx, my): (Option<f64>, Option<f64>) = (r.get(2)?, r.get(3)?);
        Ok(StabilizationFrame {
            sequence_index: r.get::<_, i64>(0)? as usize,
            image_id: r.get(1)?,
            measured: mx.zip(my),
            confidence: r.get(4)?,
            path: (r.get(5)?, r.get(6)?),
            smoothed: (r.get(7)?, r.get(8)?),
            shift: (r.get(9)?, r.get(10)?),
        })
    }))?;
    db(rows.collect())
}
