# Zap — Vision Document

## One-liner

A lightning-fast, cross-platform command palette and productivity launcher — Raycast's power, everywhere.

## Problem

Raycast is the best productivity launcher ever built. It has no Linux support and has stated it has no plans to add it. Vicinae fills the Linux gap but doesn't run on macOS. Every other alternative (Ulauncher, Albert, ueli, Alfred) is either single-platform, slow, or lacks a real extension ecosystem.

If you use macOS at work and Linux at home, there is no single launcher that gives you the same experience, shortcuts, extensions, and muscle memory across both. You're forced to learn two tools, maintain two configs, and lose your flow every time you switch machines.

## Solution

Zap is a single launcher binary that runs on macOS, Linux (X11, Wayland/Hyprland, GNOME, KDE), and eventually Windows. It launches in <100ms, lives in the background, and is summoned with a hotkey. It provides a universal command palette for launching apps, running commands, searching files, managing clipboard history, controlling windows, and running extensions — all from the keyboard.

## Core Principles

1. **Speed is a feature.** If it's not perceptibly instant, it's broken. Rust backend, zero-copy where possible, always-resident process, pre-rendered window. Target: <50ms from hotkey to visible, <5ms per search query.

2. **Same experience everywhere.** Config, extensions, themes, and shortcuts sync or at least transfer across platforms. You should never have to think about which OS you're on.

3. **Everything is a plugin.** App search, clipboard history, calculator, window management — all built as first-party plugins using the same API third-party developers use. This forces the plugin API to be good, and lets users disable or replace anything.

4. **Keyboard-first, not keyboard-only.** Optimized for keyboard flow, but clickable when you want it. No mouse required, but no mouse hostility either.

5. **Open source, local-first.** All data stays on your machine. No accounts required. No telemetry. Extensions run sandboxed. The project is MIT or Apache-2.0 licensed.

## Architecture

### Runtime

- **Tauri 2.x** — Cross-platform desktop shell using native webview
- **Rust backend** — App indexing, fuzzy search, plugin host, platform abstraction, IPC
- **Svelte 5 frontend** — Minimal reactive UI rendered in the webview
- **Always-resident process** — Starts on login, window pre-created but hidden, hotkey toggles visibility

### Platform Abstraction

A Rust trait layer that normalizes OS-specific behavior:

| Capability    | macOS                      | Linux X11             | Linux Wayland                |
| ------------- | -------------------------- | --------------------- | ---------------------------- |
| Global hotkey | CGEvent / Tauri plugin     | XGrabKey              | Compositor bind + IPC socket |
| App discovery | /Applications .app bundles | .desktop files (XDG)  | .desktop files (XDG)         |
| App launch    | `open -a` / NSWorkspace    | `exec` from .desktop  | `exec` from .desktop         |
| Clipboard     | NSPasteboard               | xclip / xsel          | wl-clipboard                 |
| Window list   | CGWindowListCopyWindowInfo | xcb \_NET_CLIENT_LIST | hyprctl / compositor IPC     |
| Window mgmt   | Accessibility API          | xcb / wmctrl          | compositor IPC               |
| File search   | mdfind (Spotlight)         | locate / fd           | locate / fd                  |
| Notifications | NSUserNotification         | libnotify             | libnotify                    |
| Tray icon     | NSStatusItem               | Tauri tray plugin     | Tauri tray plugin            |

New platform implementations (Windows, BSD) just implement the same trait.

### Plugin System

This is the core architectural bet. Zap's plugin system defines what Zap is — the launcher itself is just a thin shell that hosts plugins.

#### Design

- Plugins are the unit of functionality. There is no "built-in" behavior that isn't a plugin.
- Each plugin declares: a name, commands it exposes, UI components it can render, and permissions it needs.
- Plugins can render views into the main Zap window using a declarative component model (list views, detail views, form views, grid views).
- Plugins communicate with the host through a defined API — they cannot access the filesystem, network, or OS directly without declared permissions.

#### Execution Models (support both)

1. **WASM plugins** — Compiled to WebAssembly, run sandboxed in the Rust host via `wasmtime`. Best for performance-critical, self-contained plugins (calculator, unit converter, hash generator). Near-native speed, true sandboxing, no runtime dependency.

2. **Node/Deno sidecar plugins** — Run as a managed child process. Best for plugins that need NPM packages, network access, or Raycast API compatibility. This is the path to Raycast extension interop — mock their API surface, run their React-based extensions in a sidecar, and pipe the rendered output to Zap's frontend.

#### Plugin API Surface (v1 target)

