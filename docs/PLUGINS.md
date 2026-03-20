# Plugin Architecture

Zap's plugin system is the product. The launcher itself is a thin shell that hosts plugins — there is no "built-in" behavior. App search, calculator, clipboard history, and everything else are plugins using the same API.

This document describes the current architecture, the design decisions behind it, and where it's headed. Raycast is the primary inspiration.

## Core Concepts

### Plugins declare intent, the runtime executes

This is the single most important design principle, borrowed directly from Raycast.

A plugin does **not** copy text to the clipboard. A plugin returns a result with `Action::Copy { content: "..." }` and the runtime handles clipboard access, user feedback, and error handling. A plugin does **not** open a URL. It returns `Action::OpenUrl { url: "..." }`.

Why this matters:
- **Plugins stay simple.** The calculator plugin is pure math — no system dependencies, no clipboard crate, no platform-specific code.
- **Consistent UX.** Every "copy" action shows the same "Copied to clipboard" feedback. Users build muscle memory.
- **Sandboxing becomes possible.** When we move to WASM plugins, they physically can't access the clipboard. The runtime does it on their behalf, after checking permissions.
- **New platforms are free.** Adding Windows support means teaching the runtime about Windows clipboard, not updating every plugin.

Raycast works exactly this way: `Action.CopyToClipboard` is a React component rendered by the extension, but Raycast's runtime does the actual copy and shows the HUD. The extension never touches `NSPasteboard`.

### Results carry their actions

Every `PluginResult` has a typed `action` field:

```rust
enum Action {
    Open,                          // plugin handles via execute()
    Copy { content: String },      // runtime copies to clipboard
    OpenUrl { url: String },       // runtime opens in browser
}
```

This is unlike Alfred (where `arg` is an opaque string piped to workflow nodes) or Rofi (where the script re-invokes itself). It's closest to Ulauncher's `on_enter = CopyToClipboardAction(text)` pattern, but serialized as data rather than a callback.

`Action::Open` is the escape hatch — the plugin's `execute()` method is called, and it can do whatever it wants (launch an app, run a script, etc.). This is the default for backwards compatibility and for plugins that need custom behavior.

### Prefix routing

Plugins can declare an optional prefix. When the user types that prefix, **only** that plugin receives the query (with the prefix stripped). No other plugin is consulted.

```rust
fn prefix(&self) -> Option<&str> { Some("=") }
```

When there's no prefix match, the query fans out to all plugins. Results are merged by score and truncated to 9.

This mirrors Raycast's "root search vs. extension search" split, and Albert's `TriggerQueryHandler` vs `GlobalQueryHandler`. It's how `= 2+2` goes exclusively to the calculator while `firefox` queries all plugins.

Why 9 results? Raycast uses 9. It's one result per digit key (for future keyboard shortcuts), and it's enough to be useful without being overwhelming.

## Plugin Trait

```rust
pub trait Plugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> &str { "" }
    fn example(&self) -> Option<&str> { None }
    fn prefix(&self) -> Option<&str> { None }
    fn init(&mut self) -> anyhow::Result<()> { Ok(()) }
    fn search(&self, query: &str) -> Vec<PluginResult>;
    fn execute(&self, result_id: &str) -> anyhow::Result<()>;
    fn refresh(&self) {}
}
```

**`Send + Sync`** — Plugins are stored in `PluginHost` which is managed as Tauri state (shared across threads). The trait is object-safe (`Box<dyn Plugin>`) so plugins are registered dynamically at startup.

**`description(&self)`** — Human-readable description of what the plugin does. Used by the built-in `?` help system.

**`example(&self)`** — Example query showing how to use the plugin (e.g. `"= 2+2"` for calc, `"firefox"` for apps). Shown in `?` help results alongside the description.

**`search(&self, query: &str)`** — Takes `&self`, not `&mut self`. Search must be safe to call concurrently. Plugins that need mutable state (like the app index) use interior mutability (`RwLock`, `parking_lot::Mutex`).

**`execute(&self, result_id: &str)`** — Only called for `Action::Open` results. The `result_id` is opaque to the host — the plugin decides what it means. For the apps plugin, it's the app's unique ID. For other plugins, it could be anything.

**`refresh(&self)`** — Optional hook for plugins that cache data (like the app index). Called from the tray "Reindex" menu item. Not every plugin needs this.

