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
    let resolved_icon = icon.and_then(|i| resolve_icon(&i));

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
/// Otherwise search the icon theme directories.
fn resolve_icon(icon: &str) -> Option<String> {
    // Already an absolute path
    if icon.starts_with('/') {
        if Path::new(icon).exists() {
            return Some(icon.to_string());
        }
        return None;
    }

    let sizes = [
        "48x48",
        "scalable",
        "256x256",
        "128x128",
        "64x64",
        "32x32",
        "512x512",
        "1024x1024",
    ];
    let extensions = ["png", "svg"];

    // Search icon theme dirs
    let mut search_dirs: Vec<PathBuf> = Vec::new();

    // Hicolor is the universal fallback theme
    search_dirs.push(PathBuf::from("/usr/share/icons/hicolor"));

    // Also check the active theme if set
    if let Ok(xdg_dirs) = std::env::var("XDG_DATA_DIRS") {
        for dir in xdg_dirs.split(':') {
            let icons_dir = PathBuf::from(dir).join("icons/hicolor");
            if !search_dirs.contains(&icons_dir) {
                search_dirs.push(icons_dir);
            }
        }
    }

    // Flatpak icons
    search_dirs.push(PathBuf::from(
        "/var/lib/flatpak/exports/share/icons/hicolor",
    ));
    if let Some(home) = dirs::home_dir() {
        search_dirs.push(home.join(".local/share/flatpak/exports/share/icons/hicolor"));
        search_dirs.push(home.join(".local/share/icons/hicolor"));
    }

    for base in &search_dirs {
        for size in &sizes {
            for ext in &extensions {
                let path = base.join(size).join("apps").join(format!("{icon}.{ext}"));
                if path.exists() {
                    return Some(path.to_string_lossy().to_string());
                }
            }
        }
    }

    // Check pixmaps dirs as last resort
    let mut pixmap_dirs = vec![PathBuf::from("/usr/share/pixmaps")];
    if let Ok(xdg_dirs) = std::env::var("XDG_DATA_DIRS") {
        for dir in xdg_dirs.split(':') {
            let pixmap_dir = PathBuf::from(dir).join("pixmaps");
            if !pixmap_dirs.contains(&pixmap_dir) {
                pixmap_dirs.push(pixmap_dir);
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        let local_pixmaps = home.join(".local/share/pixmaps");
        if !pixmap_dirs.contains(&local_pixmaps) {
            pixmap_dirs.push(local_pixmaps);
        }
    }

    for pixmap_dir in &pixmap_dirs {
        for ext in &extensions {
            let path = pixmap_dir.join(format!("{icon}.{ext}"));
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }

    None
}

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
