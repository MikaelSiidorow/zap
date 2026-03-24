#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[derive(Clone)]
pub struct WindowEntry {
    pub window_id: u64,
    pub title: String,
    pub app_name: String,
    pub icon_path: Option<String>,
}

pub trait WindowPlatform: Send + Sync {
    fn list_windows(&self) -> Vec<WindowEntry>;
    fn activate_window(&self, window_id: u64) -> anyhow::Result<()>;
}

#[cfg(target_os = "linux")]
pub fn create_platform() -> Box<dyn WindowPlatform> {
    Box::new(linux::LinuxWindowPlatform)
}

#[cfg(target_os = "macos")]
pub fn create_platform() -> Box<dyn WindowPlatform> {
    Box::new(macos::MacosWindowPlatform)
}

#[cfg(target_os = "windows")]
pub fn create_platform() -> Box<dyn WindowPlatform> {
    Box::new(windows::WindowsWindowPlatform)
}
