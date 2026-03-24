mod monitor;
mod platform;
mod search;

use monitor::WindowMonitor;
use platform::{create_platform, WindowPlatform};
use std::sync::Arc;
use zap_core::{KeyboardHint, Plugin, PluginMeta, PluginResult};

pub struct WindowsPlugin {
    monitor: WindowMonitor,
    platform: Arc<dyn WindowPlatform>,
}

impl WindowsPlugin {
    pub fn new() -> Self {
        let platform: Arc<dyn WindowPlatform> = Arc::from(create_platform());
        Self {
            monitor: WindowMonitor::new(platform.clone()),
            platform,
        }
    }
}

impl Default for WindowsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for WindowsPlugin {
    fn meta(&self) -> PluginMeta {
        PluginMeta::new("windows", "Window Switcher")
            .description("Switch to open windows")
            .example("Firefox")
            .usage_ranking()
    }

    fn init(&mut self, _config: zap_core::serde_json::Value) -> anyhow::Result<()> {
        self.monitor.spawn_refresh_task();
        Ok(())
    }

    fn search(&self, query: &str) -> Vec<PluginResult> {
        let windows = self.monitor.windows();
        search::search(query, &windows, "windows")
    }

    fn execute(&self, result_id: &str) -> anyhow::Result<()> {
        let window_id: u64 = result_id.parse()?;
        let entry = self
            .monitor
            .find_by_id(window_id)
            .ok_or_else(|| anyhow::anyhow!("window {} not found (may have closed)", window_id))?;
        log::info!("Activating window: {} ({})", entry.title, entry.app_name);
        self.platform.activate_window(window_id)
    }

    fn hints(&self) -> Vec<KeyboardHint> {
        vec![KeyboardHint {
            key: "Enter".into(),
            label: "Switch to window".into(),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_metadata() {
        let p = WindowsPlugin::new();
        let meta = p.meta();
        assert_eq!(meta.id, "windows");
        assert_eq!(meta.name, "Window Switcher");
        assert!(meta.prefix.is_none());
        assert!(meta.usage_ranking);
    }
}
