mod fuzzy;
mod usage;

pub use fuzzy::{fuzzy_match, FuzzyMatch};
use serde::Serialize;
pub use serde_json;
use usage::UsageTracker;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Clone, Serialize, Default, specta::Type)]
#[serde(tag = "type")]
pub enum Action {
    #[default]
    Open,
    Copy {
        content: String,
    },
    OpenUrl {
        url: String,
    },
    SetQuery {
        query: String,
    },
    Paste {
        content: String,
    },
    PasteImage {
        path: String,
    },
}

#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(tag = "type")]
pub enum ViewMode {
    List,
    Grid { columns: u8 },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, specta::Type)]
pub enum Capability {
    Delete,
    Pin,
    Copy,
}

#[derive(Clone, Serialize, specta::Type)]
pub struct KeyboardHint {
    pub key: String,
    pub label: String,
}

#[derive(Clone, Serialize, specta::Type)]
pub struct PluginResult {
    pub id: String,
    pub plugin_id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub icon_path: Option<String>,
    pub score: u32,
    pub match_indices: Vec<u32>,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub action: Action,
}

impl PluginResult {
    pub fn new(plugin_id: &str, id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            plugin_id: plugin_id.to_string(),
            title: title.into(),
            subtitle: None,
            description: None,
            icon_path: None,
            score: 0,
            match_indices: vec![],
            pinned: false,
            action: Action::default(),
        }
    }

    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.subtitle = Some(s.into());
        self
    }

    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }

    pub fn icon(mut self, path: impl Into<String>) -> Self {
        self.icon_path = Some(path.into());
        self
    }

    pub fn score(mut self, s: u32) -> Self {
        self.score = s;
        self
    }

    pub fn indices(mut self, i: Vec<u32>) -> Self {
        self.match_indices = i;
        self
    }

    pub fn pinned(mut self) -> Self {
        self.pinned = true;
        self
    }

    pub fn action(mut self, a: Action) -> Self {
        self.action = a;
        self
    }
}

#[derive(Clone, Serialize, specta::Type)]
pub struct SearchResponse {
    pub results: Vec<PluginResult>,
    pub view: ViewMode,
    pub capabilities: Vec<Capability>,
}

// ---------------------------------------------------------------------------
// Plugin metadata & trait
// ---------------------------------------------------------------------------

pub struct PluginMeta {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub example: Option<&'static str>,
    pub prefix: Option<&'static str>,
    pub view: ViewMode,
    pub usage_ranking: bool,
    pub max_results: usize,
    pub capabilities: Vec<Capability>,
}

impl PluginMeta {
    pub fn new(id: &'static str, name: &'static str) -> Self {
        Self {
            id,
            name,
            description: "",
            example: None,
            prefix: None,
            view: ViewMode::List,
            usage_ranking: false,
            max_results: 9,
            capabilities: vec![],
        }
    }

    pub fn description(mut self, d: &'static str) -> Self {
        self.description = d;
        self
    }

    pub fn example(mut self, e: &'static str) -> Self {
        self.example = Some(e);
        self
    }

    pub fn prefix(mut self, p: &'static str) -> Self {
        self.prefix = Some(p);
        self
    }

    pub fn view(mut self, v: ViewMode) -> Self {
        self.view = v;
        self
    }

    pub fn usage_ranking(mut self) -> Self {
        self.usage_ranking = true;
        self
    }

    pub fn max_results(mut self, n: usize) -> Self {
        self.max_results = n;
        self
    }

    pub fn capabilities(mut self, caps: Vec<Capability>) -> Self {
        self.capabilities = caps;
        self
    }
}

pub trait Plugin: Send + Sync {
    fn meta(&self) -> PluginMeta;
    fn init(&mut self, _config: serde_json::Value) -> anyhow::Result<()> {
        Ok(())
    }
    fn search(&self, query: &str) -> Vec<PluginResult>;
    fn execute(&self, result_id: &str) -> anyhow::Result<()>;
    fn refresh(&self) {}
    fn hints(&self) -> Vec<KeyboardHint> {
        vec![]
    }
    fn delete(&self, _result_id: &str) -> anyhow::Result<()> {
        anyhow::bail!("not supported")
    }
    fn toggle_pin(&self, _result_id: &str) -> anyhow::Result<bool> {
        anyhow::bail!("not supported")
    }
}

// ---------------------------------------------------------------------------
// Plugin host
// ---------------------------------------------------------------------------

const DEFAULT_MAX_RESULTS: usize = 9;

pub struct PluginHost {
    plugins: Vec<Box<dyn Plugin>>,
    usage: Option<UsageTracker>,
}

