---
description: "Tasks of Messages as a chat between projects (v0.3.1, FEAT-039)"
---

# Tasks: Messages as a chat between projects

**Input**: `specs/011-messages-chat/spec.md`, `docs/plans/messages-chat.md`
**Tests**: tests before the code they prove (constitution IV).

## Phase 1: Provider summary (M1)

- [x] T001 [US5] Tests, then `summary` in the front matter (written only when set) and in every header
- [x] T002 [US5] `--summary` on `send` and `reply`, refused above 280 characters; `decide` and `guide` write their own
- [x] T003 Schema 1.3, capability `summary`, `cli.md` and the JSON schema

## Phase 2: Core chat model (M2)

- [ ] T004 [US2] `MessageHeader.summary`, `Draft.summary`, `UserAction::Reply.summary`; the adapter passes `--summary`
- [ ] T005 [US1] Tests, then `chat_contacts`: last summary, time, unread, waiting, order
- [ ] T006 [US2] [US4] Tests, then `chat_stream`: order, days, quotes, marks, record cards, decision messages hidden, subject fallback
- [ ] T007 Fixtures with summaries for the preview

## Phase 3: Chat view (M3)

- [ ] T008 [US1] Contacts column with avatar, last summary, time, unread badge, waiting dot, search
- [ ] T009 [US2] Chat column: bubbles, day separators, quotes, marks, counterpart filter, New messages button
- [ ] T010 [US3] Detail panel sliding in, highlight, Esc, keyboard; reuse of the message body, Reply and Step in
- [ ] T011 [US4] Decision card in the stream, closed state after the decision
- [ ] T012 [US5] Summary field in Reply and New conversation
- [ ] T013 Tokens in both themes and contrast checks; preview screenshots

## Phase 4: Project logo (M4)

- [ ] T014 [US1] `Project.logo`, `pick_project_logo`, `clear_project_logo`; Logo in the Project view; avatars use it

## Phase 5: Docs and manual tests (M5)

- [ ] T015 ADR-0013, architecture page, README Messages section
- [ ] T016 Manual tests MT-MSG rewritten for the chat; validated
