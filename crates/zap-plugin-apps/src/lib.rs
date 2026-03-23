mod indexer;
pub mod platform;
mod search;

use indexer::AppIndex;
use zap_core::{Plugin, PluginMeta, PluginResult};

pub struct AppsPlugin {
    index: AppIndex,
}

impl AppsPlugin {
    pub fn new() -> Self {
        let platform = platform::create_platform();
        Self {
            index: AppIndex::new(platform),
        }
    }
}

impl Default for AppsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for AppsPlugin {
    fn meta(&self) -> PluginMeta {
        PluginMeta::new("apps", "Applications")
            .description("Search and launch installed applications")
            .example("firefox")
            .usage_ranking()
    }

    fn init(&mut self, _config: zap_core::serde_json::Value) -> anyhow::Result<()> {
        self.index.spawn_refresh_task();
        Ok(())
    }

    fn search(&self, query: &str) -> Vec<PluginResult> {
        let apps = self.index.apps();
        search::search(query, &apps, "apps")
    }

    fn execute(&self, result_id: &str) -> anyhow::Result<()> {
        let app = self
            .index
            .find_by_id(result_id)
            .ok_or_else(|| anyhow::anyhow!("app '{}' not found", result_id))?;
        self.index.platform().launch_app(&app)
    }

    fn refresh(&self) {
        self.index.refresh();
    }
}
