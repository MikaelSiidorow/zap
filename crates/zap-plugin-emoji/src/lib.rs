mod data;
pub mod pins;

use data::EMOJIS;
use zap_core::{
    fuzzy_match, Action, Capability, KeyboardHint, Plugin, PluginMeta, PluginResult, ViewMode,
};

const MAX_GRID_RESULTS: usize = 256;

pub struct EmojiPlugin;

impl EmojiPlugin {
    pub fn new() -> Self {
        Self
    }

    fn make_result(emoji: &data::Emoji, is_pinned: bool) -> PluginResult {
        let mut r = PluginResult::new(
            "emoji",
            emoji.name,
            format!("{} {}", emoji.character, emoji.name),
        )
        .subtitle(emoji.category)
        .action(Action::Copy {
            content: emoji.character.to_string(),
        });
        if is_pinned {
            r = r.pinned();
        }
        r
    }
}

impl Default for EmojiPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for EmojiPlugin {
    fn meta(&self) -> PluginMeta {
        PluginMeta::new("emoji", "Emoji Picker")
            .description("Search and copy emojis")
            .example(":thumbs up")
            .prefix(":")
            .view(ViewMode::Grid { columns: 8 })
            .max_results(MAX_GRID_RESULTS)
            .capabilities(vec![Capability::Pin])
    }

    fn search(&self, query: &str) -> Vec<PluginResult> {
        let query = query.trim();
        let pinned_set = pins::load_pins();

        if query.is_empty() {
            let mut results = Vec::new();

            for emoji in EMOJIS {
                if pinned_set.contains(emoji.name) {
                    results.push(Self::make_result(emoji, true));
                }
            }

            for emoji in EMOJIS {
                if results.len() >= MAX_GRID_RESULTS {
                    break;
                }
                if !pinned_set.contains(emoji.name) {
                    results.push(Self::make_result(emoji, false));
                }
            }

            return results;
        }

        let mut results: Vec<PluginResult> = EMOJIS
            .iter()
            .filter_map(|emoji| {
                let search_text = format!("{} {}", emoji.name, emoji.keywords);
                let m = fuzzy_match(query, &search_text)?;
                let is_pinned = pinned_set.contains(emoji.name);
                Some(Self::make_result(emoji, is_pinned).score(m.score))
            })
            .collect();

        results.sort_by(|a, b| b.score.cmp(&a.score));
        results.truncate(MAX_GRID_RESULTS);
        results
    }

    fn execute(&self, _result_id: &str) -> anyhow::Result<()> {
        Ok(())
    }

    fn toggle_pin(&self, result_id: &str) -> anyhow::Result<bool> {
        Ok(pins::toggle_pin(result_id))
    }

    fn hints(&self) -> Vec<KeyboardHint> {
        vec![
            KeyboardHint {
                key: "Enter".into(),
                label: "Copy to clipboard".into(),
            },
            KeyboardHint {
                key: "Ctrl+P".into(),
                label: "Pin".into(),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plugin() -> EmojiPlugin {
        EmojiPlugin::new()
    }

    #[test]
    fn empty_query_returns_popular() {
        let p = plugin();
        let results = p.search("");
        assert!(!results.is_empty());
        assert!(results.len() <= MAX_GRID_RESULTS);
    }

    #[test]
    fn search_by_name() {
        let p = plugin();
        let results = p.search("thumbs up");
        assert!(!results.is_empty());
        assert!(results[0].title.contains("👍"));
    }

    #[test]
    fn search_by_keyword() {
        let p = plugin();
        let results = p.search("laugh");
        assert!(!results.is_empty());
        let titles: Vec<&str> = results.iter().map(|r| r.title.as_str()).collect();
        assert!(
            titles
                .iter()
                .any(|t| t.contains("😂") || t.contains("😆") || t.contains("🤣")),
            "expected a laughing emoji, got: {titles:?}"
        );
    }

    #[test]
    fn result_action_is_copy() {
        let p = plugin();
        let results = p.search("fire");
        assert!(!results.is_empty());
        match &results[0].action {
            Action::Copy { content } => assert!(!content.is_empty()),
            _ => panic!("expected Action::Copy"),
        }
    }

    #[test]
    fn plugin_metadata() {
        let p = plugin();
        let meta = p.meta();
        assert_eq!(meta.id, "emoji");
        assert_eq!(meta.prefix, Some(":"));
        assert!(matches!(meta.view, ViewMode::Grid { columns: 8 }));
    }

    #[test]
    fn results_capped() {
        let p = plugin();
        let results = p.search("face");
        assert!(results.len() <= MAX_GRID_RESULTS);
    }

    #[test]
    fn heart_search() {
        let p = plugin();
        let results = p.search("heart");
        assert!(!results.is_empty());
        let titles: Vec<&str> = results.iter().map(|r| r.title.as_str()).collect();
        assert!(
            titles.iter().any(|t| t.contains("❤")),
            "expected a heart emoji, got: {titles:?}"
        );
    }

    #[test]
    fn flag_search() {
        let p = plugin();
        let results = p.search("finland");
        assert!(!results.is_empty());
        assert!(results[0].title.contains("🇫🇮"));
    }
}
