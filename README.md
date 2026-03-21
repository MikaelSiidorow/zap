# Zap

A cross-platform application launcher built with Tauri, SvelteKit, and Rust.

## Features

- **App launcher** — quickly find and launch applications
- **Calculator** — inline math evaluation
- **Clipboard history** — search and paste from clipboard history
- **Plugin system** — extensible architecture for custom functionality

## Development

### Prerequisites

- [Rust](https://rustup.rs/)
- [Bun](https://bun.sh/)
- Linux only: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libgtk-3-dev`

### Setup

```sh
bun install
bun run tauri dev
```

### Project structure

```
src/            — SvelteKit frontend
src-tauri/      — Tauri app shell (commands, window management, tray)
crates/
  zap-core/     — Core plugin trait and types
  zap-plugin-apps/      — Application launcher plugin
  zap-plugin-calc/      — Calculator plugin
  zap-plugin-clipboard/ — Clipboard history plugin
```

## Contributing

This project uses [conventional commits](https://www.conventionalcommits.org/) to automate versioning and changelogs.

### Commit message format

```
<type>: <description>

[optional body]

[optional footer(s)]
```

**Types:**

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `chore` | Maintenance, deps, CI |
| `docs` | Documentation |
| `refactor` | Code restructuring |
| `test` | Adding or updating tests |

**Breaking changes:** add `!` after the type (e.g. `feat!: redesign plugin API`) or include `BREAKING CHANGE:` in the footer.

### Examples

```
feat: add web search plugin
fix: prevent clipboard listener crash on Wayland
chore: update tauri to v2.1
feat!: redesign plugin result format
```

## Release process

1. Merge PRs with conventional commit messages to `main`
2. [release-please](https://github.com/googleapis/release-please) opens a Release PR with version bumps and changelog
3. Merge the Release PR — a git tag is created automatically
4. GitHub Actions builds Linux (.deb, .AppImage) and macOS (.dmg) binaries
5. Binaries are uploaded as a **pre-release** on GitHub Releases
6. To promote to stable: run the "Promote Release" workflow with the tag
