# Implementation Plan: Foundations

**Branch**: `feat/001-foundations` | **Date**: 2026-10-27 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/001-foundations/spec.md`

## Summary

Build the first runnable Aurora: a Rust workspace of four crates following ADR-0001. A pure core holds the domain, the capability catalog, the ports and the fixtures. The engine adapters crate holds the safe command runner, the engine probe (contract, `--version` fallback, Dev Cycle stand-in detection) and the TOML registry and settings. The desktop crate is a Tauri 2 shell with one command per use case and parallel engine probes streamed as events. The interface crate is Leptos with the Aurora Borealis theme, the four views and a fixture backend for the browser preview. CI runs on Windows and Linux, and `cargo xtask arch-check` enforces the dependency rule.

## Technical Context

**Language/Version**: Rust stable, edition 2024, MSRV 1.95; the interface targets `wasm32-unknown-unknown`

**Primary Dependencies**: `tauri` 2.12, `tauri-plugin-dialog` 2.8, `leptos` 0.8 (CSR), `wasm-bindgen` 0.2, `serde-wasm-bindgen` 0.6, `serde`/`serde_json`, `toml` 1.1, `directories` 6, `dunce` 1, `which` 8, `wait-timeout` 0.2, `uuid` 1 (v4), `time` 0.3, `thiserror` 2, `tracing` 0.1, Trunk 0.22

**Storage**: `projects.toml` and `settings.toml` in the user's configuration directory (ADR-0004)

**Testing**: `cargo test` (unit tests per crate, integration tests of the adapters with a fake engine binary and temporary directories), `insta` snapshots of parsed engine replies, a contrast test of the theme tokens

**Target Platform**: Windows 10 22H2+ and 11 (WebView2), Linux x86_64 with WebKitGTK 4.1

**Project Type**: desktop application (Tauri shell, WebAssembly interface)

**Performance Goals**: window usable within 1.5 s (SC-001); each engine probe bounded by its timeout, the three in parallel (SC-003)

**Constraints**: no network access of Aurora's own; no shell; engine replies capped at 1 MiB, error output at 64 KiB; installed size under 30 MB (measured in CI from v0.1.0, enforced with packaging in v0.2.0)

**Scale/Scope**: up to about 50 registered projects; 3 engines; 5 views (Portfolio, Project, Engines, Inbox, Settings)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | How this plan complies | Status |
|---|---|---|
| I. Design: one responsibility per unit, explicit errors | Errors are `thiserror` enums per module (`ProjectError`, `ProbeError`, `CommandError`); no `unwrap` on input; fallbacks (`--version`, stand-in, recovered registry) are visible in status | Pass |
| II. Simplicity First | Three ports only, each with two implementations now (real and fixture); `CycleSource`, `CollaborationSource` and `KnowledgeSource` deferred to the release that calls them; navigation by signal, no router | Pass |
| III. Hexagonal | Core has no I/O, framework or adapter dependency (checked by `xtask arch-check` and `check_design.py`); adapters implement core ports; the desktop is the composition root; the interface only renders core types | Pass |
| IV. Development cycle: evidence | Every FR has an automated test except visual and keyboard checks, which go to the manual suite | Pass |
| IV. Security | No secrets; registry format versioned and additive; rollback is the previous build (no data migration) | Pass |
| IV. 9. No AI attribution | Plan and code carry no AI attribution; commit and pull request attribution follow the user's decision (see Open points) | Pending user decision |
| NFR Performance, Security, Accessibility | Covered by SC-001, SC-003, SC-005, FR-015 and the Tauri capability lock-down | Pass |

Re-checked after Phase 1 design: no violation, nothing in Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/001-foundations/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── engine-version.schema.json
│   └── desktop-commands.md
├── checklists/requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
Cargo.toml                    workspace: members, [workspace.dependencies], lints, publish = false
rust-toolchain.toml           stable + wasm32-unknown-unknown, rustfmt, clippy
deny.toml                     licenses (MIT, Apache-2.0, BSD-2/3, ISC, Zlib, Unicode-3.0), advisories, bans
.cargo/config.toml            alias xtask
.github/workflows/ci.yml      Windows and Linux CI
crates/
├── aurora-core/              no I/O; compiles to wasm32
│   └── src/
│       ├── lib.rs
│       ├── domain/           project.rs, engine.rs, settings.rs, feature.rs
│       ├── capabilities.rs   feature catalog and availability evaluation
│       ├── ports.rs          ProjectRegistry, SettingsStore, EngineProbe
│       ├── usecases/         projects.rs, engines.rs, settings.rs
│       └── fixtures.rs       feature "fixtures": sample data and port implementations
├── aurora-engines/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── runner.rs         command execution, timeout, size limits, envelope check
│   │   ├── probe.rs          EngineProbe: locate, version --json, --version, stand-in
│   │   ├── contract.rs       typed version reply (schema MAJOR 1)
│   │   ├── registry.rs       ProjectRegistry and SettingsStore over TOML, atomic writes
│   │   ├── repo.rs           repository root discovery, path canonicalization
│   │   └── bin/aurora-fake-engine.rs   feature "fake-engine": test double for dcp/gedo/actx
│   └── tests/                probe.rs, registry.rs, runner.rs
├── aurora-desktop/
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   ├── icons/
│   └── src/                  main.rs, state.rs (composition root), commands.rs, events.rs
└── aurora-ui/
    ├── Trunk.toml
    ├── index.html
    ├── styles/               tokens.css, app.css
    └── src/
        ├── main.rs
        ├── backend/          mod.rs, tauri.rs, fixture.rs
        ├── app.rs            shell, navigation, theme
        └── views/            portfolio.rs, project.rs, engines.rs, inbox.rs, settings.rs
xtask/                        arch-check
```

