//! Persistence for Phase 3: measurements, proposals, user decisions, thresholds.

use std::path::PathBuf;

use rusqlite::{Connection, OptionalExtension, params};

use super::analysis::{self, ANALYSIS_VERSION, AnalysisFailure, Luma};
use super::classify::{self, Reason, Thresholds};
use crate::{Error, Result};

fn tx_err<T>(r: rusqlite::Result<T>) -> Result<T> {
    r.map_err(|_| Error::StoreTransaction)
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

// ---- thresholds ---------------------------------------------------------------------

/// The project's thresholds (`projects.settings_json` → `clean`), or the defaults.
pub fn thresholds(conn: &Connection) -> Result<Thresholds> {
    let json: Option<String> = tx_err(
        conn.query_row("SELECT settings_json FROM projects WHERE id = 1", [], |r| r.get(0))
            .optional(),
    )?
    .flatten();
    let clean = json
        .and_then(|j| serde_json::from_str::<serde_json::Value>(&j).ok())
        .and_then(|v| v.get("clean").cloned())
        .and_then(|c| serde_json::from_value::<Thresholds>(c).ok());
    Ok(clean.unwrap_or_default().sanitized())
}

/// Persist `t` (sanitised) under `clean` in `projects.settings_json`, keeping other keys.
pub fn set_thresholds(conn: &Connection, t: Thresholds) -> Result<Thresholds> {
    let t = t.sanitized();
    let current: Option<String> = tx_err(
        conn.query_row("SELECT settings_json FROM projects WHERE id = 1", [], |r| r.get(0))
            .optional(),
    )?
    .flatten();
    let mut v = current
        .and_then(|j| serde_json::from_str::<serde_json::Value>(&j).ok())
        .filter(|v| v.is_object())
        .unwrap_or_else(|| serde_json::json!({}));
    v["clean"] = serde_json::to_value(t).map_err(|_| Error::StoreTransaction)?;
    tx_err(conn.execute(
        "UPDATE projects SET settings_json = ?1, updated_at = ?2 WHERE id = 1",
        params![v.to_string(), now()],
    ))?;
    Ok(t)
}

// ---- analysis -----------------------------------------------------------------------

/// An image that has no measurement of the current algorithm version for its current bytes.
#[derive(Debug, Clone)]
pub struct PendingAnalysis {
    pub id: i64,
    pub path: PathBuf,
    pub hash: Option<String>,
}

pub fn pending_analysis(conn: &Connection) -> Result<Vec<PendingAnalysis>> {
    let mut stmt = tx_err(conn.prepare(
        r#"
            SELECT i.id, COALESCE(i.current_path, i.original_path), i.file_hash
            FROM images i
            LEFT JOIN image_analysis a ON a.image_id = i.id
            LEFT JOIN image_status s ON s.image_id = i.id
            -- A quarantined original keeps its measurement: its XMP sidecar stayed behind.
            WHERE (s.status IS NULL OR s.status <> 'quarantined')
              AND (a.image_id IS NULL OR a.algorithm_version <> ?1 OR a.source_hash IS NOT i.file_hash)
            ORDER BY i.id
        "#,
    ))?;
    let rows = tx_err(stmt.query_map([ANALYSIS_VERSION], |r| {
        Ok(PendingAnalysis {
            id: r.get(0)?,
            path: PathBuf::from(r.get::<_, String>(1)?),
            hash: r.get(2)?,
        })
    }))?;
    tx_err(rows.collect())
}

/// The measured result for one image.
pub struct Measured {
    pub id: i64,
    pub hash: Option<String>,
    pub luma: std::result::Result<Luma, AnalysisFailure>,
    pub xmp_rating: Option<i32>,
}

/// Write a batch of measurements in one transaction (replacing older versions).
pub fn save_measurements(conn: &Connection, batch: &[Measured]) -> Result<()> {
    let tx = tx_err(conn.unchecked_transaction())?;
    {
        let mut stmt = tx_err(tx.prepare(
            r#"
                INSERT INTO image_analysis (image_id, algorithm_version, source_hash, width, height,
                    mean_luma, median_luma, histogram_json, failure, xmp_rating, analyzed_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                ON CONFLICT(image_id) DO UPDATE SET algorithm_version = excluded.algorithm_version,
                    source_hash = excluded.source_hash, width = excluded.width, height = excluded.height,
                    mean_luma = excluded.mean_luma, median_luma = excluded.median_luma,
                    histogram_json = excluded.histogram_json, failure = excluded.failure,
                    xmp_rating = excluded.xmp_rating, analyzed_at = excluded.analyzed_at
            "#,
        ))?;
        let at = now();
        for m in batch {
            let (w, h, mean, median, hist, failure) = match &m.luma {
                Ok(l) => (
                    Some(l.width),
                    Some(l.height),
                    Some(l.mean()),
                    Some(l.median()),
                    Some(serde_json::to_string(&l.histogram).unwrap_or_default()),
                    None,
                ),
                Err(f) => (None, None, None, None, None, Some(f.as_str())),
            };
            tx_err(stmt.execute(params![
                m.id,
                ANALYSIS_VERSION,
                m.hash,
                w,
                h,
                mean,
                median,
                hist,
                failure,
                m.xmp_rating,
                at
            ]))?;
        }
    }
    tx_err(tx.commit())
}

// ---- classification -----------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct StatusCounts {
    pub eligible: usize,
    pub flagged: usize,
    pub rejected: usize,
    pub quarantined: usize,
    /// Imported images with no measurement yet.
    pub unanalyzed: usize,
}

/// The reason JSON stored with an automatic proposal: the thresholds and what tripped.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredReason {
    pub thresholds: Thresholds,
    pub reasons: Vec<Reason>,
}

