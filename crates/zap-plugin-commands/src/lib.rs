mod commands;
mod platform;

use commands::COMMANDS;
use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use nucleo_matcher::{Config, Matcher, Utf32String};
use platform::{create_platform, Platform};
use zap_core::{Action, KeyboardHint, Plugin, PluginResult};

const PLUGIN_ID: &str = "commands";
const MAX_SCORE: u32 = 80;

pub struct CommandsPlugin {
    platform: Box<dyn Platform>,
}

impl CommandsPlugin {
    pub fn new() -> Self {
        Self {
            platform: create_platform(),
        }
    }
}

impl Default for CommandsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for CommandsPlugin {
    fn id(&self) -> &str {
        PLUGIN_ID
    }

    fn name(&self) -> &str {
        "System Commands"
    }

    fn description(&self) -> &str {
        "Lock screen, sleep, restart, shutdown, and more"
    }

    fn example(&self) -> Option<&str> {
        Some("lock")
    }

    fn search(&self, query: &str) -> Vec<PluginResult> {
        if query.is_empty() {
            return vec![];
        }

        let mut matcher = Matcher::new(Config::DEFAULT);
        let atom = Atom::new(
            query,
            CaseMatching::Smart,
            Normalization::Smart,
            AtomKind::Fuzzy,
            false,
        );

        let mut results: Vec<PluginResult> = COMMANDS
            .iter()
            .filter_map(|cmd| {
                let search_text = format!("{} {}", cmd.title, cmd.keywords);
                let haystack = Utf32String::from(search_text.as_str());
                let mut indices = Vec::new();
                let score = atom.indices(haystack.slice(..), &mut matcher, &mut indices)?;

                // Only keep indices within the title (filter out keyword matches)
                let title_len = cmd.title.chars().count() as u32;
                indices.retain(|&i| i < title_len);
                indices.sort_unstable();
                indices.dedup();

                Some(PluginResult {
                    id: cmd.id.to_string(),
                    plugin_id: PLUGIN_ID.to_string(),
                    title: cmd.title.to_string(),
                    subtitle: Some(cmd.subtitle.to_string()),
                    description: None,
                    icon_path: None,
                    score: (score as u32).min(MAX_SCORE),
                    match_indices: indices,
                    action: Action::default(),
                })
            })
            .collect();

        results.sort_by(|a, b| b.score.cmp(&a.score));
        results.truncate(9);
        results
    }

    fn execute(&self, result_id: &str) -> anyhow::Result<()> {
        log::info!("Executing system command: {result_id}");
        self.platform.execute(result_id)
    }

    fn hints(&self) -> Vec<KeyboardHint> {
        vec![KeyboardHint {
            key: "Enter".into(),
            label: "Execute".into(),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plugin() -> CommandsPlugin {
        CommandsPlugin::new()
    }

    #[test]
    fn empty_query_returns_nothing() {
        let p = plugin();
        assert!(p.search("").is_empty());
    }

    #[test]
    fn exact_title_match() {
        let p = plugin();
        let results = p.search("Restart");
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "Restart");
        assert!(results[0].score <= MAX_SCORE);
    }

    #[test]
    fn keyword_match() {
        let p = plugin();
        let results = p.search("suspend");
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "Sleep");
        // Keyword match should have no title highlight indices
        assert!(results[0].match_indices.is_empty());
    }

    #[test]
    fn title_match() {
        let p = plugin();
        let results = p.search("lock");
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "Lock Screen");
    }

    #[test]
    fn score_capped() {
        let p = plugin();
        let results = p.search("Shutdown");
        for r in &results {
            assert!(r.score <= MAX_SCORE, "score {} exceeds cap", r.score);
        }
    }

    #[test]
    fn all_commands_findable() {
        let p = plugin();
        for cmd in COMMANDS {
            let results = p.search(cmd.title);
            assert!(
                !results.is_empty(),
                "command '{}' not found by its title",
                cmd.title
            );
        }
    }

    #[test]
    fn plugin_metadata() {
        let p = plugin();
        assert_eq!(p.id(), "commands");
        assert_eq!(p.name(), "System Commands");
        assert!(p.prefix().is_none());
    }
}
