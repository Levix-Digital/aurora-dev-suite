---
plan_id: messages
project: aurora-dev-suite
title: Messages between projects
kind: feature
version: v0.3.0
status: in_progress
author: Levi
created_at: '2026-11-06T13:13:30'
updated_at: '2026-11-06T14:01:18'
session_id: null
tags: []
supersedes: null
superseded_by: null
parent_plan_id: null
backlog_items:
- FEAT-029
- FEAT-031
- FEAT-032
- FEAT-033
- FEAT-034
spec_dir: specs/008-messages
milestones:
- name: M1 Provider format and rules
  done: true
  done_at: '2026-11-06'
  by: LeviGuilherme
- name: M2 Provider transport and commands
  done: true
  done_at: '2026-11-06'
  by: LeviGuilherme
- name: M3 Aurora core, adapter and desktop
  done: true
  done_at: '2026-11-06'
  by: LeviGuilherme
- name: M4 Messages view
  done: true
  done_at: '2026-11-06'
  by: LeviGuilherme
- name: M5 Packaging, docs and manual tests
  done: false
status_log:
- at: '2026-11-06T13:13:30'
  by: Levi
  status: draft
  note: Plan created
- at: '2026-11-06T13:18:20'
  by: Levi
  status: in_progress
  note: Approved at gate 3; implementation started
- at: '2026-11-06T13:22:28'
  by: LeviGuilherme
  note: Milestone 'M1 Provider format and rules' completed
- at: '2026-11-06T13:30:56'
  by: LeviGuilherme
  note: Milestone 'M2 Provider transport and commands' completed
- at: '2026-11-06T13:51:58'
  by: LeviGuilherme
  note: Milestone 'M3 Aurora core, adapter and desktop' completed
- at: '2026-11-06T14:01:18'
  by: LeviGuilherme
  note: Milestone 'M4 Messages view' completed
---

# Messages between projects

## Backlog items

### FEAT-029: Integration subscriptions
- **Priority**: medium
- **Found**: 2026-10-29, 2026-10-29, Levi
- **Description**: A project declares in .aurora/integrations.json, committed once on its work branch, the projects it depends on, their repositories, what it consumes from each (a command line and its JSON, a schema, an API) and the branches it follows. Created and validated through the integrations provider, never edited by Aurora's interface directly. This file is the provider's only registry of counterparts (ADR-0010): no central list and no dependency on Aurora's repository.

### FEAT-031: Integration negotiation
- **Priority**: medium
- **Found**: 2026-10-29, 2026-10-29, Levi
- **Description**: At the integration step the receiving project checks each notice against what it consumes; when affected it opens a conversation (a thread numbered by its own counter, so projects never collide) with a proposal in the other project's aurora/inbox. Conversations follow ADR-0009: kinds question, proposal, notice, answer, counter, agreement, impasse, ack, decision and guidance; a round is one message from each side; at most three rounds per cycle, then only an agreement or an impasse; ack, notice, decision and guidance outside rounds; each side answers from its project's records without asking the user mid-conversation; the user may step in at any time with a request (guidance), which restarts the rounds in a new cycle; fixed templates per kind with required sections checked on send and reply. Messages unrelated to the current work are deferred with a reason and shown to the user. Commands with --json: poll, check, show, template, send, reply, record, decide, guide, defer, status, watch, the same as the provisional tool 0.3.0 (IMP-002), so no message already written changes.

### FEAT-032: Integration gate
- **Priority**: medium
- **Found**: 2026-10-29, 2026-10-29, Levi
- **Description**: Every conversation closes with an agreement or an impasse carrying the closing record for the user (ADR-0009): the projects and rounds, the subject, the outcome, conclusion, justification, positions for an impasse, the decision needed with one option recommended, and what each project does after approval. Only the project that opened it presents the record, once, in the user's language; the user's decision goes to every side as a decision message and each project records its own backlog item; no project acts before it. When the conclusion does not satisfy, the user asks for changes instead (decide --revise), which goes to every side and starts a new cycle of up to three rounds ending in a new closing record, until the user decides. Recorded in Aurora's Messages tab or with the provider's decide command; the same decision becomes the integration gate in the Dev Cycle with FEAT-036.

