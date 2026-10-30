---
plan_id: dcp-adapter
project: aurora-dev-suite
title: dcp adapter
kind: feature
version: v0.2.0
status: in_progress
author: Levi
created_at: '2026-10-29T23:37:26'
updated_at: '2026-10-30T01:15:46'
session_id: null
tags: []
supersedes: null
superseded_by: null
parent_plan_id: null
backlog_items:
- FEAT-016
spec_dir: specs/002-dcp-adapter
milestones:
- name: M1 Contract and core
  done: false
- name: M2 Adapter
  done: false
- name: M3 Desktop and preview
  done: false
- name: M4 Docs and contract request
  done: false
status_log:
- at: '2026-10-29T23:37:26'
  by: Levi
  status: draft
  note: Plan created
- at: '2026-10-30T01:15:46'
  by: Levi
  status: in_progress
  note: Approved at gate 3; implementation started
---

# dcp adapter

## Goal and decisions

Deliver FEAT-016 of v0.2.0 "Portfolio on the dcp contract": Aurora reads a registered project's development cycle state through `dcp diagnose --json --dir <root>` (schema `dcp.diagnose`, MAJOR 1), behind the `CycleSource` port, with sample states for the tests and the preview. It is the base of FEAT-014 (Portfolio), FEAT-015 (Project view) and FEAT-017 (Inbox).

Decisions (detail in `research.md`, no new ADR since ADR-0003 and ADR-0006 already decide how engines are read):

- Aurora writes the `dcp.diagnose` contract as a JSON Schema and proposes it to the Dev Cycle Protocol project through the cross-project inbox; no item for it exists there yet.
- One reply per project with everything the three views need, typed fields instead of preformatted strings; setup-only data of today's diagnosis stays out of version 1.
- The gate on the `diagnose` capability and the project's presence lives in the core use case; the adapter only translates process output.

The detail lives in the Spec Kit folder [`specs/002-dcp-adapter`](../../specs/002-dcp-adapter/): `spec.md` (3 user stories, 10 functional requirements, 5 success criteria), `plan.md`, `research.md`, `data-model.md`, `contracts/` (`dcp-diagnose.schema.json`, `desktop-commands.md`), `quickstart.md` and `tasks.md` (17 tasks).

## Milestones

The phases of `tasks.md`: M1 Contract and core (Phases 1 and 2), M2 Adapter (US1 and US2), M3 Desktop and preview (US3), M4 Docs and contract request.

## Backlog items

### FEAT-016: dcp adapter
- **Priority**: high
- **Found**: 2026-10-27
- **Description**: CycleSource over the dcp binary's versioned --json (ADR-0006), built against fixtures that follow the contract.
