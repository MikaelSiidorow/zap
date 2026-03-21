mod engines;

use engines::{default_engines, SearchEngine};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use zap_core::{Action, KeyboardHint, Plugin, PluginResult};

pub struct WebSearchPlugin {
    engines: Vec<SearchEngine>,
    default_keyword: String,
}

impl WebSearchPlugin {
    pub fn new() -> Self {
        Self {
            engines: default_engines(),
            default_keyword: "g".into(),
        }
    }

    fn build_url(template: &str, query: &str) -> String {
        let encoded = utf8_percent_encode(query, NON_ALPHANUMERIC).to_string();
        template.replace("%s", &encoded)
    }

    fn default_engine(&self) -> Option<&SearchEngine> {
        self.engines
            .iter()
            .find(|e| e.keyword == self.default_keyword)
    }
}

impl Default for WebSearchPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for WebSearchPlugin {
    fn id(&self) -> &str {
        "websearch"
    }

    fn name(&self) -> &str {
        "Web Search"
    }

    fn description(&self) -> &str {
        "Search the web with keyword shortcuts"
    }

    fn example(&self) -> Option<&str> {
        Some("g rust programming")
    }

    fn init(&mut self, config: zap_core::serde_json::Value) -> anyhow::Result<()> {
        // Override default keyword
        if let Some(d) = config.get("default").and_then(|v| v.as_str()) {
            self.default_keyword = d.to_string();
        }

        // Merge user engines: override by keyword or append
        if let Some(user_engines) = config.get("engines").and_then(|v| v.as_array()) {
            for val in user_engines {
                if let Ok(engine) = zap_core::serde_json::from_value::<SearchEngine>(val.clone()) {
                    if let Some(existing) = self
                        .engines
                        .iter_mut()
                        .find(|e| e.keyword == engine.keyword)
                    {
                        existing.name = engine.name;
                        existing.url = engine.url;
                    } else {
                        self.engines.push(engine);
                    }
                } else {
                    log::warn!("Invalid engine config entry: {val}");
                }
            }
        }

        Ok(())
    }

    fn search(&self, query: &str) -> Vec<PluginResult> {
        if query.is_empty() {
            return vec![];
        }

        let mut results = Vec::new();

        // Check for "keyword query" pattern
        if let Some((first_word, rest)) = query.split_once(' ') {
            // Keyword match with query text
            if let Some(engine) = self.engines.iter().find(|e| e.keyword == first_word) {
                if rest.is_empty() {
                    // "keyword " (trailing space, no query yet) — placeholder
                    results.push(PluginResult {
                        id: format!("ws-{}", engine.keyword),
                        plugin_id: "websearch".into(),
                        title: format!("Search {} for...", engine.name),
                        subtitle: Some(format!(
                            "Type your search query after '{} '",
                            engine.keyword
                        )),
                        description: None,
                        icon_path: None,
                        score: 100,
                        match_indices: vec![],
                        pinned: false,
                        action: Action::SetQuery {
                            query: format!("{} ", engine.keyword),
                        },
                    });
                } else {
                    // Full keyword + query
                    let url = Self::build_url(&engine.url, rest);
                    results.push(PluginResult {
                        id: format!("ws-{}", engine.keyword),
                        plugin_id: "websearch".into(),
                        title: format!("Search {} for '{}'", engine.name, rest),
                        subtitle: Some(engine.name.clone()),
                        description: None,
                        icon_path: None,
                        score: 100,
                        match_indices: vec![],
                        pinned: false,
                        action: Action::OpenUrl { url },
                    });
                }
                return results;
            }
        }

        // Exact keyword without space — hint to add space
        if let Some(engine) = self.engines.iter().find(|e| e.keyword == query) {
            results.push(PluginResult {
                id: format!("ws-{}", engine.keyword),
                plugin_id: "websearch".into(),
                title: format!("Search {}", engine.name),
                subtitle: Some("Type a space then your query".into()),
                description: None,
                icon_path: None,
                score: 50,
                match_indices: vec![],
                pinned: false,
                action: Action::SetQuery {
                    query: format!("{} ", engine.keyword),
                },
            });
            return results;
        }

        // Partial keyword match (≥2 chars)
        if query.len() >= 2 && !query.contains(' ') {
            let lower = query.to_lowercase();
            for engine in &self.engines {
                if engine.keyword.starts_with(&lower) && engine.keyword != lower {
                    results.push(PluginResult {
                        id: format!("ws-{}", engine.keyword),
                        plugin_id: "websearch".into(),
                        title: format!("{} ({})", engine.name, engine.keyword),
                        subtitle: Some("Web search shortcut".into()),
                        description: None,
                        icon_path: None,
                        score: 5,
                        match_indices: vec![],
                        pinned: false,
                        action: Action::SetQuery {
                            query: format!("{} ", engine.keyword),
                        },
                    });
                }
            }
            if !results.is_empty() {
                return results;
            }
        }

        // Fallback: "Search {default} for {query}"
        if let Some(engine) = self.default_engine() {
            let url = Self::build_url(&engine.url, query);
            results.push(PluginResult {
                id: "ws-fallback".into(),
                plugin_id: "websearch".into(),
                title: format!("Search {} for '{}'", engine.name, query),
                subtitle: Some(engine.name.clone()),
                description: None,
                icon_path: None,
                score: 1,
                match_indices: vec![],
                pinned: false,
                action: Action::OpenUrl { url },
            });
        }

        results
    }

