use crate::indexer::AppIndex;
use crate::search::SearchResult;
use tauri::Manager;

#[tauri::command]
pub fn search(query: String, state: tauri::State<'_, AppIndex>) -> Vec<SearchResult> {
    let apps = state.apps();
    crate::search::search(&query, &apps)
}

#[tauri::command]
pub fn launch(id: String, state: tauri::State<'_, AppIndex>) -> Result<(), String> {
    let app = state.find_by_id(&id).ok_or("App not found")?;
    state.platform().launch_app(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn hide_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}