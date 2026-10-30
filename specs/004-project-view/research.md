# Research: Project view

## R1. Gates from the contract

**Decision**: derive the gates in the core from `dcp.diagnose` 1.x: a "Decision" when `next.ask` is true, and one gate for an unreleased release candidate, "Release candidate acceptance" while tests are not run or failed, "Release gate" when all passed.

**Rationale**: these are the user's decisions the contract can see. Plan approval (gate 3) happens in the agent's session before a plan is saved, so the contract has no field for it; a plan in progress already passed it. A rule in the core keeps the Project view and the Inbox (FEAT-017) in agreement.

**Alternatives considered**: asking the Dev Cycle Protocol for a `gates` field in a contract MINOR (not needed for what the view shows; kept as a later option); showing `next.options` alone (does not say which ones are gates).

## R2. One candidate stage for the waiting title and the gate

**Decision**: add `CandidateStage` (tests to run, failures to look at, ready for release, released) computed from a `CandidateRun`, and use it for the Portfolio's waiting title (unchanged texts) and for the gate.

**Rationale**: the same three conditions decided the waiting title of FEAT-014; one function avoids two copies of the rule.

## R3. Opening a project and keeping the choice

**Decision**: the shell keeps the chosen project id in the shared `AppContext` (`project: RwSignal<Option<String>>`); "Open" in a Portfolio row sets it and switches to the Project view. Opened from the navigation with no choice, the view uses the first registered project. Nothing is saved.

**Rationale**: navigation is already one signal with no router (spec 001 research R11); a second signal is the smallest change. Saving the choice would add a setting for little value (ADR-0004).

**Alternatives considered**: a router with a project path (a new dependency and a second navigation model); a saved "last project" setting (a settings migration for a convenience).

## R4. One read, latest answer wins

**Decision**: the view reads through `project_cycle_state`, with a request counter: an answer is applied only when its number is the latest. Refresh keeps the last state, marked as refreshing.

**Rationale**: only one project is read at a time, so the Portfolio's loader (a parallel scheduler) is more than needed; a counter covers quick switching (SC-002).

## R5. One failure text for both views

**Decision**: move the failure text and the engine output disclosure of the Portfolio row into a shared component in `views/mod.rs`, used by both views.

**Rationale**: FR-008; the texts cannot drift apart.
