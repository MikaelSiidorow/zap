use serde::Serialize;

/// Typed action declared on a result. The runtime handles execution and feedback,
/// inspired by Raycast's action model (e.g. Action.CopyToClipboard).
#[derive(Clone, Serialize, Default)]
#[serde(tag = "type")]
pub enum Action {
    /// Plugin handles execution via execute() callback.
    #[default]
    Open,
    /// Copy text to clipboard. Runtime shows "Copied to clipboard" feedback.
    Copy { content: String },
    /// Open a URL in the default browser.
    OpenUrl { url: String },
}

#[derive(Clone, Serialize)]
pub struct PluginResult {
    pub id: String,
    pub plugin_id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub icon_path: Option<String>,
    pub score: u32,
    pub match_indices: Vec<u32>,
    #[serde(default)]
    pub action: Action,
}

pub trait Plugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn prefix(&self) -> Option<&str> {
        None
    }
    fn init(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
    fn search(&self, query: &str) -> Vec<PluginResult>;
    /// Called only for results with Action::Open.
    fn execute(&self, result_id: &str) -> anyhow::Result<()>;
    fn refresh(&self) {}
}

pub struct PluginHost {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginHost {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn init_all(&mut self) -> anyhow::Result<()> {
        for plugin in &mut self.plugins {
            plugin.init()?;
        }
        Ok(())
    }

    pub fn search(&self, query: &str) -> Vec<PluginResult> {
        // Check for prefix match → exclusive routing
        if let Some(plugin) = self
            .plugins
            .iter()
            .find(|p| p.prefix().is_some_and(|pfx| query.starts_with(pfx)))
        {
            let prefix = plugin.prefix().unwrap();
            let sub_query = &query[prefix.len()..];
            return plugin.search(sub_query);
        }

        // No prefix → fan out to all plugins, merge by score
        let mut results: Vec<PluginResult> = self
            .plugins
            .iter()
            .flat_map(|p| p.search(query))
            .collect();

        results.sort_by(|a, b| b.score.cmp(&a.score));
        results.truncate(9);
        results
    }

    pub fn execute(&self, plugin_id: &str, result_id: &str) -> anyhow::Result<()> {
        let plugin = self
            .plugins
            .iter()
            .find(|p| p.id() == plugin_id)
            .ok_or_else(|| anyhow::anyhow!("plugin '{}' not found", plugin_id))?;
        plugin.execute(result_id)
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
