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

## v0.4.0: Web application
- **Status**: in_progress
- **Goal**: Aurora runs as a self-hosted web application (ADR-0014): one server per organization with the same core, engines, integrations and interface, packaged as a container image and a Linux binary; without a governing Admin Hub it is a read-only view, and until Admin Hub v0.4.0 it uses a single local development account. It also delivers the Messages chat view (FEAT-039), whose desktop release candidate v0.3.1-rc.1 was cancelled because the move to the web would make that acceptance run obsolete (Levi, 2026-11-10). v0.3.0 "Actions through dcp" (project settings through dcp, Messages through the integrations provider) is not released as desktop either: its items are delivered and ship here, accepted on the web candidate (Levi, 2026-11-10). Messages are read as round tables (ADR-0015): direct messages become two-project tables in the new view, so the pairwise chat is not accepted on its own.
- **Items**: FEAT-027, BUG-004, ADJ-001, FEAT-029, FEAT-031, FEAT-032, FEAT-033, FEAT-034, BUG-005, BUG-006, ADJ-002, BUG-007, ADJ-003, BUG-008, IMP-004, BUG-009, ADJ-005, IMP-005, IMP-006, FEAT-039, FEAT-041, FEAT-042, FEAT-046, FEAT-047, FEAT-048, FEAT-050

## v0.6.0: Integrations
- **Status**: planned
- **Target**: after dcp v2.11.0 (DCP FEAT-018)
- **Goal**: Projects that depend on each other learn about each other's merges automatically and their conversations and decisions (delivered in v0.4.0) become the integration step and the integration gate of the Dev Cycle, through the integrations provider plugged into dcp; waits for dcp v2.11.0 with the extension point (DCP FEAT-018). Approvals go through the Admin Hub (ADR-0014), so this release also waits for Admin Hub v0.4.0 (Hub FEAT-047 sign-in, FEAT-048 signed approval attestations, FEAT-049 e-mail; agreed 2026-11-10). The coordinator's round tables, the integration hold and the required verify check at merge follow ADR-0015.
- **Items**: FEAT-018, FEAT-030, FEAT-036, FEAT-037, FEAT-043, FEAT-044, FEAT-045, FEAT-049, FEAT-051, FEAT-052, FEAT-053

## v0.7.0: Agents
- **Status**: planned
- **Goal**: Parallel agents and Gedo collisions are visible per project; waits for Gedo v0.4.0 (command line) and v0.5.0 (agent streams). Moved after the web application and the integrations (Levi, 2026-11-10).
- **Items**: FEAT-019, FEAT-020

## v0.8.0: Knowledge
- **Status**: planned
- **Goal**: Questions about a project are answered with citations inside Aurora; waits for AnyContext's versioned JSON output. Moved after the web application and the integrations (Levi, 2026-11-10).
- **Items**: FEAT-021, FEAT-022

## v1.0.0: The suite
- **Status**: planned
- **Goal**: One installer, one license and one update for Aurora and the three engines, ready for the public freemium release; waits for levix-platform licensing and catalog.
- **Items**: FEAT-023, FEAT-024, FEAT-025, FEAT-026
