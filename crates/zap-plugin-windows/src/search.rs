use crate::platform::WindowEntry;
use zap_core::{fuzzy_match, PluginResult};

const EMPTY_QUERY_SCORE: u32 = 150;

fn result_from(win: &WindowEntry, plugin_id: &str) -> PluginResult {
    let mut r =
        PluginResult::new(plugin_id, win.window_id.to_string(), &win.title).subtitle(&win.app_name);
    if let Some(icon) = &win.icon_path {
        r = r.icon(icon);
    }
    r
}

pub fn search(query: &str, windows: &[WindowEntry], plugin_id: &str) -> Vec<PluginResult> {
    if query.is_empty() {
        return windows
            .iter()
            .take(9)
            .map(|win| result_from(win, plugin_id).score(EMPTY_QUERY_SCORE))
            .collect();
    }

    let mut results: Vec<PluginResult> = windows
        .iter()
        .filter_map(|win| {
            let search_text = format!("{} {}", win.title, win.app_name);
            let m = fuzzy_match(query, &search_text)?;

            let title_len = win.title.chars().count() as u32;
            let indices: Vec<u32> = m.indices.into_iter().filter(|&i| i < title_len).collect();

            Some(result_from(win, plugin_id).score(m.score).indices(indices))
        })
        .collect();

    results.sort_by(|a, b| b.score.cmp(&a.score));
    results.truncate(9);
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_windows() -> Vec<WindowEntry> {
        vec![
            WindowEntry {
                window_id: 1001,
                title: "main.rs - zap - Visual Studio Code".into(),
                app_name: "Visual Studio Code".into(),
                icon_path: None,
            },
            WindowEntry {
                window_id: 2002,
                title: "GitHub - Mozilla Firefox".into(),
                app_name: "Firefox".into(),
                icon_path: None,
            },
            WindowEntry {
                window_id: 3003,
                title: "Terminal".into(),
                app_name: "Alacritty".into(),
                icon_path: None,
            },
        ]
    }

    #[test]
    fn empty_query_returns_all_windows() {
        let results = search("", &test_windows(), "windows");
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.score == EMPTY_QUERY_SCORE));
    }

    #[test]
    fn matches_window_title() {
        let results = search("GitHub", &test_windows(), "windows");
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "GitHub - Mozilla Firefox");
    }

    #[test]
    fn matches_app_name() {
        let results = search("firefox", &test_windows(), "windows");
        assert!(!results.is_empty());
        assert!(results[0].title.contains("Firefox"));
    }

    #[test]
    fn result_id_is_window_id() {
        let results = search("Firefox", &test_windows(), "windows");
        assert_eq!(results[0].id, "2002");
    }

    #[test]
    fn subtitle_is_app_name() {
        let results = search("Firefox", &test_windows(), "windows");
        assert_eq!(results[0].subtitle.as_deref(), Some("Firefox"));
    }

    #[test]
    fn no_match_returns_empty() {
        let results = search("nonexistent", &test_windows(), "windows");
        assert!(results.is_empty());
    }
}
