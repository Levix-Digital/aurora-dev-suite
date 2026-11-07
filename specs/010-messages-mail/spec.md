# Feature Specification: Messages like a mailbox

**Feature Branch**: `feat/010-messages-mail`
**Created**: 2026-11-07
**Status**: Agreed with the user on mockups (2026-11-07)
**Input**: Backlog ADJ-006, ADJ-007, IMP-007; plan `docs/plans/messages-mailbox.md`

## User Scenarios

### US1 - See who talks to whom and where each conversation stands (P1)

The user opens Messages and, at a glance, sees each conversation as `opener → counterpart`, its subject and its status by color, with active and closed conversations in two tabs.

**Acceptance**
1. Each row shows `opener → counterpart`, the subject and one line `Last: <kind> · <date> · <who>`, nothing else.
2. The bar on the left of a row has the status color: Waiting for you, Awaiting reply, Answered, Deferred, Closed.
3. Tabs Active and Closed show their counts; a status filter and a legend sit next to the project filter.
4. A thread between two projects both registered in Aurora shows once.
5. A conversation with unread received messages has an unread dot.
6. The navigation shows `Messages · N active` and an amber dot when one waits for the user.

### US2 - Read a conversation like an e-mail thread (P1)

**Acceptance**
1. The header shows `opener → counterpart` large, the subject, the status, `N messages · round R of 3` and `Next: ...` from the latest message with a Next step.
2. The newest message is open; older ones are one line each (from → to, kind, date, preview) and open on a click.
3. Ask is shown as the lead sentence without a label; Next step in a highlighted box; Authority and Open points folded.
4. Status notes leave the body: a handled message shows `✓ Handled by <receiver> · <date>`.
5. Reply and Step in forms are closed until their button is pressed.

### US3 - Decide with one click (P1)

**Acceptance**
1. A closing record waiting for the user shows a card with one button per option; the recommended one is marked.
2. A click shows "sending in 5 s" with Undo; without Undo the decision is sent.
3. "Something else…" opens a text box and Send decision; a link asks the projects to rework it.

### US4 - Messages are simply there (P1)

**Acceptance**
1. Opening the Messages view or a conversation shows content at once from the snapshot, with no provider call on the way.
2. After a restart, the last snapshot shows at once, also without network.
3. New messages appear by themselves: every 20 s with the window in focus, 1 min in the background, at once on focus and after each write.
4. Writes run in order; a refused write is rolled back in the view with the provider's reason.

## Requirements

- **FR-001** The provider has `export [--cached]` (schema `dcp-integrations.export`, capability `export`) returning, for one project, its counterparts, its conversations with unread counts, every message with sections, body without status notes and the notes, and the closing records.
- **FR-002** The closing record has `recommended` (1-based) from a `Recommended: <n>` line of Decision needed or an option ending with "(recommended)"; the agreement and impasse templates ask for the line.
- **FR-003** The core derives the five statuses and merges a thread seen by two registered projects into one conversation.
- **FR-004** A desktop worker owns every provider call and keeps the snapshot in memory and in `messages-snapshot.json`; the view reads only the snapshot.
- **FR-005** Writes go through one ordered queue; each write is followed by a refresh of the projects involved.
- **FR-006** The background interval is a setting in seconds (default 20, minimum 10, 0 off); in the background it is at least 1 min.
- **FR-007** Without the `export` capability, the worker builds the snapshot from `projects`, `check`, `thread` and `record`.

## Success Criteria

- **SC-001** The Messages view and an opened conversation render from the snapshot in under 100 ms on the user's machine (MT-MSG-008, MT-MSG-009).
- **SC-002** A message written by another project shows in Aurora within 30 s while the window has the focus.
