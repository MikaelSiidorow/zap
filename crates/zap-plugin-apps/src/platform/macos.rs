use super::{AppEntry, Platform};
use compact_str::CompactString;
use std::fs;
use std::path::Path;

pub struct MacOSPlatform;

impl Platform for MacOSPlatform {
    fn discover_apps(&self) -> Vec<AppEntry> {
        let mut apps = Vec::new();
        for dir in &["/Applications", "/System/Applications"] {
            scan_app_dir(Path::new(dir), &mut apps);
        }
        apps
    }

    fn launch_app(&self, app: &AppEntry) -> anyhow::Result<()> {
        std::process::Command::new("open")
            .arg("-a")
            .arg(&app.exec_path)
            .spawn()?;
        Ok(())
    }
}

fn scan_app_dir(dir: &Path, apps: &mut Vec<AppEntry>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "app") {
            if let Some(app) = parse_app_bundle(&path) {
                apps.push(app);
            }
        } else if path.is_dir() && path.extension().is_none() {
            scan_app_dir(&path, apps);
        }
    }
}

fn parse_app_bundle(path: &Path) -> Option<AppEntry> {
    let plist_path = path.join("Contents/Info.plist");
    let value = plist::Value::from_file(&plist_path).ok()?;
    let dict = value.as_dictionary()?;

    let name = dict
        .get("CFBundleName")
        .or_else(|| dict.get("CFBundleDisplayName"))
        .and_then(|v| v.as_string())
        .or_else(|| path.file_stem().and_then(|s| s.to_str()))?;

    let bundle_id = dict
        .get("CFBundleIdentifier")
        .and_then(|v| v.as_string())
        .unwrap_or(name);

    Some(AppEntry {
        id: bundle_id.to_string(),
        name: CompactString::from(name),
        exec_path: path.to_string_lossy().to_string(),
        icon_path: None,
        category: None,
    })
}
