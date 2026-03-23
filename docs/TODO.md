# Zap — Prioritized TODO

What needs to happen, in order, to make Zap a real Raycast competitor.

## P0 — Make it daily-drivable ✅

Both P0 features are complete.

### Clipboard History (`zap-plugin-clipboard`) ✅

- ✅ Background clipboard monitoring (poll-based)
- ✅ SQLite storage with WAL mode
- ✅ Text and image support
- ✅ Fuzzy search across clipboard entries
- ✅ Paste into frontmost app on Enter (`Action::Paste`)
- ✅ Pin/favorite entries
- ✅ Configurable retention (`max_age_days`, `max_entries` via config.toml)
- ✅ Sensitive content detection (PEM keys, OTP URIs, PIN codes)
- ✅ Prefix: `cb `

### Usage-Based Ranking ✅

- ✅ SQLite counter with exponential time decay (14-day half-life)
- ✅ Blended into search score (`fuzzy_score + usage_bonus`, capped at 50)
- ✅ Per-plugin opt-in via `PluginMeta::usage_ranking()`

## P1 — Essential daily features

Features that make you stop reaching for other tools entirely.

### Emoji Picker (`zap-plugin-emoji`) ✅

- ✅ ~400 bundled emojis, fuzzy search, copy on Enter
- ✅ Grid view (8 columns) with pinned/unpinned sections
- ✅ Prefix: `:`

### System Commands (`zap-plugin-commands`) ✅

- ✅ Lock, sleep, restart, shutdown, logout, empty trash
- ✅ Platform-specific execution (Linux/macOS/Windows)
- ✅ No prefix — shows alongside apps in global search

### Snippets (`zap-plugin-snippets`)

Text expansion with dynamic placeholders: `{date}`, `{time}`, `{clipboard}`, `{cursor}`. Prefix: `sn` or `/`. Store snippets in SQLite. Create/edit snippets from within Zap.

### Window Management (`zap-plugin-windows`)

List open windows and focus on Enter. This alone replaces Alt-Tab for many users. Later: move, resize, tiling shortcuts. Platform-specific: X11 (`_NET_CLIENT_LIST`), Wayland (compositor IPC), macOS (Accessibility API).

## P2 — Power user features that differentiate

### Action Panel

Raycast's killer UX pattern. Tab or Cmd+K on any result opens a submenu of contextual actions. App result → "Show in Finder", "Copy path", "Uninstall". File result → "Open with...", "Move to trash". This requires extending `PluginResult` with a list of secondary actions.

### File Search (`zap-plugin-files`)

Deep file search via `fd`/`locate` on Linux, Spotlight (`mdfind`) on macOS. Preview in a detail pane (the first use of `PluginView::Detail`). Prefix: `f` or `~`.

### Quicklinks

User-defined URL templates: `gh {query}` → `https://github.com/search?q={query}`. Stored in config. Raycast users love these — it's the fastest way to search specific sites.

### Bookmarks (`zap-plugin-bookmarks`)

Search Chrome/Firefox/Arc bookmarks. Read SQLite databases directly (Chrome) or JSON (Firefox). No browser extension needed.

### Theming

CSS custom properties are already in place (`--hue` etc.). Ship 5-10 themes. Let users create custom themes via `~/.config/zap/themes/`. The current `--hue` variable already supports one-knob re-theming.

### Settings UI

Hotkey config, plugin enable/disable, theme selection, clipboard retention. Only build this once there are enough things to configure. Until then, `config.toml` is fine.

## P3 — Ecosystem play

### Raycast Extension Compatibility

The long game. Run Raycast extensions in a Node sidecar, mock their React API, pipe rendered output to Zap's view model. This instantly gives Zap thousands of extensions. Hard but game-changing.

### AI Integration (`zap-plugin-ai`)

Inline answers from local LLMs (Ollama) or API (Claude, OpenAI). "What's the capital of France?" → answer inline. Text transform: select text → rewrite/summarize/translate. This is where launchers are headed.

### Scripts (`zap-plugin-scripts`)

Run shell scripts as Zap commands. Point Zap at a directory of scripts, each becomes a command. Like Alfred's workflows but simpler. Prefix: `>`.

### Plugin Marketplace

Web registry at `zap.dev/extensions`. Install from within Zap. Ratings, verified publishers. Only build this when there's community demand — premature marketplaces are graveyards.

## Infrastructure (build as needed, not upfront)

- **SQLite storage layer** ✅ — Used by clipboard and usage tracking. WAL mode for concurrent reads.
- **Plugin API** ✅ — `Plugin` trait with `PluginMeta` builder, `PluginResult` builder, `fuzzy_match` helper, capability system. Type-safe TS bindings via tauri-specta.
- **Config file** ✅ — `~/.config/zap/config.toml`. Per-plugin sections.
- **WASM plugin runtime** — Needed for third-party plugins. Not needed for first-party (Rust trait objects compiled in). Build when the first external developer asks.
- **Node sidecar runtime** — Needed for Raycast compat (P3). Don't touch until P3.
- **Plugin permissions** — Needed when plugins can access network/filesystem. Not needed while all plugins are first-party and compiled in.