```
// Core
zap.search.register(query => results[])
zap.command.register(name, handler)
zap.navigation.push(view)
zap.navigation.pop()

// UI Components (declarative)
zap.ui.List({ items, onSelect })
zap.ui.Detail({ markdown, metadata })
zap.ui.Form({ fields, onSubmit })
zap.ui.Grid({ items, columns })
zap.ui.Action({ title, shortcut, handler })
zap.ui.ActionPanel({ actions[] })

// Platform
zap.clipboard.read()
zap.clipboard.write(content)
zap.clipboard.history()
zap.apps.list()
zap.apps.launch(id)
zap.windows.list()
zap.windows.focus(id)
zap.fs.search(query, scope)
zap.shell.exec(command)  // requires permission
zap.fetch(url)           // requires permission
zap.storage.get(key)     // per-plugin sandboxed storage
zap.storage.set(key, value)

// Lifecycle
zap.onActivate(() => {})
zap.onDeactivate(() => {})
zap.preferences.get(key)
```

#### First-Party Plugins (built by us, using the public API)

These are the "product" — they ship with Zap but are implemented as plugins:

| Plugin               | Description                                                    | Phase |
| -------------------- | -------------------------------------------------------------- | ----- |
| `zap-apps`           | App discovery, indexing, fuzzy search, launch                  | 0     |
| `zap-clipboard`      | Clipboard history with search, pinning, categories             | 1     |
| `zap-calc`           | Calculator with unit conversion, currency, math expressions    | 1     |
| `zap-snippets`       | Text snippets with expansion, dynamic placeholders             | 1     |
| `zap-windows`        | Window list, focus, move, resize, tiling shortcuts             | 2     |
| `zap-files`          | File search via Spotlight (macOS) or fd/locate (Linux)         | 2     |
| `zap-commands`       | System commands (sleep, lock, restart, empty trash, etc.)      | 2     |
| `zap-emoji`          | Emoji picker with search                                       | 1     |
| `zap-bookmarks`      | Browser bookmark search (Chrome, Firefox, Arc)                 | 2     |
| `zap-scripts`        | Run shell scripts as commands                                  | 2     |
| `zap-theme`          | Theme engine, custom themes                                    | 2     |
| `zap-ai`             | LLM integration (local or API), inline answers, text transform | 3     |
| `zap-notes`          | Quick capture, scratchpad, floating notes                      | 3     |
| `zap-passwords`      | 1Password / Bitwarden / KeePassXC integration                  | 3     |
| `zap-ssh`            | SSH connection manager                                         | 3     |
| `zap-docker`         | Container management                                           | 3     |
| `zap-k8s`            | Kubernetes context/pod management                              | 3     |
| `zap-raycast-compat` | Raycast extension compatibility layer                          | 3     |

#### Third-Party Extension Distribution

