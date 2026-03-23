use crate::store::{recent_entries, search_candidates, ClipboardEntry};
use rusqlite::Connection;
use time::OffsetDateTime;
use zap_core::{fuzzy_match, Action, PluginResult};

fn first_line(content: &str, max_len: usize) -> String {
    let line = content.lines().next().unwrap_or(content);
    if line.len() > max_len {
        format!("{}…", &line[..max_len])
    } else {
        line.to_string()
    }
}

fn relative_time(epoch_secs: i64) -> String {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let diff = now - epoch_secs;

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else if diff < 172800 {
        "Yesterday".to_string()
    } else if diff < 604800 {
        format!("{}d ago", diff / 86400)
    } else {
        let dt =
            OffsetDateTime::from_unix_timestamp(epoch_secs).unwrap_or(OffsetDateTime::UNIX_EPOCH);
        format!("{} {}", dt.month(), dt.day())
    }
}

fn subtitle_for(entry: &ClipboardEntry) -> String {
    let time = relative_time(entry.last_used);
    if entry.pinned {
        format!("Pinned · {time}")
    } else {
        time
    }
}

fn entry_to_result(entry: &ClipboardEntry, plugin_id: &str) -> PluginResult {
    let title = first_line(&entry.content, 80);
    let subtitle = subtitle_for(entry);

    if entry.content_type == "image" {
        let blob_path = entry.blob_path.clone().unwrap_or_default();
        let mut r = PluginResult::new(plugin_id, entry.id.to_string(), title)
            .subtitle(subtitle)
            .action(Action::PasteImage {
                path: blob_path.clone(),
            });
        if entry.pinned {
            r = r.pinned();
        }
        if !blob_path.is_empty() {
            r = r.icon(blob_path);
        }
        r
    } else {
        let mut r = PluginResult::new(plugin_id, entry.id.to_string(), title)
            .subtitle(subtitle)
            .action(Action::Paste {
                content: entry.content.clone(),
            });
        if entry.pinned {
            r = r.pinned();
        }
        r
    }
}

pub fn search(conn: &Connection, query: &str, plugin_id: &str) -> Vec<PluginResult> {
    if query.is_empty() {
        let entries = recent_entries(conn, 9).unwrap_or_default();
        return entries
            .iter()
            .map(|e| entry_to_result(e, plugin_id))
            .collect();
    }

    let candidates = search_candidates(conn, 200).unwrap_or_default();
    if candidates.is_empty() {
        return vec![];
    }

    let mut scored: Vec<(u32, &ClipboardEntry, Vec<u32>)> = candidates
        .iter()
        .filter_map(|entry| {
            let haystack_text = first_line(&entry.content, 200);
            let m = fuzzy_match(query, &haystack_text)?;
            Some((m.score, entry, m.indices))
        })
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.truncate(9);

    scored
        .into_iter()
        .map(|(score, entry, indices)| {
            entry_to_result(entry, plugin_id)
                .score(score)
                .indices(indices)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{open_memory_db, toggle_pin, upsert_entry};

    #[test]
    fn test_first_line() {
        assert_eq!(first_line("hello\nworld", 80), "hello");
        assert_eq!(first_line("short", 80), "short");
        assert_eq!(first_line("abcdef", 3), "abc…");
    }

    #[test]
    fn test_empty_query_returns_recent() {
        let conn = open_memory_db().unwrap();
        upsert_entry(&conn, "alpha", "h1").unwrap();
        upsert_entry(&conn, "beta", "h2").unwrap();

        let results = search(&conn, "", "clipboard");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "beta");
    }

    #[test]
    fn test_fuzzy_search() {
        let conn = open_memory_db().unwrap();
        upsert_entry(&conn, "hello world", "h1").unwrap();
        upsert_entry(&conn, "goodbye world", "h2").unwrap();
        upsert_entry(&conn, "something else", "h3").unwrap();

        let results = search(&conn, "hello", "clipboard");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "hello world");
    }

    #[test]
    fn test_pinned_show_in_subtitle() {
        let conn = open_memory_db().unwrap();
        upsert_entry(&conn, "pinned item", "h1").unwrap();
        let entries = crate::store::recent_entries(&conn, 10).unwrap();
        toggle_pin(&conn, entries[0].id).unwrap();

        let results = search(&conn, "", "clipboard");
        assert!(results[0].subtitle.as_ref().unwrap().starts_with("Pinned"));
    }

    #[test]
    fn test_paste_action() {
        let conn = open_memory_db().unwrap();
        upsert_entry(&conn, "paste me", "h1").unwrap();

        let results = search(&conn, "", "clipboard");
        match &results[0].action {
            Action::Paste { content } => assert_eq!(content, "paste me"),
            _ => panic!("expected Paste action"),
        }
    }

    #[test]
    fn test_image_entry_returns_paste_image_action() {
        let conn = open_memory_db().unwrap();
        crate::store::upsert_image_entry(&conn, "Image (800x600)", "imghash1", "/tmp/blob.png")
            .unwrap();

        let results = search(&conn, "", "clipboard");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Image (800x600)");
        assert_eq!(results[0].icon_path.as_deref(), Some("/tmp/blob.png"));
        match &results[0].action {
            Action::PasteImage { path } => assert_eq!(path, "/tmp/blob.png"),
            _ => panic!("expected PasteImage action"),
        }
    }
}
