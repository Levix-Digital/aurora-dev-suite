---
description: "Tasks of Messages like a mailbox (v0.3.0, ADJ-006, ADJ-007, IMP-007)"
---

# Tasks: Messages like a mailbox

**Input**: `specs/010-messages-mail/spec.md`, `docs/plans/messages-mailbox.md`
**Tests**: tests before the code they prove (constitution IV).

## Phase 1: Provider (M1)

- [ ] T001 [US2] [US4] Scenarios, then `export` with status notes taken out of the body in `crates/aurora-integrations`
- [ ] T002 [US3] Tests, then `Record.recommended` and the `Recommended:` hint in the agreement and impasse templates
- [ ] T003 Capability `export`, usage text, `specs/008-messages/contracts/cli.md` and the JSON schema (1.2)

## Phase 2: Core and adapter (M2)

- [ ] T004 [US1] Tests, then the five statuses and `Conversation::route()`
- [ ] T005 [US1] Tests, then the merge of a thread seen by two registered projects
- [ ] T006 [US2] Tests, then `Message.notes`, `lead()`, `next_step()`, `handled()`
- [ ] T007 [US4] `MessagesSnapshot`, the port's `export` with its fallback, the adapter for `export`
- [ ] T008 [US4] `messages_poll_seconds` with the conversion of the minutes setting

## Phase 3: Worker (M3)

- [ ] T009 [US4] Tests, then the worker: jobs, refresh merging, ordered writes, snapshot file, event
- [ ] T010 [US4] Adaptive interval and focus refresh; commands `messages_snapshot`, `messages_refresh`, `messages_act`

## Phase 4: View (M4)

- [ ] T011 [US1] List with tabs, status bars, filter, legend, unread dot; navigation `N active`
- [ ] T012 [US2] Conversation panel like an e-mail thread, Handled chip, Next step, collapsed forms
- [ ] T013 [US3] Decision card with Undo and Something else
- [ ] T014 Settings field in seconds; fixture backend; preview screenshots

## Phase 5: Docs and manual tests (M5)

- [ ] T015 ADR-0012, architecture pages, README Messages section
- [ ] T016 MT-MSG-003, MT-MSG-005 updated; MT-MSG-009, MT-MSG-010 added; validated