### FEAT-033: Integrations view
- **Priority**: medium
- **Found**: 2026-10-29, 2026-10-29, Levi
- **Description**: A tab with conversations waiting for the user first (their closing record shown as fields: conclusion, justification, positions, decision needed, after approval, with the decision recorded from there, or a request for changes written right there), a field to write to a conversation at any time (the user's request goes to every side and restarts the rounds), then notices, open conversations with their rounds, and sent messages still unread after a set time, read from local clones through the provider's --json (show --json returns the sections as fields). Aurora stays air-gapped: it never fetches or pushes; the provider does through Git.

### FEAT-034: Integrations provider
- **Priority**: high
- **Found**: 2026-10-29, 2026-10-29, Levi
- **Description**: The Rust binary shipped with Aurora (dcp-integrations) that implements the messages and integrations commands as a standalone command line, usable from a terminal and from Aurora's Messages tab without dcp; plugging it into the Dev Cycle is FEAT-037. It reads and writes the aurora/inbox branch with Git plumbing, without touching the working tree, and is the only writer of integration data. Without Aurora there is no provider and the step does nothing. It offers poll (one git ls-remote, no fetch) for every touchpoint and an optional watch that runs the user's local command in a project's folder when news arrives (ADR-0009). Shipped with Aurora's installer (signed once FEAT-035 lands) and updated with it, and published per platform as a separate download for machines without Aurora; nothing executable is stored on an aurora/inbox branch, which holds only messages and inbox.json with the format version (ADR-0010). It replaces the provisional Python tool, which is then removed.

## 1. Goal and decisions

Bring the messages between projects into Aurora in v0.3.0 (Levi, 2026-11-06), so the user decides on closing records, asks for changes and steps into conversations from the window that stays open, instead of a terminal. The engine is `dcp-integrations`, the Rust integrations provider shipped with Aurora, which also replaces the provisional Python tool.

Decisions: ADR-0009 (conversation model, user requests restart the rounds), ADR-0010 (the provider ships with Aurora, each project keeps its counterparts in `.aurora/integrations.json`, the inbox branch holds only data). Feature-level choices in `specs/008-messages/research.md` R1 to R9: a new crate behind the provider's command line; Git through the `git` command on bare caches; identity and counterparts from the project's file; Aurora polls with `poll --json`; bodies by temporary file; one schema file for every reply; Tauri `externalBin` for shipping; retiring the Python tool after the release.

## 2. Design (`software-design`)

`crates/aurora-integrations`: `format` (header, parse, render, sections), `rules` (kinds, templates, rounds, cycles, waiting, record), `registry` (`.aurora/integrations.json`), `transport` (Git), `commands`, binary. In Aurora: `aurora-core::domain::messages`, port `MessagesSource`, `usecases::messages`; adapter `aurora-engines::messages` over `--json`; six desktop commands and a poll timer; `aurora-ui::views::messages`. `aurora-core` never depends on the provider crate (ADR-0003). Errors explicit (`MessagesError`, `dcp-integrations.error`). Detail in `specs/008-messages/plan.md` and `data-model.md`.

## 3. Affected surfaces and files

Surface `desktop` (new Messages view, navigation entry, background poll). New crate `crates/aurora-integrations`; files listed in `specs/008-messages/plan.md`, "Source Code"; `.github/workflows/release.yml`; `crates/aurora-desktop/tauri.conf.json`.

## 4. Tests (`quality-assurance`)

As in `specs/008-messages/plan.md`, "Tests": provider unit and bare-repository scenario tests reproducing tool 0.3.0, contract validation, core and adapter tests with a fake provider, the poll timer, preview fixtures, manual tests MT-MSG-001 to MT-MSG-007. Acceptance: US1 to US6, SC-001 to SC-006 of the spec.

## 5. Documentation (`architecture-docs`, `user-docs`)

New `docs/architecture/integrations.md`; in place `core.md`, `engines.md`, `ui.md`, `desktop.md`; README sections for the Messages view, the command line and `.aurora/integrations.json`. No new ADR beyond ADR-0009 and ADR-0010.

## 6. CI, release and observability (`devops-release`)

CI covers the new crate (needs `git` on runners). `release.yml` builds and bundles the provider and uploads it alone with its checksum; `release-check` checks its version. Minor version impact inside v0.3.0. Logs without message bodies, the provider's with `--verbose`.

## 7. Security, migration and rollback

No secrets; Git with the user's own credentials; bodies never in arguments or logs. The provider reads every message of the provisional tool and adds `inbox.json`, which that tool ignores. Rollback: revert the pull request.

## 8. Execution steps and milestones

The phases of `specs/008-messages/tasks.md` (28 tasks): M1 Provider format and rules, M2 Provider transport and commands, M3 Aurora core, adapter and desktop, M4 Messages view, M5 Packaging, docs and manual tests. After the release: the projects add `.aurora/integrations.json`, switch their instructions and the Python tool is removed (research R9).
