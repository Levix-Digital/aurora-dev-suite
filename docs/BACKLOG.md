# Backlog

Work captured for later planning: bugs, features, improvements and adjustments, grouped under epics when the product is planned as a whole. Capturing an item takes no discovery; when the user decides to work on items, they are promoted into a plan (`plan-memory/backlog.py promote`), where the discovery and the planning happen. Open bugs come before new work.

Each item: `### <ID>: <title>` with the fields `Severity` (bugs: critical, high, medium, low) or `Priority` (others: high, medium, low), an optional `Epic`, `Status` (open, planned, in_progress), `Found`, `Plan` and `Description`. Epics (`EPIC-NNN`) group items of the complete product; they have no `Plan` and are never promoted: their items are. The roadmap (`docs/ROADMAP.md`) says which items go into which release. Closed items (done, dropped, duplicate, superseded by better defined items) move to `Closed` with a `Closed` note and keep their ids, which are never reused. Edit this file with `plan-memory/backlog.py` or by hand, and check it with `backlog.py validate`.

## Bugs

## Features

### FEAT-001: Rust workspace with the four crates
- **Priority**: high
- **Epic**: EPIC-001
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: aurora-core, aurora-engines, aurora-desktop and aurora-ui with workspace dependencies, edition 2024, MSRV 1.95, publish = false, cargo-deny and check_design passing.

### FEAT-002: Tauri 2 shell with a Leptos interface
- **Priority**: high
- **Epic**: EPIC-001
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: The desktop application opens a window that renders the Leptos interface built by Trunk; one sample command round-trips a core type.

### FEAT-003: Aurora Borealis theme and navigation
- **Priority**: high
- **Epic**: EPIC-001
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: CSS tokens from the Aurora Borealis palette, light and dark, and the navigation between Portfolio, Project, Engines and Inbox views.

### FEAT-004: CI on Windows and Linux
- **Priority**: high
- **Epic**: EPIC-001
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: Pull request CI: fmt, clippy, tests, cargo-deny, check_design and a desktop build on windows-latest and ubuntu-latest.

### FEAT-005: Desktop packages
- **Priority**: medium
- **Epic**: EPIC-001
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: Windows installer (NSIS or MSI) and Linux packages (AppImage and deb) built in CI for release candidates.

### FEAT-006: Interface preview with fixture data
- **Priority**: medium
- **Epic**: EPIC-001
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: The interface runs in a browser during development with fixture data, without the desktop shell.

### FEAT-007: Command runner
- **Priority**: high
- **Epic**: EPIC-002
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: Run an engine command without a shell, with timeout, working directory and standard error capture, and check the JSON envelope (schema, schema_version) with size limits.

### FEAT-008: Engine probe and capabilities
- **Priority**: high
- **Epic**: EPIC-002
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: Find each engine binary and read its version and capabilities with version --json; each feature declares the capabilities it needs and is disabled with a clear message when one is missing.

### FEAT-009: Fixture adapters for every port
- **Priority**: high
- **Epic**: EPIC-002
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: Fixture implementations of every core port, used by tests and by the interface preview.

### FEAT-010: Project registry
- **Priority**: high
- **Epic**: EPIC-002
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: Add, rename and remove projects in projects.toml in the user's configuration directory; validate paths; recover from a corrupt file (ADR-0004).

### FEAT-011: Engines view
- **Priority**: high
- **Epic**: EPIC-003
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: A view with each engine's status (ready, degraded, missing), version, capabilities, and the features they enable.

### FEAT-012: Refresh and diagnostics
- **Priority**: medium
- **Epic**: EPIC-003
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: Refresh engine detection on demand and show the last command's standard error for a degraded engine.

### FEAT-013: Dev Cycle stand-in adapter
- **Priority**: high
- **Epic**: EPIC-004
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: Read .dev-cycle.json, docs/BACKLOG.md, docs/ROADMAP.md and docs/plans frontmatter, and the current scripts' --json when Python is available.

### FEAT-014: Portfolio view
- **Priority**: high
- **Epic**: EPIC-004
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: All registered projects with name, phase, current release and a count of what waits for the user, loaded in parallel and shown progressively.

