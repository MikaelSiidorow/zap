#[cfg(target_os = "linux")]
use std::collections::HashMap;
use std::path::{Path, PathBuf};
#[cfg(target_os = "linux")]
use std::sync::OnceLock;

/// Resolve a freedesktop icon name to an absolute file path.
/// Searches hicolor icon theme dirs, XDG data dirs, flatpak exports, and pixmaps.
#[cfg(target_os = "linux")]
pub fn resolve_icon(icon: &str) -> Option<String> {
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

    let mut search_dirs: Vec<PathBuf> = Vec::new();

    // Build from XDG_DATA_DIRS (the proper freedesktop way)
    let xdg = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());
    for dir in xdg.split(':') {
        let icons_dir = PathBuf::from(dir).join("icons/hicolor");
        if !search_dirs.contains(&icons_dir) {
            search_dirs.push(icons_dir);
        }
    }

    // User-local dirs (XDG_DATA_HOME, flatpak, nix)
    if let Some(home) = dirs::home_dir() {
        search_dirs.push(home.join(".local/share/icons/hicolor"));
        search_dirs.push(home.join(".local/share/flatpak/exports/share/icons/hicolor"));
        search_dirs.push(home.join(".nix-profile/share/icons/hicolor"));
    }
    search_dirs.push(PathBuf::from(
        "/var/lib/flatpak/exports/share/icons/hicolor",
    ));

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

    // Pixmaps fallback (built from XDG_DATA_DIRS)
    let mut pixmap_dirs: Vec<PathBuf> = xdg
        .split(':')
        .map(|d| PathBuf::from(d).join("pixmaps"))
        .collect();
    if let Some(home) = dirs::home_dir() {
        pixmap_dirs.push(home.join(".local/share/pixmaps"));
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

#[cfg(not(target_os = "linux"))]
pub fn resolve_icon(_icon: &str) -> Option<String> {
    None
}

/// Info resolved from a .desktop file by matching StartupWMClass.
#[derive(Clone)]
pub struct DesktopInfo {
    pub name: String,
    pub icon_path: Option<String>,
}

/// Look up app name and icon from .desktop files by matching `StartupWMClass`
/// against the WM_CLASS instance or class name.
///
/// The desktop file index is built once on first call and cached for the
/// lifetime of the process. Desktop files rarely change during a session.
#[cfg(target_os = "linux")]
pub fn desktop_info_for_class(wm_instance: &str, wm_class: &str) -> Option<DesktopInfo> {
    static INDEX: OnceLock<HashMap<String, DesktopInfo>> = OnceLock::new();
    let index = INDEX.get_or_init(build_desktop_index);

    let lower_instance = wm_instance.to_lowercase();
    let lower_class = wm_class.to_lowercase();

    index
        .get(&lower_instance)
        .or_else(|| index.get(&lower_class))
        .cloned()
}

#[cfg(not(target_os = "linux"))]
pub fn desktop_info_for_class(_wm_instance: &str, _wm_class: &str) -> Option<DesktopInfo> {
    None
}

/// Build a HashMap from lowercase StartupWMClass to DesktopInfo.
#[cfg(target_os = "linux")]
fn build_desktop_index() -> HashMap<String, DesktopInfo> {
    let mut index = HashMap::new();

    for dir in &desktop_app_dirs() {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                continue;
            }

            let Ok(contents) = std::fs::read_to_string(&path) else {
                continue;
            };

            let mut startup_wm_class = None;
            let mut name = None;
            let mut icon_name = None;
            let mut in_desktop_entry = false;

            for line in contents.lines() {
                if line.starts_with('[') {
                    in_desktop_entry = line == "[Desktop Entry]";
                    continue;
                }
                if !in_desktop_entry {
                    continue;
                }
                if let Some(v) = line.strip_prefix("StartupWMClass=") {
                    startup_wm_class = Some(v.to_string());
                } else if let Some(v) = line.strip_prefix("Name=") {
                    if name.is_none() {
                        name = Some(v.to_string());
                    }
                } else if let Some(v) = line.strip_prefix("Icon=") {
                    if icon_name.is_none() {
                        icon_name = Some(v.to_string());
                    }
                }
            }

            let Some(wm) = startup_wm_class else {
                continue;
            };

            let key = wm.to_lowercase();
            if index.contains_key(&key) {
                continue;
            }

            let icon_path = icon_name.and_then(|i| resolve_icon(&i));
            let fallback_name = name.unwrap_or_else(|| wm.clone());
            index.insert(
                key,
                DesktopInfo {
                    name: fallback_name,
                    icon_path,
                },
            );
        }
    }

    index
}

#[cfg(target_os = "linux")]
fn desktop_app_dirs() -> Vec<PathBuf> {
    let xdg = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());
    let mut dirs: Vec<PathBuf> = xdg
        .split(':')
        .map(|d| PathBuf::from(d).join("applications"))
        .collect();
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/share/applications"));
        dirs.push(home.join(".local/share/flatpak/exports/share/applications"));
        dirs.push(home.join(".nix-profile/share/applications"));
    }
    dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));
    dirs
}
