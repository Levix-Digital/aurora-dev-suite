---
plan_id: desktop-packages
project: aurora-dev-suite
title: Desktop packages
kind: feature
version: v0.2.0
status: in_progress
author: Levi
created_at: '2026-10-30T15:57:17'
updated_at: '2026-10-30T16:48:00'
session_id: null
tags: []
supersedes: null
superseded_by: null
parent_plan_id: null
backlog_items:
- FEAT-005
spec_dir: specs/006-desktop-packages
milestones:
- name: M1 Tag rule
  done: true
  done_at: '2026-10-30'
  by: Levi
- name: M2 Packages and release workflows
  done: true
  done_at: '2026-10-30'
  by: Levi
- name: M3 Manual tests and docs
  done: true
  done_at: '2026-10-30'
  by: Levi
status_log:
- at: '2026-10-30T15:57:17'
  by: Levi
  status: draft
  note: Plan created
- at: '2026-10-30T16:01:39'
  by: Levi
  status: in_progress
  note: Approved at gate 3 with the Tauri CLI 2.x build tool; implementation started
- at: '2026-10-30T16:48:00'
  by: Levi
  note: Milestone 'M1 Tag rule' completed
- at: '2026-10-30T16:48:00'
  by: Levi
  note: Milestone 'M2 Packages and release workflows' completed
- at: '2026-10-30T16:48:00'
  by: Levi
  note: Milestone 'M3 Manual tests and docs' completed
- at: '2026-10-30T16:48:00'
  by: Levi
  note: All milestones done; the release workflow's dry run built the three packages; waits for pull request #15 to merge
---

# Desktop packages

## Goal and decisions

Deliver FEAT-005, the last item of v0.2.0 "Portfolio on the dcp contract": Aurora ships as a Windows installer and Linux `.deb` and AppImage packages, built and published by CI for every release candidate and final release, so release candidates can be installed and accepted without building from source.

Decisions (detail in `research.md`; ADR-0008 records R1 to R4 and R6):

- The Tauri bundler builds the packages from `tauri.conf.json` with the Tauri CLI 2.x as a build tool (no new crate).
- Windows: NSIS only, per user without administrator rights, with the embedded WebView2 bootstrapper; MSI later if a managed deployment needs it.
- Linux: `.deb` (declares WebKitGTK 4.1 and GTK 3) and AppImage (carries its libraries; its size is reported, not gated).
- The Dev Cycle's release and promote workflows (trunk model) are added: the tag is checked against the workspace version by `cargo xtask release-check`, tests and packages run on both platforms, and one publish job creates the release with `SHA256SUMS.txt` only after both succeed; candidates are pre-releases. Publishing uses the GitHub CLI, not a third-party action. A dry run on pull requests that change the packaging, and by hand, keeps the packages as run artifacts.
- Releases go to the repository in `RELEASES_REPOSITORY` (with `RELEASES_TOKEN`) when set, the source repository otherwise, until ADR-0005's public releases repository exists.
- No signing and no updater in v0.2.0; code signing becomes a backlog item.

The detail lives in the Spec Kit folder [`specs/006-desktop-packages`](../../specs/006-desktop-packages/): `spec.md` (3 user stories, 9 functional requirements, 4 success criteria), `plan.md`, `research.md`, `quickstart.md` and `tasks.md` (11 tasks).

## Milestones

M1 Tag rule (Phase 1 of `tasks.md`), M2 Packages and release workflows (Phases 2 and 3), M3 Manual tests and docs (Phase 4).

## Backlog items

### FEAT-005: Desktop packages
- **Priority**: medium
- **Found**: 2026-10-27
- **Description**: Windows installer (NSIS or MSI) and Linux packages (AppImage and deb) built in CI for release candidates.