struct StoredMeasure {
    id: i64,
    histogram: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    failure: Option<String>,
    xmp_rating: Option<i32>,
}

fn decode_luma(hist: Option<&str>, w: Option<u32>, h: Option<u32>) -> Option<Luma> {
    let histogram: Vec<u64> = serde_json::from_str(hist?).ok()?;
    (histogram.len() == 256).then(|| Luma {
        histogram,
        width: w.unwrap_or(0),
        height: h.unwrap_or(0),
    })
}

/// Re-propose `eligible` / `flagged` for every measured image the user has not decided,
/// using the stored thresholds. Rows with `user_override = 1` are never touched.
pub fn classify_all(conn: &Connection) -> Result<StatusCounts> {
    let t = thresholds(conn)?;
    let measures: Vec<StoredMeasure> = {
        let mut stmt = tx_err(conn.prepare(
            r#"
                SELECT a.image_id, a.histogram_json, a.width, a.height, a.failure, a.xmp_rating
                FROM image_analysis a
                LEFT JOIN image_status s ON s.image_id = a.image_id
                WHERE s.user_override IS NULL OR s.user_override = 0
            "#,
        ))?;
        let rows = tx_err(stmt.query_map([], |r| {
            Ok(StoredMeasure {
                id: r.get(0)?,
                histogram: r.get(1)?,
                width: r.get(2)?,
                height: r.get(3)?,
                failure: r.get(4)?,
                xmp_rating: r.get(5)?,
            })
        }))?;
        tx_err(rows.collect())?
    };
    let tx = tx_err(conn.unchecked_transaction())?;
    {
        let mut stmt = tx_err(tx.prepare(
            r#"
                INSERT INTO image_status (image_id, status, reason, user_override, updated_at)
                VALUES (?1, ?2, ?3, 0, ?4)
                ON CONFLICT(image_id) DO UPDATE SET status = excluded.status,
                    reason = excluded.reason, updated_at = excluded.updated_at
                WHERE image_status.user_override = 0
            "#,
        ))?;
        let at = now();
        for m in &measures {
            let luma = decode_luma(m.histogram.as_deref(), m.width, m.height);
            let failure =
                if luma.is_none() { m.failure.as_deref().or(Some("undecodable")) } else { None };
            let p = classify::classify(luma.as_ref(), failure, m.xmp_rating, &t);
            let status = if p.flagged() { "flagged" } else { "eligible" };
            let reason = serde_json::to_string(&StoredReason { thresholds: t, reasons: p.reasons })
                .map_err(|_| Error::StoreTransaction)?;
            tx_err(stmt.execute(params![m.id, status, reason, at]))?;
        }
    }
    tx_err(tx.commit())?;
    status_counts(conn)
}

pub fn status_counts(conn: &Connection) -> Result<StatusCounts> {
    let mut c = StatusCounts::default();
    let mut stmt = tx_err(conn.prepare(
        "SELECT COALESCE(s.status, ''), COUNT(*) FROM images i
         LEFT JOIN image_status s ON s.image_id = i.id GROUP BY 1",
    ))?;
    let rows =
        tx_err(stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as usize))))?;
    for row in rows {
        let (status, n) = tx_err(row)?;
        match status.as_str() {
            "eligible" => c.eligible = n,
            "flagged" => c.flagged = n,
            "rejected" => c.rejected = n,
            "quarantined" => c.quarantined = n,
            _ => c.unanalyzed += n,
        }
    }
    Ok(c)
}

// ---- user decisions -----------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    /// Keep the image (status `eligible`, user override).
    Accept,
    /// Reject it (status `rejected`, user override); it can then be quarantined.
    Reject,
    /// Drop the user's decision; the next classification proposes again.
    Clear,
}

