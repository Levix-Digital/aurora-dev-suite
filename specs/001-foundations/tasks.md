---
description: "Tasks of the Foundations feature (v0.1.0)"
---

# Tasks: Foundations

**Input**: Design documents from `/specs/001-foundations/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/
**Tests**: required by the constitution (principle IV.2, evidence) and by FR-017 and SC-006; test tasks come before the code they prove.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: can run in parallel (different files, no dependency on an unfinished task)
- **[Story]**: US1 to US4 of spec.md

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: the workspace, toolchain and checks every later task builds on (FEAT-001)

- [ ] T001 Create the workspace `Cargo.toml` (members `crates/*` and `xtask`, `[workspace.package]` version 0.1.0, edition 2024, rust-version 1.95, `publish = false`, proprietary license notice, `[workspace.dependencies]` with the versions of plan.md, `[workspace.lints]` with `unsafe_code = "forbid"` and clippy `all` warnings)
- [ ] T002 [P] Add `rust-toolchain.toml` (stable, components rustfmt and clippy, target wasm32-unknown-unknown), `rustfmt.toml` and `.gitignore` (target, dist)
- [ ] T003 [P] Add `deny.toml` (licenses MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, Unicode-3.0, advisories deny, unlicensed workspace crates ignored). If Tauri's dependency tree needs a weak-copyleft license such as MPL-2.0, list the crates as exceptions and amend the license constraint of `docs/product/TECH_SPEC.md` in the same change, with the user's approval
- [ ] T004 Create the four crates with empty `lib.rs`/`main.rs`: `crates/aurora-core`, `crates/aurora-engines`, `crates/aurora-desktop`, `crates/aurora-ui`
- [ ] T005 [P] Create `xtask` with `arch-check` (research R9) and the alias in `.cargo/config.toml`; test that it fails on a forbidden dependency using a fixture metadata file in `xtask/tests/`

**Checkpoint**: `cargo build --workspace`, `cargo xtask arch-check` and `cargo deny check` pass.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: the core domain, the ports, the fixtures, the shell and the theme every story needs (FEAT-002, FEAT-003 base, FEAT-009)

- [ ] T006 [P] Domain types with serde in `crates/aurora-core/src/domain/` (`project.rs`, `engine.rs`, `settings.rs`, `feature.rs`) per data-model.md
- [ ] T007 [P] Ports in `crates/aurora-core/src/ports.rs` (`ProjectRegistry`, `SettingsStore`, `EngineProbe`)
- [ ] T008 Fixtures in `crates/aurora-core/src/fixtures.rs` behind the `fixtures` feature: four sample projects (one missing), an engine report per status, and in-memory implementations of the three ports; unit tests that each status is covered
- [ ] T009 Tauri shell in `crates/aurora-desktop` (`build.rs`, `tauri.conf.json` with `withGlobalTauri`, strict CSP and the interface's `dist` as frontend, `capabilities/default.json` with only Aurora's commands and dialog `open`, default icons), `src/main.rs` starting the window and the `tracing` log file (plan: observability), with a unit test of the log directory resolution
- [ ] T010 Leptos application skeleton in `crates/aurora-ui` (`Trunk.toml`, `index.html`, `src/main.rs`, `src/app.rs`) and the `Backend` abstraction with the Tauri implementation (`src/backend/tauri.rs`, research R1) and the fixture implementation (`src/backend/fixture.rs`, research R2), chosen at start-up
- [ ] T011 [P] Theme tokens `crates/aurora-ui/styles/tokens.css` (dark from AnyContext's `AuroraTheme`, light derived) and `app.css`; contrast test of every token pair in both themes in `crates/aurora-ui/src/theme_contrast.rs` (research R11)

**Checkpoint**: the desktop opens an empty themed window; `trunk serve` shows the same window in a browser.

---

## Phase 3: User Story 1 - Open Aurora and move between its views (Priority: P1) 🎯 MVP

**Goal**: the window, navigation, theme setting and per-section states (FR-001 to FR-004)

**Independent Test**: open on Windows and Linux, move through every view with the keyboard only, switch light and dark.

### Tests for User Story 1

- [ ] T012 [P] [US1] Unit tests of the theme resolution (system, light, dark) and of the settings use case over fixtures in `crates/aurora-core/src/usecases/settings.rs`
- [ ] T013 [P] [US1] Manual tests "Foundations: window and navigation" in `tests/MANUAL_TESTS.md` (open on both systems, keyboard-only navigation, visible focus, theme follows system and setting)

### Implementation for User Story 1

- [ ] T014 [US1] Settings use cases (`get`, `save`) in `crates/aurora-core/src/usecases/settings.rs`
- [ ] T015 [US1] `SettingsStore` over `settings.toml` with atomic write in `crates/aurora-engines/src/registry.rs`, with integration tests in `crates/aurora-engines/tests/registry.rs`
- [ ] T016 [US1] Commands `get_settings` and `save_settings` in `crates/aurora-desktop/src/commands.rs`, composition root in `src/state.rs`
- [ ] T017 [US1] Navigation (`View` signal, keyboard handling, focus styles) and the shell layout in `crates/aurora-ui/src/app.rs`
- [ ] T018 [P] [US1] Settings view (theme choice; per-engine location and time limit, FR-011) in `crates/aurora-ui/src/views/settings.rs`
- [ ] T019 [P] [US1] Project and Inbox views with the "coming in v0.2.0" state (FR-004) in `crates/aurora-ui/src/views/project.rs` and `inbox.rs`
- [ ] T020 [US1] Linux start-up check for WebKitGTK with a readable message (edge case) in `crates/aurora-desktop/src/main.rs`

**Checkpoint**: US1 acceptance scenarios 1 to 4 pass.

---

## Phase 4: User Story 2 - Register the projects to follow (Priority: P2)

**Goal**: the project registry and the Portfolio list (FEAT-010, FR-005 to FR-010)

**Independent Test**: add three, rename one, remove one, restart; refuse a non-repository folder.

### Tests for User Story 2

- [ ] T021 [P] [US2] Core tests of the project use cases over fixtures (add, duplicate, invalid name, rename, remove, relocate, recovered and read-only notices) in `crates/aurora-core/src/usecases/projects.rs`
- [ ] T022 [P] [US2] Adapter tests in `crates/aurora-engines/tests/registry.rs`: sub-folder resolves to the root, `.git` file (worktree), duplicate through another path and through letter case on Windows, missing folder, damaged file moved aside, read-only directory, atomic write survives a failed write

### Implementation for User Story 2

- [ ] T023 [US2] Project use cases in `crates/aurora-core/src/usecases/projects.rs`
- [ ] T024 [US2] Repository root discovery and canonicalization in `crates/aurora-engines/src/repo.rs`
- [ ] T025 [US2] `ProjectRegistry` over `projects.toml` in `crates/aurora-engines/src/registry.rs`
- [ ] T026 [US2] Commands `list_projects`, `add_project`, `pick_and_add_project`, `rename_project`, `remove_project`, `relocate_project` in `crates/aurora-desktop/src/commands.rs`
- [ ] T027 [US2] Portfolio view (list, add with the folder dialog, rename, remove, missing state with locate, notices) in `crates/aurora-ui/src/views/portfolio.rs`
- [ ] T028 [P] [US2] Manual tests "Foundations: projects" in `tests/MANUAL_TESTS.md` (folder dialog, restart keeps the list, damaged file notice)

**Checkpoint**: US2 acceptance scenarios 1 to 5 pass.

---

## Phase 5: User Story 3 - See which engines are installed and what they enable (Priority: P3)

**Goal**: the runner, the probe, the capability catalog and the Engines view (FEAT-007, FEAT-008, FEAT-011, FEAT-012, FR-011 to FR-016)

**Independent Test**: no engine, then fake engines in each mode; refresh without restart.

### Tests for User Story 3

- [ ] T029 [P] [US3] Fake engine `crates/aurora-engines/src/bin/aurora-fake-engine.rs` (feature `fake-engine`; modes from `AURORA_FAKE_MODE`: ready, no-contract, unsupported, invalid, huge, hang, crash; answers as `dcp`, `gedo` or `actx` from its file name)
- [ ] T030 [P] [US3] Core tests of the capability catalog and availability for every feature and status in `crates/aurora-core/src/capabilities.rs`
- [ ] T031 [P] [US3] Runner tests in `crates/aurora-engines/tests/runner.rs` (reply, timeout and kill within the limit, 1 MiB cap, invalid JSON, wrong schema, error output kept, no shell interpretation of arguments)
- [ ] T032 [P] [US3] Probe tests in `crates/aurora-engines/tests/probe.rs` (each status; `--version` fallback; configured path wins over the search path) and `insta` snapshots of parsed version replies

### Implementation for User Story 3

- [ ] T033 [US3] Capability catalog and availability evaluation in `crates/aurora-core/src/capabilities.rs` (data-model.md table)
- [ ] T034 [US3] Engine use cases (`engine_status`, `overview`) in `crates/aurora-core/src/usecases/engines.rs`
- [ ] T035 [US3] Command runner in `crates/aurora-engines/src/runner.rs` (research R5)
- [ ] T036 [US3] Version contract types in `crates/aurora-engines/src/contract.rs` (contracts/engine-version.schema.json)
- [ ] T037 [US3] `EngineProbe` in `crates/aurora-engines/src/probe.rs` (research R6), logging command, duration and outcome
- [ ] T038 [US3] Commands `engines_overview` and `refresh_engines` with parallel probes and the `engine-report` and `engines-ready` events in `crates/aurora-desktop/src/commands.rs` and `src/events.rs`
- [ ] T039 [US3] Engines view (status per engine, version, capabilities, enabled features, hints, last error output, refresh) in `crates/aurora-ui/src/views/engines.rs`
- [ ] T040 [P] [US3] Manual tests "Foundations: engines" in `tests/MANUAL_TESTS.md` (refresh after installing a fake engine, hang mode)

**Checkpoint**: US3 acceptance scenarios 1 to 6 pass.

---

## Phase 6: User Story 4 - Develop and demonstrate views without the engines (Priority: P4)

**Goal**: the interface preview and engine-free tests (FEAT-006, FR-017, FR-018)

**Independent Test**: `trunk serve` shows every view; tests pass with no engine installed.

- [ ] T041 [US4] Fixture backend completeness: every command of contracts/desktop-commands.md answered from the core fixtures, including events simulated after a short delay, in `crates/aurora-ui/src/backend/fixture.rs`, with a unit test listing the commands
- [ ] T042 [US4] Preview banner ("sample data") shown only when the fixture backend is active, in `crates/aurora-ui/src/app.rs`

**Checkpoint**: US4 acceptance scenarios 1 and 2 pass.

---

## Phase 7: Polish & Cross-Cutting Concerns

- [ ] T043 CI workflow `.github/workflows/ci.yml` on Windows and Linux (research R10), including the release binary size report (FEAT-004)
- [ ] T044 [P] Edit in place `docs/architecture/core.md`, `engines.md`, `desktop.md`, `ui.md`; run `build_index.py` and `check_docs.py`
- [ ] T045 [P] Rewrite `README.md` with `user-docs` and run `check_readme.py`
- [ ] T046 Validate the manual suite (`manual_tests.py validate`) and run quickstart.md on Linux
- [ ] T047 Run `check_design.py`, the software-design checklist, and the security review of the capability file and CSP

---

## Dependencies & Execution Order

- Phase 1 → Phase 2 → stories. US1 first (MVP); US2 and US3 depend only on Phase 2 and can run in parallel; US4 completes the fixture backend after US2 and US3 commands exist; Phase 7 last, except T043, which can start right after Phase 1.
- Within a story, tests come first and must fail before the code that makes them pass.

## Milestones (plan file)

M1 Setup (Phase 1) · M2 Foundation (Phase 2) · M3 Window and navigation (US1) · M4 Projects (US2) · M5 Engines (US3) · M6 Preview (US4) · M7 CI and docs (Phase 7)
