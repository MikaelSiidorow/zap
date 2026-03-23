use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use nucleo_matcher::{Config, Matcher, Utf32String};

pub struct FuzzyMatch {
    pub score: u32,
    pub indices: Vec<u32>,
}

/// Perform a fuzzy match of `query` against `haystack`.
/// Returns `None` if there is no match.
pub fn fuzzy_match(query: &str, haystack: &str) -> Option<FuzzyMatch> {
    let mut matcher = Matcher::new(Config::DEFAULT);
    let atom = Atom::new(
        query,
        CaseMatching::Smart,
        Normalization::Smart,
        AtomKind::Fuzzy,
        false,
    );
    let hay = Utf32String::from(haystack);
    let mut indices = Vec::new();
    let score = atom.indices(hay.slice(..), &mut matcher, &mut indices)?;
    indices.sort_unstable();
    indices.dedup();
    Some(FuzzyMatch {
        score: score as u32,
        indices,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match() {
        let m = fuzzy_match("hello", "hello").unwrap();
        assert!(m.score > 0);
        assert_eq!(m.indices, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn fuzzy_substring() {
        let m = fuzzy_match("fox", "Firefox").unwrap();
        assert!(m.score > 0);
    }

    #[test]
    fn no_match() {
        assert!(fuzzy_match("xyz", "hello").is_none());
    }
}
