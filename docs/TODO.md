# Zap — Prioritized TODO

What needs to happen, in order, to make Zap a real Raycast competitor.

## P0 — Make it daily-drivable

These two features are what make people stay. Without clipboard history, Zap is just an app launcher. Without usage ranking, it feels dumb.

### Clipboard History (`zap-plugin-clipboard`)

The single most important feature after app launch. Once you have clipboard history, you physically cannot go back to a launcher without it.

- Background clipboard monitoring (poll or native events)
- SQLite storage (introduces shared storage layer for all plugins)
- Text and image support
- Search across clipboard entries
- Paste into frontmost app on Enter (new `Action::Paste`)
- Pin/favorite entries
- Configurable retention (default: 30 days, max entries)
- Sensitive content detection (don't store passwords from password managers)
- Prefix: `cb` or dedicated hotkey

### Usage-Based Ranking

Apps you launch daily should be at the top. This is small code, big daily impact.

- SQLite counter: increment on every `Action::Open` execute
- Blend usage frequency into search score (e.g., `fuzzy_score + usage_bonus`)
- Decay over time (recent usage weights more than old)
- Per-plugin opt-in (apps yes, calc no)

## P1 — Essential daily features

Features that make you stop reaching for other tools entirely.

### Emoji Picker (`zap-plugin-emoji`)

Quick win. Ship a bundled emoji dataset, fuzzy search, copy on Enter. Prefix: `:` (like Slack). Skin tone support via modifier.

### System Commands (`zap-plugin-commands`)

Lock screen, sleep, restart, empty trash, toggle dark mode, logout. These are the commands people run from Raycast 5x/day. Small plugin, high value. No prefix — show alongside apps in global search.

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

- **SQLite storage layer** — Needed for clipboard history (P0). Shared across plugins via a `Storage` API on the plugin host. WAL mode for concurrent reads.
- **WASM plugin runtime** — Needed for third-party plugins. Not needed for first-party (Rust trait objects compiled in). Build when the first external developer asks.
- **Node sidecar runtime** — Needed for Raycast compat (P3). Don't touch until P3.
- **Config file** — `~/.config/zap/config.toml`. Add fields as features need them. Don't build a settings UI until P2.
- **Plugin permissions** — Needed when plugins can access network/filesystem. Not needed while all plugins are first-party and compiled in.
