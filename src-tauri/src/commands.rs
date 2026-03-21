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

fn simulate_paste() {
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};
    let Ok(mut enigo) = Enigo::new(&Settings::default()) else {
        return;
    };

    #[cfg(target_os = "macos")]
    {
        let _ = enigo.key(Key::Meta, Direction::Press);
        let _ = enigo.key(Key::Unicode('v'), Direction::Click);
        let _ = enigo.key(Key::Meta, Direction::Release);
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = enigo.key(Key::Control, Direction::Press);
        let _ = enigo.key(Key::Unicode('v'), Direction::Click);
        let _ = enigo.key(Key::Control, Direction::Release);
    }
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

#[tauri::command]
pub fn paste_image_to_frontmost(path: String, app: tauri::AppHandle) -> Result<(), String> {
    // Read PNG file and set as clipboard image
    let img = image::open(&path).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let img_data = arboard::ImageData {
        width: width as usize,
        height: height as usize,
        bytes: std::borrow::Cow::Owned(rgba.into_raw()),
    };
    arboard::Clipboard::new()
        .and_then(|mut c| c.set_image(img_data))
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

#[tauri::command]
pub fn copy_image_to_clipboard(path: String) -> Result<(), String> {
    let img = image::open(&path).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let img_data = arboard::ImageData {
        width: width as usize,
        height: height as usize,
        bytes: std::borrow::Cow::Owned(rgba.into_raw()),
    };
    arboard::Clipboard::new()
        .and_then(|mut c| c.set_image(img_data))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clipboard_delete(id: i64) -> Result<(), String> {
    let db_path = clipboard_db_path();
    let conn = zap_plugin_clipboard::store::open_db(&db_path).map_err(|e| e.to_string())?;
    let blob_path =
        zap_plugin_clipboard::store::delete_entry(&conn, id).map_err(|e| e.to_string())?;
    // Clean up blob file if this was an image entry
    if let Some(path) = blob_path {
        let _ = std::fs::remove_file(path);
    }
    Ok(())
}

#[tauri::command]
pub fn clipboard_toggle_pin(id: i64) -> Result<bool, String> {
    let db_path = clipboard_db_path();
    let conn = zap_plugin_clipboard::store::open_db(&db_path).map_err(|e| e.to_string())?;
    zap_plugin_clipboard::store::toggle_pin(&conn, id).map_err(|e| e.to_string())
}
