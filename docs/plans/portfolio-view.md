---
plan_id: portfolio-view
project: aurora-dev-suite
title: Portfolio view
kind: feature
version: v0.2.0
status: in_progress
author: Levi
created_at: '2026-10-30T12:57:14'
updated_at: '2026-10-30T13:16:19'
session_id: null
tags: []
supersedes: null
superseded_by: null
parent_plan_id: null
backlog_items:
- FEAT-014
spec_dir: specs/003-portfolio-view
milestones:
- name: M1 Core rules
  done: true
  done_at: '2026-10-30'
  by: Levi
- name: M2 Portfolio rows and loader
  done: true
  done_at: '2026-10-30'
  by: Levi
- name: M3 Manual tests and docs
  done: true
  done_at: '2026-10-30'
  by: Levi
status_log:
- at: '2026-10-30T12:57:14'
  by: Levi
  status: draft
  note: Plan created
- at: '2026-10-30T12:57:14'
  by: Levi
  status: in_progress
  note: Approved at gate 3; implementation started
- at: '2026-10-30T13:16:08'
  by: Levi
  note: Milestone 'M1 Core rules' completed
- at: '2026-10-30T13:16:08'
  by: Levi
  note: Milestone 'M2 Portfolio rows and loader' completed
- at: '2026-10-30T13:16:08'
  by: Levi
  note: Milestone 'M3 Manual tests and docs' completed
- at: '2026-10-30T13:16:08'
  by: Levi
  status: completed
  note: All milestones completed
- at: '2026-10-30T13:16:19'
  by: Levi
  status: in_progress
  note: All milestones done; waits for the pull request to merge
---

# Portfolio view

## Goal and decisions

Deliver FEAT-014 of v0.2.0 "Portfolio on the dcp contract": the Portfolio shows every registered project with its phase, current release and the number of things waiting for the user, read through the dcp adapter (FEAT-016), in parallel and shown as each answer arrives.

Decisions (detail in `research.md`, no new ADR):

- The phase is derived in the core from `dcp.diagnose` 1.x (release candidate, implementation, review, planning, idle); the contract has no phase field.
- What waits for the user: the unreleased release candidate, pull requests that need action, blocking bugs and untriaged bugs; the same rules serve the Project view (FEAT-015) and the Inbox (FEAT-017).
- The interface reads through the existing `project_cycle_state` command, at most four reads at once, dropping stale answers; the desktop shares a single first engine probe. No new command, no new dependency.

The detail lives in the Spec Kit folder [`specs/003-portfolio-view`](../../specs/003-portfolio-view/): `spec.md` (3 user stories, 11 functional requirements, 5 success criteria), `plan.md`, `research.md`, `data-model.md`, `quickstart.md` and `tasks.md` (10 tasks).

## Milestones

M1 Core rules (Phase 1 of `tasks.md`), M2 Portfolio rows and loader (Phases 2 to 4), M3 Manual tests and docs (Phase 5).

## Backlog items

### FEAT-014: Portfolio view
- **Priority**: high
- **Found**: 2026-10-27
- **Description**: All registered projects with name, phase, current release and a count of what waits for the user, loaded in parallel and shown progressively.
