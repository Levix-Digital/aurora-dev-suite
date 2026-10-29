# Backlog

Work captured for later planning: bugs, features, improvements and adjustments, grouped under epics when the product is planned as a whole. Capturing an item takes no discovery; when the user decides to work on items, they are promoted into a plan (`plan-memory/backlog.py promote`), where the discovery and the planning happen. Open bugs come before new work.

Each item: `### <ID>: <title>` with the fields `Severity` (bugs: critical, high, medium, low) or `Priority` (others: high, medium, low), an optional `Epic`, `Status` (open, planned, in_progress), `Found`, `Plan` and `Description`. Epics (`EPIC-NNN`) group items of the complete product; they have no `Plan` and are never promoted: their items are. The roadmap (`docs/ROADMAP.md`) says which items go into which release. Closed items (done, dropped, duplicate, superseded by better defined items) move to `Closed` with a `Closed` note and keep their ids, which are never reused. Edit this file with `plan-memory/backlog.py` or by hand, and check it with `backlog.py validate`.

## Bugs

## Features

### FEAT-005: Desktop packages
- **Priority**: medium
- **Epic**: EPIC-001
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: Windows installer (NSIS or MSI) and Linux packages (AppImage and deb) built in CI for release candidates.

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
- **Priority**: high
- **Epic**: EPIC-004
- **Status**: open
- **Found**: 2026-10-27
- **Plan**: -
- **Description**: CycleSource over the dcp binary's versioned --json (ADR-0006), built against fixtures that follow the contract.

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

### FEAT-027: Project settings
- **Priority**: medium
- **Epic**: EPIC-004
- **Status**: open
- **Found**: 2026-10-28, Dev Cycle Protocol suggestion 2026-10-28
- **Plan**: -
- **Description**: A Settings tab per project rendered from dcp config get --json (schema dcp.config: value, allowed values, default, type, read-only, group); changes previewed with dcp config set --dry-run --json and applied with dcp config set, errors shown next to the field. Aurora never reads or writes .dev-cycle.json and does not commit: the user commits the change. Needs the dcp capability config (ADR-0003, ADR-0006).

### FEAT-028: Cycle metrics
- **Priority**: medium
- **Epic**: EPIC-004
- **Status**: open
- **Found**: 2026-10-28, Dev Cycle Protocol suggestion 2026-10-28
- **Plan**: -
- **Description**: A Metrics tab per project and release (first-pass gate rate, rework, time per phase, escaped bugs, tokens when reported, trends across releases) and a combined view across the portfolio with totals and the protocol's baseline marker, read only from dcp metrics --json (schema dcp.metrics, capability metrics). No stand-in: without the capability the views say to update dcp (ADR-0006). Release assigned when dcp metrics has a date.

### FEAT-029: Integration subscriptions
- **Priority**: medium
- **Epic**: EPIC-010
- **Status**: open
- **Found**: 2026-10-29, 2026-10-29, Levi
- **Plan**: -
- **Description**: A project declares in .aurora/integrations.json, committed once on its work branch, the projects it depends on, their repositories, what it consumes from each (a command line and its JSON, a schema, an API) and the branches it follows. Created and validated through the integrations provider, never edited by Aurora's interface directly.

### FEAT-030: Merge notices
- **Priority**: medium
- **Epic**: EPIC-010
- **Status**: open
- **Found**: 2026-10-29, 2026-10-29, Levi
- **Plan**: -
- **Description**: When a change is merged into a followed branch, the provider writes a notice (title, description, branch, version or tag, changed files and contracts, taken from Git) to the aurora/inbox branch of every project that declared a dependency. Written by the provider at the Dev Cycle's merge or release step, so it works on any Git host (GitHub, Azure DevOps, GitLab, Bitbucket, a local server) without host automation.