## PluginResult

```rust
pub struct PluginResult {
    pub id: String,
    pub plugin_id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub icon_path: Option<String>,
    pub score: u32,
    pub match_indices: Vec<u32>,
    pub action: Action,
}
```

**`description: Option<String>`** — Optional longer description, rendered as a third line below title and subtitle. When present, the result item switches to a stacked (vertical) layout. Used by the `?` help system to show plugin descriptions. Most results leave this `None`.

**`icon_path: Option<String>`** — File path to an icon. `None` means no icon is displayed (no placeholder, no fallback). This is intentional: the calculator result "4" should not show a gray square with "4" in it. When `icon_path` is `Some` but the image fails to load, the frontend falls back to a letter placeholder (first character of title).

**`match_indices: Vec<u32>`** — Character positions in `title` that matched the query, used for highlight rendering. The fuzzy matcher (nucleo) provides these.

**`score: u32`** — Used for ranking when multiple plugins return results for the same query. Higher is better. When a prefix routes to a single plugin, score still determines ordering within that plugin's results.

## Action Handling

The frontend receives serialized results including the action. On Enter (or click), it dispatches based on action type:

| Action | Frontend behavior | Feedback |
|--------|------------------|----------|
| `Open` | Calls `execute(plugin_id, result_id)` Tauri command, then hides window | None (app launches, file opens, etc.) |
| `Copy` | Calls `copy_to_clipboard(content)` Tauri command | Shows "Copied to clipboard" for 600ms, then hides |
| `OpenUrl` | Opens URL via system browser | Hides window |
| `SetQuery` | Sets the search bar to the given query | Stays open (triggers new search) |

Clipboard is handled by `arboard` in the Tauri layer — not in any plugin crate. This keeps plugin dependencies minimal.

The feedback pattern is inspired by Raycast's HUD (the compact overlay that appears after the window closes, saying "Copied to Clipboard"). Ours is simpler — a brief message in the results area before the window hides.

## Data Flow

```
User types "= 2+2"
    │
    ▼
PluginHost::search("= 2+2")
    │
    ├─ prefix "=" matches CalcPlugin
    │  └─ CalcPlugin::search("2+2")  (prefix stripped)
    │     └─ returns PluginResult {
    │          title: "4",
    │          subtitle: "= 2+2",
    │          action: Copy { content: "4" },
    │          icon_path: None,
    │        }
    │
    ▼
Frontend renders result (no icon, just title + subtitle)
    │
User presses Enter
    │
    ▼
Frontend reads action.type == "Copy"
    │
    ├─ invoke("copy_to_clipboard", { text: "4" })
    ├─ show "Copied to clipboard" feedback
    └─ hide window after 600ms
```

For non-prefixed queries:

```
User types "firefox"
    │
    ▼
PluginHost::search("firefox")
    │
    ├─ No prefix match → fan out to all plugins
    │  ├─ AppsPlugin::search("firefox") → [Firefox result, score: 85]
    │  ├─ CalcPlugin::search("firefox") → [] (parse error, empty)
    │  └─ ... other plugins ...
    │
    ├─ Merge all results, sort by score, truncate to 9
    │
    ▼
Frontend renders result list
    │
User presses Enter
    │
    ▼
Frontend reads action.type == "Open"
    │
    ├─ invoke("execute", { pluginId: "apps", resultId: "firefox.desktop" })
    │  └─ AppsPlugin::execute("firefox.desktop") → launches Firefox
    └─ hide window
```

## Built-in Help (`?`)

Typing `?` lists all prefixed plugins. This is built into `PluginHost`, not a separate plugin — it reads `name()`, `description()`, and `example()` from each registered plugin. Prefix-less plugins (like apps) are excluded since they have no special syntax to teach.

Each result shows three lines for quick scanning:
- **Title** — plugin name (e.g., "Calculator")
- **Subtitle** — example command in monospace (e.g., `= 2+2`)
- **Description** — what it does (e.g., "Inline calculator for math expressions")

Typing `? calc` filters the list. Pressing Enter fills the plugin's prefix into the search bar via `Action::SetQuery`, so you can immediately start using it.

The search placeholder reads "Search or type ? for help" to hint at this feature.

## Current Plugins

### `zap-plugin-apps`

