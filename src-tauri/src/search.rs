use crate::platform::AppEntry;
use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use nucleo_matcher::{Config, Matcher, Utf32String};
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct SearchResult {
    pub id: String,
    pub name: String,
    pub exec_path: String,
    pub icon_path: Option<String>,
    pub category: Option<String>,
    pub score: u32,
    pub match_indices: Vec<u32>,
}

pub fn search(query: &str, apps: &[AppEntry]) -> Vec<SearchResult> {
    if query.is_empty() {
        return apps
            .iter()
            .take(9)
            .map(|app| SearchResult {
                id: app.id.clone(),
                name: app.name.to_string(),
                exec_path: app.exec_path.clone(),
                icon_path: app.icon_path.clone(),
                category: app.category.clone(),
                score: 0,
                match_indices: vec![],
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

    let mut results: Vec<SearchResult> = apps
        .iter()
        .filter_map(|app| {
            let haystack = Utf32String::from(app.name.as_str());
            let mut indices = Vec::new();
            let score = atom.indices(haystack.slice(..), &mut matcher, &mut indices)?;
            indices.sort_unstable();
            indices.dedup();
            Some(SearchResult {
                id: app.id.clone(),
                name: app.name.to_string(),
                exec_path: app.exec_path.clone(),
                icon_path: app.icon_path.clone(),
                category: app.category.clone(),
                score: score as u32,
                match_indices: indices,
            })
        })
        .collect();

    results.sort_by(|a, b| b.score.cmp(&a.score));
    results.truncate(9);
    results
}
