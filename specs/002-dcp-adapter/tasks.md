---
description: "Tasks of the dcp adapter feature (v0.2.0, FEAT-016)"
---

# Tasks: dcp adapter

**Input**: Design documents from `/specs/002-dcp-adapter/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/
**Tests**: required by the constitution (principle IV.2, evidence); test tasks come before the code they prove.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: can run in parallel (different files, no dependency on an unfinished task)
- **[Story]**: US1 to US3 of spec.md

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: the contract and its samples, which everything else follows

- [x] T001 Add the sample replies under `crates/aurora-engines/tests/fixtures/dcp-diagnose/` (`busy.json`, `idle.json`, `release-candidate.json`, `unknown-fields.json`), each with `schema` `dcp.diagnose` and `schema_version` `1.0`
- [x] T002 Add `jsonschema` as a dev-dependency of `aurora-engines` (default features off) and a test in `crates/aurora-engines/tests/contract_samples.rs` that validates every sample against `specs/002-dcp-adapter/contracts/dcp-diagnose.schema.json` and catches broken samples; allow MIT-0 in `deny.toml` and `TECH_SPEC.md` (approved by Levi, 2026-10-30) and run `cargo deny check`

**Checkpoint**: the samples are valid against the schema.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: the core types and port every story uses

- [x] T003 Add `crates/aurora-core/src/domain/cycle.rs` with `CycleState`, `ItemRef`, `UntriagedBug`, `CurrentRelease`, `PlanProgress`, `CandidateRun`, `PullRequestState`, `NextStep`, `NextOption` and `CycleError` (data-model.md), re-exported from `domain/mod.rs`; unit test that every `CycleError` variant has a non-empty message
- [x] T004 Add the `CycleSource` port to `crates/aurora-core/src/ports.rs`

**Checkpoint**: `cargo build --workspace` and `cargo xtask arch-check` pass.

---

## Phase 3: User Story 1 - Read a project's cycle state through dcp (Priority: P1) 🎯 MVP

**Goal**: a ready `dcp` gives Aurora the full cycle state of a project.

**Independent Test**: the adapter reads each sample reply from the fake engine into the expected `CycleState`.

- [x] T005 [US1] Write the adapter tests in `crates/aurora-engines/tests/cycle.rs` for a valid reply (equal to the core's busy sample), unknown fields ignored, an empty state, a path with spaces and non-ASCII characters, warnings on standard error with exit 0
- [x] T006 [US1] Teach the fake engine (`crates/aurora-engines/src/bin/aurora-fake-engine.rs`) `diagnose --json --dir <root>`: in `ready` mode it prints `busy.json` with `root` replaced by the given root; add `diagnose` (schema_major 1) to its `dcp` version reply
- [x] T007 [US1] Implement `DcpCycleSource` and the reply parser in `crates/aurora-engines/src/cycle.rs` (runner with the `dcp` time limit, envelope and MAJOR check, typed parsing, root check with the registry's canonical comparison, logging of id, duration and outcome)

**Checkpoint**: T005 tests pass.

---

## Phase 4: User Story 2 - Know why the cycle state is not available (Priority: P2)

**Goal**: every failure comes back as exactly one reason, and nothing runs when it cannot succeed.

**Independent Test**: each failing situation gives its reason.

- [x] T008 [P] [US2] Write the core tests in `crates/aurora-core/src/usecases/cycle.rs` for the gate: `Unavailable` for missing, without contract, degraded, without `diagnose`, `diagnose` at another MAJOR (a counting source proves the port is not called); `ProjectMissing`; pass-through otherwise
- [x] T009 [US2] Implement `usecases::cycle::cycle_state` and the dcp requirement lookup in `crates/aurora-core/src/capabilities.rs`
- [x] T010 [P] [US2] Add the failure tests to `crates/aurora-engines/tests/cycle.rs`: `hang` (timed out within limit plus one second), `crash` (failed with error output), `huge` (too large), `invalid`, `unsupported` (newer and older), reply for another root; make the fake engine's existing modes apply to `diagnose`
- [x] T011 [US2] Complete the error mapping in `crates/aurora-engines/src/cycle.rs` until T010 passes

**Checkpoint**: T008 and T010 tests pass.

---

## Phase 5: User Story 3 - Build the views before dcp ships the contract (Priority: P3)

**Goal**: the tests, the desktop and the preview all reach the cycle state through the same contract.

**Independent Test**: the preview and the desktop command answer for every sample project.

- [x] T012 [US3] Add `FixtureCycleSource` to `crates/aurora-core/src/fixtures.rs`, built from the same sample states (busy, idle, release candidate) plus the failures `TimedOut`, `Invalid` and `Unsupported`, keyed by sample project root; test that every sample project has an answer
- [x] T013 [US3] Add `project_cycle_state` to `crates/aurora-desktop/src/commands.rs` and the cycle source to `state.rs`; allow it in `capabilities/default.json`; test that a `CycleError` keeps its kind in the command error
- [x] T014 [US3] Add `project_cycle_state` to both backends in `crates/aurora-ui/src/backend/` (Tauri and fixture); test that the fixture backend answers every sample project

**Checkpoint**: `cargo test --workspace --all-features` passes; the preview builds.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [x] T015 [P] Edit in place `docs/architecture/core.md`, `engines.md`, `desktop.md` and `ui.md`; regenerate the index (`build_index`) and run `check_docs`
- [x] T016 Send the contract to `dev-cycle-protocol` through the inbox tool (plan, "Cross-project request")
- [x] T017 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -D warnings`, `cargo test --workspace --all-features`, `cargo deny check`, `cargo xtask arch-check` and the quickstart

---

## Dependencies & Execution Order

- Phase 1 before everything (the samples are the contract's evidence); Phase 2 before the stories.
- US1 (T005 to T007) before US2's adapter tasks (T010, T011); the core gate (T008, T009) only needs Phase 2.
- US3 needs US1 and US2 (the command uses the use case and the adapter).

## Milestones (plan file)

- M1 Contract and core (Phases 1 and 2)
- M2 Adapter (Phases 3 and 4)
- M3 Desktop and preview (Phase 5)
- M4 Docs and contract request (Phase 6)
