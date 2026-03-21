use std::collections::HashSet;
use std::path::PathBuf;

pub fn pins_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zap")
        .join("emoji_pins.json")
}

pub fn load_pins() -> HashSet<String> {
    let path = pins_path();
    match std::fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => HashSet::new(),
    }
}

fn save_pins(pins: &HashSet<String>) {
    let path = pins_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_string(pins) {
        let _ = std::fs::write(&path, data);
    }
}

/// Toggle pin state for an emoji. Returns `true` if now pinned.
pub fn toggle_pin(name: &str) -> bool {
    let mut pins = load_pins();
    let pinned = if pins.contains(name) {
        pins.remove(name);
        false
    } else {
        pins.insert(name.to_string());
        true
    };
    save_pins(&pins);
    pinned
}
