use tauri::Manager;
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

#[tauri::command]
#[specta::specta]
pub fn copy_to_clipboard(text: String) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())?;
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(1));
        drop(clipboard);
    });
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn hide_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
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
#[specta::specta]
pub fn paste_to_frontmost(text: String, app: tauri::AppHandle) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())?;
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(80));
        simulate_paste();
        std::thread::sleep(std::time::Duration::from_millis(500));
        drop(clipboard);
    });
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn paste_image_to_frontmost(path: String, app: tauri::AppHandle) -> Result<(), String> {
    let img = image::open(&path).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let img_data = arboard::ImageData {
        width: width as usize,
        height: height as usize,
        bytes: std::borrow::Cow::Owned(rgba.into_raw()),
    };
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_image(img_data).map_err(|e| e.to_string())?;

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(80));
        simulate_paste();
        std::thread::sleep(std::time::Duration::from_millis(500));
        drop(clipboard);
    });
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn copy_image_to_clipboard(path: String) -> Result<(), String> {
    let img = image::open(&path).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let img_data = arboard::ImageData {
        width: width as usize,
        height: height as usize,
        bytes: std::borrow::Cow::Owned(rgba.into_raw()),
    };
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_image(img_data).map_err(|e| e.to_string())?;
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(1));
        drop(clipboard);
    });
    Ok(())
}
