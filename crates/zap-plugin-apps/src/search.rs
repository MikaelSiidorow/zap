use crate::platform::AppEntry;
use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use nucleo_matcher::{Config, Matcher, Utf32String};
use zap_core::{Action, PluginResult};

pub fn search(query: &str, apps: &[AppEntry], plugin_id: &str) -> Vec<PluginResult> {
    if query.is_empty() {
        return apps
            .iter()
            .take(9)
            .map(|app| PluginResult {
                id: app.id.clone(),
                plugin_id: plugin_id.to_string(),
                title: app.name.to_string(),
                subtitle: app.category.clone(),
                icon_path: app.icon_path.clone(),
                score: 0,
                match_indices: vec![],
                action: Action::default(),
            })
            .collect();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let atom = Atom::new(
        query,
        CaseMatching::Smart,
        Normalization::Smart,
        AtomKind::Fuzzy,
        false,
    );

    let mut results: Vec<PluginResult> = apps
        .iter()
        .filter_map(|app| {
            let haystack = Utf32String::from(app.name.as_str());
            let mut indices = Vec::new();
            let score = atom.indices(haystack.slice(..), &mut matcher, &mut indices)?;
            indices.sort_unstable();
            indices.dedup();
            Some(PluginResult {
                id: app.id.clone(),
                plugin_id: plugin_id.to_string(),
                title: app.name.to_string(),
                subtitle: app.category.clone(),
                icon_path: app.icon_path.clone(),
                score: score as u32,
                match_indices: indices,
                action: Action::default(),
            })
        })
        .collect();

    results.sort_by(|a, b| b.score.cmp(&a.score));
    results.truncate(9);
    results
}
