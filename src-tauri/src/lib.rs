//! Thin Tauri shell: commands are adapters over `timelapse-core`.
//!
//! Errors cross IPC as `{ kind, args }`; the UI renders them through Fluent.
//! The shell owns no business logic and no database state — every command calls a
//! core function that opens a short-lived connection (ADR-007, ADR-011) and closes
//! it on return, so concurrent commands never hold locks on each other.

use serde::Serialize;
use std::path::PathBuf;

#[cfg(desktop)]
use tauri::Manager as _;

#[derive(Serialize)]
struct IpcError {
    kind: &'static str,
    args: serde_json::Value,
}

impl From<timelapse_core::Error> for IpcError {
    fn from(e: timelapse_core::Error) -> Self {
        Self { kind: e.kind(), args: e.args() }
    }
}

#[derive(Serialize)]
struct AppInfo {
    version: &'static str,
}

#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo { version: env!("CARGO_PKG_VERSION") }
}

/// Phase 1 "import": the UI passes `root` (the folder the user picked). Core derives
/// the project name + database location and does scan → hash → dedup → upsert,
/// returning a report. No long-lived writer remains in the shell.
#[tauri::command]
fn import_folder(
    root: PathBuf,
) -> std::result::Result<timelapse_core::import::ImportReport, IpcError> {
    Ok(timelapse_core::import::import_folder(&root)?)
}

/// List the inventory rows for `root`.
#[tauri::command]
fn list_images(
    root: PathBuf,
) -> std::result::Result<Vec<timelapse_core::import::store::ImageRow>, IpcError> {
    Ok(timelapse_core::import::list_for_root(&root)?)
}

/// Integrity snapshot for `root`: missing files + duplicate groups.
#[tauri::command]
fn integrity_check(
    root: PathBuf,
) -> std::result::Result<timelapse_core::import::store::IntegrityResult, IpcError> {
    Ok(timelapse_core::import::integrity_for_root(&root)?)
}

/// Phase 2: resolve and persist capture times for every imported image of `root` that
/// has none yet (EXIF → filename → filesystem). Reads originals only.
#[tauri::command]
fn backfill_metadata(
    root: PathBuf,
) -> std::result::Result<timelapse_core::import::BackfillReport, IpcError> {
    Ok(timelapse_core::import::backfill_for_root(&root)?)
}

/// Phase 2: the chronological timeline of `root` (unresolved images last).
#[tauri::command]
fn get_timeline(
    root: PathBuf,
) -> std::result::Result<Vec<timelapse_core::import::store::TimelineEntry>, IpcError> {
    Ok(timelapse_core::import::timeline_for_root(&root)?)
}

/// Phase 2: image counts per hour / day / month over an optional `[from, to)` period.
#[tauri::command]
fn timeline_distribution(
    root: PathBuf,
    granularity: timelapse_core::import::store::Granularity,
    from: Option<String>,
    to: Option<String>,
) -> std::result::Result<Vec<timelapse_core::import::store::DistributionBucket>, IpcError> {
    Ok(timelapse_core::import::distribution_for_root(
        &root,
        granularity,
        from.as_deref(),
        to.as_deref(),
    )?)
}

/// Phase 2: unusually long intervals on the timeline; omitted options take core's defaults.
#[tauri::command]
fn detect_timeline_gaps(
    root: PathBuf,
    factor: Option<f64>,
    min_seconds: Option<i64>,
) -> std::result::Result<timelapse_core::import::store::GapReport, IpcError> {
    let opts = timelapse_core::import::store::GapOptions::from_parts(factor, min_seconds);
    Ok(timelapse_core::import::gaps_for_root(&root, opts)?)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Dialog is used by the frontend for folder picking on every platform; the
    // window_state / single_instance plugins are desktop-only (GUIDELINES §8.2).
    let builder = tauri::Builder::default().plugin(tauri_plugin_dialog::init());

    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_window_state::Builder::default().build()).plugin(
        tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.set_focus();
            }
        }),
    );

    builder
        .invoke_handler(tauri::generate_handler![
            app_info,
            import_folder,
            list_images,
            integrity_check,
            backfill_metadata,
            get_timeline,
            timeline_distribution,
            detect_timeline_gaps
        ])
        .run(tauri::generate_context!())
        .expect("error while running Timelapse Studio");
}
