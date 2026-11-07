---
plan_id: messages-mailbox
project: aurora-dev-suite
title: Messages like a mailbox
kind: feature
version: v0.3.0
status: in_progress
author: LeviGuilherme
created_at: '2026-11-07T14:59:13'
updated_at: '2026-11-07T15:14:18'
session_id: null
tags: []
supersedes: null
superseded_by: null
parent_plan_id: null
backlog_items:
- ADJ-006
- ADJ-007
- IMP-007
spec_dir: specs/010-messages-mail
milestones:
- name: M1 Provider
  done: true
  done_at: '2026-11-07'
  by: LeviGuilherme
- name: M2 Core and adapter
  done: true
  done_at: '2026-11-07'
  by: LeviGuilherme
- name: M3 Worker
  done: true
  done_at: '2026-11-07'
  by: LeviGuilherme
- name: M4 View
  done: false
- name: M5 Docs and manual tests
  done: false
status_log:
- at: '2026-11-07T14:59:13'
  by: LeviGuilherme
  status: draft
  note: Plan created
- at: '2026-11-07T15:00:34'
  by: LeviGuilherme
  status: in_progress
  note: Design agreed with Levi on mockups (list, conversation, decision card, worker, adaptive interval), 2026-11-07.
- at: '2026-11-07T15:04:21'
  by: LeviGuilherme
  note: 'Milestone ''M1 Provider'' completed: export, notes, recommended'
- at: '2026-11-07T15:09:39'
  by: LeviGuilherme
  note: Milestone 'M2 Core and adapter' completed
- at: '2026-11-07T15:14:18'
  by: LeviGuilherme
  note: Milestone 'M3 Worker' completed
---

# Messages like a mailbox

Plan for the backlog items below (`dev-cycle-protocol` Phases 1 and 2). Specification and tasks: `specs/010-messages-mail/`.

## Backlog items

### ADJ-006: Messages list reads like a mailbox: Active and Closed tabs, status colors, active count
- **Priority**: high
- **Found**: 2026-11-07, v0.3.0-rc.5
- **Description**: Levi, 2026-11-07: the list was noisy. Each row shows opener → counterpart (larger, accent), the subject below and one muted line 'Last: <kind> · date · who'; opened by, cycle, round, message count and No answer yet go. Active and Closed are tabs with counts. The left bar's color is the status: Waiting for you (amber), Awaiting reply (blue), Answered (purple), Deferred (gray), Closed (green, own tab); a status filter and a small legend sit next to the project filter. An unread dot marks conversations with unread messages. The navigation shows 'N active' (conversations not closed), with an amber dot when one waits for the user. Mockup: mockup-list.png.

### ADJ-007: Opened conversation reads like an e-mail thread, with Handled, Next step and a decision card
- **Priority**: high
- **Found**: 2026-11-07, v0.3.0-rc.5
- **Description**: Levi, 2026-11-07: header with opener → counterpart large, the subject, the status pill, 'N messages · round R of 3' and 'Next: <latest Next step>'. Messages grouped like e-mail: the newest open, older ones collapsed to one line (from → to, kind, date, preview). The Ask section becomes the unlabelled lead sentence; Next step in an amber callout; Authority and Open points folded. The '> done (...)' notes leave the body and become one 'Handled by <project> · date' chip (Done read as the end of the conversation). Reply and Step in forms collapsed behind two buttons. A closing record shows a decision card: one button per numbered option (Recommended marked from a 'Recommended: <n>' line), sent on click with a 5 s Undo, 'Something else…' opens a text box (decide --choice) with a link to ask for rework (--revise). Mockups: mockup-conversation.png, mockup-decision-card.png.

