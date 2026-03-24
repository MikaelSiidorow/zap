use crate::platform::{WindowEntry, WindowPlatform};
use parking_lot::RwLock;
use std::sync::Arc;

pub struct WindowMonitor {
    windows: Arc<RwLock<Vec<WindowEntry>>>,
    platform: Arc<dyn WindowPlatform>,
}

impl WindowMonitor {
    pub fn new(platform: Arc<dyn WindowPlatform>) -> Self {
        let initial = filter_self(platform.list_windows());
        Self {
            windows: Arc::new(RwLock::new(initial)),
            platform,
        }
    }

    pub fn windows(&self) -> Vec<WindowEntry> {
        self.windows.read().clone()
    }

    pub fn find_by_id(&self, window_id: u64) -> Option<WindowEntry> {
        self.windows
            .read()
            .iter()
            .find(|w| w.window_id == window_id)
            .cloned()
    }

    pub fn spawn_refresh_task(&self) {
        let windows = self.windows.clone();
        let platform = self.platform.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(2));
            *windows.write() = filter_self(platform.list_windows());
        });
    }
}

/// Filter out Zap's own window from the list.
fn filter_self(windows: Vec<WindowEntry>) -> Vec<WindowEntry> {
    windows
        .into_iter()
        .filter(|w| !w.app_name.eq_ignore_ascii_case("zap"))
        .collect()
}
