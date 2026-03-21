use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const CURRENT_VERSION: u32 = 1;

#[derive(Debug, Clone)]
pub struct ClipboardEntry {
    pub id: i64,
    pub content: String,
    pub pinned: bool,
    pub created_at: i64,
    pub last_used: i64,
    pub use_count: i64,
}

pub fn open_db(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    migrate(&conn)?;
    Ok(conn)
}

pub fn open_memory_db() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    migrate(&conn)?;
    Ok(conn)
}

pub fn migrate(conn: &Connection) -> Result<()> {
    let version: u32 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;

    if version < 1 {
        conn.execute_batch(
            "CREATE TABLE clipboard_entries (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                content     TEXT NOT NULL,
                hash        TEXT NOT NULL,
                pinned      INTEGER NOT NULL DEFAULT 0,
                created_at  INTEGER NOT NULL,
                last_used   INTEGER NOT NULL,
                use_count   INTEGER NOT NULL DEFAULT 0
            );
            CREATE UNIQUE INDEX idx_hash ON clipboard_entries(hash);
            CREATE INDEX idx_created ON clipboard_entries(created_at);",
        )?;
    }

    if version < CURRENT_VERSION {
        conn.pragma_update(None, "user_version", CURRENT_VERSION)?;
    }
    Ok(())
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub fn upsert_entry(conn: &Connection, content: &str, hash: &str) -> Result<()> {
    let now = now_unix();
    conn.execute(
        "INSERT INTO clipboard_entries (content, hash, pinned, created_at, last_used, use_count)
         VALUES (?1, ?2, 0, ?3, ?3, 1)
         ON CONFLICT(hash) DO UPDATE SET
             last_used = ?3,
             use_count = use_count + 1",
        params![content, hash, now],
    )?;
    Ok(())
}

pub fn recent_entries(conn: &Connection, limit: usize) -> Result<Vec<ClipboardEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, pinned, created_at, last_used, use_count
         FROM clipboard_entries
         ORDER BY pinned DESC, last_used DESC, id DESC
         LIMIT ?1",
    )?;
    let entries = stmt
        .query_map(params![limit as i64], |row| {
            Ok(ClipboardEntry {
                id: row.get(0)?,
                content: row.get(1)?,
                pinned: row.get::<_, i64>(2)? != 0,
                created_at: row.get(3)?,
                last_used: row.get(4)?,
                use_count: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(entries)
}

pub fn search_candidates(conn: &Connection, limit: usize) -> Result<Vec<ClipboardEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, pinned, created_at, last_used, use_count
         FROM clipboard_entries
         ORDER BY last_used DESC, id DESC
         LIMIT ?1",
    )?;
    let entries = stmt
        .query_map(params![limit as i64], |row| {
            Ok(ClipboardEntry {
                id: row.get(0)?,
                content: row.get(1)?,
                pinned: row.get::<_, i64>(2)? != 0,
                created_at: row.get(3)?,
                last_used: row.get(4)?,
                use_count: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(entries)
}

pub fn delete_entry(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM clipboard_entries WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn toggle_pin(conn: &Connection, id: i64) -> Result<bool> {
    conn.execute(
        "UPDATE clipboard_entries SET pinned = 1 - pinned WHERE id = ?1",
        params![id],
    )?;
    let pinned: bool = conn.query_row(
        "SELECT pinned FROM clipboard_entries WHERE id = ?1",
        params![id],
        |row| row.get::<_, i64>(0).map(|v| v != 0),
    )?;
    Ok(pinned)
}

pub fn enforce_retention(conn: &Connection, max_age_days: u32, max_entries: usize) -> Result<()> {
    let cutoff = now_unix() - (max_age_days as i64 * 86400);
    // Delete unpinned entries older than max_age_days
    conn.execute(
        "DELETE FROM clipboard_entries WHERE pinned = 0 AND created_at < ?1",
        params![cutoff],
    )?;
    // Delete oldest unpinned entries beyond max_entries (pinned entries don't count toward limit)
    conn.execute(
        "DELETE FROM clipboard_entries WHERE pinned = 0 AND id NOT IN (
            SELECT id FROM clipboard_entries WHERE pinned = 0
            ORDER BY last_used DESC, id DESC LIMIT ?1
        )",
        params![max_entries as i64],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_creates_table() {
        let conn = open_memory_db().unwrap();
        let version: u32 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(version, CURRENT_VERSION);
    }

    #[test]
    fn test_upsert_and_recent() {
        let conn = open_memory_db().unwrap();
        upsert_entry(&conn, "hello", "hash1").unwrap();
        upsert_entry(&conn, "world", "hash2").unwrap();

        let entries = recent_entries(&conn, 10).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].content, "world");
        assert_eq!(entries[1].content, "hello");
    }

    #[test]
    fn test_upsert_deduplicates() {
        let conn = open_memory_db().unwrap();
        upsert_entry(&conn, "hello", "hash1").unwrap();
        upsert_entry(&conn, "hello", "hash1").unwrap();

        let entries = recent_entries(&conn, 10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].use_count, 2);
    }

    #[test]
    fn test_delete_entry() {
        let conn = open_memory_db().unwrap();
        upsert_entry(&conn, "hello", "hash1").unwrap();
        let entries = recent_entries(&conn, 10).unwrap();
        let id = entries[0].id;

        delete_entry(&conn, id).unwrap();
        let entries = recent_entries(&conn, 10).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_toggle_pin() {
        let conn = open_memory_db().unwrap();
        upsert_entry(&conn, "hello", "hash1").unwrap();
        let entries = recent_entries(&conn, 10).unwrap();
        let id = entries[0].id;
        assert!(!entries[0].pinned);

        let pinned = toggle_pin(&conn, id).unwrap();
        assert!(pinned);

        let pinned = toggle_pin(&conn, id).unwrap();
        assert!(!pinned);
    }

    #[test]
    fn test_pinned_entries_sort_first() {
        let conn = open_memory_db().unwrap();
        upsert_entry(&conn, "first", "hash1").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        upsert_entry(&conn, "second", "hash2").unwrap();

        // Pin the first (older) entry
        let entries = recent_entries(&conn, 10).unwrap();
        let first_id = entries.iter().find(|e| e.content == "first").unwrap().id;
        toggle_pin(&conn, first_id).unwrap();

        let entries = recent_entries(&conn, 10).unwrap();
        assert_eq!(entries[0].content, "first");
        assert!(entries[0].pinned);
    }

    #[test]
    fn test_enforce_retention_max_entries() {
        let conn = open_memory_db().unwrap();
        for i in 0..5 {
            upsert_entry(&conn, &format!("entry{i}"), &format!("hash{i}")).unwrap();
        }
        enforce_retention(&conn, 365, 3).unwrap();
        let entries = recent_entries(&conn, 10).unwrap();
        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn test_retention_preserves_pinned() {
        let conn = open_memory_db().unwrap();
        for i in 0..5 {
            upsert_entry(&conn, &format!("entry{i}"), &format!("hash{i}")).unwrap();
        }
        // Pin the first entry
        let entries = recent_entries(&conn, 10).unwrap();
        let first_id = entries.last().unwrap().id;
        toggle_pin(&conn, first_id).unwrap();

        enforce_retention(&conn, 365, 2).unwrap();
        let entries = recent_entries(&conn, 10).unwrap();
        // Pinned entry + 2 most recent unpinned = 3
        assert_eq!(entries.len(), 3);
        assert!(entries[0].pinned);
    }
}
