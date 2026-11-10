---
id: 12
title: A messages worker in Aurora owns every provider call and serves a saved snapshot
status: accepted
date: '2026-11-07'
version: v0.3.0
subsystems: [desktop, ui, core, integrations]
supersedes: []
superseded_by: []
amends: [11]
amended_by: [13, 14]
---

# ADR-0012: A messages worker in Aurora owns every provider call and serves a saved snapshot

## Context

Levi tested v0.3.0 on 2026-11-07 and still found Messages slow, and asked for an e-mail feel: the list there at once, news arriving by itself, no incoherent states while writing. Facts at that point:

- The view called the provider itself: one `check` per registered project when it opened, one `thread` and one `record` per conversation opened, and a reload of everything after each write. Every call waited on the provider's cache lock, so a write or a sync in progress made reading wait too.
- ADR-0011's fast reads were merged (PR #39) but tag `v0.3.0-rc.5` points before them, so no published build had them.
- The same thread was read once per registered project that takes part in it, so it was listed twice.
- The poll ran every minute, also while Aurora was minimized, and the focus check at most every 30 s.

## Decision

This amends ADR-0011: the provider still owns the local copies and the inbox branch, and Aurora still talks only to `dcp-integrations`.

- **`export`.** A new provider command answers `dcp-integrations.export`: the project's conversations as `check` gives them, each with all its messages (sections, body without status notes, the `> open|seen|done` notes parsed into `notes`) and its closing record when one exists, in one process. `version` declares the capability `export`; documents move to schema 1.2. With an older provider, Aurora builds the same reading from `check`, `thread` and `record`.
- **One worker.** A thread in the desktop (`worker.rs`) owns every provider call. It takes jobs from one queue: readings (coalesced, so many asks make one reading) and writes (sent in the order they were made). One reading follows a burst of writes, 300 ms after the queue is quiet.
- **A saved snapshot.** Each reading (`sync`, then `export --cached` per project) builds a `MessagesSnapshot` with a version number. The worker keeps it in memory and in `messages-snapshot.json` in Aurora's configuration folder (written to a temporary file and renamed). A project that cannot be read keeps its last good inbox and is listed as stale. After a reading that changed something the worker emits `messages-snapshot` with the version, the counts and what changed.
- **The view reads only the snapshot.** `messages_snapshot` answers from memory at once (the saved file at start-up); `messages_refresh` ("Check now") waits for a reading; `messages_act` puts a write in the queue. The view applies what the user just did at once (read, decided) and drops it when a newer snapshot catches up; a failed write shows its error and the next snapshot shows the true state. The list is keyed by thread, so an update does not reset an open form.
- **One row per thread.** `conversation_views` merges the readings of one thread seen by two projects and acts through the project that received the last message.
- **Interval.** 20 s while the window has the focus, 60 s otherwise, one reading when the window gets the focus (at most once per 20 s) and after each write. The setting `messagesPollSeconds` (minimum 10, 0 turns it off) replaces `messagesPollMinutes`, which is converted when settings are read.

## Alternatives Considered

- **Keep calling the provider from the view, only faster** (ADR-0011 alone): every opening and every write still waits for processes, and two calls can interleave with a write.
- **A database owned by Aurora**: a second copy of every message to keep in step with statuses written by other sessions; the snapshot is a derived file that can always be rebuilt from the provider.
- **Push from the provider (a long-running process or file watcher)**: messages arrive through remote repositories, so something must still fetch; a long-running provider process would also need its own lifecycle in every host.
- **Polling every few seconds**: each reading fetches every inbox over the network; 20 s in front and 60 s behind keeps the cost low (one `ls-remote` per inbox when nothing changed) while the focus reading covers coming back from a terminal.

## Consequences

- Messages opens from the snapshot without starting any process, also with the network down; the list may be up to one reading old, and stale projects are named.
- Writes never race with readings; a write shows in the list after the reading that follows it (about a second), and at once where the view applies it itself.
- The provider gains one command and the contract one schema version; older providers keep working through the fallback.
- The snapshot file grows with the messages it holds (a few hundred kilobytes for a few hundred messages); deleting it only costs one reading.
- Verification: `tests/scenarios.rs` covers `export`; `tests/messages.rs` in the engines covers `export` with and without the capability; `worker.rs` tests the order of writes, the coalesced reading and a broken snapshot file; the core tests cover the snapshot, merged rows, statuses and the interval; manual tests MT-MSG-001, 003, 005 and 009.
