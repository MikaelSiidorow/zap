use super::{AppEntry, Platform};
use compact_str::CompactString;
use log::debug;
use std::path::{Path, PathBuf};

pub struct WindowsPlatform;

impl Platform for WindowsPlatform {
    fn discover_apps(&self) -> Vec<AppEntry> {
        let mut apps = Vec::new();

        for dir in shortcut_dirs() {
            scan_dir(&dir, &mut apps);
        }

        apps.sort_by(|a, b| a.name.cmp(&b.name));
        apps.dedup_by(|a, b| a.id == b.id);
        debug!("Discovered {} apps", apps.len());
        apps
    }

    fn launch_app(&self, app: &AppEntry) -> anyhow::Result<()> {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &app.exec_path])
            .spawn()?;
        Ok(())
    }
}

fn shortcut_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // Start Menu shortcuts (per-user)
    if let Some(appdata) = std::env::var_os("APPDATA") {
        dirs.push(PathBuf::from(appdata).join("Microsoft\\Windows\\Start Menu\\Programs"));
    }

    // Start Menu shortcuts (system-wide)
    if let Some(programdata) = std::env::var_os("PROGRAMDATA") {
        dirs.push(PathBuf::from(programdata).join("Microsoft\\Windows\\Start Menu\\Programs"));
    }

    dirs
}

fn scan_dir(dir: &Path, apps: &mut Vec<AppEntry>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, apps);
        } else if path.extension().is_some_and(|e| e == "lnk") {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                apps.push(AppEntry {
                    id: path.to_string_lossy().to_string(),
                    name: CompactString::from(name),
                    exec_path: path.to_string_lossy().to_string(),
                    icon_path: None,
                    category: parent_folder_name(&path),
                });
            }
        }
    }
}

fn parent_folder_name(path: &Path) -> Option<String> {
    let parent = path.parent()?.file_name()?.to_str()?;
    if parent == "Programs" {
        return None;
    }
    Some(parent.to_string())
}
