# Changelog

## [0.1.6](https://github.com/MikaelSiidorow/zap/compare/v0.1.5...v0.1.6) (2026-03-29)


### Features

* add autostart with tray toggle, default on ([0cb3898](https://github.com/MikaelSiidorow/zap/commit/0cb389807063414c44712d689f3093e793f3b2e1))
* add emoji picker plugin with grid layout, pinning, and keyboard nav ([88a07b6](https://github.com/MikaelSiidorow/zap/commit/88a07b64b388814c375f9249f71c7b4e5304c17d))
* add Nix flake for cross-platform builds and installation ([5de8c24](https://github.com/MikaelSiidorow/zap/commit/5de8c24865d854118efb5b74a1344871f311c84b))
* add usage-based ranking with time decay for search results ([411b4cf](https://github.com/MikaelSiidorow/zap/commit/411b4cfd56cf4ecf867d3d83253c3c20a1a90521))
* add window switcher plugin with X11 support and shared icon resolver ([5fdd36e](https://github.com/MikaelSiidorow/zap/commit/5fdd36e347b8a32e63da0c7b892aa5e15e36afba))


### Bug Fixes

* improve Rust idioms — remove panics, deduplicate, safe casts ([cb10c06](https://github.com/MikaelSiidorow/zap/commit/cb10c06529e81c94353f25b576d83adccb8bf659))
* use new apple-sdk pattern for darwin and add CI nix cache ([230bd89](https://github.com/MikaelSiidorow/zap/commit/230bd89c0f03e707a08608686cbc4fd5e3b90363))

## [0.1.5](https://github.com/MikaelSiidorow/zap/compare/v0.1.4...v0.1.5) (2026-03-21)


### Features

* add system commands plugin for lock, sleep, restart, shutdown, logout, and empty trash ([fa16f6c](https://github.com/MikaelSiidorow/zap/commit/fa16f6c5ce367d5368a8617864fe53dd7d80958e))

## [0.1.4](https://github.com/MikaelSiidorow/zap/compare/v0.1.3...v0.1.4) (2026-03-21)


### Features

* add config system and web search plugin ([dad80cc](https://github.com/MikaelSiidorow/zap/commit/dad80ccd4bce5e18209d36835fc35c1e8ac61d87))
* wire up Action::OpenUrl via open crate ([a4bd8d7](https://github.com/MikaelSiidorow/zap/commit/a4bd8d799b18c504b3abd757539b33c743376e2f))


### Bug Fixes

* exclude prefixed plugins from global fan-out and show all plugins in help ([92704e7](https://github.com/MikaelSiidorow/zap/commit/92704e7695a098883412c10d1b45cdd2335da43c))
* gate Unix socket IPC for Windows compatibility ([951b87b](https://github.com/MikaelSiidorow/zap/commit/951b87b13cfebdd944449a1cbf5573bb9760ae38))

## [0.1.3](https://github.com/MikaelSiidorow/zap/compare/v0.1.2...v0.1.3) (2026-03-21)


### Features

* add Windows build support ([cb50203](https://github.com/MikaelSiidorow/zap/commit/cb50203b5cc1f35b2eaa8b735f6302598e9b5330))

## [0.1.2](https://github.com/MikaelSiidorow/zap/compare/v0.1.1...v0.1.2) (2026-03-21)


### Features

* **calc:** add unit conversion support ([3d17486](https://github.com/MikaelSiidorow/zap/commit/3d174864e15da2fea8a81ffcbd11db5874468eed))
* **calc:** add year and month to time unit conversions ([609b76d](https://github.com/MikaelSiidorow/zap/commit/609b76df300ca8da803ed296bec9d360736f9e17))


### Bug Fixes

* **clipboard:** make schema migration crash-safe ([0ce9d5c](https://github.com/MikaelSiidorow/zap/commit/0ce9d5cbf83ead3f121bb3f0dac503c30b7a6a24))

## [0.1.1](https://github.com/MikaelSiidorow/zap/compare/v0.1.0...v0.1.1) (2026-03-21)


### Features

* add built-in ? help showing prefixed plugins ([6579771](https://github.com/MikaelSiidorow/zap/commit/657977136086fb846011d9a129e0b4874cb5d168))
* add calculator plugin (zap-plugin-calc) ([88b3432](https://github.com/MikaelSiidorow/zap/commit/88b3432175d993611e6958a14c5aa053e2053aa4))
* add clipboard history plugin (zap-plugin-clipboard) ([dc983d5](https://github.com/MikaelSiidorow/zap/commit/dc983d5313f58cc24d78a085cbb02b285814b904))
* add image clipboard support, enigo paste, and Shift+Enter copy ([a2780e7](https://github.com/MikaelSiidorow/zap/commit/a2780e7e4327abaf0fe3ec3ad3a6eae9dc4fd4d6))
* add keyboard shortcut hints footer for plugins ([6dea759](https://github.com/MikaelSiidorow/zap/commit/6dea75984917faee77fe3ff6407b0598099d2202))
* add timezone conversion to calc plugin ([9faf001](https://github.com/MikaelSiidorow/zap/commit/9faf0019946538e59e5fe82d2dcae93a23cf6f72))
* add typed Action enum to plugin results (Raycast-inspired) ([0bbc133](https://github.com/MikaelSiidorow/zap/commit/0bbc1335c3418c9a0a8acb729d84375d81e9acc4))
* enable colored log output for env_logger ([93e3c19](https://github.com/MikaelSiidorow/zap/commit/93e3c196b030bd29bf9d360fff9b91f9b3f93310))
* phase 0 app launcher with fuzzy search, hotkey toggle, and system tray ([126cd68](https://github.com/MikaelSiidorow/zap/commit/126cd68cd79c0e4ae4224b226eb417b701fcb4d9))
* plugin-defined keyboard shortcut hints ([606a6de](https://github.com/MikaelSiidorow/zap/commit/606a6de2b4a8d6f9e46722fe15fbde4c76ac4f9a))


### Bug Fixes

* keep clipboard alive longer to prevent X11 ownership loss ([7c27eee](https://github.com/MikaelSiidorow/zap/commit/7c27eee087a236ae8338244d246aac1ce765f111))
* resolve CI failures and update actions to latest versions ([23007ed](https://github.com/MikaelSiidorow/zap/commit/23007ed81fae0167d78b7484b73b9e09b6cc1fee))
