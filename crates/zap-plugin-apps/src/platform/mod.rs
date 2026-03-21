#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub id: String,
    pub name: CompactString,
    pub exec_path: String,
    pub icon_path: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
}

pub trait Platform: Send + Sync {
    fn discover_apps(&self) -> Vec<AppEntry>;
    fn launch_app(&self, app: &AppEntry) -> anyhow::Result<()>;
}

#[cfg(target_os = "macos")]
pub fn create_platform() -> Arc<dyn Platform> {
    Arc::new(macos::MacOSPlatform)
}

#[cfg(target_os = "linux")]
pub fn create_platform() -> Arc<dyn Platform> {
    Arc::new(linux::LinuxPlatform)
}
