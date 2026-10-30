---
plan_id: inbox
project: aurora-dev-suite
title: Inbox
kind: feature
version: v0.2.0
status: draft
author: Levi
created_at: '2026-10-30T15:16:38'
updated_at: '2026-10-30T15:16:38'
session_id: null
tags: []
supersedes: null
superseded_by: null
parent_plan_id: null
backlog_items:
- FEAT-017
spec_dir: specs/005-inbox
milestones:
- name: M1 Core rules
  done: false
- name: M2 Inbox view
  done: false
- name: M3 Manual tests and docs
  done: false
status_log:
- at: '2026-10-30T15:16:38'
  by: Levi
  status: draft
  note: Plan created
---

# Inbox

## Goal and decisions

Deliver FEAT-017 of v0.2.0 "Portfolio on the dcp contract": the Inbox lists what waits for the user across every registered project, ordered by urgency, read through the dcp adapter (FEAT-016) with the waiting rules of the Portfolio (FEAT-014) and the candidate stage of the Project view (FEAT-015).

Decisions (detail in `research.md`, no new ADR):

- The Inbox items are the Portfolio's waiting items, so both views always agree; the protocol's Decision is not an item (its options point at the same items) and stays in the Project view.
- Four urgency levels computed in the core: blocks a release (blocking bugs, candidates with failed tests), waits for your acceptance (candidates to test or to release), waits for your review (pull requests that need action), to triage (untriaged bugs).
- Order: urgency, then the Portfolio's project order, then the item's order in its project.
- The Portfolio's parallel reads move to a shared module used by both views; each view reads when it opens. "Open" on an item opens the Project view. No new command, no new dependency.

The detail lives in the Spec Kit folder [`specs/005-inbox`](../../specs/005-inbox/): `spec.md` (3 user stories, 8 functional requirements, 4 success criteria), `plan.md`, `research.md`, `data-model.md`, `quickstart.md` and `tasks.md` (9 tasks).

## Milestones

M1 Core rules (Phase 1 of `tasks.md`), M2 Inbox view (Phases 2 to 4), M3 Manual tests and docs (Phase 5).

## Backlog items

### FEAT-017: Inbox view
- **Priority**: medium
- **Found**: 2026-10-27
- **Description**: Gates to approve, release candidates to test, review threads, untriaged and blocking bugs, across projects, ordered by urgency.
