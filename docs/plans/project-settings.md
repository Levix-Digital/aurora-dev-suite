---
plan_id: project-settings
project: aurora-dev-suite
title: Project settings
kind: feature
version: v0.3.0
status: completed
author: Levi
created_at: '2026-10-31T15:09:24'
updated_at: '2026-10-31T15:38:29'
session_id: null
tags: []
supersedes: null
superseded_by: null
parent_plan_id: null
backlog_items:
- FEAT-027
spec_dir: specs/007-project-settings
milestones:
- name: M1 Core and contracts
  done: true
  done_at: '2026-10-31'
  by: Levi
- name: M2 Adapter and desktop
  done: true
  done_at: '2026-10-31'
  by: Levi
- name: M3 Settings tab
  done: true
  done_at: '2026-10-31'
  by: Levi
- name: M4 Manual tests and docs
  done: true
  done_at: '2026-10-31'
  by: Levi
status_log:
- at: '2026-10-31T15:09:24'
  by: Levi
  status: draft
  note: Plan created
- at: '2026-10-31T15:30:52'
  by: Levi
  status: in_progress
  note: Approved at gate 3; implementation started
- at: '2026-10-31T15:30:53'
  by: Levi
  note: Milestone 'M1 Core and contracts' completed
- at: '2026-10-31T15:30:54'
  by: Levi
  note: Milestone 'M2 Adapter and desktop' completed
- at: '2026-10-31T15:30:55'
  by: Levi
  note: Milestone 'M3 Settings tab' completed
- at: '2026-10-31T15:30:56'
  by: Levi
  note: Milestone 'M4 Manual tests and docs' completed
- at: '2026-10-31T15:30:56'
  by: Levi
  note: All milestones done; waits for the pull request to merge
- at: '2026-10-31T15:38:29'
  by: Levi
  status: completed
  note: 'Pull request #23 merged; FEAT-027 delivered'
---

# Project settings

## Goal and decisions

Deliver FEAT-027 of v0.3.0 "Actions through dcp": a Settings tab per project in the Project view, rendered from `dcp config get --json`, where the user edits settings, previews the change with `dcp config set --dry-run --json` and applies it with `dcp config set --json`. Aurora never reads or writes `.dev-cycle.json` and never commits.

Decisions (detail in `research.md`, no new ADR):

- `dcp config get` and `dcp config set` do not exist yet (DCP FEAT-017, protocol v2.12.0). Aurora authors the `dcp.config` and `dcp.config-set` MAJOR 1 contracts here and sends them to the Dev Cycle Protocol project, as for `dcp.diagnose`. Until `dcp` lists the capability `config`, the desktop shows the feature as unavailable; the preview works with sample settings (ADR-0006).
- Values go to `dcp config set` as one `--value <key>=<JSON>` per key; writes are all or nothing and guarded by a `--revision` from `config get`, so a file changed meanwhile is never overwritten.
- The core checks read-only, type, allowed values and null before running anything; `dcp` stays the authority and its errors are shown next to the field.
- The tab lives in the Project view ("Cycle" and "Settings" tabs); running `dcp` and the envelope checks are shared with the cycle adapter. No new dependency.

The detail lives in the Spec Kit folder [`specs/007-project-settings`](../../specs/007-project-settings/): `spec.md` (3 user stories, 11 functional requirements, 4 success criteria), `plan.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md` and `tasks.md` (16 tasks).

## Milestones

M1 Core and contracts (Phase 1 of `tasks.md`), M2 Adapter and desktop (Phase 2), M3 Settings tab (Phase 3), M4 Manual tests and docs (Phase 4).

## Backlog items

### FEAT-027: Project settings

A Settings tab per project rendered from dcp config get --json (schema dcp.config: value, allowed values, default, type, read-only, group); changes previewed with dcp config set --dry-run --json and applied with dcp config set, errors shown next to the field. Aurora never reads or writes .dev-cycle.json and does not commit: the user commits the change. Needs the dcp capability config (ADR-0003, ADR-0006).
