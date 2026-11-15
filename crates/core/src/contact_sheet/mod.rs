//! Contact sheets (ADR-024): PDF pages of thumbnails with captions, generated from SQLite
//! state only.
//!
//! Which photographs, their order and every caption come from the database. Each image is
//! the cached review thumbnail of the photograph at its location of record (ADR-020); the
//! original is read only when no thumbnail is cached, and a photograph that cannot be read
//! gets a placeholder cell instead of failing the sheet. The PDF goes to
//! `<root>/.timelapse/contact-sheets/<id>.pdf` through a temporary name.

pub mod layout;
pub mod pdf;
pub mod store;
pub mod text;

#[cfg(test)]
mod exit_tests;

use std::fs;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, TryLockError};

use rayon::prelude::*;

use crate::import::{open_root, project_dir};
use crate::{Error, Result};

pub use layout::{Geometry, Layout, Orientation, PageSize, Rect};
pub use pdf::Labels;
pub use store::{SheetRecord, Source};

/// The design target (PRD §6): larger sources are refused.
pub const MAX_PHOTOS: usize = 100_000;

static SHEET_LOCK: Mutex<()> = Mutex::new(());
static CANCEL_TOKEN: AtomicU64 = AtomicU64::new(0);

pub fn sheets_dir(root: &Path) -> PathBuf {
    project_dir(root).join("contact-sheets")
}

fn fail(reason: &'static str) -> Error {
    Error::ContactSheet { reason }
}

fn try_lock() -> Option<MutexGuard<'static, ()>> {
    match SHEET_LOCK.try_lock() {
        Ok(g) => Some(g),
        Err(TryLockError::Poisoned(p)) => Some(p.into_inner()),
        Err(TryLockError::WouldBlock) => None,
    }
}

/// Ask the sheet started with `token` to stop; it removes its temporary file.
pub fn cancel(token: u64) {
    CANCEL_TOKEN.store(token, Ordering::SeqCst);
}

fn cancelled(token: u64) -> bool {
    token != 0 && CANCEL_TOKEN.load(Ordering::SeqCst) == token
}

fn part_path(dir: &Path, id: i64) -> PathBuf {
    dir.join(format!("{id}.pdf.part"))
}

/// Rows left `running` by a crash become `interrupted` and their temporary files go. Only
/// while the lock is held, so a live sheet is never touched.
fn settle(conn: &rusqlite::Connection, root: &Path) -> Result<()> {
    for id in store::mark_interrupted(conn)? {
        let _ = fs::remove_file(part_path(&sheets_dir(root), id));
    }
    Ok(())
}

pub fn sheets_for_root(root: &Path) -> Result<Vec<SheetRecord>> {
    let conn = open_root(root)?;
    if let Some(_idle) = try_lock() {
        settle(&conn, root)?;
    }
    store::list(&conn)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct SheetProgress {
    /// Photographs placed so far.
    pub done: usize,
    pub total: usize,
}

/// One cell of the preview: where its image goes and what it says.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct PreviewCell {
    pub image_id: i64,
    pub image_box: Rect,
    pub filename: Option<String>,
    pub timestamp: Option<String>,
}

/// Counts for the chosen source and layout, and the first page's cells.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SheetPreview {
    pub photos: usize,
    pub pages: usize,
    pub geometry: Geometry,
    pub first_page: Vec<PreviewCell>,
    pub too_many: bool,
}

/// `2024-05-01T08:31:05.000Z` → `2024-05-01 08:31:05` (UTC, as the app shows it).
pub fn caption_time(normalized: &str) -> String {
    let s = normalized.trim_end_matches('Z');
    let s = s.split('.').next().unwrap_or(s);
    s.replacen('T', " ", 1)
}

fn captions(layout: &Layout, item: &store::Item) -> (Option<String>, Option<String>) {
    (
        // `/` on every platform, as the collection is shown in the app.
        layout.show_filename.then(|| item.relative_path.replace('\\', "/")),
        layout
            .show_timestamp
            .then(|| item.normalized_datetime.as_deref().map(caption_time))
            .flatten(),
    )
}

pub fn preview(root: &Path, source: &Source, layout: &Layout) -> Result<SheetPreview> {
    let conn = open_root(root)?;
    let layout = layout.clamped();
    let items = store::items(&conn, source)?;
    let geometry = Geometry::new(&layout);
    let first_page = items
        .iter()
        .take(layout.per_page())
        .enumerate()
        .map(|(i, item)| {
            let (filename, timestamp) = captions(&layout, item);
            PreviewCell {
                image_id: item.image_id,
                image_box: geometry.image_box(geometry.cell(i)),
                filename,
                timestamp,
            }
        })
        .collect();
    Ok(SheetPreview {
        photos: items.len(),
        pages: layout.pages(items.len()),
        geometry,
        first_page,
        too_many: items.len() > MAX_PHOTOS,
    })
}

/// If anything unwinds out of a generation, the temporary file goes and the row says so.
struct Cleanup<'a> {
    conn: &'a rusqlite::Connection,
    id: i64,
    part: PathBuf,
    armed: bool,
}

