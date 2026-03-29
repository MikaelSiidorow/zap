use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;
use zap_core::{KeyboardHint, PluginHost, SearchResponse};

#[tauri::command]
#[specta::specta]
pub fn search(query: String, state: tauri::State<'_, PluginHost>) -> SearchResponse {
    state.search(&query)
}

#[tauri::command]
#[specta::specta]
pub fn plugin_hints(plugin_id: String, state: tauri::State<'_, PluginHost>) -> Vec<KeyboardHint> {
    state.plugin_hints(&plugin_id)
}

#[tauri::command]
#[specta::specta]
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
#[specta::specta]
pub fn delete_result(
    plugin_id: String,
    result_id: String,
    state: tauri::State<'_, PluginHost>,
) -> Result<(), String> {
    state
        .delete(&plugin_id, &result_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn toggle_pin(
    plugin_id: String,
    result_id: String,
    state: tauri::State<'_, PluginHost>,
) -> Result<bool, String> {
    state
        .toggle_pin(&plugin_id, &result_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn open_url(url: String) -> Result<(), String> {
    open::that(url).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Autostart
// ---------------------------------------------------------------------------

#[tauri::command]
#[specta::specta]
pub fn get_autostart(app: tauri::AppHandle) -> Result<bool, String> {
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn set_autostart(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(|e| e.to_string())
    } else {
        manager.disable().map_err(|e| e.to_string())
    }
}

// ---------------------------------------------------------------------------
// Clipboard helpers
// ---------------------------------------------------------------------------

fn load_image(path: &str) -> Result<arboard::ImageData<'static>, String> {
    let img = image::open(path).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(arboard::ImageData {
        width: width as usize,
        height: height as usize,
        bytes: std::borrow::Cow::Owned(rgba.into_raw()),
    })
}

/// Keep clipboard alive in a background thread so clipboard managers can read
/// the content (X11 ownership model). Optionally simulates Ctrl+V / Cmd+V.
fn keep_alive_and_paste(clipboard: arboard::Clipboard, paste: bool) {
    std::thread::spawn(move || {
        if paste {
            std::thread::sleep(std::time::Duration::from_millis(80));
            simulate_paste();
            std::thread::sleep(std::time::Duration::from_millis(500));
        } else {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        drop(clipboard);
    });
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

// ---------------------------------------------------------------------------
// Clipboard commands
// ---------------------------------------------------------------------------

#[tauri::command]
#[specta::specta]
pub fn copy_to_clipboard(text: String) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())?;
    keep_alive_and_paste(clipboard, false);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn hide_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tauri::command]
#[specta::specta]
pub fn paste_to_frontmost(text: String, app: tauri::AppHandle) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())?;
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    keep_alive_and_paste(clipboard, true);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn paste_image_to_frontmost(path: String, app: tauri::AppHandle) -> Result<(), String> {
    let img_data = load_image(&path)?;
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_image(img_data).map_err(|e| e.to_string())?;
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    keep_alive_and_paste(clipboard, true);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn copy_image_to_clipboard(path: String) -> Result<(), String> {
    let img_data = load_image(&path)?;
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_image(img_data).map_err(|e| e.to_string())?;
    keep_alive_and_paste(clipboard, false);
    Ok(())
}
