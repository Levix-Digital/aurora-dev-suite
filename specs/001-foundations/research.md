# Research: Foundations

Decisions taken while planning v0.1.0. Founding decisions are in the ADRs (`docs/adr/`); this file records the smaller choices of this feature.

## R1. Calling the desktop from the Leptos interface

- **Decision**: the interface calls Tauri commands through a small `wasm-bindgen` binding to `window.__TAURI__.core.invoke` (`app.withGlobalTauri: true`), with arguments and results serialized by `serde-wasm-bindgen`. Results are the core's own types.
- **Rationale**: it is the pattern of Tauri's official Leptos template, needs no extra crate, and keeps one type definition (the core) on both sides.
- **Alternatives considered**: `tauri-sys` (community bindings, lags behind Tauri releases); a hand-written JSON protocol (duplicates types).

## R2. The interface preview without the desktop

- **Decision**: the interface talks to a `Backend` with two implementations: the Tauri one, and a fixture one built from the core's fixtures. At start-up it uses Tauri when `window.__TAURI__` exists, otherwise the fixtures. `trunk serve` then shows every view in a browser.
- **Rationale**: FR-017 and FR-018 with no second code path for the views themselves.
- **Alternatives considered**: a mock HTTP server (another process and protocol); Tauri's mock runtime (only for Rust tests).

## R3. Where the fixtures live

- **Decision**: the fixture data and the fixture implementations of the ports live in `aurora-core` behind the `fixtures` cargo feature (pure data, no I/O), so both the interface (WebAssembly) and the tests of every crate use them. ADR-0003's "fixtures" are this module; the engine adapters crate has no fixtures of its own.
- **Rationale**: `aurora-engines` runs processes and reads files, so it cannot be linked into the WebAssembly interface; the core can.
- **Alternatives considered**: a separate `aurora-fixtures` crate (one more crate for a few hundred lines).

## R4. Ports: synchronous, parallelism at the edge

- **Decision**: the ports are synchronous traits. The desktop runs each use case on Tauri's blocking pool and probes the three engines in parallel, publishing each engine's result as it arrives.
- **Rationale**: the core stays free of an async runtime and compiles to WebAssembly; engine calls are blocking processes anyway; SC-003 (a hanging engine never blocks the others) is met by running one probe per engine at the same time.
- **Alternatives considered**: async traits in the core (needs boxing or `trait_variant` for dynamic dispatch, and a runtime choice in the core).

## R5. Running engine commands

- **Decision**: `std::process::Command` without a shell, `wait-timeout` for the time limit (default 10 s, kill on expiry), standard output read on a separate thread up to 1 MiB (more marks the reply as too large), standard error kept up to 64 KiB for diagnostics. Binaries are found with the `which` crate on the search path, unless the settings give a path.
- **Rationale**: FR-015 with small, well-known crates.
- **Alternatives considered**: `tokio::process` (brings an async runtime into the adapters for no gain); `duct` (a shell-like API we do not need).

## R6. Recognizing engines without the contract

- **Decision**: `<engine> version --json` is tried first. If it fails or does not answer with the envelope (`schema`, `schema_version`), `<engine> --version` is tried: a successful answer means "installed without the contract". For the Dev Cycle, when no `dcp` is found, the stand-in is usable when the protocol's scripts are found (the setting `devcycle.skills_root`, else `~/.ai-code-agent-skills/dev-cycle-protocol`) and a Python 3 interpreter is on the search path.
- **Rationale**: FR-013 and the real state of the engines today (no contract anywhere; AnyContext answers `--version`; the Dev Cycle is Python scripts).

## R7. The version reply Aurora expects

- **Decision**: the contract proposed to all three engines (following Gedo's ADR-0013): `{"schema": "<engine>.version", "schema_version": "1.0", "version": "X.Y.Z", "capabilities": [{"name": "radar", "schema_major": 1}, ...]}`. Aurora 0.1.0 supports `schema_version` MAJOR 1 of `<engine>.version`. The exact shape is in `contracts/engine-version.schema.json`.
- **Rationale**: Gedo already decided the envelope and the capability list; the suggestions sent on 2026-10-27 ask `dcp` and `actx` for the same.

## R8. The project registry

- **Decision**: `projects.toml` and `settings.toml` in `directories::ProjectDirs::from("digital", "Levix", "Aurora")`'s configuration directory, each with a `version = 1` key. Writes go to a temporary file in the same directory and are renamed over the old one. The repository root is found by walking up to a `.git` directory or file (worktrees), without running Git. Paths are canonicalized with `dunce` (no `\\?\` prefixes on Windows) and compared case-insensitively on Windows. Ids are UUID v4.
- **Rationale**: ADR-0004; atomic writes make FR-010's damaged-file case rare; no Git process needed.
- **Alternatives considered**: JSON (less pleasant to read and fix by hand); asking Git for the root (a process per add).

## R9. Checking the dependency rule in CI

- **Decision**: `cargo xtask arch-check` reads `cargo metadata` and fails when `aurora-core`'s normal dependencies include `tauri*`, `leptos*`, `aurora-engines`, `aurora-desktop` or `aurora-ui`, or when a core source file uses `std::process`. `check_design.py` of the Dev Cycle runs locally with the same list from `.dev-cycle.json`.
- **Rationale**: CI cannot depend on the private skills repository; the same rule is enforced from Cargo's own data, as Gedo does.

## R10. CI

- **Decision**: GitHub Actions on `ubuntu-latest` and `windows-latest`: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-features`, `cargo xtask arch-check`, and a desktop build (`trunk build` of the interface, then `cargo build -p aurora-desktop --release`), reporting the binary's size. `cargo deny check` runs once, on Linux. Linux installs `libwebkit2gtk-4.1-dev`, `libsoup-3.0-dev`, `librsvg2-dev` and `libayatana-appindicator3-dev`. Caching with `Swatinem/rust-cache`; Trunk and cargo-deny through `taiki-e/install-action`.
- **Rationale**: FR-019; the Tauri CLI is not needed to build (only to bundle, which is v0.2.0).

## R11. Navigation and theme

- **Decision**: one `View` signal (Portfolio, Project, Engines, Inbox, Settings) instead of URL routing; the theme is CSS custom properties under `:root[data-theme]`, following `prefers-color-scheme` when the setting is `system`. The Aurora Borealis palette is taken from AnyContext's tokens (`AuroraTheme` in `crates/any-context-core-rs/src/theme/mod.rs`, ADR-0114 and ADR-0117 of that repository), which define the dark theme; the light theme is derived from it here. Both are checked for WCAG 2.2 AA contrast by a test (for example, AnyContext's muted text `#64748B` on `#0B0F19` is below 4.5:1, so Aurora lightens it for body-size text).
- **Rationale**: a desktop app has no URLs to share; a signal is the simplest navigation that works with the keyboard.
- **Alternatives considered**: `leptos_router` (routes add nothing in a single window, and can come later if deep links are needed).