impl PluginHost {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            usage: UsageTracker::open_default(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn init_all(
        &mut self,
        config: &std::collections::HashMap<String, serde_json::Value>,
    ) -> anyhow::Result<()> {
        for plugin in &mut self.plugins {
            let section = config
                .get(plugin.meta().id)
                .cloned()
                .unwrap_or(serde_json::Value::Object(Default::default()));
            plugin.init(section)?;
        }
        Ok(())
    }

    pub fn search(&self, query: &str) -> SearchResponse {
        // Built-in "?" prefix → list installed plugins
        if let Some(filter) = query.strip_prefix('?') {
            return SearchResponse {
                results: self.help_results(filter.trim()),
                view: ViewMode::List,
                capabilities: vec![],
            };
        }

        // Check for prefix match → exclusive routing
        if let Some(plugin) = self
            .plugins
            .iter()
            .find(|p| p.meta().prefix.is_some_and(|pfx| query.starts_with(pfx)))
        {
            let meta = plugin.meta();
            let sub_query = &query[meta.prefix.unwrap().len()..];
            let mut results = plugin.search(sub_query);
            results.truncate(meta.max_results);
            return SearchResponse {
                results,
                view: meta.view,
                capabilities: meta.capabilities,
            };
        }

        // No prefix → fan out to non-prefixed plugins, merge by score
        let mut results: Vec<PluginResult> = self
            .plugins
            .iter()
            .filter(|p| p.meta().prefix.is_none())
            .flat_map(|p| p.search(query))
            .collect();

        // Boost scores based on usage frequency
        if let Some(tracker) = &self.usage {
            for result in &mut results {
                if self
                    .plugins
                    .iter()
                    .any(|p| p.meta().id == result.plugin_id && p.meta().usage_ranking)
                {
                    result.score = result
                        .score
                        .saturating_add(tracker.get_bonus(&result.plugin_id, &result.id));
                }
            }
        }

        results.sort_by(|a, b| b.score.cmp(&a.score));
        results.truncate(DEFAULT_MAX_RESULTS);
        SearchResponse {
            results,
            view: ViewMode::List,
            capabilities: vec![],
        }
    }

    fn help_results(&self, filter: &str) -> Vec<PluginResult> {
        let filter_lower = filter.to_lowercase();
        self.plugins
            .iter()
            .filter(|p| {
                let meta = p.meta();
                filter.is_empty()
                    || meta.name.to_lowercase().contains(&filter_lower)
                    || meta.id.to_lowercase().contains(&filter_lower)
            })
            .map(|p| {
                let meta = p.meta();
                let subtitle = meta.example.map(String::from);
                let desc = meta.description;
                let description = if desc.is_empty() {
                    None
                } else {
                    Some(desc.to_string())
                };
                let action = if let Some(prefix) = meta.prefix {
                    Action::SetQuery {
                        query: prefix.to_string(),
                    }
                } else if let Some(example) = meta.example {
                    Action::SetQuery {
                        query: example.to_string(),
                    }
                } else {
                    Action::default()
                };
                PluginResult::new("_help", meta.id, meta.name)
                    .subtitle(subtitle.unwrap_or_default())
                    .action(action)
                    .description(description.unwrap_or_default())
            })
            .collect()
    }

    pub fn execute(&self, plugin_id: &str, result_id: &str) -> anyhow::Result<()> {
        let plugin = self
            .plugins
            .iter()
            .find(|p| p.meta().id == plugin_id)
            .ok_or_else(|| anyhow::anyhow!("plugin '{}' not found", plugin_id))?;
        plugin.execute(result_id)?;

        // Record usage after successful execution
        if let Some(tracker) = &self.usage {
            if plugin.meta().usage_ranking {
                tracker.record(plugin_id, result_id);
            }
        }

        Ok(())
    }

    pub fn delete(&self, plugin_id: &str, result_id: &str) -> anyhow::Result<()> {
        let plugin = self
            .plugins
            .iter()
            .find(|p| p.meta().id == plugin_id)
            .ok_or_else(|| anyhow::anyhow!("plugin '{}' not found", plugin_id))?;
        plugin.delete(result_id)
    }

    pub fn toggle_pin(&self, plugin_id: &str, result_id: &str) -> anyhow::Result<bool> {
        let plugin = self
            .plugins
            .iter()
            .find(|p| p.meta().id == plugin_id)
            .ok_or_else(|| anyhow::anyhow!("plugin '{}' not found", plugin_id))?;
        plugin.toggle_pin(result_id)
    }

    pub fn plugin_hints(&self, plugin_id: &str) -> Vec<KeyboardHint> {
        self.plugins
            .iter()
            .find(|p| p.meta().id == plugin_id)
            .map(|p| p.hints())
            .unwrap_or_default()
    }

    pub fn refresh_all(&self) {
        for plugin in &self.plugins {
            plugin.refresh();
        }
    }
}

impl Default for PluginHost {
    fn default() -> Self {
        Self::new()
    }
}
