use crate::platform::AppEntry;
use zap_core::{fuzzy_match, Action, PluginResult};

pub fn search(query: &str, apps: &[AppEntry], plugin_id: &str) -> Vec<PluginResult> {
    if query.is_empty() {
        return apps
            .iter()
            .take(9)
            .map(|app| {
                let mut r = PluginResult::new(plugin_id, app.id.to_string(), app.name.to_string());
                if let Some(cat) = &app.category {
                    r = r.subtitle(cat.as_str());
                }
                if let Some(icon) = &app.icon_path {
                    r = r.icon(icon.as_str());
                }
                r
            })
            .collect();
    }

    let mut results: Vec<PluginResult> = apps
        .iter()
        .filter_map(|app| {
            let m = fuzzy_match(query, &app.name)?;
            let mut r = PluginResult::new(plugin_id, app.id.to_string(), app.name.to_string())
                .score(m.score)
                .indices(m.indices);
            if let Some(cat) = &app.category {
                r = r.subtitle(cat.as_str());
            }
            if let Some(icon) = &app.icon_path {
                r = r.icon(icon.as_str());
            }
            Some(r)
        })
        .collect();

    results.sort_by(|a, b| b.score.cmp(&a.score));
    results.truncate(9);
    results
}