impl Drop for Cleanup<'_> {
    fn drop(&mut self) {
        if self.armed {
            let _ = fs::remove_file(&self.part);
            let _ = store::end(self.conn, self.id, "failed", Some("interrupted"));
        }
    }
}

/// Generate a sheet of `source`. `token` identifies it for [`cancel`]. `expected` is the
/// photo count the caller showed (and printed in `labels`): a source that changed since
/// is refused (`source-changed`) rather than printed with a wrong count.
#[allow(clippy::too_many_arguments)]
pub fn generate(
    root: &Path,
    source: &Source,
    layout: &Layout,
    labels: &Labels,
    expected: Option<usize>,
    token: u64,
    mut on_progress: impl FnMut(SheetProgress),
) -> Result<SheetRecord> {
    let _guard = try_lock().ok_or_else(|| fail("busy"))?;
    let conn = open_root(root)?;
    settle(&conn, root)?;
    let layout = layout.clamped();
    let items = store::items(&conn, source)?;
    if items.is_empty() {
        return Err(fail("empty"));
    }
    if items.len() > MAX_PHOTOS {
        return Err(fail("too-many"));
    }
    if expected.is_some_and(|n| n != items.len()) {
        return Err(fail("source-changed"));
    }
    let pages = layout.pages(items.len());
    let dir = sheets_dir(root);
    fs::create_dir_all(&dir).map_err(|_| fail("write-failed"))?;
    let id = store::begin(&conn, source, &layout, labels, items.len(), pages)?;
    let output = dir.join(format!("{id}.pdf"));
    store::set_output(&conn, id, &output.to_string_lossy())?;
    let part = part_path(&dir, id);
    // Either name already there is someone else's file: refuse before arming the cleanup.
    let file = match fs::OpenOptions::new().write(true).create_new(true).open(&part) {
        Ok(f) if !output.exists() => f,
        Ok(_) => {
            let _ = fs::remove_file(&part);
            store::end(&conn, id, "failed", Some("output-exists"))?;
            return store::get(&conn, id)?.ok_or(Error::StoreTransaction);
        }
        Err(e) => {
            let reason = if e.kind() == std::io::ErrorKind::AlreadyExists {
                "output-exists"
            } else {
                "write-failed"
            };
            store::end(&conn, id, "failed", Some(reason))?;
            return store::get(&conn, id)?.ok_or(Error::StoreTransaction);
        }
    };
    let mut cleanup = Cleanup { conn: &conn, id, part: part.clone(), armed: true };

    let outcome = write(root, &items, &layout, labels, pages, file, token, &mut on_progress);
    cleanup.armed = false;
    match outcome {
        Ok(unreadable) => match fs::rename(&part, &output) {
            Ok(()) => {
                let size = fs::metadata(&output).map(|m| m.len()).unwrap_or(0);
                // A sheet the database does not record as complete is not kept: the row
                // never stays `running` next to a finished file.
                if store::complete(&conn, id, unreadable, size).is_err() {
                    let _ = fs::remove_file(&output);
                    store::end(&conn, id, "failed", Some("write-failed"))?;
                }
            }
            Err(_) => {
                let _ = fs::remove_file(&part);
                store::end(&conn, id, "failed", Some("write-failed"))?;
            }
        },
        Err(reason) => {
            let _ = fs::remove_file(&part);
            let status = if reason == "cancelled" { "cancelled" } else { "failed" };
            store::end(&conn, id, status, (status == "failed").then_some(reason))?;
        }
    }
    store::get(&conn, id)?.ok_or(Error::StoreTransaction)
}

/// Writes every page; returns how many photographs were unreadable.
#[allow(clippy::too_many_arguments)]
fn write(
    root: &Path,
    items: &[store::Item],
    layout: &Layout,
    labels: &Labels,
    pages: usize,
    file: fs::File,
    token: u64,
    on_progress: &mut impl FnMut(SheetProgress),
) -> std::result::Result<usize, &'static str> {
    let size = Geometry::new(layout).thumbnail_size();
    let mut writer = pdf::SheetWriter::new(BufWriter::new(file), layout, labels, pages)
        .map_err(|_| "write-failed")?;
    let total = items.len();
    let mut done = 0;
    let mut unreadable = 0;
    on_progress(SheetProgress { done, total });
    for page in items.chunks(layout.per_page()) {
        if cancelled(token) {
            return Err("cancelled");
        }
        let cells: Vec<pdf::Cell> = page
            .par_iter()
            .map(|item| {
                let (filename, timestamp) = captions(layout, item);
                let jpeg = crate::clean::thumbs::thumbnail(
                    root,
                    item.image_id,
                    size,
                    &item.location,
                    item.hash.as_deref(),
                )
                .ok();
                pdf::Cell { filename: filename.unwrap_or_default(), timestamp, jpeg }
            })
            .collect();
        unreadable +=
            cells.iter().filter(|c| c.jpeg.as_deref().and_then(pdf::usable).is_none()).count();
        writer.page(&cells).map_err(|_| "write-failed")?;
        done += page.len();
        on_progress(SheetProgress { done, total });
    }
    let out = writer.finish().map_err(|_| "write-failed")?;
    let file = out.into_inner().map_err(|_| "write-failed")?;
    file.sync_all().map_err(|_| "write-failed")?;
    Ok(unreadable)
}
