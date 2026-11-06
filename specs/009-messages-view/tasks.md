---
description: "Tasks of the Messages view adjustments (v0.3.0, ADJ-002)"
---

# Tasks: Messages view adjustments

**Input**: Design documents from `/specs/009-messages-view/`
**Prerequisites**: plan.md, spec.md
**Tests**: required by the constitution (principle IV.2, evidence); test tasks come before the code they prove.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: can run in parallel (different files, no dependency on an unfinished task)
- **[Story]**: US1 to US4 of spec.md

---

## Phase 1: Core and adapter (M1)

- [ ] T001 [US2] Tests for `Conversation::state()` (waiting, in progress, decision, cycle 2 after a revision, notice only, tool 0.1) in `crates/aurora-core/src/domain/messages.rs`
- [ ] T002 [US2] `ConversationState` and `Conversation::state()` in `crates/aurora-core/src/domain/messages.rs`
- [ ] T003 [US4] Tests, then `Conversation.unread` (serde default), `ProjectInbox::open_count()` and `open_total()` in the core
- [ ] T004 [US1] [US2] [US4] Samples in `crates/aurora-core/src/fixtures/messages.rs` covering the three states and unread messages
- [ ] T005 [US4] Test, then `unread` per thread from `check`'s `messages` in `crates/aurora-engines/src/messages.rs`

## Phase 2: View (M2)

- [ ] T006 [US1] `from → to` first on rows and message headers in `crates/aurora-ui/src/views/messages.rs`
- [ ] T007 [US2] Groups Waiting for you, In progress and Closed (collapsed `<details>`), with their colors in `crates/aurora-ui/styles/app.css`
- [ ] T008 [US3] The conversation panel inside the row's item, one at a time, `aria-expanded`, focus in and out, Escape closes
- [ ] T009 [US4] The navigation count from `open_total` in `crates/aurora-ui/src/app.rs`, at start and after every poll
- [ ] T010 Preview check by screenshots (groups, inline panel, count on Portfolio)

## Phase 3: Docs and manual tests (M3)

- [ ] T011 [P] `docs/architecture/core.md` and `ui.md`
- [ ] T012 [P] README Messages section
- [ ] T013 MT-MSG-003 and MT-MSG-005 in `tests/MANUAL_TESTS.md`, checked with `quality-assurance manual_tests validate`
- [ ] T014 Mark the milestones done in `docs/plans/messages-view-adjustments.md`

## Dependencies

Phase 1 → Phase 2 → Phase 3 (T011 and T012 may start with Phase 2).
