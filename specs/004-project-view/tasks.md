---
description: "Tasks of the Project view feature (v0.2.0, FEAT-015)"
---

# Tasks: Project view

**Input**: Design documents from `/specs/004-project-view/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md
**Tests**: required by the constitution (principle IV.2, evidence); test tasks come before the code they prove.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: can run in parallel (different files, no dependency on an unfinished task)
- **[Story]**: US1 to US3 of spec.md

---

## Phase 1: Foundational (core rules)

- [x] T001 [US2] Write the tests of the candidate stage, the gates, the next step order and plan progress in `crates/aurora-core/src/domain/project_view.rs` over the sample cycle states and hand-built states
- [x] T002 [US2] Implement `CandidateStage`, `GateKind`, `Gate`, `NextStep::ordered` and the plan progress in `crates/aurora-core/src/domain/project_view.rs`, re-exported from `domain/mod.rs`; the candidate's waiting title in `portfolio.rs` follows the stage

**Checkpoint**: `cargo test -p aurora-core --all-features` and `cargo xtask arch-check` pass.

---

## Phase 2: User Story 1 - See one project's whole cycle (Priority: P1) 🎯 MVP

- [x] T003 [US1] Add the chosen project to `AppContext` in `crates/aurora-ui/src/app.rs` and "Open" on each Portfolio row in `crates/aurora-ui/src/views/portfolio.rs`
- [x] T004 [US1] Implement the Project view sections (overview, waiting, plans, roadmap, release candidate, pull requests, bugs) in `crates/aurora-ui/src/views/project.rs`, with styles in `crates/aurora-ui/styles/app.css`

---

## Phase 3: User Story 2 - Know which decision is the user's (Priority: P2)

- [x] T005 [US2] Show the gates and the next step in `crates/aurora-ui/src/views/project.rs`

---

## Phase 4: User Story 3 - Switch, refresh and understand failures (Priority: P3)

- [x] T006 [US3] Move the failure text and engine output of a row into `CycleFailure` in `crates/aurora-ui/src/views/mod.rs`, used by the Portfolio
- [x] T007 [US3] Project choice, Refresh keeping the last state, latest answer wins and failures through `CycleFailure` in `crates/aurora-ui/src/views/project.rs`

---

## Phase 5: Polish

- [x] T008 [P] Manual tests MT-PVW-001 to MT-PVW-004 in `tests/MANUAL_TESTS.md`; check with the quality-assurance validator
- [x] T009 [P] Architecture pages `core.md` and `ui.md`, and the README
- [x] T010 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-features`, `cargo xtask arch-check`, the interface build (`trunk build`) and the quickstart
