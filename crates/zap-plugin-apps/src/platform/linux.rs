use super::{AppEntry, Platform};
use compact_str::CompactString;
use log::debug;
use std::fs;
use std::path::{Path, PathBuf};

pub struct LinuxPlatform;

impl Platform for LinuxPlatform {
    fn discover_apps(&self) -> Vec<AppEntry> {
        let mut apps = Vec::new();

        for dir in desktop_dirs() {
            let entries = match fs::read_dir(&dir) {
                Ok(e) => e,
                Err(_) => continue,
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "desktop") {
                    if let Some(app) = parse_desktop_file(&path) {
                        apps.push(app);
                    }
                }
            }
        }

        // Deduplicate by id, keeping first occurrence
        apps.sort_by(|a, b| a.id.cmp(&b.id));
        apps.dedup_by(|a, b| a.id == b.id);
        debug!("Discovered {} apps", apps.len());
        apps
    }

    fn launch_app(&self, app: &AppEntry) -> anyhow::Result<()> {
        let exec = strip_field_codes(&app.exec_path);
        let parts: Vec<&str> = exec.split_whitespace().collect();
        if parts.is_empty() {
            anyhow::bail!("empty exec command");
        }
        std::process::Command::new(parts[0])
            .args(&parts[1..])
            .spawn()?;
        Ok(())
    }
}

fn desktop_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
    ];

    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/share/applications"));
        dirs.push(home.join(".local/share/flatpak/exports/share/applications"));
    }

    dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));

    if let Ok(xdg_dirs) = std::env::var("XDG_DATA_DIRS") {
        for dir in xdg_dirs.split(':') {
            let app_dir = PathBuf::from(dir).join("applications");
            if !dirs.contains(&app_dir) {
                dirs.push(app_dir);
            }
        }
    }

    dirs
}

fn parse_desktop_file(path: &Path) -> Option<AppEntry> {
    let content = fs::read_to_string(path).ok()?;

    let mut name = None;
    let mut exec = None;
    let mut icon = None;
    let mut categories = None;
    let mut no_display = false;
    let mut hidden = false;
    let mut in_desktop_entry = false;

    for line in content.lines() {
        let line = line.trim();

        if line == "[Desktop Entry]" {
            in_desktop_entry = true;
            continue;
        }

        if line.starts_with('[') {
            if in_desktop_entry {
                break;
            }
            continue;
        }

        if !in_desktop_entry {
            continue;
        }

        if let Some(value) = line.strip_prefix("Name=") {
            if name.is_none() {
                name = Some(value.to_string());
            }
        } else if let Some(value) = line.strip_prefix("Exec=") {
            exec = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("Icon=") {
            icon = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("Categories=") {
            categories = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("NoDisplay=") {
            no_display = value.trim().eq_ignore_ascii_case("true");
        } else if let Some(value) = line.strip_prefix("Hidden=") {
            hidden = value.trim().eq_ignore_ascii_case("true");
        }
    }

    if no_display || hidden {
        return None;
    }

    let name = name?;
    let exec = exec?;
    let id = path.file_name()?.to_string_lossy().to_string();

    // Resolve icon name to a file path
    let resolved_icon = icon.and_then(|i| zap_core::icons::resolve_icon(&i));

    // Pick a user-friendly category from the Categories= field
    let category = categories.and_then(|c| pick_category(&c));

    Some(AppEntry {
        id,
        name: CompactString::from(name),
        exec_path: exec,
        icon_path: resolved_icon,
        category,
    })
}

/// Resolve an icon name to an absolute file path.
/// If it's already an absolute path, check it exists and return it.
/// Pick the most user-friendly category from a semicolon-separated list.
fn pick_category(categories: &str) -> Option<String> {
    // Map freedesktop categories to user-friendly labels
    let priority = [
        ("Game", "Game"),
        ("Development", "Development"),
        ("Graphics", "Graphics"),
        ("AudioVideo", "Media"),
        ("Audio", "Media"),
        ("Video", "Media"),
        ("Network", "Internet"),
        ("Office", "Office"),
        ("Education", "Education"),
        ("Science", "Science"),
        ("System", "System"),
        ("Settings", "Settings"),
        ("Utility", "Utility"),
    ];

    let cats: Vec<&str> = categories.split(';').map(|s| s.trim()).collect();

    for (key, label) in &priority {
        if cats.iter().any(|c| c == key) {
            return Some(label.to_string());
        }
    }

    None
}

fn strip_field_codes(exec: &str) -> String {
    exec.replace("%u", "")
        .replace("%U", "")
        .replace("%f", "")
        .replace("%F", "")
        .replace("%i", "")
        .replace("%c", "")
        .replace("%k", "")
        .trim()
        .to_string()
}
