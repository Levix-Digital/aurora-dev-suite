# Research: Portfolio view

## R1. Where the phase comes from

**Decision**: derive the phase in the core from the fields of `dcp.diagnose` 1.x, by precedence: release candidate not released, plan in progress, open pull request, current release with open items, idle.

**Rationale**: the contract agreed with the Dev Cycle Protocol (DCP FEAT-019) has no phase field, and every phase the Portfolio needs can be read from the fields it has. A rule in the core is tested once and shared by the Portfolio, the Project view and the Inbox.

**Alternatives considered**: asking the Dev Cycle Protocol for a `phase` field in a MINOR of the contract (a cross-project request for something Aurora can compute; kept as a later option if the rules diverge from the protocol's own phases); showing the engine's `summary` line instead (free text, not comparable across projects).

## R2. What counts as waiting for the user

**Decision**: the unreleased release candidate (one item), each pull request with `needs_action`, each blocking bug and each untriaged bug. The recommended next step is not counted.

**Rationale**: these are the decisions the protocol gives to the user (release candidate acceptance and release gate, review threads, bug promotion, triage). The options of `next` point at the same items (`rc:`, `pr:`, `bugs`, `triage`), so counting them would count twice. The Inbox (FEAT-017) lists the same items, ordered by urgency.

**Alternatives considered**: counting `threads_for_user` instead of pull requests (inflates a single review into many items); counting `next.ask` as one decision (duplicates the items above).

## R3. Loading in parallel

**Decision**: the interface reads the projects through the existing `project_cycle_state` command, with a small loader that keeps at most four requests in flight and applies each answer as it arrives, tagged with a generation number so older answers are dropped.

**Rationale**: the desktop already runs each command on the blocking pool, so concurrent commands run in parallel without a new command or event. The limit keeps a portfolio of 50 projects from starting 50 `dcp` processes at once; four matches the engine probe's parallelism and a typical machine. No new desktop command means no contract change.

**Alternatives considered**: one `portfolio` command that reads all projects and emits an event per project (a second path to the same data, plus events to subscribe and unsubscribe); a semaphore in the desktop (the interface is the only caller that fans out, and it would still need to apply answers progressively).

## R4. One probe for many reads

**Decision**: the desktop serializes the first engine probe behind a lock and checks the last overview again after taking it.

**Rationale**: `project_cycle_state` calls `engines_overview`, which probes when nothing is cached; four reads arriving together at start-up would otherwise probe the three engines four times.

## R5. Where the unavailable hint is shown

**Decision**: the view keeps the Portfolio availability line (`FeatureState`) at the top, which already says what to install or update; rows with the `unavailable` reason only say "Cycle state unavailable".

**Rationale**: until `dcp` ships the contract, every row would repeat the same sentence.
