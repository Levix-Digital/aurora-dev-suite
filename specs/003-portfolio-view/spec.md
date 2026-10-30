# Feature Specification: Portfolio view

**Feature Branch**: `feat/003-portfolio-view`

**Created**: 2026-10-30

**Status**: Draft

**Input**: User description: "Portfolio view: all registered projects with name, phase, current release and a count of what waits for the user, loaded in parallel and shown progressively. Backlog item FEAT-014, release v0.2.0 Portfolio on the dcp contract."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See where every project stands (Priority: P1)

A Levix developer opens Aurora and the Portfolio shows every registered project with its name, the phase of its development cycle, its current release and how many things wait for them, all read through the Dev Cycle Protocol (`dcp`) without opening each project.

**Why this priority**: it is the first screen of Aurora and the goal of release v0.2.0; without it the cycle state read by FEAT-016 is invisible.

**Independent Test**: In the preview, open the Portfolio and check that each sample project shows the phase, release and count of its sample cycle state.

**Acceptance Scenarios**:

1. **Given** registered projects whose cycle state can be read, **When** the Portfolio opens, **Then** each project shows its phase, its current release (version and name, or "No roadmap") and the number of things waiting for the user.
2. **Given** a project with a release candidate not released, pull requests that need action, blocking bugs or untriaged bugs, **When** its row is shown, **Then** the count is the sum of those items and the row names their kinds.
3. **Given** a project with nothing in progress, **When** its row is shown, **Then** the phase reads "Idle" and the count reads "Nothing waits for you".
4. **Given** the cycle states of all projects, **When** they are loaded, **Then** the header shows the total of things waiting across the projects.

---

### User Story 2 - Projects appear as they are read (Priority: P2)

With many projects, or a slow project, the developer sees each project's state as soon as it is read, and a slow or failing project never holds the others back.

**Why this priority**: each read starts a `dcp` process that may take seconds (pull requests are read from the platform); waiting for all of them would make the first screen feel broken.

**Independent Test**: With one project whose `dcp` read hangs, open the Portfolio and check that the other projects show their state while the slow one still says it is reading.

**Acceptance Scenarios**:

1. **Given** registered projects, **When** the Portfolio opens, **Then** the list of projects is shown at once and each row says it is reading the cycle state until its answer arrives.
2. **Given** several projects, **When** their states are read, **Then** at most four reads run at the same time and each row is updated as soon as its own answer arrives.
3. **Given** a project whose read times out or fails, **When** its answer arrives, **Then** only that row shows the failure, with the reason and the engine's error output available to open.
4. **Given** the user presses "Refresh", **When** the reads run again, **Then** every row keeps its last state, marked as refreshing, until the new answer arrives.

---

### User Story 3 - Know why a project has no state (Priority: P3)

When the cycle state of a project cannot be read, its row says why in a few words and what to do, without hiding the project's name, path and actions (rename, locate, remove).

**Why this priority**: until a `dcp` release ships the contract, every real installation sees this state (ADR-0006); it must read as expected, not as broken.

**Independent Test**: With no `dcp`, open the Portfolio and check that every present project shows "Cycle state unavailable" and the view says what to install once.

**Acceptance Scenarios**:

1. **Given** `dcp` missing, without the contract or without the `diagnose` capability, **When** the Portfolio opens, **Then** the view shows once what to install or update, and each present project's row says "Cycle state unavailable", without starting any process.
2. **Given** a project whose folder is missing, **When** its row is shown, **Then** it keeps the "Missing" badge and says to locate the folder to read its cycle state.
3. **Given** a reply with a schema version Aurora does not support, **When** the row is shown, **Then** it says which side to update.

---

### Edge Cases

- No registered project: the empty state of v0.1.0 is kept; no read runs.
- A project is added, renamed, located or removed while reads run: the list reloads and only the projects without a state are read; an answer for a removed project is dropped.
- The same project is refreshed while its previous read is still running: the later answer wins; an older answer never replaces a newer one.
- The engines have not been probed yet when the Portfolio opens: the desktop probes once and every read waits for that single probe.
- More than four projects: the remaining reads wait their turn; the order of the rows never changes while answers arrive.
- The engine reports findings (for example the platform could not be reached): the row shows the state and notes that some data could not be read.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The Portfolio MUST list every registered project in the registry's order, with its name, path and presence, as in v0.1.0, plus its cycle summary.
- **FR-002**: The cycle summary of a project MUST be derived only from its cycle state (FEAT-016) and carry: the phase, the current release (version and name) or none, the items waiting for the user, and whether the engine reported findings.
- **FR-003**: The phase MUST be one of, in this order of precedence: "Release candidate" (a candidate not released), "Implementation" (a plan in progress), "Review" (an open pull request), "Planning" (a current release with open items), "Idle" (none of these).
- **FR-004**: The items waiting for the user MUST be: the release candidate not released (tests to run, failures to look at, or the release gate when all passed), each pull request that needs action, each blocking bug and each untriaged bug; the count is their number.
- **FR-005**: The phase and the waiting items MUST be computed in the core, so the Project view (FEAT-015) and the Inbox (FEAT-017) use the same rules.
- **FR-006**: The Portfolio MUST read the projects' cycle states in parallel, at most four at a time, and update each row as soon as its answer arrives; a slow or failing read MUST NOT delay the other rows.
- **FR-007**: Each row MUST show exactly one of: reading, the summary, or the failure reason (unavailable, project missing, timed out, too large, failed, invalid, unsupported) with the engine's error output when there is one.
- **FR-008**: The Portfolio MUST offer "Refresh" to read every project again, keeping the last state of each row until its new answer arrives, and MUST drop answers that are older than the latest request for that row or that belong to a project no longer listed.
- **FR-009**: The desktop MUST probe the engines only once when several reads arrive before the first probe has finished.
- **FR-010**: When `dcp` cannot answer for any project (unavailable), the view MUST say what to install or update once, not in every row.
- **FR-011**: The preview MUST show the Portfolio with the sample cycle states and sample failures.

### Key Entities

- **Cycle summary**: what the Portfolio shows of one project's cycle state: phase, current release, waiting items, findings flag.
- **Phase**: where the project stands in its cycle, derived from its cycle state.
- **Waiting item**: one thing that waits for the user in a project: its kind (release candidate, pull request, blocking bug, untriaged bug), a key and a title.
- **Row state**: reading, read (with the summary, possibly refreshing) or failed (with the reason).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In the preview, every sample project shows the phase, release and count expected from its sample state, and the header total equals the sum of the rows.
- **SC-002**: With one project whose read hangs, every other row shows its state before the hanging read times out.
- **SC-003**: With 20 projects, never more than four `dcp` reads run at the same time.
- **SC-004**: With `dcp` missing, opening the Portfolio starts no process and every row shows its unavailable state within one second.
- **SC-005**: 100% of the automated tests pass on Windows and Linux with no `dcp` installed.

## Assumptions

- The phase is derived from the fields of `dcp.diagnose` 1.x; the contract has no phase field, and asking the Dev Cycle Protocol for one is not needed for this view.
- The recommended next step (`next`) is not counted as a waiting item: its options repeat the items above. The Project view (FEAT-015) shows it.
- The count is what the Inbox (FEAT-017) will list; ordering by urgency belongs to FEAT-017.
- The cycle states are not kept between sessions (ADR-0004); opening the Portfolio reads them again.
- The view keeps its "Coming in v0.2.0" availability text when `dcp` is ready, until the v0.2.0 release sets the build's release; with no `dcp` with the contract, it shows what to install.
- Clicking a project to open the Project view comes with FEAT-015.
