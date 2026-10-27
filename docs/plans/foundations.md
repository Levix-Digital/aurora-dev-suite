---
plan_id: foundations
project: aurora-dev-suite
title: Foundations
kind: feature
version: v0.1.0
status: in_progress
author: Levi
created_at: '2026-10-27T17:18:26'
updated_at: '2026-10-27T17:52:06'
session_id: null
tags: []
supersedes: null
superseded_by: null
parent_plan_id: null
backlog_items:
- FEAT-001
- FEAT-002
- FEAT-003
- FEAT-004
- FEAT-006
- FEAT-007
- FEAT-008
- FEAT-009
- FEAT-010
- FEAT-011
- FEAT-012
spec_dir: specs/001-foundations
milestones:
- name: M1 Setup
  done: false
- name: M2 Foundation
  done: false
- name: M3 Window and navigation
  done: false
- name: M4 Projects
  done: false
- name: M5 Engines
  done: false
- name: M6 Preview
  done: false
- name: M7 CI and docs
  done: false
status_log:
- at: '2026-10-27T17:18:26'
  by: Levi
  status: draft
  note: Plan created
- at: '2026-10-27T17:23:59'
  by: Levi
  status: planned
  note: Spec, plan and tasks written; waiting for the user's approval (gate 3)
- at: '2026-10-27T17:24:06'
  by: Levi
  status: active
  note: Plan complete, submitted for approval (gate 3)
- at: '2026-10-27T17:52:06'
  by: Levi
  status: in_progress
  note: Approved by Levi on 2026-10-27 after removing the Dev Cycle stand-in (ADR-0006); keeping the interface preview
---

# Foundations

## Goal and decisions

Deliver v0.1.0 "Foundations": the Levix team opens Aurora on Windows and Linux, registers its projects and sees which engines are installed and what each Aurora feature needs, with every engine behind a port and fixtures in place. The founding decisions are ADR-0001 to ADR-0005; the smaller choices of this plan are in the spec folder's `research.md` (no new ADR).

The detail lives in the Spec Kit folder [`specs/001-foundations`](../../specs/001-foundations/): `spec.md` (4 user stories, 20 functional requirements, 7 success criteria), `plan.md` (design, structure, tests, documentation, CI, security), `research.md`, `data-model.md`, `contracts/`, `quickstart.md` and `tasks.md` (47 tasks).

## Milestones

The phases of `tasks.md`: M1 Setup, M2 Foundation, M3 Window and navigation (US1), M4 Projects (US2), M5 Engines (US3), M6 Preview (US4), M7 CI and docs.

## Backlog items

### FEAT-001: Rust workspace with the four crates
- **Priority**: high
- **Found**: 2026-10-27
- **Description**: aurora-core, aurora-engines, aurora-desktop and aurora-ui with workspace dependencies, edition 2024, MSRV 1.95, publish = false, cargo-deny and check_design passing.

### FEAT-002: Tauri 2 shell with a Leptos interface
- **Priority**: high
- **Found**: 2026-10-27
- **Description**: The desktop application opens a window that renders the Leptos interface built by Trunk; one sample command round-trips a core type.

### FEAT-003: Aurora Borealis theme and navigation
- **Priority**: high
- **Found**: 2026-10-27
- **Description**: CSS tokens from the Aurora Borealis palette, light and dark, and the navigation between Portfolio, Project, Engines and Inbox views.

### FEAT-004: CI on Windows and Linux
- **Priority**: high
- **Found**: 2026-10-27
- **Description**: Pull request CI: fmt, clippy, tests, cargo-deny, check_design and a desktop build on windows-latest and ubuntu-latest.

### FEAT-006: Interface preview with fixture data
- **Priority**: medium
- **Found**: 2026-10-27
- **Description**: The interface runs in a browser during development with fixture data, without the desktop shell.

### FEAT-007: Command runner
- **Priority**: high
- **Found**: 2026-10-27
- **Description**: Run an engine command without a shell, with timeout, working directory and standard error capture, and check the JSON envelope (schema, schema_version) with size limits.

### FEAT-008: Engine probe and capabilities
- **Priority**: high
- **Found**: 2026-10-27
- **Description**: Find each engine binary and read its version and capabilities with version --json; each feature declares the capabilities it needs and is disabled with a clear message when one is missing.

### FEAT-009: Fixture adapters for every port
- **Priority**: high
- **Found**: 2026-10-27
- **Description**: Fixture implementations of every core port, used by tests and by the interface preview.

### FEAT-010: Project registry
- **Priority**: high
- **Found**: 2026-10-27
- **Description**: Add, rename and remove projects in projects.toml in the user's configuration directory; validate paths; recover from a corrupt file (ADR-0004).

### FEAT-011: Engines view
- **Priority**: high
- **Found**: 2026-10-27
- **Description**: A view with each engine's status (ready, degraded, missing), version, capabilities, and the features they enable.

### FEAT-012: Refresh and diagnostics
- **Priority**: medium
- **Found**: 2026-10-27
- **Description**: Refresh engine detection on demand and show the last command's standard error for a degraded engine.

## Plan sections

Each section of the protocol's plan is in the spec folder, not copied here:

| Section | Where |
|---|---|
| Design | [`plan.md`](../../specs/001-foundations/plan.md) Summary, Technical Context, Project Structure; [`research.md`](../../specs/001-foundations/research.md); [`data-model.md`](../../specs/001-foundations/data-model.md) |
| Affected surfaces and files | `desktop` (the only surface); files in `plan.md` Project Structure; commands in [`contracts/desktop-commands.md`](../../specs/001-foundations/contracts/desktop-commands.md) |
| Tests | `plan.md` Dev Cycle Protocol, Tests; test tasks of `tasks.md` |
| Documentation | `plan.md` Dev Cycle Protocol, Documentation |
| CI, release and observability | `plan.md` Dev Cycle Protocol, CI; `research.md` R10 |
| Security, migration and rollback | `plan.md` Dev Cycle Protocol, Security |
| Execution steps | [`tasks.md`](../../specs/001-foundations/tasks.md), phases mapped to the milestones above |
