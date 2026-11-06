# Implementation Plan: Messages view adjustments

**Branch**: `feat/009-messages-view` | **Date**: 2026-11-06 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/009-messages-view/spec.md`

## Summary

Make the Messages view read at a glance: `from → to` on rows and messages, conversations grouped by a state derived in the core (Waiting for you, In progress, Closed), a conversation opened right below its row, and a navigation count of what needs the user. Unread comes from the received open messages the provider already returns in `check --json`.

## Technical Context

**Language/Version**: Rust (workspace toolchain), Leptos 0.8 (WASM) for the view
**Primary Dependencies**: none new
**Storage**: none
**Testing**: `cargo test` (core and engines), browser preview screenshots, manual tests MT-MSG-003 and MT-MSG-005
**Target Platform**: Windows 10+ and Linux desktop (Tauri 2)
**Project Type**: desktop application, hexagonal workspace
**Performance Goals**: no extra provider call; the count reuses the overview read
**Constraints**: no change to `dcp-integrations` or its JSON contracts; keyboard first
**Scale/Scope**: one adjustment, three milestones

## Constitution Check

| Principle | How this plan complies | Status |
|---|---|---|
| I. Design: one responsibility per unit | State and counts in the core domain; the adapter only maps `check`; the view only renders | Pass |
| II. Simplicity First | No new dependency, no new command, no cache; `<details>` for the collapsed group | Pass |
| III. Hexagonal | Domain logic in `aurora-core`; the provider stays behind `MessagesSource` | Pass |
| IV. Development cycle: evidence | Unit tests for every state and the counts; adapter test on the contract sample; preview check; manual tests updated | Pass |
| IV. Security | No input or permission change | Pass |
| IV. 9. No AI attribution | None in plan, code, commits or pull request | Pass |
| ADR-0003, ADR-0006 | The provider only through its versioned JSON; no stand-in | Pass |

## Project Structure

### Documentation (this feature)

```text
specs/009-messages-view/
├── spec.md
├── plan.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/aurora-core/src/domain/messages.rs     # ConversationState, Conversation::state, unread, open counts
crates/aurora-core/src/fixtures/messages.rs   # samples in the three states, with unread
crates/aurora-engines/src/messages.rs         # unread per thread from check's messages
crates/aurora-ui/src/views/messages.rs        # groups, from → to, inline panel
crates/aurora-ui/src/app.rs                   # navigation count
crates/aurora-ui/styles/app.css               # group colors
```

## Complexity Tracking

None.

## Dev Cycle Protocol

### Backlog items and release

ADJ-002, release v0.3.0 (with BUG-005 and BUG-006 in v0.3.0-rc.2). Plan file: `docs/plans/messages-view-adjustments.md`.

### Tests (quality-assurance)

- Core: `state()` for waiting, rounds running, a decision, cycle 2 after a revision, notice only, tool 0.1 message; `open_count()` and the total across projects.
- Engines: the `check` sample yields `unread` per thread; a thread with nothing open has 0; a payload without the field reads as 0.
- View: preview screenshots of the groups, the inline panel and the count on another screen.
- Manual: MT-MSG-003 (groups, colors, `from → to`, inline opening) and MT-MSG-005 (count on another screen).

### Documentation (architecture-docs, user-docs)

`docs/architecture/core.md` (state and counts), `ui.md` (list, groups, inline panel, count), README Messages section.

### CI, release and observability (devops-release)

No workflow change; ships in v0.3.0-rc.2. No new failure mode, so no new log.

### Security, migration and rollback

`unread` has a serde default, so older payloads still read. Rollback: revert the pull request.

## Milestones

1. M1 Core and adapter
2. M2 View
3. M3 Docs and manual tests
