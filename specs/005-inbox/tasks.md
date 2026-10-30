---
description: "Tasks of the Inbox feature (v0.2.0, FEAT-017)"
---

# Tasks: Inbox

**Input**: Design documents from `/specs/005-inbox/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md
**Tests**: required by the constitution (principle IV.2, evidence); test tasks come before the code they prove.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: can run in parallel (different files, no dependency on an unfinished task)
- **[Story]**: US1 to US3 of spec.md

---

## Phase 1: Foundational (core rules)

- [x] T001 [US1] Write the tests of each waiting item's urgency and detail in `crates/aurora-core/src/domain/portfolio.rs` and of the Inbox order, groups and project count in `crates/aurora-core/src/domain/inbox.rs`, over the sample cycle states and hand-built states
- [x] T002 [US1] Implement `Urgency`, `InboxEntry` and `Inbox` in `crates/aurora-core/src/domain/inbox.rs`, the urgency and detail of `WaitingItem` in `portfolio.rs`, re-exported from `domain/mod.rs`

**Checkpoint**: `cargo test -p aurora-core --all-features` and `cargo xtask arch-check` pass.

---

## Phase 2: User Story 1 - See everything that waits for me (Priority: P1) 🎯 MVP

- [x] T003 [US1] Move the row state, `read` and `run` from `crates/aurora-ui/src/views/portfolio.rs` to `crates/aurora-ui/src/views/reads.rs`, used by the Portfolio unchanged
- [x] T004 [US1] Implement the Inbox header, groups and items in `crates/aurora-ui/src/views/inbox.rs`, with styles in `crates/aurora-ui/styles/app.css`

---

## Phase 3: User Story 2 - Go to the project of an item (Priority: P2)

- [x] T005 [US2] "Open" on each item through `AppContext::open_project` in `crates/aurora-ui/src/views/inbox.rs`

---

## Phase 4: User Story 3 - Refresh and know what could not be read (Priority: P3)

- [x] T006 [US3] Refresh keeping the last items, the projects still being read, "Not read" through `CycleFailure` and one line when `dcp` is unavailable, in `crates/aurora-ui/src/views/inbox.rs`

---

## Phase 5: Polish

- [x] T007 [P] Manual tests MT-INB-001 to MT-INB-004 in `tests/MANUAL_TESTS.md`; check with the quality-assurance validator
- [x] T008 [P] Architecture pages `core.md` and `ui.md`, and the README
- [x] T009 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-features`, `cargo xtask arch-check`, the interface build (`trunk build`) and the quickstart