### FEAT-031: Integration negotiation
- **Priority**: medium
- **Epic**: EPIC-010
- **Status**: open
- **Found**: 2026-10-29, 2026-10-29, Levi
- **Plan**: -
- **Description**: At the integration step the receiving project checks each notice against what it consumes; when affected it opens an integration (numbered by its own counter, so projects never collide) with a proposal in the other project's aurora/inbox, which answers with an agreement or a counter-proposal. At most three rounds, then the user decides. Messages unrelated to the current work are deferred with a reason and shown to the user. Commands with --json: check, show, propose, counter, agree, defer.

### FEAT-032: Integration gate
- **Priority**: medium
- **Epic**: EPIC-010
- **Status**: open
- **Found**: 2026-10-29, 2026-10-29, Levi
- **Plan**: -
- **Description**: An agreed integration is shown to the user, who approves or rejects it; both projects receive the decision and each records its own backlog item. The gate is registered by the integrations provider through the Dev Cycle's extension point, not built into the protocol.

### FEAT-033: Integrations view
- **Priority**: medium
- **Epic**: EPIC-010
- **Status**: open
- **Found**: 2026-10-29, 2026-10-29, Levi
- **Plan**: -
- **Description**: A tab with notices, open integrations and their conversations, and the integration gate, read from local clones through the provider's --json. Aurora stays air-gapped: it never fetches or pushes; the provider does through Git.

### FEAT-034: Integrations provider
- **Priority**: high
- **Epic**: EPIC-010
- **Status**: open
- **Found**: 2026-10-29, 2026-10-29, Levi
- **Plan**: -
- **Description**: The Rust binary shipped with Aurora (dcp-integrations) that implements the integrations commands and plugs into dcp through its extension point: external subcommand, registered provider, cycle points discovery, after-merge and before-release-gate, and the integration gate (DCP FEAT-018, dcp v2.12.0). Registered at user level by Aurora's installer, it only adds items, warnings and its own gate, never weakens the protocol's gates. It reads and writes the aurora/inbox branch with Git plumbing, without touching the working tree, and is the only writer of integration data. Without Aurora there is no provider and the step does nothing.

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
- **Description**: Run the engines through their Rust binaries' versioned JSON and discover their capabilities (ADR-0003, ADR-0006).

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

### EPIC-010: Aurora Integrations
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-29, 2026-10-29, Levi
- **Description**: Projects that depend on each other learn about each other's merges and agree on integrations before changing code: notices and messages travel through plain Git on an orphan aurora/inbox branch of each project's repository, the Dev Cycle's integration step runs through dcp with any agent or a person, and the user approves each agreed integration at an integration gate. Agnostic to coding agents and Git hosts; available only with Aurora (ADR-0007).

## Closed

### FEAT-013: Dev Cycle stand-in adapter
- **Priority**: high
- **Epic**: EPIC-004
- **Status**: dropped
- **Found**: 2026-10-27
- **Plan**: -
- **Closed**: 2026-10-27, No stand-in adapters (ADR-0006): the Dev Cycle is plugged in only through the Rust dcp binary.
- **Description**: Read .dev-cycle.json, docs/BACKLOG.md, docs/ROADMAP.md and docs/plans frontmatter, and the current scripts' --json when Python is available.

### FEAT-001: Rust workspace with the four crates
- **Priority**: high
- **Epic**: EPIC-001
- **Status**: done
- **Found**: 2026-10-27
- **Plan**: foundations
- **Closed**: 2026-10-27, plan foundations
- **Description**: aurora-core, aurora-engines, aurora-desktop and aurora-ui with workspace dependencies, edition 2024, MSRV 1.95, publish = false, cargo-deny and check_design passing.

### FEAT-002: Tauri 2 shell with a Leptos interface
- **Priority**: high
- **Epic**: EPIC-001
- **Status**: done
- **Found**: 2026-10-27
- **Plan**: foundations
- **Closed**: 2026-10-27, plan foundations
- **Description**: The desktop application opens a window that renders the Leptos interface built by Trunk; one sample command round-trips a core type.