### IMP-007: Messages open like e-mail: one worker, a local snapshot and an ordered write queue
- **Priority**: high
- **Found**: 2026-11-07, v0.3.0-rc.5
- **Description**: Levi, 2026-11-07: reading and opening messages still felt slow (rc.5 predates PR #39). One worker in Aurora owns every provider call: it synchronizes in the background and reads everything at once with a new provider command, export --cached (conversations and message bodies in one JSON); the view reads only a snapshot kept in memory and on disk, so the list and a conversation show at once, also right after start; the worker sends only what changed; writes (reply, decide, mark seen, defer) go to one ordered queue, shown at once in the view and rolled back with the error if they fail. Git stays the source of truth; the snapshot is a cache (ADR-0012).

## 1. Goal and decisions

Levi tested v0.3.0 on 2026-11-07 and asked for a Messages view that reads like a mailbox and opens like e-mail. Every point below was agreed with him on mockups the same day (project files `aurora/v0.3.0-messages-adjustments/`).

- **List (ADJ-006)**: one row per conversation: `opener → counterpart` first (larger, accent color), the subject below, one muted line `Last: <kind> · <date> · <who>`. Everything else on the row goes (opened by, cycle, round, message count, badges). Tabs **Active** and **Closed** with their counts. The left bar's color is the status, with a status filter and a small legend next to the project filter.
- **Statuses**, derived in the core from data the provider already returns: **Waiting for you** (amber; a closing record waits for the user), **Awaiting reply** (blue; the last message asks for an answer: question, proposal, counter, guidance), **Answered** (purple; the last message answers, the opener continues or closes), **Deferred** (gray; the receiver deferred the last message), **Closed** (green; decision, ack, a notice outside rounds).
- **Navigation**: `Messages · N active`, N = conversations not closed, with an amber dot when one waits for the user.
- **One row per conversation**: a thread between two projects that are both registered in Aurora appears once (today once per project).
- **Opened conversation (ADJ-007)**: header with `opener → counterpart` large, the subject, the status pill, `N messages · round R of 3` and `Next: <the latest Next step>`. Messages like an e-mail thread: the newest open, older ones collapsed to one line (from → to, kind, date, preview) that a click opens. The Ask section is the unlabelled lead sentence; Next step sits in an amber callout; Authority and Open points are folded; other sections keep small headings. Reply and Step in are two buttons that open their forms.
- **Handled instead of done**: the provider appends `> done (<date>): <note>` to a message body when its receiver handles it. The view takes these notes out of the body and shows one chip `✓ Handled by <receiver> · <date>`. Levi: "done" read as the end of the conversation, which it is not.
- **Decision card**: a closing record shows one button per numbered option of "Decision needed", the recommended one marked; a click sends the decision after a 5 s Undo window; "Something else…" opens a text box sent as the choice; a link asks the projects to rework it (`--revise`). The provider reads an optional `Recommended: <n>` line of "Decision needed" (or an option ending in "(recommended)") and its agreement and impasse templates ask for it.
- **E-mail feel (IMP-007)**: one worker in the desktop process owns every provider call. It keeps a snapshot of every project's conversations, message bodies and closing records in memory and on disk; the view reads only the snapshot, so the list and a conversation show at once, also right after Aurora starts. The worker brings the snapshot up to date in the background (`sync`, then the new `export --cached`, one provider process per project), and tells the view when something changed; the view renders keyed rows, so only changed rows redraw. Writes (send, reply, decide, step in, mark read) go to one ordered queue in the same worker; the view shows them at once and rolls back with the error when the provider refuses. Git stays the source of truth; the snapshot is a cache that can be deleted at any time. ADR-0012.
- **Adaptive interval** (Levi, 2026-11-07): every 20 s while the window has the focus, every 1 min in the background, at once when the window gets the focus (at most once per 20 s) and after each write. The setting moves from minutes to seconds (minimum 10 s, 0 turns the background updates off).

Alternatives left out: a database such as SQLite (a second source of truth, against ADR-0010 and ADR-0011); pushing the whole snapshot through each event (the event carries only a version, the view asks for the snapshot, which costs one in-process read); a filesystem watcher (messages arrive through the network, not the disk).

## 2. Design (`software-design`)

- **Provider** (`crates/aurora-integrations`): new command `export [--cached]`, schema `dcp-integrations.export` 1.x: the project, its counterparts (slug, name, url), every conversation as in `check` plus `unread`, its messages with sections, the body without status notes, and the notes (`status`, `date`, `note`), and the closing record of each conversation that has one. Capability `export` in `version`. `Record` gets `recommended: Option<usize>` (1-based). Status notes parsed by one function shared by `export` and `thread`.
- **Core** (`aurora-core`): `ConversationState` gets `AwaitingReply`, `Answered`, `Deferred` (replacing `InProgress`). `MessagesSnapshot { version, taken_at, inboxes, threads }` with the threads' messages and records; `Message` gets `notes: Vec<StatusNote>` and helpers (`lead`, `next_step`, `handled`); `Conversation::route()`; `merge_conversations` (one row per thread, keeping the inbox of the project that acts). Port `MessagesSource::export` with a default built from `inbox`, `thread` and `record`, so a provider without `export` still works.
- **Desktop** (`aurora-desktop`): `MessagesWorker` (one thread, one channel of jobs: refresh, act, stop). State: the snapshot behind an `RwLock`, persisted to `messages-snapshot.json` in Aurora's configuration folder after each change (written to a temporary file, then renamed). Commands: `messages_snapshot` (memory read), `messages_refresh` (Check now: a refresh job, returns what changed), `messages_act` (an act job, waits for its result, then a refresh of the projects involved). Event `messages-snapshot` with the new version. The timer and the focus hook send refresh jobs; a refresh while one runs is merged.
- **View** (`aurora-ui`): the list and the conversation read the snapshot; `<For>` keyed by thread and last message id; optimistic changes (read marks, sent replies shown as "Sending…", decisions) applied to a local copy and dropped when the next snapshot arrives or the action fails.
- **Errors**: a project the worker cannot read keeps its last snapshot entry, marked with the error and its time; the view shows a muted note on that project, not a blocking banner.

## 3. Affected surfaces and files

- `dcp-integrations` CLI: new `export`, `version` capability `export`, record field `recommended`; templates of agreement and impasse mention `Recommended:`. Additive, schema 1.2.
- Desktop commands: `messages_snapshot`, `messages_refresh` added; `messages_overview`, `messages_conversation`, `messages_record`, `messages_poll` removed from the view (kept until the view no longer calls them, then removed). Event `messages-snapshot` replaces `messages-news`.
- Settings: `messages_poll_seconds` (default 20) replaces `messages_poll_minutes` (read once and converted).
- Files: `crates/aurora-integrations/src/{commands,rules,format}.rs`, `bin/dcp-integrations.rs`; `crates/aurora-core/src/domain/{messages,settings}.rs`, `ports.rs`, `usecases/messages.rs`, `fixtures/messages.rs`; `crates/aurora-engines/src/messages.rs`; `crates/aurora-desktop/src/{messages,state,main}.rs`; `crates/aurora-ui/src/{app.rs,views/messages.rs,views/settings.rs,backend/*}`, `styles/app.css`; contracts in `specs/008-messages/contracts/`.

## 4. Tests (`quality-assurance`)

- Provider: scenarios for `export` (bodies, notes taken out, unread, records), `recommended` from both spellings.
- Core: every status, merge of a thread seen by two projects, lead / next step / handled, snapshot diff (what changed), settings conversion.
- Engines: `export` parsed from a contract sample; the fallback without the capability.
- Desktop: the worker's queue order, refresh merging, the snapshot written and read back, the adaptive interval.
- View: preview screenshots of the list, the conversation and the card (fixture backend).
- Manual tests: MT-MSG-003 and MT-MSG-005 updated; new MT-MSG-009 (open Aurora with the network off: the last snapshot shows at once) and MT-MSG-010 (decision card with Undo).

## 5. Documentation (`architecture-docs`, `user-docs`)

ADR-0012 (the worker and the snapshot); `docs/architecture/{integrations,desktop,ui,core}.md`; README Messages section; `specs/008-messages/contracts/cli.md` and the JSON schema.

## 6. CI, release and observability (`devops-release`)

No workflow change. Logs: each refresh (projects, elapsed ms, changed), each queued action (kind, project, elapsed ms, outcome), snapshot load and write failures. Released in the next v0.3.0 release candidate, cut after the merge (it also brings PR #39, which no published candidate has).

## 7. Security, migration and rollback

The snapshot holds message bodies that are already on disk in the provider's caches; it lives in Aurora's configuration folder with the same permissions. No secret. The old minutes setting is read once and converted. Rollback: revert the pull request; the snapshot file is ignored by older versions.

## 8. Execution steps and milestones

1. M1 Provider: `export`, notes, `recommended`, capability, contracts.
2. M2 Core and adapter: statuses, snapshot, merge, helpers, port and fallback, settings.
3. M3 Worker: snapshot, queue, persistence, events, adaptive interval.
4. M4 View: list, navigation, conversation, decision card, settings field.
5. M5 Docs and manual tests.
