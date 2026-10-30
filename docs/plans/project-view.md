---
plan_id: project-view
project: aurora-dev-suite
title: Project view
kind: feature
version: v0.2.0
status: completed
author: Levi
created_at: '2026-10-30T14:17:31'
updated_at: '2026-10-30T14:41:00'
session_id: null
tags: []
supersedes: null
superseded_by: null
parent_plan_id: null
backlog_items:
- FEAT-015
spec_dir: specs/004-project-view
milestones:
- name: M1 Core rules
  done: true
  done_at: '2026-10-30'
  by: Levi
- name: M2 Project view and Open
  done: true
  done_at: '2026-10-30'
  by: Levi
- name: M3 Manual tests and docs
  done: true
  done_at: '2026-10-30'
  by: Levi
status_log:
- at: '2026-10-30T14:17:31'
  by: Levi
  status: draft
  note: Plan created
- at: '2026-10-30T14:17:31'
  by: Levi
  status: in_progress
  note: Approved at gate 3; implementation started
- at: '2026-10-30T14:35:00'
  by: Levi
  note: Milestone 'M1 Core rules' completed
- at: '2026-10-30T14:35:00'
  by: Levi
  note: Milestone 'M2 Project view and Open' completed
- at: '2026-10-30T14:35:00'
  by: Levi
  note: Milestone 'M3 Manual tests and docs' completed
- at: '2026-10-30T14:35:00'
  by: Levi
  note: All milestones done; waits for the pull request to merge
- at: '2026-10-30T14:41:00'
  by: Levi
  status: completed
  note: 'Pull request #11 merged; FEAT-015 delivered'
---

# Project view

## Goal and decisions

Deliver FEAT-015 of v0.2.0 "Portfolio on the dcp contract": the Project view shows one project's whole cycle state, read through the dcp adapter (FEAT-016), with the phase and waiting rules of the Portfolio (FEAT-014), the gates that wait for the user and the recommended next step.

Decisions (detail in `research.md`, no new ADR):

- The gates are derived in the core from `dcp.diagnose` 1.x: a decision when the protocol asks, the release candidate's acceptance, and the release gate. Plan approval is not in the contract.
- One release candidate stage serves the Portfolio's waiting title and the gate.
- "Open" in a Portfolio row sets the chosen project in the shared context; nothing is saved.
- One read at a time through `project_cycle_state`; the latest answer wins. The failure text is shared with the Portfolio. No new command, no new dependency.

The detail lives in the Spec Kit folder [`specs/004-project-view`](../../specs/004-project-view/): `spec.md` (3 user stories, 10 functional requirements, 4 success criteria), `plan.md`, `research.md`, `data-model.md`, `quickstart.md` and `tasks.md` (10 tasks).

## Milestones

M1 Core rules (Phase 1 of `tasks.md`), M2 Project view and Open (Phases 2 to 4), M3 Manual tests and docs (Phase 5).

## Backlog items

### FEAT-015: Project view
- **Priority**: high
- **Found**: 2026-10-27
- **Description**: One project's cycle state: plans and milestones, backlog summary, roadmap, release candidate, pull requests, gates and blocking bugs.
