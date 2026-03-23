mod monitor;
mod search;
pub mod store;

use monitor::{spawn_monitor, ClipboardConfig};
use std::path::PathBuf;
use zap_core::{Capability, KeyboardHint, Plugin, PluginMeta, PluginResult};

pub struct ClipboardPlugin {
    db_path: PathBuf,
    blob_dir: PathBuf,
    config: ClipboardConfig,
}

impl ClipboardPlugin {
    pub fn new() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("zap");
        Self {
            db_path: data_dir.join("clipboard.db"),
            blob_dir: data_dir.join("clipboard_blobs"),
            config: ClipboardConfig::default(),
        }
    }
}

impl Default for ClipboardPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for ClipboardPlugin {
    fn meta(&self) -> PluginMeta {
        PluginMeta::new("clipboard", "Clipboard History")
            .description("Search and paste from clipboard history")
            .example("cb search term")
            .prefix("cb ")
            .capabilities(vec![Capability::Delete, Capability::Pin, Capability::Copy])
    }

    fn init(&mut self, config: zap_core::serde_json::Value) -> anyhow::Result<()> {
        if let Some(v) = config.get("max_age_days").and_then(|v| v.as_u64()) {
            self.config.max_age_days = v as u32;
        }
        if let Some(v) = config.get("max_entries").and_then(|v| v.as_u64()) {
            self.config.max_entries = v as usize;
        }
        if let Some(v) = config.get("poll_interval_ms").and_then(|v| v.as_u64()) {
            self.config.poll_interval_ms = v;
        }

        if let Some(parent) = self.db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::create_dir_all(&self.blob_dir)?;

        let _conn = store::open_db(&self.db_path)?;

        spawn_monitor(
            self.db_path.clone(),
            self.blob_dir.clone(),
            ClipboardConfig {
                max_age_days: self.config.max_age_days,
                max_entries: self.config.max_entries,
                poll_interval_ms: self.config.poll_interval_ms,
            },
        );

        Ok(())
    }

    fn search(&self, query: &str) -> Vec<PluginResult> {
        let conn = match store::open_db(&self.db_path) {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        search::search(&conn, query, "clipboard")
    }

    fn execute(&self, _result_id: &str) -> anyhow::Result<()> {
        Ok(())
    }

    fn delete(&self, result_id: &str) -> anyhow::Result<()> {
        let id: i64 = result_id.parse()?;
        let conn = store::open_db(&self.db_path)?;
        let blob_path = store::delete_entry(&conn, id)?;
        if let Some(path) = blob_path {
            let _ = std::fs::remove_file(path);
        }
        Ok(())
    }

    fn toggle_pin(&self, result_id: &str) -> anyhow::Result<bool> {
        let id: i64 = result_id.parse()?;
        let conn = store::open_db(&self.db_path)?;
        store::toggle_pin(&conn, id)
    }

    fn hints(&self) -> Vec<KeyboardHint> {
        #[cfg(target_os = "macos")]
        let (delete_key, mod_key) = ("Cmd+⌫", "Cmd");
        #[cfg(not(target_os = "macos"))]
        let (delete_key, mod_key) = ("Del", "Ctrl");

        vec![
            KeyboardHint {
                key: "Enter".into(),
                label: "Paste".into(),
            },
            KeyboardHint {
                key: "Shift+Enter".into(),
                label: "Copy".into(),
            },
            KeyboardHint {
                key: delete_key.into(),
                label: "Delete".into(),
            },
            KeyboardHint {
                key: format!("{mod_key}+P"),
                label: "Pin".into(),
            },
        ]
    }
}
