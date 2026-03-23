mod commands;
mod platform;

use commands::COMMANDS;
use platform::{create_platform, Platform};
use zap_core::{fuzzy_match, KeyboardHint, Plugin, PluginMeta, PluginResult};

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
    fn meta(&self) -> PluginMeta {
        PluginMeta::new("commands", "System Commands")
            .description("Lock screen, sleep, restart, shutdown, and more")
            .example("lock")
            .usage_ranking()
    }

    fn search(&self, query: &str) -> Vec<PluginResult> {
        if query.is_empty() {
            return vec![];
        }

        let mut results: Vec<PluginResult> = COMMANDS
            .iter()
            .filter_map(|cmd| {
                let search_text = format!("{} {}", cmd.title, cmd.keywords);
                let m = fuzzy_match(query, &search_text)?;

                // Only keep indices within the title (filter out keyword matches)
                let title_len = cmd.title.chars().count() as u32;
                let indices: Vec<u32> = m.indices.into_iter().filter(|&i| i < title_len).collect();

                Some(
                    PluginResult::new("commands", cmd.id, cmd.title)
                        .subtitle(cmd.subtitle)
                        .score(m.score.min(MAX_SCORE))
                        .indices(indices),
                )
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
        let meta = p.meta();
        assert_eq!(meta.id, "commands");
        assert_eq!(meta.name, "System Commands");
        assert!(meta.prefix.is_none());
    }
}