App discovery and launch. Indexes `.desktop` files on Linux, `.app` bundles on macOS. Caches to disk with MessagePack. Background refresh every 30 seconds. Fuzzy search via `nucleo-matcher`.

- Prefix: none (always participates in global search)
- Action: `Open` (launches the app)
- Icon: app icon from `.desktop` file or `.app` bundle

### `zap-plugin-calc`

Inline calculator. Recursive descent parser for arithmetic expressions. Supports `+`, `-`, `*`, `/`, `%`, `^`/`**`, parentheses, unary minus, decimals, constants (`pi`, `e`), and functions (`sqrt`, `sin`, `cos`, `tan`, `log`, `ln`, `abs`, `floor`, `ceil`, `round`).

- Prefix: `=`
- Action: `Copy` (copies the result)
- Icon: none
- Zero dependencies beyond `zap-core` and `anyhow`

## What We Learned from Other Launchers

We researched Raycast, Alfred, Rofi, Albert, and Ulauncher before designing this system.

**Raycast** (the gold standard): Multiple view types (List, Grid, Detail, Form). Typed action components. Built-in icon enum with hundreds of icons. HUD + Toast feedback system. The key insight we took: actions are declared, not executed. The extension says "copy this" and the runtime does it.

**Alfred**: Script Filters output JSON with `arg` piped to workflow graph nodes. The `uid` field enables learning/ranking across sessions — we should adopt this. The `mods` object (alt/cmd/shift variants per result) is powerful and we'll want something similar.

**Ulauncher**: Closest to our action model — `on_enter = CopyToClipboardAction(text)`. Clean and simple. Limited to flat list rendering.

**Albert**: `TriggerQueryHandler` vs `GlobalQueryHandler` maps exactly to our prefix routing. Their `InputActionText` (Tab to refine) is worth stealing.

**Rofi**: Stateless CGI-like model where the script re-invokes itself on each selection. Elegant but too limited for rich plugins. Pango markup in rows is a clever way to add formatting without new view types.

## Where This Is Headed

### View types (next)

The current model forces every plugin into a flat list of title/subtitle/icon rows. This works for app search and calculator, but breaks down for:

- Icon browsers → need a grid
- Clipboard history → needs preview/detail pane
- Color pickers → needs swatches
- AI answers → needs markdown rendering

The plan is a `PluginView` enum, inspired by Raycast's view types:

```
PluginView::List(Vec<ListItem>)     // current behavior
PluginView::Detail { markdown, metadata }  // rich content
PluginView::Grid { items, columns }        // image-first layout
```

Plugins would return `PluginView` from `search()` instead of `Vec<PluginResult>`. The frontend renders the appropriate component. We'll add this when the second view type is actually needed — not before.

### Action panel

Raycast's `ActionPanel` attaches multiple actions to each result. Primary action = Enter, secondary actions appear in a submenu (Cmd+K or Tab). This lets you do things like:

- App result → Enter to launch, Tab → "Show in Finder", "Copy path", "Uninstall"
- Clipboard item → Enter to paste, Tab → "Copy", "Delete", "Pin"
- File result → Enter to open, Tab → "Open with...", "Copy path", "Move to trash"

Alfred does this with modifier keys (`mods` object). We'll likely do both — action panel for discovery, keyboard shortcuts for speed.

### Richer icons

Move from `icon_path: Option<String>` to an `Icon` enum:

```
Icon::Path(String)         // file path (current)
Icon::Name(String)         // named icon from built-in set
Icon::Emoji(String)        // single emoji
Icon::Theme(String)        // XDG icon theme name (Linux)
```

Raycast has hundreds of built-in icons. We don't need hundreds, but a core set (clipboard, calculator, globe, gear, folder, terminal, etc.) would let plugins look consistent without bundling images.

### WASM plugin runtime

Currently plugins are Rust crates compiled into the binary. Phase 1 adds WASM plugins via `wasmtime` — third-party plugins compiled to `.wasm`, loaded at runtime, sandboxed by default. The `Plugin` trait maps naturally to WASM imports/exports.

### Node sidecar runtime

Phase 2 adds Node/Deno sidecars for plugins that need NPM packages or want Raycast API compatibility. The Raycast compat layer (`zap-raycast-compat`) would run Raycast extensions in a Node sidecar, translate their React component tree to Zap's view model, and pipe results back.