- Extensions are Git repositories with a `zap-plugin.toml` manifest
- No central server required (like Gauntlet's approach), but we'll host a curated registry at `zap.dev/extensions` (eventual)
- Install via: `zap install github:user/repo` or from within Zap's UI
- Auto-update via Git tags

## Phased Roadmap

### Phase 0 — The Core Loop (current)

**Goal:** Hotkey → search → launch feels instant on macOS and Linux.

- Always-resident Tauri process with hidden window
- App indexing (macOS .app bundles, Linux .desktop files)
- Fuzzy search via `nucleo`
- App launching
- Global hotkey (Tauri plugin on macOS/X11, IPC socket for Wayland)
- Tray icon with quit/reindex
- Cache app index to disk (MessagePack)

**Definition of done:** Press hotkey, type "fire", hit enter, Firefox launches, window hides. Works on macOS and Hyprland. Total time from keypress to window visible: <100ms.

### Phase 1 — Essential Productivity

**Goal:** Daily-drivable. You stop reaching for Raycast/Alfred.

- Plugin system v1 (WASM plugins, API surface for search + commands + basic UI)
- Refactor app search as `zap-apps` plugin
- `zap-clipboard` — encrypted clipboard history with search, image support, pin items
- `zap-calc` — inline calculator (integrate SoulverCore or build a Rust expression parser)
- `zap-snippets` — text expansion with dynamic placeholders ({date}, {clipboard}, {cursor})
- `zap-emoji` — emoji picker with skin tone support
- Theming engine — CSS custom properties, ship 5-10 themes
- Settings UI — hotkey config, plugin enable/disable, theme selection
- SQLite for persistent storage (clipboard history, usage frequency, plugin data)
- Usage-based ranking — frequently launched apps bubble to the top

### Phase 2 — Power User Features

**Goal:** Comprehensive enough that power users choose Zap over native tools.

- `zap-windows` — list open windows, focus, move, resize, basic tiling
- `zap-files` — deep file search with preview
- `zap-commands` — system commands (lock, sleep, empty trash, eject, toggle dark mode, etc.)
- `zap-bookmarks` — browser bookmark search
- `zap-scripts` — run arbitrary shell scripts as Zap commands
- Node/Deno sidecar plugin runtime (for richer third-party extensions)
- Plugin permissions model (network, filesystem, shell access require user approval)
- Action panel — contextual actions on any result (copy path, open in terminal, reveal in finder, etc.)
- Quicklinks — user-defined URL templates with placeholder substitution
- Extension registry website

### Phase 3 — Ecosystem & Intelligence

**Goal:** Extension ecosystem takes off. Zap becomes the universal desktop interface.

- `zap-raycast-compat` — compatibility layer for running Raycast extensions
- `zap-ai` — local LLM or API-backed AI assistant (inline answers, text rewriting, code generation)
- `zap-notes` — floating scratchpad / quick capture
- Service integrations via community plugins (GitHub, Jira, Slack, Notion, Linear, etc.)
- Plugin marketplace with ratings and verified publishers
- Cross-machine config sync (Git-based or optional cloud)
- Windows support
- Accessibility audit and screen reader support

### Phase 4 — Beyond the Launcher

**Goal:** Zap becomes the keyboard-first operating layer across your machines.

- Focus/pomodoro mode with app blocking
- Meeting controls (mute mic, toggle camera, join next meeting)
- Notification center integration
- Automation/workflow builder (chain commands)
- Mobile companion (trigger desktop commands from phone)

## On Building Everything as Plugins from Day 0

**Yes. This is the single most important architectural decision.**

The temptation is to hardcode app search in phase 0 for speed, then "refactor into a plugin later." Don't. You'll never refactor it, and you'll end up with a split between privileged built-in features and second-class plugins.

Instead:

- Phase 0 ships with a minimal plugin host that can load exactly one plugin: `zap-apps`.
- The plugin API surface in phase 0 is tiny: `register_search_handler` and `register_command`.
- `zap-apps` uses this API to register itself as a search provider.
- The host routes search queries to registered plugins and merges results.

This means even in phase 0, the data flow is: user types → host calls plugin search handlers → plugin returns results → host renders. When you add `zap-clipboard` in phase 1, it's just another search handler in the same pipeline. No refactoring needed.

The plugin host in phase 0 doesn't need WASM or Node sidecars. It can be simple Rust trait objects loaded at compile time (first-party plugins are statically linked). Dynamic loading comes in phase 1 with WASM, and Node sidecars in phase 2.

This is exactly how Raycast, VS Code, and Obsidian succeeded — the extension API isn't an afterthought, it IS the product.

## Non-Goals

- **Not a window manager.** Zap can move/resize windows, but it doesn't replace i3/Hyprland/yabai.
- **Not an app store.** Zap launches apps, it doesn't install them.
- **Not a terminal.** Zap can run commands, but it's not a shell replacement.
- **Not a note-taking app.** Quick capture yes, but Obsidian/Notion territory is out of scope.
- **Not a cloud service.** Local-first. Cloud sync is optional and comes late.

## Tech Stack Summary

| Layer          | Technology                                        | Rationale                                                       |
| -------------- | ------------------------------------------------- | --------------------------------------------------------------- |
| Desktop shell  | Tauri 2.x                                         | Cross-platform, native webview, Rust backend, <10MB binary      |
| Backend        | Rust                                              | Performance, safety, platform FFI, WASM host                    |
| Frontend       | Svelte 5                                          | Tiny runtime, zero VDOM overhead, reactive, fast                |
| Fuzzy search   | nucleo                                            | Fastest fuzzy matcher in Rust ecosystem                         |
| Storage        | MessagePack (phase 0), SQLite/WAL (phase 1+)      | Binary serde for cache, SQL for structured data                 |
| Plugin runtime | Rust traits → WASM (wasmtime) → Node/Deno sidecar | Progressive capability: compile-time → sandboxed → full runtime |
| Distribution   | GitHub releases, Nix flake, Homebrew, AUR         | Cover macOS and Linux power users where they are                |

## Distribution Strategy

- **NixOS:** Nix flake from day 0. `nix run github:user/zap`. Declarative config via `zap.toml`.
- **macOS:** Homebrew cask. DMG download on GitHub releases.
- **Arch Linux:** AUR package.
- **Ubuntu/Fedora:** AppImage on GitHub releases. Flatpak eventually.
- **Config format:** TOML file in `~/.config/zap/config.toml`. Portable across machines.

## Success Metrics

Phase 0: You use Zap as your daily launcher on both machines. It replaces Spotlight and any Linux launcher you were using.

Phase 1: 10 people outside your circle use it daily and don't go back to their previous launcher.

Phase 2: 100+ GitHub stars. Community-contributed plugins appear without you asking.

Phase 3: Featured on Hacker News front page. Plugin ecosystem is self-sustaining.

## Name

**Zap** — short, fast, memorable, implies electricity and instant action. Four keystrokes to summon everything.
