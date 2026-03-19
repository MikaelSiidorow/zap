use crate::platform::{AppEntry, Platform};
use parking_lot::RwLock;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

pub struct AppIndex {
    apps: Arc<RwLock<Vec<AppEntry>>>,
    platform: Arc<dyn Platform>,
}

fn cache_path() -> Option<PathBuf> {
    let dir = dirs::cache_dir()?.join("zap");
    Some(dir.join("app_index.bin"))
}

fn load_cache() -> Option<Vec<AppEntry>> {
    let path = cache_path()?;
    let file = fs::File::open(&path).ok()?;
    let reader = std::io::BufReader::new(file);
    rmp_serde::from_read(reader).ok()
}

fn save_cache(apps: &[AppEntry]) {
    if let Some(path) = cache_path() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(data) = rmp_serde::to_vec(apps) {
            let _ = fs::write(&path, data);
        }
    }
}

impl AppIndex {
    pub fn new(platform: Arc<dyn Platform>) -> Self {
        let apps = if let Some(cached) = load_cache() {
            cached
        } else {
            let discovered = platform.discover_apps();
            save_cache(&discovered);
            discovered
        };

        Self {
            apps: Arc::new(RwLock::new(apps)),
            platform,
        }
    }

    pub fn apps(&self) -> Vec<AppEntry> {
        self.apps.read().clone()
    }

    pub fn find_by_id(&self, id: &str) -> Option<AppEntry> {
        self.apps.read().iter().find(|a| a.id == id).cloned()
    }

    pub fn platform(&self) -> &dyn Platform {
        self.platform.as_ref()
    }

    /// Perform a re-scan and update the index. Saves cache if changed.
    pub fn refresh(&self) {
        let new_apps = self.platform.discover_apps();
        let changed = {
            let current = self.apps.read();
            current.len() != new_apps.len()
                || current
                    .iter()
                    .zip(new_apps.iter())
                    .any(|(a, b)| a.id != b.id || a.name != b.name || a.exec_path != b.exec_path)
        };
        if changed {
            save_cache(&new_apps);
        }
        *self.apps.write() = new_apps;
    }

    pub fn spawn_refresh_task(&self) {
        let apps = self.apps.clone();
        let platform = self.platform.clone();
        std::thread::spawn(move || {
            // If we loaded from cache, do an immediate background rescan
            let new_apps = platform.discover_apps();
            {
                let current = apps.read();
                let changed = current.len() != new_apps.len()
                    || current.iter().zip(new_apps.iter()).any(|(a, b)| {
                        a.id != b.id || a.name != b.name || a.exec_path != b.exec_path
                    });
                if changed {
                    save_cache(&new_apps);
                }
            }
            *apps.write() = new_apps;

            // Then periodic rescans
            loop {
                std::thread::sleep(std::time::Duration::from_secs(30));
                let new_apps = platform.discover_apps();
                {
                    let current = apps.read();
                    let changed = current.len() != new_apps.len()
                        || current.iter().zip(new_apps.iter()).any(|(a, b)| {
                            a.id != b.id || a.name != b.name || a.exec_path != b.exec_path
                        });
                    if changed {
                        save_cache(&new_apps);
                    }
                }
                *apps.write() = new_apps;
            }
        });
    }
}