### FEAT-003: Aurora Borealis theme and navigation
- **Priority**: high
- **Epic**: EPIC-001
- **Status**: done
- **Found**: 2026-10-27
- **Plan**: foundations
- **Closed**: 2026-10-27, plan foundations
- **Description**: CSS tokens from the Aurora Borealis palette, light and dark, and the navigation between Portfolio, Project, Engines and Inbox views.

### FEAT-004: CI on Windows and Linux
- **Priority**: high
- **Epic**: EPIC-001
- **Status**: done
- **Found**: 2026-10-27
- **Plan**: foundations
- **Closed**: 2026-10-27, plan foundations
- **Description**: Pull request CI: fmt, clippy, tests, cargo-deny, check_design and a desktop build on windows-latest and ubuntu-latest.

### FEAT-006: Interface preview with fixture data
- **Priority**: medium
- **Epic**: EPIC-001
- **Status**: done
- **Found**: 2026-10-27
- **Plan**: foundations
- **Closed**: 2026-10-27, plan foundations
- **Description**: The interface runs in a browser during development with fixture data, without the desktop shell.

### FEAT-007: Command runner
- **Priority**: high
- **Epic**: EPIC-002
- **Status**: done
- **Found**: 2026-10-27
- **Plan**: foundations
- **Closed**: 2026-10-27, plan foundations
- **Description**: Run an engine command without a shell, with timeout, working directory and standard error capture, and check the JSON envelope (schema, schema_version) with size limits.

### FEAT-008: Engine probe and capabilities
- **Priority**: high
- **Epic**: EPIC-002
- **Status**: done
- **Found**: 2026-10-27
- **Plan**: foundations
- **Closed**: 2026-10-27, plan foundations
- **Description**: Find each engine binary and read its version and capabilities with version --json; each feature declares the capabilities it needs and is disabled with a clear message when one is missing.

### FEAT-009: Fixture adapters for every port
- **Priority**: high
- **Epic**: EPIC-002
- **Status**: done
- **Found**: 2026-10-27
- **Plan**: foundations
- **Closed**: 2026-10-27, plan foundations
- **Description**: Fixture implementations of every core port, used by tests and by the interface preview.

### FEAT-010: Project registry
- **Priority**: high
- **Epic**: EPIC-002
- **Status**: done
- **Found**: 2026-10-27
- **Plan**: foundations
- **Closed**: 2026-10-27, plan foundations
- **Description**: Add, rename and remove projects in projects.toml in the user's configuration directory; validate paths; recover from a corrupt file (ADR-0004).

### FEAT-011: Engines view
- **Priority**: high
- **Epic**: EPIC-003
- **Status**: done
- **Found**: 2026-10-27
- **Plan**: foundations
- **Closed**: 2026-10-27, plan foundations
- **Description**: A view with each engine's status (ready, degraded, missing), version, capabilities, and the features they enable.

### FEAT-012: Refresh and diagnostics
- **Priority**: medium
- **Epic**: EPIC-003
- **Status**: done
- **Found**: 2026-10-27
- **Plan**: foundations
- **Closed**: 2026-10-27, plan foundations
- **Description**: Refresh engine detection on demand and show the last command's standard error for a degraded engine.

### IMP-001: Provisional cross-project messages
- **Priority**: high
- **Epic**: EPIC-010
- **Status**: done
- **Found**: 2026-10-29, 2026-10-29, Levi
- **Plan**: -
- **Closed**: 2026-10-29, aurora/inbox branch of aurora-dev-suite, first message sent 2026-10-29
- **Description**: Until the integrations provider exists, a small provisional tool kept on this repository's aurora/inbox branch (not in the product) sends free messages between Levix projects through each project's aurora/inbox branch, in the message format of ADR-0007, with the same command names and --json output the provider will have.
