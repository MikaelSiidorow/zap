mod data;
pub mod pins;

use data::EMOJIS;
use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use nucleo_matcher::{Config, Matcher, Utf32String};
use zap_core::{Action, KeyboardHint, Plugin, PluginResult};

const PLUGIN_ID: &str = "emoji";
const MAX_GRID_RESULTS: usize = 256;

pub struct EmojiPlugin;

impl EmojiPlugin {
    pub fn new() -> Self {
        Self
    }

    fn make_result(emoji: &data::Emoji, pinned: bool) -> PluginResult {
        PluginResult {
            id: emoji.name.to_string(),
            plugin_id: PLUGIN_ID.to_string(),
            title: format!("{} {}", emoji.character, emoji.name),
            subtitle: Some(emoji.category.to_string()),
            description: None,
            icon_path: None,
            score: 0,
            match_indices: vec![],
            pinned,
            action: Action::Copy {
                content: emoji.character.to_string(),
            },
        }
    }
}

impl Default for EmojiPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for EmojiPlugin {
    fn id(&self) -> &str {
        PLUGIN_ID
    }

    fn name(&self) -> &str {
        "Emoji Picker"
    }

    fn description(&self) -> &str {
        "Search and copy emojis"
    }

    fn example(&self) -> Option<&str> {
        Some(":thumbs up")
    }

    fn prefix(&self) -> Option<&str> {
        Some(":")
    }

    fn view(&self) -> &str {
        "grid"
    }

    fn search(&self, query: &str) -> Vec<PluginResult> {
        let query = query.trim();
        let pinned_set = pins::load_pins();

        // Empty query: show pinned first, then popular emojis
        if query.is_empty() {
            let mut results = Vec::new();

            // Pinned emojis first
            for emoji in EMOJIS {
                if pinned_set.contains(emoji.name) {
                    results.push(Self::make_result(emoji, true));
                }
            }

            // Fill remaining slots with non-pinned emojis
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

        let mut matcher = Matcher::new(Config::DEFAULT);
        let atom = Atom::new(
            query,
            CaseMatching::Smart,
            Normalization::Smart,
            AtomKind::Fuzzy,
            false,
        );

        let mut results: Vec<PluginResult> = EMOJIS
            .iter()
            .filter_map(|emoji| {
                let search_text = format!("{} {}", emoji.name, emoji.keywords);
                let haystack = Utf32String::from(search_text.as_str());
                let mut indices = Vec::new();
                let score = atom.indices(haystack.slice(..), &mut matcher, &mut indices)?;

                let is_pinned = pinned_set.contains(emoji.name);
                let mut result = Self::make_result(emoji, is_pinned);
                result.score = score as u32;
                Some(result)
            })
            .collect();

        results.sort_by(|a, b| b.score.cmp(&a.score));
        results.truncate(MAX_GRID_RESULTS);
        results
    }

    fn execute(&self, _result_id: &str) -> anyhow::Result<()> {
        Ok(())
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
        assert_eq!(p.id(), "emoji");
        assert_eq!(p.prefix(), Some(":"));
        assert_eq!(p.view(), "grid");
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
