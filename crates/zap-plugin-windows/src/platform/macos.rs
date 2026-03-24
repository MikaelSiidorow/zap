use super::{WindowEntry, WindowPlatform};
use std::process::Command;

pub struct MacosWindowPlatform;

impl WindowPlatform for MacosWindowPlatform {
    fn list_windows(&self) -> Vec<WindowEntry> {
        // Use osascript to list visible windows
        let script = r#"
            tell application "System Events"
                set windowList to {}
                repeat with proc in (every process whose visible is true)
                    repeat with win in (every window of proc)
                        set end of windowList to (name of proc) & tab & (name of win) & linefeed
                    end repeat
                end repeat
            end tell
            return windowList as text
        "#;
        let output = match Command::new("osascript").args(["-e", script]).output() {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
            _ => return vec![],
        };

        output
            .lines()
            .filter(|l| !l.is_empty())
            .enumerate()
            .filter_map(|(i, line)| {
                let (app_name, title) = line.split_once('\t')?;
                if title.is_empty() {
                    return None;
                }
                Some(WindowEntry {
                    window_id: i as u64,
                    title: title.to_string(),
                    app_name: app_name.to_string(),
                    icon_path: None,
                })
            })
            .collect()
    }

    fn activate_window(&self, _window_id: u64) -> anyhow::Result<()> {
        // macOS doesn't have stable window IDs via osascript; activate by app name
        // For now, this is a stub — a real implementation would use Accessibility API
        anyhow::bail!("macOS window activation requires Accessibility API (not yet implemented)")
    }
}
