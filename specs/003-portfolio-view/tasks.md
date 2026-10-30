---
description: "Tasks of the Portfolio view feature (v0.2.0, FEAT-014)"
---

# Tasks: Portfolio view

**Input**: Design documents from `/specs/003-portfolio-view/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md
**Tests**: required by the constitution (principle IV.2, evidence); test tasks come before the code they prove.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: can run in parallel (different files, no dependency on an unfinished task)
- **[Story]**: US1 to US3 of spec.md

---

## Phase 1: Foundational (core rules)

- [x] T001 [US1] Write the tests of the phase, the waiting items and the summary in `crates/aurora-core/src/domain/portfolio.rs` over the sample cycle states and hand-built states
- [x] T002 [US1] Implement `CyclePhase`, `WaitingKind`, `WaitingItem` and `CycleSummary` in `crates/aurora-core/src/domain/portfolio.rs`, re-exported from `domain/mod.rs`

**Checkpoint**: `cargo test -p aurora-core --all-features` and `cargo xtask arch-check` pass.

---

## Phase 2: User Story 1 - See where every project stands (Priority: P1) 🎯 MVP

- [x] T003 [US1] Show the cycle summary in each Portfolio row (phase badge, release, waiting count with kinds, findings note) and the header total in `crates/aurora-ui/src/views/portfolio.rs`, with styles in `crates/aurora-ui/styles/app.css`

---

## Phase 3: User Story 2 - Projects appear as they are read (Priority: P2)

- [x] T004 [US2] Write the loader tests (limit, every project answered once, any order, stale and removed answers dropped) in `crates/aurora-ui/src/views/loader.rs`
- [x] T005 [US2] Implement the loader in `crates/aurora-ui/src/views/loader.rs` and use it from the Portfolio, with Refresh keeping the last state
- [x] T006 [P] [US2] Make concurrent first calls of `engines_overview` share one probe in `crates/aurora-desktop/src/commands.rs` and `state.rs`, with a unit test

---

## Phase 4: User Story 3 - Know why a project has no state (Priority: P3)

- [x] T007 [US3] Render each failure reason in the row (unavailable without the hint, missing with "locate", the others with the reason and the error output in a disclosure) in `crates/aurora-ui/src/views/portfolio.rs`

---

## Phase 5: Polish

- [x] T008 [P] Manual tests MT-POR-001 to MT-POR-004 in `tests/MANUAL_TESTS.md`; check with the quality-assurance validator
- [x] T009 [P] Architecture pages `core.md`, `ui.md`, `desktop.md` and the README
- [x] T010 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-features`, `cargo xtask arch-check`, the interface build (`trunk build`) and the quickstart
