---
plan_id: messages-chat
project: aurora-dev-suite
title: Messages as a chat between projects
kind: feature
version: v0.3.1
status: draft
author: LeviGuilherme
created_at: '2026-11-08T21:29:15'
updated_at: '2026-11-08T21:29:15'
session_id: null
tags: []
supersedes: null
superseded_by: null
parent_plan_id: null
backlog_items:
- FEAT-039
milestones:
- name: M1 Provider summary
  done: false
- name: M2 Core chat model
  done: false
- name: M3 Chat view
  done: false
- name: M4 Project logo
  done: false
- name: M5 Docs and manual tests
  done: false
status_log:
- at: '2026-11-08T21:29:15'
  by: LeviGuilherme
  status: draft
  note: Plan created
---

# Messages as a chat between projects

Plan for the backlog item below (`dev-cycle-protocol` Phases 1 and 2). Specification and tasks: `specs/011-messages-chat/`. Mockups agreed with Levi on 2026-11-08 (Aurora palette only, both themes).

## Backlog items

### FEAT-039: Messages as a chat between projects
- **Priority**: high
- **Found**: 2026-11-08, v0.3.0-rc.6
- **Description**: see `docs/BACKLOG.md`.

## 1. Goal and decisions

The Messages view becomes a chat between projects: projects as contacts on the left, the selected project's stream of sent and received messages as bubbles with the sender's first-person summary, the full message in a panel that slides in on the right. Decisions (Levi, 2026-11-08): two columns plus the detail panel; the summary is written by the sender (`--summary`), older messages show the subject; closing records render as the existing decision card inside the stream; Aurora's palette only; release v0.3.1, after v0.3.0 final. Gedo alerts in the same style are FEAT-040, later.

## 2. Design (`software-design`)

- Provider (`aurora-integrations`): optional `summary` in the message front matter, written after the fixed fields only when set; `--summary` on `send` and `reply`, at most 280 characters, refused when longer; `decide` and `guide` write their own; every header in the replies carries it; schema 1.3, capability `summary`.
- Core (`aurora-core`): `MessageHeader.summary`; `Draft.summary` and `UserAction::Reply.summary`; a pure chat model: `chat_contacts` (one row per registered project: last summary, time, unread, waiting) and `chat_stream` (one project's messages and closing records in date order, with day separators, the quote of the replied message, the delivery mark and the decision taken). Decision messages are not bubbles: they close the record card. No new port.
- Engines adapter: passes `--summary` when the draft or reply has one; reads `summary` from the export.
- Desktop: `pick_project_logo` and `clear_project_logo`; the logo is a small image (PNG, JPEG, WebP or SVG, at most 256 KB) stored as a data URL in the project entry of Aurora's own list; nothing is written in the repository.
- UI: the Messages view is three columns (contacts fixed, chat flexible, detail closed by default); decision card, message body, Reply and Step in, New conversation and the write queue are reused. New tokens `--bubble-out`, `--bubble-out-border`, `--bubble-in`, `--row-sel`, `--quote-bg` in both themes, checked by `theme_contrast.rs`.

## 3. Affected surfaces and files

Desktop surface only. `crates/aurora-integrations/src/{format,commands,rules}.rs` and its binary; `crates/aurora-core/src/domain/{messages,project}.rs`, a new `domain/chat.rs`, fixtures; `crates/aurora-engines/src/messages.rs`; `crates/aurora-desktop/src/commands.rs`; `crates/aurora-ui/src/views/{messages,project}.rs`, `styles/{tokens,app}.css`, `theme_contrast.rs`, the fixture backend.

## 4. Tests (`quality-assurance`)

Provider: summary round trip, refusal above 280, `decide` and `guide` summaries, export carries it (scenarios against the contract). Core: contacts order and counts, stream order, day separators, quotes, marks, hidden decision messages, record states, fallback to the subject. UI: pure helpers (day and time labels, initials, avatar color). Manual: MT-MSG cases rewritten for the chat (contacts, bubbles, detail panel, decision in the stream, summary in Reply and New conversation, logo).

## 5. Documentation (`architecture-docs`, `user-docs`)

ADR-0013 (chat layout and sender summaries); the messages architecture page; README Messages section; contracts `specs/008-messages/contracts/{cli.md,dcp-integrations.schema.json}`.

## 6. CI, release and observability (`devops-release`)

No workflow change. Version v0.3.1 after v0.3.0 final. Logs unchanged.

## 7. Security, migration and rollback

The summary is optional and additive: older tools keep reading the messages; older Aurora ignores the field. The logo is read only from a file the user picks, size and type checked. Rollback: revert the release; messages written with a summary stay valid.

## 8. Execution steps and milestones

M1 Provider summary, M2 Core chat model, M3 Chat view, M4 Project logo, M5 Docs and manual tests (tasks in `specs/011-messages-chat/tasks.md`).
