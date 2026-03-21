mod monitor;
mod search;
pub mod store;

use monitor::{spawn_monitor, ClipboardConfig};
use std::path::PathBuf;
use zap_core::{KeyboardHint, Plugin, PluginResult};

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

    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }
}

impl Default for ClipboardPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for ClipboardPlugin {
    fn id(&self) -> &str {
        "clipboard"
    }

    fn name(&self) -> &str {
        "Clipboard History"
    }

    fn description(&self) -> &str {
        "Search and paste from clipboard history"
    }

    fn example(&self) -> Option<&str> {
        Some("cb search term")
    }

    fn prefix(&self) -> Option<&str> {
        Some("cb ")
    }

    fn init(&mut self, config: zap_core::serde_json::Value) -> anyhow::Result<()> {
        // Apply config overrides
        if let Some(v) = config.get("max_age_days").and_then(|v| v.as_u64()) {
            self.config.max_age_days = v as u32;
        }
        if let Some(v) = config.get("max_entries").and_then(|v| v.as_u64()) {
            self.config.max_entries = v as usize;
        }
        if let Some(v) = config.get("poll_interval_ms").and_then(|v| v.as_u64()) {
            self.config.poll_interval_ms = v;
        }

        // Ensure data directory exists
        if let Some(parent) = self.db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Ensure blob directory exists
        std::fs::create_dir_all(&self.blob_dir)?;

        // Open DB and run migrations to verify schema
        let _conn = store::open_db(&self.db_path)?;

        // Spawn background monitor
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
        search::search(&conn, query, self.id())
    }

    fn execute(&self, _result_id: &str) -> anyhow::Result<()> {
        // Primary action is Paste/PasteImage, handled by the runtime
        Ok(())
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
