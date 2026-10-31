---
description: "Tasks of the Project settings feature (v0.3.0, FEAT-027)"
---

# Tasks: Project settings

**Input**: Design documents from `/specs/007-project-settings/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/
**Tests**: required by the constitution (principle IV.2, evidence); test tasks come before the code they prove.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: can run in parallel (different files, no dependency on an unfinished task)
- **[Story]**: US1 to US3 of spec.md

---

## Phase 1: Foundational (core and contracts)

- [ ] T001 [US1] Write the sample replies in `crates/aurora-engines/tests/fixtures/dcp-config/` (get, preview, applied, refused, stale) and validate them against the schemas in `tests/contract_samples.rs`
- [ ] T002 [US2] Write the tests of the checks, input parsing, draft and errors in `crates/aurora-core/src/domain/config.rs`
- [ ] T003 [US2] Implement the config domain in `crates/aurora-core/src/domain/config.rs`, re-exported from `domain/mod.rs`
- [ ] T004 [US3] Add `Feature::ProjectSettings`, the capability `config` and `usable_dcp_for` in `capabilities.rs`, with tests
- [ ] T005 [US1] Add the `ConfigSource` port, the use cases read, preview and apply in `usecases/config.rs` with tests, and the sample settings with `FixtureConfigSource` in `fixtures.rs`

**Checkpoint**: `cargo test -p aurora-core --all-features` and `cargo xtask arch-check` pass.

---

## Phase 2: Adapter and desktop

- [ ] T006 [US1] Move running `dcp` and the envelope checks to `crates/aurora-engines/src/dcp.rs`, used by `cycle.rs` unchanged
- [ ] T007 [US1] Teach the fake engine `config get` and `config set` with modes; write `tests/config.rs`
- [ ] T008 [US1] Implement `DcpConfigSource` in `crates/aurora-engines/src/config.rs`
- [ ] T009 [US1] Add the commands `project_config`, `preview_project_config`, `apply_project_config` in `aurora-desktop` (commands, state, build permissions, capability file)

---

## Phase 3: User Stories 1 to 3 - the Settings tab

- [ ] T010 [US1] Add the three calls to `crates/aurora-ui/src/backend/` with fixture answers and the contract test
- [ ] T011 [US1] Add the tabs to the Project view and the Settings tab in `crates/aurora-ui/src/views/project_settings.rs`, with styles
- [ ] T012 [US2] Edit, preview, apply, back and discard; errors next to fields; stale revision
- [ ] T013 [US3] Unavailable and failure states with the shared text

---

## Phase 4: Polish

- [ ] T014 Manual tests MT-SET-001 to MT-SET-004 in `tests/MANUAL_TESTS.md`
- [ ] T015 [P] Architecture pages and README
- [ ] T016 Check the preview with the quickstart; `cargo fmt`, `clippy`, tests, `xtask arch-check`