### FEAT-015: Project view
- **Priority**: high
- **Epic**: EPIC-004
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: One project's cycle state: plans and milestones, backlog summary, roadmap, release candidate, pull requests, gates and blocking bugs.

### FEAT-016: dcp adapter
- **Priority**: medium
- **Epic**: EPIC-004
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: CycleSource over the dcp binary's versioned --json, replacing the stand-in once dcp publishes its contract.

### FEAT-017: Inbox view
- **Priority**: medium
- **Epic**: EPIC-005
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: Gates to approve, release candidates to test, review threads, untriaged and blocking bugs, across projects, ordered by urgency.

### FEAT-018: Inbox actions
- **Priority**: low
- **Epic**: EPIC-005
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: Approve a gate, record a manual test result or open the pull request from the inbox, through dcp commands.

### FEAT-019: Gedo adapter
- **Priority**: medium
- **Epic**: EPIC-006
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: CollaborationSource over gedo --json (doctor, radar, scope, stream, check contracts), from Gedo v0.4.0.

### FEAT-020: Agents view
- **Priority**: medium
- **Epic**: EPIC-006
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: Per project: active streams labeled by plan and milestone, their scopes and the collisions found by radar.

### FEAT-021: AnyContext adapter
- **Priority**: medium
- **Epic**: EPIC-007
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: KnowledgeSource over actx --json: workspaces, sources, status and ask with streamed answers and citations.

### FEAT-022: Knowledge view
- **Priority**: medium
- **Epic**: EPIC-007
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: Ask questions about a project and open the cited files; create a workspace for a project from its repository.

### FEAT-023: Suite installer
- **Priority**: medium
- **Epic**: EPIC-008
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: Install and update Aurora and the three engines from their releases repositories on Windows and Linux, using the shared installer core proposed to levix-platform.

### FEAT-024: Release channels
- **Priority**: low
- **Epic**: EPIC-008
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: Stable and release candidate channels per product, chosen in Aurora's settings.

### FEAT-025: License verification
- **Priority**: low
- **Epic**: EPIC-009
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: Verify the suite license and each engine's entry offline with the levix-platform licensing crate.

### FEAT-026: Plan view
- **Priority**: low
- **Epic**: EPIC-009
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: Show the user's plan, what it includes and how to upgrade.

## Improvements

## Adjustments

## Epics

### EPIC-001: Desktop foundation
- **Priority**: high
- **Status**: open
- **Found**: 2026-10-27
- **Description**: The Rust workspace, the Tauri 2 shell, the Leptos interface with the Aurora Borealis theme, CI on Windows and Linux, and packaging (ADR-0001, ADR-0002).

### EPIC-002: Engine integration contract
- **Priority**: high
- **Status**: open
- **Found**: 2026-10-27
- **Description**: Run the engines through their command lines with versioned JSON, discover capabilities, and stand in for engines that are not ready (ADR-0003).

### EPIC-003: Engine health
- **Priority**: high
- **Status**: open
- **Found**: 2026-10-27
- **Description**: Show which engines are installed, their versions, capabilities and what each Aurora feature needs.

### EPIC-004: Project portfolio
- **Priority**: high
- **Status**: open
- **Found**: 2026-10-27
- **Description**: Every registered project and its development cycle state, read from the Dev Cycle.

### EPIC-005: Decision inbox
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-27
- **Description**: Everything waiting for the user across projects, first as a view and later with actions.

### EPIC-006: Agent work view
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-27
- **Description**: Agents running on each repository, their Gedo streams and scopes, and the collisions and contract breaks Gedo reports.

### EPIC-007: Knowledge
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-27
- **Description**: AnyContext questions and answers with citations over each project's sources, inside Aurora.

### EPIC-008: Suite installer and updates
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-27
- **Description**: One installer and updater for Aurora, gedo, dcp and actx with verified downloads and release channels.

### EPIC-009: Licensing and plans
- **Priority**: low
- **Status**: open
- **Found**: 2026-10-27
- **Description**: Suite and per-engine licenses verified offline through levix-platform, with a free tier (ADR-0005).

## Closed
