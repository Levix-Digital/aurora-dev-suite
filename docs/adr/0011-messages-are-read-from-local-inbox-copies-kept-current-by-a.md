---
id: 11
title: Messages are read from local inbox copies kept current by a background sync
status: accepted
date: '2026-11-07'
version: v0.3.0
subsystems: [engines, desktop]
supersedes: []
superseded_by: []
amends: [10]
amended_by: []
---

# ADR-0011: Messages are read from local inbox copies kept current by a background sync

## Context

Levi found reading messages in Aurora slow (2026-11-07) and proposed a worker that polls more often into a local SQLite database, with Aurora reading from it. Measured on dcp-integrations 0.3.0 with the real inboxes (136 messages in 7 inboxes):

- The provider already reads from local bare caches, but one `check` started 153 Git processes: one `git show` per message file of every inbox it read. Aurora runs one `check` per registered project in the Messages view, and on Windows a Git process takes tens of milliseconds to start, so the view waited several seconds per project, growing with every message.
- Once the 30 s fetch reuse (BUG-008) expired, every read fetched the project's inbox and each counterpart's again, one after another (the cache lock serializes them), and `check` also ran `ls-remote`: 5 s cold here, more on Windows through Git's credential helper.
- The background poll (`poll`, `ls-remote` of the project's own inbox) found news but did not bring the caches up to date, so the read that followed paid for the fetches.

## Decision

This amends ADR-0010: Aurora still talks only to `dcp-integrations`, and the provider keeps owning the local copies.

- **One read, no Git when nothing changed.** The provider reads an inbox with one `ls-tree` and one `git cat-file --batch`, and keeps the parsed files of the tip it read in `<cache>/<project>.git/aurora-index.json` (`tip` and `files`, replaced atomically). A read whose tip matches the index starts no Git process. The index is keyed by the commit, so it can never be stale; a missing or unreadable index only costs one Git read.
- **`sync`.** A new command fetches the project's inbox and every counterpart's at once (one thread per inbox), each unless it was fetched less than `--max-age` seconds ago (default 10 s, so a sweep over projects that share counterparts fetches each inbox once), and indexes them. It answers `dcp-integrations.sync` with `changed` and, per inbox, its `tip`, `changed` and `error` (an unreachable inbox does not fail the others).
- **`--cached`.** The reading commands (`projects`, `check`, `list`, `show`, `thread`, `record`) take `--cached`: they read the local copies and fetch only an inbox never fetched before; `check --cached` records the cached tip instead of running `ls-remote`. Writes always fetch first, as before, and leave the cache on their own commit.
- **Capability.** `version` adds the capability `cache` (MAJOR 1) and the documents move to schema 1.1. Aurora passes `--cached` and runs `sync` only when the provider declares `cache`; an older provider keeps the previous behavior.
- **Aurora.** The Messages view answers from the copies; opening it, the background poll, the window's focus and "Check now" run `poll` and then `sync` for every project (one sync at a time, `AppState.syncing`), and the projects where something changed arrive as `messages-news`. Opening a conversation clears its "new" badge at once and marks its unread messages as seen in the background (IMP-005).
- **No new store.** The Git object database in the caches is the local store; the index is a derived file.

## Alternatives Considered

- **A worker writing to SQLite, read by Aurora** (Levi's proposal): it separates the network from reading, which this decision keeps. Set aside because it adds a second copy of every message that must follow status changes written by other sessions, a schema and its migrations in two binaries, and it would have Aurora read the provider's storage directly, against ADR-0003 and ADR-0010. The index per tip gives the same speed without a second source of truth. SQLite can come back if search across messages needs it.
- **Only a longer fetch reuse**: fewer fetches, but every read still started one Git process per message.
- **An operating system file watcher**: messages arrive through the remote repositories, not through local files, so there is nothing local to watch.

## Consequences

- With current copies, `check` runs in about 20 ms with 2 Git processes (it was 153 processes); a cold `sync` of 7 inboxes took 1 s instead of 5 s one after another (measured on Linux, 2026-11-07). The Windows gain is larger, as process start is slower there.
- The view can show copies up to one sync old; the news banner offers the update, so nothing being typed is lost.
- Marking as seen writes one commit per message in the background; a failure is logged and the message stays unread at the next read.
- Verification: `tests/scenarios.rs` covers the index (a read with an unchanged tip, a read after a new message), `sync` and `--cached`; `tests/messages.rs` in the engines checks that `--cached` and `sync` are used only with the `cache` capability.