/// Apply a decision to each image. Quarantined images are skipped (restore them first);
/// unmeasured images cannot be cleared back to a proposal. Returns how many changed.
pub fn decide(conn: &Connection, ids: &[i64], decision: Decision) -> Result<usize> {
    let tx = tx_err(conn.unchecked_transaction())?;
    let mut changed = 0;
    let at = now();
    for id in ids {
        let quarantined: bool = tx_err(
            tx.query_row(
                "SELECT status = 'quarantined' FROM image_status WHERE image_id = ?1",
                [id],
                |r| r.get(0),
            )
            .optional(),
        )?
        .unwrap_or(false);
        if quarantined {
            continue;
        }
        changed += match decision {
            Decision::Accept | Decision::Reject => {
                let status = if decision == Decision::Accept { "eligible" } else { "rejected" };
                tx_err(tx.execute(
                    r#"
                        INSERT INTO image_status (image_id, status, reason, user_override, updated_at)
                        SELECT id, ?2, NULL, 1, ?3 FROM images WHERE id = ?1
                        ON CONFLICT(image_id) DO UPDATE SET status = excluded.status,
                            user_override = 1, updated_at = excluded.updated_at
                    "#,
                    params![id, status, at],
                ))?
            }
            Decision::Clear => {
                // An image never measured has no proposal to return to: drop its status row
                // (back to "not analysed") instead of leaving a decision-less `rejected`.
                let dropped = tx_err(tx.execute(
                    "DELETE FROM image_status WHERE image_id = ?1
                     AND NOT EXISTS (SELECT 1 FROM image_analysis a WHERE a.image_id = ?1)",
                    params![id],
                ))?;
                dropped
                    + tx_err(tx.execute(
                        "UPDATE image_status SET user_override = 0, updated_at = ?2 WHERE image_id = ?1",
                        params![id, at],
                    ))?
            }
        };
    }
    tx_err(tx.commit())?;
    if decision == Decision::Clear {
        classify_all(conn)?;
    }
    Ok(changed)
}

// ---- review -------------------------------------------------------------------------

/// One image as the review screen shows it.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ReviewItem {
    pub id: i64,
    pub relative_path: String,
    pub normalized_datetime: Option<String>,
    pub status: Option<String>,
    pub user_override: bool,
    pub reasons: Vec<Reason>,
    pub mean_luma: Option<f64>,
    pub median_luma: Option<u8>,
    /// At the project's current dark level.
    pub dark_fraction: Option<f64>,
    pub histogram: Vec<u64>,
    pub xmp_rating: Option<i32>,
    pub failure: Option<String>,
    /// Where the original is now, while quarantined (`None` = at its original path).
    pub current_path: Option<String>,
}

/// Images whose status is in `statuses` (all images when empty), chronological.
pub fn review(conn: &Connection, statuses: &[String]) -> Result<Vec<ReviewItem>> {
    let t = thresholds(conn)?;
    let mut stmt = tx_err(conn.prepare(
        r#"
            SELECT i.id, i.relative_path, i.normalized_datetime, s.status, COALESCE(s.user_override, 0),
                   s.reason, a.mean_luma, a.median_luma, a.histogram_json, a.width, a.height,
                   a.xmp_rating, a.failure, i.current_path
            FROM images i
            LEFT JOIN image_status s ON s.image_id = i.id
            LEFT JOIN image_analysis a ON a.image_id = i.id
            ORDER BY i.normalized_datetime IS NULL, i.normalized_datetime, i.original_path
        "#,
    ))?;
    let rows = tx_err(stmt.query_map([], |r| {
        let hist: Option<String> = r.get(8)?;
        let luma = decode_luma(hist.as_deref(), r.get(9)?, r.get(10)?);
        let reason: Option<String> = r.get(5)?;
        Ok(ReviewItem {
            id: r.get(0)?,
            relative_path: r.get(1)?,
            normalized_datetime: r.get(2)?,
            status: r.get(3)?,
            user_override: r.get::<_, i64>(4)? != 0,
            reasons: reason
                .and_then(|j| serde_json::from_str::<StoredReason>(&j).ok())
                .map(|s| s.reasons)
                .unwrap_or_default(),
            mean_luma: r.get(6)?,
            median_luma: r.get(7)?,
            dark_fraction: luma.as_ref().map(|l| l.fraction_at_or_below(t.dark_level)),
            histogram: luma.as_ref().map(Luma::display_bins).unwrap_or_default(),
            xmp_rating: r.get(11)?,
            failure: r.get(12)?,
            current_path: r.get(13)?,
        })
    }))?;
    let all: Vec<ReviewItem> = tx_err(rows.collect())?;
    Ok(if statuses.is_empty() {
        all
    } else {
        all.into_iter()
            .filter(|i| i.status.as_ref().is_some_and(|s| statuses.contains(s)))
            .collect()
    })
}

/// Current location of an image (quarantine path while quarantined) and its hash.
pub fn location(conn: &Connection, id: i64) -> Result<Option<(PathBuf, Option<String>)>> {
    tx_err(
        conn.query_row(
            "SELECT COALESCE(current_path, original_path), file_hash FROM images WHERE id = ?1",
            [id],
            |r| Ok((PathBuf::from(r.get::<_, String>(0)?), r.get(1)?)),
        )
        .optional(),
    )
}

/// Analyse one file (decode + XMP); used by the parallel runner.
pub fn measure_file(p: &PendingAnalysis) -> Measured {
    Measured {
        id: p.id,
        hash: p.hash.clone(),
        luma: analysis::analyze(&p.path),
        xmp_rating: super::xmp::rating(&p.path),
    }
}
