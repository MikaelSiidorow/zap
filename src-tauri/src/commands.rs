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
