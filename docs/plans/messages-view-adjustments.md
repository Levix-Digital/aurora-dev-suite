---
plan_id: messages-view-adjustments
project: aurora-dev-suite
title: Messages view adjustments
kind: feature
version: v0.3.0
status: in_progress
author: LeviGuilherme
created_at: '2026-11-06T16:46:02'
updated_at: '2026-11-06T16:53:00'
session_id: null
tags: []
supersedes: null
superseded_by: null
parent_plan_id: null
backlog_items:
- ADJ-002
spec_dir: specs/009-messages-view
milestones:
- name: M1 Core and adapter
  done: true
  done_at: '2026-11-06'
  by: LeviGuilherme
- name: M2 View
  done: false
- name: M3 Docs and manual tests
  done: false
status_log:
- at: '2026-11-06T16:46:02'
  by: LeviGuilherme
  status: draft
  note: Plan created
- at: '2026-11-06T16:47:31'
  by: LeviGuilherme
  status: active
  note: Plan written, waiting for gate 3.
- at: '2026-11-06T16:49:58'
  by: LeviGuilherme
  status: in_progress
  note: Approved by Levi at gate 3 with Spec Kit (2026-11-06).
- at: '2026-11-06T16:53:00'
  by: LeviGuilherme
  note: Milestone 'M1 Core and adapter' completed
---

# Messages view adjustments

Plan for the backlog items below (`dev-cycle-protocol` Phases 1 and 2). Replace every instruction paragraph with the plan.

## Backlog items

### ADJ-002: Messages view: From → To, states by color, inline conversation, open count
- **Priority**: high
- **Found**: 2026-11-06, v0.3.0-rc.1
- **Description**: Levi, 2026-11-06, testing v0.3.0-rc.1: each message and row shows sender → receiver; conversations grouped in Waiting for you, In progress and Closed (muted, collapsed), each with its color; a conversation opens right below its row, not at the bottom of the page; the navigation count shows what waits for the user plus received messages not read yet, on every screen.

## 1. Goal and decisions

Levi tested v0.3.0-rc.1 and asked for a Messages view that reads at a glance (2026-11-06; agreed on the details the same day):

- **Who sends to whom**: every message and every conversation row shows `sender → receiver` (the slugs), first, before the subject.
- **Conversations, not messages**: the list keeps one row per thread and says so: who opened it, the projects in it, how many messages it has.
- **State by group and color**: three groups, in this order: **Waiting for you** (warning color: a closing record waits for the user), **In progress** (normal color: open, rounds still running or an answer pending), **Closed** (muted, collapsed by default: the last message of the current cycle is a decision, or the thread holds only a notice or acks).
- **Inline conversation**: a conversation opens right below its row (one at a time, like an accordion), not at the bottom of the page; Escape or the row again closes it; focus moves to the opened panel and back to the row on close. A popup was considered and left out: it hides the list and is harder with the keyboard.
- **The navigation count**: next to Messages on every screen, the number of conversations that wait for the user plus those with received messages not read yet; computed when Aurora starts and after every poll (it already starts at launch; today it counts only what waits for the user).

No ADR: the view's layout and a derived state; no contract or interface changes.

## 2. Design (`software-design`)

- **Core (`domain/messages.rs`)**: `ConversationState { WaitingForYou, InProgress, Closed }` and `Conversation::state()`, derived from fields the core already has (`waiting_for_user`, `last.kind`, the cycle). `Conversation` gains `unread: u32` (received messages of the thread with status open, for this project); `ProjectInbox::open_count()` counts conversations that wait for the user or have `unread > 0`. Pure functions, unit tested.
- **Engines (`messages.rs`)**: the adapter fills `unread` from `check --json`, whose `messages` array already lists the received messages still open; counted per `thread`. No provider change.
- **UI (`views/messages.rs`)**: `ConversationList` renders the three groups (the Closed group in a `<details>`, closed by default); each row is a button with `aria-expanded` and, when open, the `ConversationPanel` right after it inside the same list item; the panel at the bottom goes away. Message headers in the panel and rows lead with `from → to`. `app.rs` uses `open_total` instead of `waiting_total` for the count, with an aria label that says what it counts.
- **Fixtures**: the sample conversations cover the three states and unread messages, for the browser preview and the tests.

## 3. Affected surfaces and files

No interface in `architecture.surfaces` changes (`dcp-integrations` JSON and desktop commands keep their shape; `Conversation` gains a field with a default for older payloads).

Files: `crates/aurora-core/src/domain/messages.rs`, `crates/aurora-core/src/fixtures/messages.rs`, `crates/aurora-engines/src/messages.rs` (+ tests), `crates/aurora-ui/src/views/messages.rs`, `crates/aurora-ui/src/app.rs`, `crates/aurora-ui/styles/app.css`, `crates/aurora-ui/src/backend/fixture.rs`.

## 4. Tests (`quality-assurance`)

- Core unit tests: `state()` for each state (waiting, rounds running, decision, cycle 2 after a revision, notice only); `open_count()`.
- Engines: the `check` sample yields `unread` per thread; a thread with nothing open has 0.
- UI: the browser preview shows the three groups, `from → to`, the inline panel and the count (checked by screenshot).
- Manual tests: MT-MSG-003 changes (groups, colors, inline opening, `from → to`); MT-MSG-005 checks the count on another screen. Run in v0.3.0-rc.2.
- Acceptance: the five points of section 1, seen in the release candidate on Windows and Linux.

## 5. Documentation (`architecture-docs`, `user-docs`)

`docs/architecture/ui.md` (the list, groups and inline panel) and `core.md` (state and count); README Messages section (groups, colors, count).

## 6. CI, release and observability (`devops-release`)

No workflow change. Ships in v0.3.0-rc.2 with BUG-005 and BUG-006. No new logs (no new failure mode).

## 7. Security, migration and rollback

No data or settings change; `unread` has a serde default. Rollback: revert the pull request.

## 8. Execution steps and milestones

1. M1 Core and adapter: state, unread, open count, fixtures, tests.
2. M2 View: groups and colors, from → to, inline panel, count in the navigation, preview check.
3. M3 Docs and manual tests: ui.md, core.md, README, MT-MSG-003 and MT-MSG-005.