**Structure Decision**: the four crates of ADR-0001 under `crates/`, plus `xtask`. The fixtures sit in `aurora-core` behind a feature so the WebAssembly interface can use them (research R3).

## Complexity Tracking

No violations.

## Dev Cycle Protocol

### Backlog items and release

FEAT-001, FEAT-002, FEAT-003, FEAT-004, FEAT-006, FEAT-007, FEAT-008, FEAT-009, FEAT-010, FEAT-011 and FEAT-012, release v0.1.0 "Foundations" (`docs/ROADMAP.md`).

### Tests (quality-assurance)

- **Core unit tests**: name validation; capability evaluation for every feature and engine status (ready, without contract, stand-in, degraded, missing); availability messages; use cases over the fixtures (add, duplicate, rename, remove, relocate, recovered registry notice).
- **Adapter integration tests** (`aurora-engines/tests`): the runner with the fake engine (normal reply, timeout and kill, reply over 1 MiB, invalid JSON, wrong schema, error output kept); the probe for each status, including the `--version` fallback and the Dev Cycle stand-in detection in a temporary home; the registry in a temporary configuration directory (add from a sub-folder, duplicates through another path and through letter case on Windows, missing folder, damaged file moved aside, read-only directory, atomic write); `insta` snapshots of parsed version replies.
- **Interface tests**: contrast of every text and background token pair in both themes (WCAG 2.2 AA, 4.5:1 for body text, 3:1 for large text and focus indicators); the fixture backend returns every state.
- **Manual tests** (new curated suite `tests/MANUAL_TESTS.md`, feature "Foundations"): window opens on Windows and Linux; keyboard-only navigation with visible focus; theme follows the system and the setting; folder dialog add; engine refresh after installing a fake engine; Linux message when WebKitGTK is missing.
- **Acceptance criteria**: the acceptance scenarios of US1 to US4 and SC-001 to SC-007 of the spec.

### Documentation (architecture-docs, user-docs)

- Edit in place `docs/architecture/core.md` (modules as built, the fixtures feature), `engines.md` (fixtures moved to the core, the fake engine, detection order), `desktop.md` (commands and events of `contracts/desktop-commands.md`, capability file) and `ui.md` (backend selection, theme tokens, views).
- No new ADR: the choices of `research.md` do not change a founding decision; fixtures in the core is recorded on the engines page.
- `README.md` rewritten with `user-docs` for this release: what Aurora is, building from source on Windows and Linux, quick start, the preview, troubleshooting (WebKitGTK, WebView2), proprietary license notice.

### CI, release and observability (devops-release)

- New workflow `.github/workflows/ci.yml` on pull requests and pushes to `main` (research R10).
- Version: the workspace starts at `0.1.0` (`Cargo.toml`, the only version file); no release candidate in this plan (packaging is v0.2.0).
- Observability: `tracing` with a rolling log file in the user's data directory (`aurora.log`, 5 files of 1 MB), level from `AURORA_LOG` (default `info`); each engine probe logs the command, duration and outcome (never the output body). Tested by a unit test of the log directory resolution and checked manually in the suite. No telemetry.

### Security, migration and rollback

- No secrets; nothing leaves the machine; the Tauri capability file allows only Aurora's commands and the dialog `open`; strict content security policy; engine output treated as untrusted (size limits, typed parsing, rendered as text).
- Data: `projects.toml` and `settings.toml` start at `version = 1`; later changes are additive; an unknown future version is read-only with a notice rather than overwritten.
- Rollback: run the previous build; the files of 0.1.0 stay readable by it.

## Open points for the user

- **Attribution in commits and pull requests**: principle 9 of the constitution forbids AI attribution, while this environment adds a co-author trailer to commits and a footer to pull requests by default. The user decides which rule applies to this repository.
