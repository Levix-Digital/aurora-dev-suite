# Roadmap

The releases of the product in delivery order, from the MVP onwards. Each release has a goal and the backlog items (`docs/BACKLOG.md`) it delivers; the roadmap is the only place that says which item goes into which release. Discovery starts from the open items of the current release (the first one not released), after the bugs that come first.

Each release: `## vX.Y.Z: <name>` with the fields `Status` (planned, in_progress, released), an optional `Target`, `Released` (the date, once released), `Goal` and `Items` (backlog ids separated by commas, `-` when empty). Edit this file with `plan-memory/roadmap.py` or by hand, and check it with `roadmap.py validate`.

## v0.1.0: Foundations
- **Status**: released
- **Target**: shipped with v0.2.0, no separate release
- **Released**: 2026-10-31
- **Goal**: The Levix team opens Aurora on Windows and Linux, registers its projects and sees which engines are installed and what each feature needs, with every engine behind a port and fixtures in place.
- **Items**: FEAT-001, FEAT-002, FEAT-003, FEAT-004, FEAT-006, FEAT-007, FEAT-008, FEAT-009, FEAT-010, FEAT-011, FEAT-012

## v0.2.0: Portfolio on the dcp contract
- **Status**: released
- **Target**: lights up with dcp v2.11.0 (DCP FEAT-019)
- **Released**: 2026-10-31
- **Goal**: Every registered project's cycle state and what waits for the user, read through the dcp binary's versioned JSON; built and tested against fixtures, it lights up when the first dcp release with the contract is installed. Plus installable packages.
- **Items**: FEAT-014, FEAT-015, FEAT-017, FEAT-005, FEAT-016

## v0.3.0: Actions through dcp
- **Status**: in_progress
- **Target**: project settings light up with dcp v2.11.0 (DCP FEAT-017)
- **Goal**: Change a project's Dev Cycle settings through dcp commands; and talk to the other projects from Aurora's Messages tab, through the integrations provider shipped with Aurora, deciding on each conversation's closing record or stepping into it (Levi, 2026-11-06). The inbox actions (FEAT-018, on hold) moved to v0.6.0, which also waits for dcp v2.11.0.
- **Items**: FEAT-027, BUG-004, ADJ-001, FEAT-029, FEAT-031, FEAT-032, FEAT-033, FEAT-034, BUG-005, BUG-006, ADJ-002, BUG-007, ADJ-003, BUG-008, IMP-004, BUG-009, ADJ-005

## v0.4.0: Agents
- **Status**: planned
- **Goal**: Parallel agents and Gedo collisions are visible per project; waits for Gedo v0.4.0 (command line) and v0.5.0 (agent streams).
- **Items**: FEAT-019, FEAT-020

## v0.5.0: Knowledge
- **Status**: planned
- **Goal**: Questions about a project are answered with citations inside Aurora; waits for AnyContext's versioned JSON output.
- **Items**: FEAT-021, FEAT-022

## v0.6.0: Integrations
- **Status**: planned
- **Target**: after dcp v2.11.0 (DCP FEAT-018)
- **Goal**: Projects that depend on each other learn about each other's merges automatically and their conversations and decisions (delivered in v0.3.0) become the integration step and the integration gate of the Dev Cycle, through the integrations provider plugged into dcp; waits for dcp v2.11.0 with the extension point (DCP FEAT-018).
- **Items**: FEAT-018, FEAT-030, FEAT-036, FEAT-037

## v1.0.0: The suite
- **Status**: planned
- **Goal**: One installer, one license and one update for Aurora and the three engines, ready for the public freemium release; waits for levix-platform licensing and catalog.
- **Items**: FEAT-023, FEAT-024, FEAT-025, FEAT-026
