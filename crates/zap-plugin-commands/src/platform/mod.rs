#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub trait Platform: Send + Sync {
    fn execute(&self, command_id: &str) -> anyhow::Result<()>;
}

#[cfg(target_os = "linux")]
pub fn create_platform() -> Box<dyn Platform> {
    Box::new(linux::LinuxPlatform)
}

#[cfg(target_os = "macos")]
pub fn create_platform() -> Box<dyn Platform> {
    Box::new(macos::MacOSPlatform)
}

#[cfg(target_os = "windows")]
pub fn create_platform() -> Box<dyn Platform> {
    Box::new(windows::WindowsPlatform)
}
