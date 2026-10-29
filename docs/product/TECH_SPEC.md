# Aurora Dev Suite: Technical Specification

Aurora Dev Suite is a native desktop application written entirely in Rust: a Tauri 2 shell with a Leptos interface compiled to WebAssembly. It runs on the user's machine, has no server, and reads the three Levix engines (the Dev Cycle Protocol `dcp`, Gedo `gedo` and AnyContext `actx`) only through their command lines with versioned JSON. The architecture style is **hexagonal** (`architecture.style` in `.dev-cycle.json`, ADR-0001): a pure core with the domain, use cases and ports, engine adapters, the desktop shell as composition root and the interface as the driving adapter. How each part works is described in [TECDOC.md](../../TECDOC.md); this document holds the product-wide technical decisions and requirements.

## Platforms and Stack

**Target platforms**

| Platform | When | Targets | Web view |
|---|---|---|---|
| Windows 10 22H2 and Windows 11 | Foundations | `x86_64-pc-windows-msvc` | WebView2 (bundled bootstrapper) |
| Linux (Ubuntu 22.04 or later and equivalents) | Foundations | `x86_64-unknown-linux-gnu` | WebKitGTK 4.1 |
| macOS 13 or later | Later, with the engines | `aarch64-apple-darwin` | WKWebView |

**Toolchain**: Rust stable, edition 2024, minimum supported version **1.95** (the same as Gedo), the `wasm32-unknown-unknown` target for the interface and Trunk to bundle it.

**Main libraries** (latest stable versions on 2026-10-27, declared once in `[workspace.dependencies]`; `Cargo.lock` is committed):

| Crate | Version | Used in | Why |
|---|---|---|---|
| `tauri` + `tauri-build` | 2.12 + 2.7 | `aurora-desktop` | Desktop shell, windows, commands and events (ADR-0002) |
| `leptos` (client-side rendering) | 0.8 | `aurora-ui` | Reactive interface in Rust compiled to WebAssembly (ADR-0002) |
| `wasm-bindgen` | 0.2 | `aurora-ui` | Calls into the Tauri `invoke` API |
| `serde` + `serde_json` | 1.0 + 1.0 | all crates | Core types shared with the interface; engine JSON |
| `toml` | 1.1 | `aurora-engines` | The project registry file (ADR-0004) |
| `directories` | 6.0 | `aurora-engines` | The user's configuration directory per platform |
| `thiserror` | 2.0 | all crates | Typed errors |
| `tracing` | 0.1 | adapters, desktop | Structured logs |
| `insta` | 1.49 | tests | Snapshot tests of parsed engine output and views' data |
| Trunk | 0.22 | build | Builds and bundles the WebAssembly interface |

## Architecture Overview

Four crates (ADR-0001): `aurora-core` (domain, use cases, ports; no I/O; also compiles to WebAssembly), `aurora-engines` (command runner, engine probe, one adapter per engine and contract version, the project registry); fixtures live in the core (`fixtures` feature), `aurora-desktop` (Tauri, composition root, one command per use case) and `aurora-ui` (Leptos views). The interface talks only to the desktop commands; the desktop builds the adapters and calls the core; the adapters run the engines' command lines. See [TECDOC.md](../../TECDOC.md).

## Non-Functional Requirements

### Performance

- Cold start to the first painted window: under 1.5 s on a mid-range laptop (4 cores, SSD).
- Portfolio with 20 registered projects: every project's summary shown within 3 s of start, each project appearing as soon as it is read.
- One engine call: timeout 10 s by default (configurable per command); a timeout marks only that section as degraded.
- Installed size of Aurora itself (without the engines): under 30 MB.

### Security and Privacy

- No network access of Aurora's own in the foundations; the engines decide their own network use.
- Code, documents and engine output never leave the machine; no telemetry.
- Engine commands run without a shell, with fixed arguments and the project directory as working directory; paths from the registry are validated (existing directory, inside a Git repository).
- The web view gets no file system, shell or HTTP access: Tauri capabilities allow only Aurora's own commands, with a strict content security policy.
- Engine output is untrusted input: size-limited, parsed with typed structs, rendered as text (Markdown rendered without raw HTML).

### Availability and Recovery

A desktop application without a service. Recovery means: a missing or broken engine never blocks the rest of the application, and a corrupt registry file is moved aside and recreated empty with a message.

### Accessibility and Localization

- Keyboard navigation for every view, visible focus, contrast of WCAG 2.2 AA in light and dark themes.
- Interface in English first; text kept in one catalog so Portuguese (pt-BR) can follow.

## Integrations

| System | Direction | How | When unavailable |
|---|---|---|---|
| Dev Cycle Protocol | Aurora reads | `dcp … --json` once the Rust `dcp` ships it (ADR-0006); nothing before | Portfolio, Project and Inbox views unavailable with "install dcp" |
| Gedo | Aurora reads | `gedo … --json` (from Gedo v0.4.0, schemas of its ADR-0013) | Agents view disabled with "install Gedo" |
| AnyContext | Aurora reads, sends questions | `actx … --json` once it exists | Knowledge view disabled with "update AnyContext" |
| `levix-platform` | Aurora links crates | Licensing and verified catalog, by git tag, once built | No license check (internal use) |
| GitHub | None directly | Pull request data comes through `dcp` | n/a |

Every engine call goes through the versioned JSON contract and capability discovery of ADR-0003.

## Data

- **Project registry** (Aurora's only own data, ADR-0004): a TOML file `projects.toml` in the user's configuration directory, with one entry per project (path, display name, date added). Format versioned by a `version` key; changes are additive.
- **Settings**: `settings.toml` beside it (theme, timeouts, engine paths).
- **Derived data**: cycle states, engine health, inbox items; held in memory, rebuilt on refresh, never stored.
- Retention: the files live until the user removes them; uninstalling leaves them, and a documented path lets the user delete them.

## Constraints

- Proprietary, binary-only distribution; every crate `publish = false`; dependencies limited to permissive licenses (MIT, Apache-2.0, BSD, ISC, Zlib, Unicode-3.0), checked with `cargo-deny` (ADR-0005). Named exceptions in `deny.toml`, each used unmodified: MPL-2.0 (file-level copyleft) for `cssparser`, `cssparser-macros`, `selectors` and `dtoa-short` of the Tauri tree and `option-ext` of `directories`; CC0-1.0 for `base16` and BSL-1.0 for `xxhash-rust`. Any other crate outside the list needs a new review.
- No engine crate is linked (ADR-0003); the engines' schedules decide when each feature can ship.
- One small team (Levi and coding agents): the foundations stay small, and every feature is testable with fixtures before its engine exists.
- Windows and Linux are both first-class: CI builds and tests on both.

## Open Questions

- How the suite installer updates Aurora: Tauri's own updater, or the shared installer core proposed to `levix-platform`.
- How long an engine call may take before a progress indicator replaces the section (proposed 300 ms).
