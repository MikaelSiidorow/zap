use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const CURRENT_VERSION: u32 = 2;

#[derive(Debug, Clone)]
pub struct ClipboardEntry {
    pub id: i64,
    pub content: String,
    pub content_type: String,
    pub blob_path: Option<String>,
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

    if version < 2 {
        // Add content_type and blob_path columns for image support
        if version >= 1 {
            // Only ALTER if table already existed from v1
            conn.execute_batch(
                "ALTER TABLE clipboard_entries ADD COLUMN content_type TEXT NOT NULL DEFAULT 'text';
                 ALTER TABLE clipboard_entries ADD COLUMN blob_path TEXT;",
            )?;
        } else {
            // Fresh install: columns already part of v1 CREATE above won't exist,
            // so add them now (table was just created in v<1 block above)
            conn.execute_batch(
                "ALTER TABLE clipboard_entries ADD COLUMN content_type TEXT NOT NULL DEFAULT 'text';
                 ALTER TABLE clipboard_entries ADD COLUMN blob_path TEXT;",
            )?;
        }
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
        "INSERT INTO clipboard_entries (content, hash, pinned, created_at, last_used, use_count, content_type)
         VALUES (?1, ?2, 0, ?3, ?3, 1, 'text')
         ON CONFLICT(hash) DO UPDATE SET
             last_used = ?3,
             use_count = use_count + 1",
        params![content, hash, now],
    )?;
    Ok(())
}

pub fn upsert_image_entry(
    conn: &Connection,
    content: &str,
    hash: &str,
    blob_path: &str,
) -> Result<()> {
    let now = now_unix();
    conn.execute(
        "INSERT INTO clipboard_entries (content, hash, pinned, created_at, last_used, use_count, content_type, blob_path)
         VALUES (?1, ?2, 0, ?3, ?3, 1, 'image', ?4)
         ON CONFLICT(hash) DO UPDATE SET
             last_used = ?3,
             use_count = use_count + 1",
        params![content, hash, now, blob_path],
    )?;
    Ok(())
}

fn map_entry(row: &rusqlite::Row) -> rusqlite::Result<ClipboardEntry> {
    Ok(ClipboardEntry {
        id: row.get(0)?,
        content: row.get(1)?,
        pinned: row.get::<_, i64>(2)? != 0,
        created_at: row.get(3)?,
        last_used: row.get(4)?,
        use_count: row.get(5)?,
        content_type: row.get(6)?,
        blob_path: row.get(7)?,
    })
}

pub fn recent_entries(conn: &Connection, limit: usize) -> Result<Vec<ClipboardEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, pinned, created_at, last_used, use_count, content_type, blob_path
         FROM clipboard_entries
         ORDER BY pinned DESC, last_used DESC, id DESC
         LIMIT ?1",
    )?;
    let entries = stmt
        .query_map(params![limit as i64], map_entry)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(entries)
}

pub fn search_candidates(conn: &Connection, limit: usize) -> Result<Vec<ClipboardEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, pinned, created_at, last_used, use_count, content_type, blob_path
         FROM clipboard_entries
         ORDER BY last_used DESC, id DESC
         LIMIT ?1",
    )?;
    let entries = stmt
        .query_map(params![limit as i64], map_entry)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(entries)
}

