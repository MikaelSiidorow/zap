use rusqlite::Connection;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const DECAY_HALF_LIFE_DAYS: f64 = 14.0;
const MAX_USAGE_BONUS: u32 = 50;

pub struct UsageTracker {
    conn: Mutex<Connection>,
}

impl UsageTracker {
    pub fn open_default() -> Option<Self> {
        let db_path = dirs::data_dir()?.join("zap").join("usage.db");
        match Self::open_file(&db_path) {
            Ok(tracker) => Some(tracker),
            Err(e) => {
                log::warn!("Failed to open usage database: {e}");
                None
            }
        }
    }

    fn open_file(path: &std::path::Path) -> anyhow::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        Self::init(conn)
    }

    #[cfg(test)]
    fn open_in_memory() -> Self {
        Self::init(Connection::open_in_memory().unwrap()).unwrap()
    }

    fn init(conn: Connection) -> anyhow::Result<Self> {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 5000;
             CREATE TABLE IF NOT EXISTS usage (
                 plugin_id TEXT NOT NULL,
                 item_id TEXT NOT NULL,
                 use_count INTEGER NOT NULL DEFAULT 0,
                 last_used INTEGER NOT NULL,
                 PRIMARY KEY (plugin_id, item_id)
             );",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn record(&self, plugin_id: &str, item_id: &str) {
        let now = now_secs();
        let conn = self.conn.lock().expect("usage db lock poisoned");
        if let Err(e) = conn.execute(
            "INSERT INTO usage (plugin_id, item_id, use_count, last_used)
             VALUES (?1, ?2, 1, ?3)
             ON CONFLICT(plugin_id, item_id) DO UPDATE SET
                 use_count = use_count + 1,
                 last_used = ?3",
            rusqlite::params![plugin_id, item_id, now],
        ) {
            log::warn!("Failed to record usage: {e}");
        }
    }

    pub fn get_bonus(&self, plugin_id: &str, item_id: &str) -> u32 {
        let conn = self.conn.lock().expect("usage db lock poisoned");
        conn.query_row(
            "SELECT use_count, last_used FROM usage WHERE plugin_id = ?1 AND item_id = ?2",
            rusqlite::params![plugin_id, item_id],
            |row| {
                let use_count: i64 = row.get(0)?;
                let last_used: i64 = row.get(1)?;
                Ok(compute_bonus(use_count, last_used))
            },
        )
        .unwrap_or(0)
    }
}

fn compute_bonus(use_count: i64, last_used: i64) -> u32 {
    let days_elapsed = (now_secs() - last_used) as f64 / 86400.0;
    let decay = 0.5_f64.powf(days_elapsed / DECAY_HALF_LIFE_DAYS);
    let bonus = (use_count as f64 * decay).round() as u32;
    bonus.min(MAX_USAGE_BONUS)
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_usage_returns_zero() {
        let tracker = UsageTracker::open_in_memory();
        assert_eq!(tracker.get_bonus("apps", "firefox"), 0);
    }

    #[test]
    fn record_and_retrieve() {
        let tracker = UsageTracker::open_in_memory();
        tracker.record("apps", "firefox");
        let bonus = tracker.get_bonus("apps", "firefox");
        // 1 use just now → bonus of 1
        assert_eq!(bonus, 1);
    }

    #[test]
    fn multiple_uses_increase_bonus() {
        let tracker = UsageTracker::open_in_memory();
        for _ in 0..10 {
            tracker.record("apps", "firefox");
        }
        let bonus = tracker.get_bonus("apps", "firefox");
        assert_eq!(bonus, 10);
    }

    #[test]
    fn bonus_capped_at_max() {
        let tracker = UsageTracker::open_in_memory();
        for _ in 0..100 {
            tracker.record("apps", "firefox");
        }
        let bonus = tracker.get_bonus("apps", "firefox");
        assert_eq!(bonus, MAX_USAGE_BONUS);
    }

    #[test]
    fn different_items_tracked_separately() {
        let tracker = UsageTracker::open_in_memory();
        for _ in 0..5 {
            tracker.record("apps", "firefox");
        }
        tracker.record("apps", "chrome");
        assert_eq!(tracker.get_bonus("apps", "firefox"), 5);
        assert_eq!(tracker.get_bonus("apps", "chrome"), 1);
    }

    #[test]
    fn different_plugins_tracked_separately() {
        let tracker = UsageTracker::open_in_memory();
        for _ in 0..5 {
            tracker.record("apps", "firefox");
        }
        assert_eq!(tracker.get_bonus("apps", "firefox"), 5);
        assert_eq!(tracker.get_bonus("commands", "firefox"), 0);
    }

    #[test]
    fn decay_reduces_bonus() {
        // Test the pure computation function with a stale timestamp
        let one_week_ago = now_secs() - 7 * 86400;
        let bonus = compute_bonus(10, one_week_ago);
        // After 7 days (half of 14-day half-life): 10 * 0.707 ≈ 7
        assert!(bonus < 10, "bonus should decay: got {bonus}");
        assert!(bonus > 0, "bonus should still be positive: got {bonus}");

        let one_month_ago = now_secs() - 30 * 86400;
        let bonus_old = compute_bonus(10, one_month_ago);
        assert!(
            bonus_old < bonus,
            "older usage should have lower bonus: {bonus_old} vs {bonus}"
        );
    }
}
