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
            integrity_check
        ])
        .run(tauri::generate_context!())
        .expect("error while running Timelapse Studio");
}
