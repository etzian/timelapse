//! Thin Tauri shell: commands are adapters over `timelapse-core`.
//! Errors cross IPC as `{ kind, args }`; the UI renders them through Fluent.

use serde::Serialize;

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

#[tauri::command]
fn greet(name: String) -> Result<timelapse_core::Greeting, IpcError> {
    Ok(timelapse_core::greet(&name)?)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();

    // Desktop only: remember window geometry, focus the running instance
    // instead of opening a second one (GUIDELINES §8.2).
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_window_state::Builder::default().build()).plugin(
        tauri_plugin_single_instance::init(|app, _args, _cwd| {
            use tauri::Manager;
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.set_focus();
            }
        }),
    );

    builder
        .invoke_handler(tauri::generate_handler![app_info, greet])
        .run(tauri::generate_context!())
        .expect("error while running Timelapse Studio");
}
