use super::{WindowEntry, WindowPlatform};

pub struct WindowsWindowPlatform;

impl WindowPlatform for WindowsWindowPlatform {
    fn list_windows(&self) -> Vec<WindowEntry> {
        // TODO: Use windows-rs EnumWindows
        vec![]
    }

    fn activate_window(&self, _window_id: u64) -> anyhow::Result<()> {
        anyhow::bail!("window activation not yet implemented on Windows")
    }
}