/// Deletes an entry and returns its blob_path if it had one (so caller can delete the file).
pub fn delete_entry(conn: &Connection, id: i64) -> Result<Option<String>> {
    let blob_path: Option<String> = conn
        .query_row(
            "SELECT blob_path FROM clipboard_entries WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .ok()
        .flatten();
    conn.execute("DELETE FROM clipboard_entries WHERE id = ?1", params![id])?;
    Ok(blob_path)
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

/// Enforces retention policy. Returns blob_paths of deleted entries (for file cleanup).
pub fn enforce_retention(
    conn: &Connection,
    max_age_days: u32,
    max_entries: usize,
) -> Result<Vec<String>> {
    let cutoff = now_unix() - (max_age_days as i64 * 86400);

    // Collect blob_paths of entries that will be deleted
    let mut stmt = conn.prepare(
        "SELECT blob_path FROM clipboard_entries
         WHERE pinned = 0 AND created_at < ?1 AND blob_path IS NOT NULL",
    )?;
    let mut blob_paths: Vec<String> = stmt
        .query_map(params![cutoff], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();

    conn.execute(
        "DELETE FROM clipboard_entries WHERE pinned = 0 AND created_at < ?1",
        params![cutoff],
    )?;

    // Collect blob_paths of entries beyond max_entries
    let mut stmt2 = conn.prepare(
        "SELECT blob_path FROM clipboard_entries
         WHERE pinned = 0 AND blob_path IS NOT NULL AND id NOT IN (
             SELECT id FROM clipboard_entries WHERE pinned = 0
             ORDER BY last_used DESC, id DESC LIMIT ?1
         )",
    )?;
    let overflow_paths: Vec<String> = stmt2
        .query_map(params![max_entries as i64], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();
    blob_paths.extend(overflow_paths);

    conn.execute(
        "DELETE FROM clipboard_entries WHERE pinned = 0 AND id NOT IN (
            SELECT id FROM clipboard_entries WHERE pinned = 0
            ORDER BY last_used DESC, id DESC LIMIT ?1
        )",
        params![max_entries as i64],
    )?;

    Ok(blob_paths)
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
        assert_eq!(entries[0].content_type, "text");
        assert!(entries[0].blob_path.is_none());
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
    fn test_upsert_image_entry() {
        let conn = open_memory_db().unwrap();
        upsert_image_entry(&conn, "Image (800x600)", "imghash1", "/tmp/blob.png").unwrap();

        let entries = recent_entries(&conn, 10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content_type, "image");
        assert_eq!(entries[0].blob_path.as_deref(), Some("/tmp/blob.png"));
        assert_eq!(entries[0].content, "Image (800x600)");
    }

    #[test]
    fn test_delete_entry_returns_blob_path() {
        let conn = open_memory_db().unwrap();
        upsert_image_entry(&conn, "Image (800x600)", "imghash1", "/tmp/blob.png").unwrap();
        let entries = recent_entries(&conn, 10).unwrap();
        let id = entries[0].id;

        let blob_path = delete_entry(&conn, id).unwrap();
        assert_eq!(blob_path.as_deref(), Some("/tmp/blob.png"));

        let entries = recent_entries(&conn, 10).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_delete_text_entry_returns_none() {
        let conn = open_memory_db().unwrap();
        upsert_entry(&conn, "hello", "hash1").unwrap();
        let entries = recent_entries(&conn, 10).unwrap();
        let id = entries[0].id;

        let blob_path = delete_entry(&conn, id).unwrap();
        assert!(blob_path.is_none());
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
        let blob_paths = enforce_retention(&conn, 365, 3).unwrap();
        assert!(blob_paths.is_empty()); // text entries have no blobs
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

        let blob_paths = enforce_retention(&conn, 365, 2).unwrap();
        assert!(blob_paths.is_empty());
        let entries = recent_entries(&conn, 10).unwrap();
        // Pinned entry + 2 most recent unpinned = 3
        assert_eq!(entries.len(), 3);
        assert!(entries[0].pinned);
    }

    #[test]
    fn test_retention_returns_image_blob_paths() {
        let conn = open_memory_db().unwrap();
        upsert_image_entry(&conn, "Image (100x100)", "img1", "/tmp/a.png").unwrap();
        upsert_image_entry(&conn, "Image (200x200)", "img2", "/tmp/b.png").unwrap();
        upsert_image_entry(&conn, "Image (300x300)", "img3", "/tmp/c.png").unwrap();

        let blob_paths = enforce_retention(&conn, 365, 1).unwrap();
        // 2 entries should be deleted, returning their blob paths
        assert_eq!(blob_paths.len(), 2);
    }
}