    fn execute(&self, _result_id: &str) -> anyhow::Result<()> {
        // All actions are OpenUrl or SetQuery, handled by the runtime
        Ok(())
    }

    fn hints(&self) -> Vec<KeyboardHint> {
        vec![KeyboardHint {
            key: "Enter".into(),
            label: "Open in browser".into(),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plugin() -> WebSearchPlugin {
        WebSearchPlugin::new()
    }

    #[test]
    fn keyword_with_query() {
        let p = plugin();
        let results = p.search("g rust programming");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Search Google for 'rust programming'");
        assert_eq!(results[0].score, 100);
        match &results[0].action {
            Action::OpenUrl { url } => {
                assert!(url.contains("google.com"));
                assert!(url.contains("rust%20programming"));
            }
            _ => panic!("expected OpenUrl"),
        }
    }

    #[test]
    fn keyword_with_trailing_space() {
        let p = plugin();
        let results = p.search("gh ");
        assert_eq!(results.len(), 1);
        assert!(results[0].title.contains("Search GitHub for..."));
        assert_eq!(results[0].score, 100);
        assert!(matches!(&results[0].action, Action::SetQuery { .. }));
    }

    #[test]
    fn exact_keyword_no_space() {
        let p = plugin();
        let results = p.search("gh");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Search GitHub");
        assert_eq!(results[0].score, 50);
        match &results[0].action {
            Action::SetQuery { query } => assert_eq!(query, "gh "),
            _ => panic!("expected SetQuery"),
        }
    }

    #[test]
    fn partial_keyword() {
        let p = plugin();
        let results = p.search("cra");
        assert_eq!(results.len(), 1);
        assert!(results[0].title.contains("crates.io"));
        assert_eq!(results[0].score, 5);
    }

    #[test]
    fn fallback_search() {
        let p = plugin();
        let results = p.search("hello world");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Search Google for 'hello world'");
        assert_eq!(results[0].score, 1);
        assert!(matches!(&results[0].action, Action::OpenUrl { .. }));
    }

    #[test]
    fn empty_query() {
        let p = plugin();
        assert!(p.search("").is_empty());
    }

    #[test]
    fn url_encoding() {
        let url = WebSearchPlugin::build_url("https://example.com/?q=%s", "hello world!");
        assert_eq!(url, "https://example.com/?q=hello%20world%21");
    }

    #[test]
    fn config_override_default() {
        let mut p = plugin();
        let config = zap_core::serde_json::json!({
            "default": "ddg"
        });
        p.init(config).unwrap();
        let results = p.search("hello");
        assert_eq!(results[0].title, "Search DuckDuckGo for 'hello'");
    }

    #[test]
    fn config_add_custom_engine() {
        let mut p = plugin();
        let config = zap_core::serde_json::json!({
            "engines": [
                {
                    "keyword": "arch",
                    "name": "Arch Wiki",
                    "url": "https://wiki.archlinux.org/index.php?search=%s"
                }
            ]
        });
        p.init(config).unwrap();
        let results = p.search("arch pacman");
        assert_eq!(results.len(), 1);
        assert!(results[0].title.contains("Arch Wiki"));
    }

    #[test]
    fn config_override_builtin_engine() {
        let mut p = plugin();
        let config = zap_core::serde_json::json!({
            "engines": [
                {
                    "keyword": "g",
                    "name": "Custom Google",
                    "url": "https://custom.google.com/search?q=%s"
                }
            ]
        });
        p.init(config).unwrap();
        let results = p.search("g test");
        assert!(results[0].title.contains("Custom Google"));
    }
}
