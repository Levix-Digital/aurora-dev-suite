# Roadmap

The releases of the product in delivery order, from the MVP onwards. Each release has a goal and the backlog items (`docs/BACKLOG.md`) it delivers; the roadmap is the only place that says which item goes into which release. Discovery starts from the open items of the current release (the first one not released), after the bugs that come first.

Each release: `## vX.Y.Z: <name>` with the fields `Status` (planned, in_progress, released), an optional `Target`, `Released` (the date, once released), `Goal` and `Items` (backlog ids separated by commas, `-` when empty). Edit this file with `plan-memory/roadmap.py` or by hand, and check it with `roadmap.py validate`.

## v0.1.0: Foundations
- **Status**: planned
- **Goal**: The Levix team opens Aurora on Windows and Linux, registers its projects and sees which engines are installed and what each feature needs, with every engine behind a port and fixtures in place.
- **Items**: FEAT-001, FEAT-002, FEAT-003, FEAT-004, FEAT-006, FEAT-007, FEAT-008, FEAT-009, FEAT-010, FEAT-011, FEAT-012

## v0.2.0: Portfolio from today's Dev Cycle
- **Status**: planned
- **Goal**: Every registered project's cycle state and what waits for the user, read from the Dev Cycle files and scripts that exist today, and installable packages.
- **Items**: FEAT-013, FEAT-014, FEAT-015, FEAT-017, FEAT-005

## v0.3.0: dcp contract
- **Status**: planned
- **Goal**: Aurora reads the Dev Cycle through the dcp binary's versioned JSON and can act on the inbox; waits for the first dcp release with the contract.
- **Items**: FEAT-016, FEAT-018

## v0.4.0: Agents
- **Status**: planned
- **Goal**: Parallel agents and Gedo collisions are visible per project; waits for Gedo v0.4.0 (command line) and v0.5.0 (agent streams).
- **Items**: FEAT-019, FEAT-020

## v0.5.0: Knowledge
- **Status**: planned
- **Goal**: Questions about a project are answered with citations inside Aurora; waits for AnyContext's versioned JSON output.
- **Items**: FEAT-021, FEAT-022

## v1.0.0: The suite
- **Status**: planned
- **Goal**: One installer, one license and one update for Aurora and the three engines, ready for the public freemium release; waits for levix-platform licensing and catalog.
- **Items**: FEAT-023, FEAT-024, FEAT-025, FEAT-026
