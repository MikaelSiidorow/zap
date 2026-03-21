mod monitor;
mod search;
pub mod store;

use monitor::{spawn_monitor, ClipboardConfig};
use std::path::PathBuf;
use zap_core::{Plugin, PluginResult};

pub struct ClipboardPlugin {
    db_path: PathBuf,
    config: ClipboardConfig,
}

impl ClipboardPlugin {
    pub fn new() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("zap");
        Self {
            db_path: data_dir.join("clipboard.db"),
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

    fn init(&mut self) -> anyhow::Result<()> {
        // Ensure data directory exists
        if let Some(parent) = self.db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Open DB and run migrations to verify schema
        let _conn = store::open_db(&self.db_path)?;

        // Spawn background monitor
        spawn_monitor(
            self.db_path.clone(),
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
        // Primary action is Paste, handled by the runtime
        Ok(())
    }
}
