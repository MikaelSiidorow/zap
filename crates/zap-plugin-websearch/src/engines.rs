use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct SearchEngine {
    pub keyword: String,
    pub name: String,
    pub url: String,
}

pub fn default_engines() -> Vec<SearchEngine> {
    vec![
        SearchEngine {
            keyword: "g".into(),
            name: "Google".into(),
            url: "https://www.google.com/search?q=%s".into(),
        },
        SearchEngine {
            keyword: "ddg".into(),
            name: "DuckDuckGo".into(),
            url: "https://duckduckgo.com/?q=%s".into(),
        },
        SearchEngine {
            keyword: "gh".into(),
            name: "GitHub".into(),
            url: "https://github.com/search?q=%s".into(),
        },
        SearchEngine {
            keyword: "yt".into(),
            name: "YouTube".into(),
            url: "https://www.youtube.com/results?search_query=%s".into(),
        },
        SearchEngine {
            keyword: "wiki".into(),
            name: "Wikipedia".into(),
            url: "https://en.wikipedia.org/w/index.php?search=%s".into(),
        },
        SearchEngine {
            keyword: "so".into(),
            name: "Stack Overflow".into(),
            url: "https://stackoverflow.com/search?q=%s".into(),
        },
        SearchEngine {
            keyword: "nix".into(),
            name: "Nix Packages".into(),
            url: "https://search.nixos.org/packages?query=%s".into(),
        },
        SearchEngine {
            keyword: "crates".into(),
            name: "crates.io".into(),
            url: "https://crates.io/search?q=%s".into(),
        },
        SearchEngine {
            keyword: "npm".into(),
            name: "npm".into(),
            url: "https://www.npmjs.com/search?q=%s".into(),
        },
        SearchEngine {
            keyword: "r".into(),
            name: "Reddit".into(),
            url: "https://www.reddit.com/search/?q=%s".into(),
        },
        SearchEngine {
            keyword: "maps".into(),
            name: "Google Maps".into(),
            url: "https://www.google.com/maps/search/%s".into(),
        },
        SearchEngine {
            keyword: "mdn".into(),
            name: "MDN Web Docs".into(),
            url: "https://developer.mozilla.org/en-US/search?q=%s".into(),
        },
        SearchEngine {
            keyword: "rust".into(),
            name: "Rust Docs".into(),
            url: "https://doc.rust-lang.org/std/?search=%s".into(),
        },
        SearchEngine {
            keyword: "pypi".into(),
            name: "PyPI".into(),
            url: "https://pypi.org/search/?q=%s".into(),
        },
        SearchEngine {
            keyword: "hn".into(),
            name: "Hacker News".into(),
            url: "https://hn.algolia.com/?q=%s".into(),
        },
    ]
}
