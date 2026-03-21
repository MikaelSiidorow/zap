use std::path::PathBuf;
use tauri::Manager;
use zap_core::{PluginHost, PluginResult};

#[tauri::command]
pub fn search(query: String, state: tauri::State<'_, PluginHost>) -> Vec<PluginResult> {
    state.search(&query)
}

#[tauri::command]
pub fn execute(
    plugin_id: String,
    result_id: String,
    state: tauri::State<'_, PluginHost>,
) -> Result<(), String> {
    state
        .execute(&plugin_id, &result_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn copy_to_clipboard(text: String) -> Result<(), String> {
    arboard::Clipboard::new()
        .and_then(|mut c| c.set_text(text))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn hide_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

fn clipboard_db_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zap")
        .join("clipboard.db")
}

#[tauri::command]
pub fn paste_to_frontmost(text: String, app: tauri::AppHandle) -> Result<(), String> {
    arboard::Clipboard::new()
        .and_then(|mut c| c.set_text(text))
        .map_err(|e| e.to_string())?;
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(80));
        simulate_paste();
    });
    Ok(())
}

fn simulate_paste() {
    #[cfg(target_os = "linux")]
    {
        // Try xdotool first (X11), fall back to wtype (Wayland)
        let status = std::process::Command::new("xdotool")
            .args(["key", "ctrl+v"])
            .status();
        if status.is_err() || !status.unwrap().success() {
            let _ = std::process::Command::new("wtype")
                .args(["-M", "ctrl", "-P", "v", "-p", "v", "-m", "ctrl"])
                .status();
        }
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("osascript")
            .args([
                "-e",
                "tell application \"System Events\" to keystroke \"v\" using command down",
            ])
            .status();
    }
}

#[tauri::command]
pub fn clipboard_delete(id: i64) -> Result<(), String> {
    let db_path = clipboard_db_path();
    let conn = zap_plugin_clipboard::store::open_db(&db_path).map_err(|e| e.to_string())?;
    zap_plugin_clipboard::store::delete_entry(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clipboard_toggle_pin(id: i64) -> Result<bool, String> {
    let db_path = clipboard_db_path();
    let conn = zap_plugin_clipboard::store::open_db(&db_path).map_err(|e| e.to_string())?;
    zap_plugin_clipboard::store::toggle_pin(&conn, id).map_err(|e| e.to_string())
}
