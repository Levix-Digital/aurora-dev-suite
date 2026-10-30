# Feature Specification: Project view

**Feature Branch**: `feat/004-project-view`

**Created**: 2026-10-30

**Status**: Draft

**Input**: User description: "Project view: one project's cycle state: plans and milestones, backlog summary, roadmap, release candidate, pull requests, gates and blocking bugs. Backlog item FEAT-015, release v0.2.0 Portfolio on the dcp contract."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See one project's whole cycle (Priority: P1)

From the Portfolio, a Levix developer opens a project and the Project view shows everything the Dev Cycle Protocol (`dcp`) reports about it: the phase, the current release and its open items, the plans in progress with their milestones, the release candidate, the open pull requests, the blocking and untriaged bugs, and what waits for them, with the same phase and waiting rules as the Portfolio.

**Why this priority**: the Portfolio says where each project stands; the Project view says why and what to do, without opening the repository.

**Independent Test**: In the preview, open `gedo` from the Portfolio and check that each section shows the data of the busy sample.

**Acceptance Scenarios**:

1. **Given** the Portfolio, **When** the user presses "Open" on a project, **Then** the Project view opens on that project and reads its cycle state.
2. **Given** a cycle state, **When** it is shown, **Then** the view shows the phase and the count of waiting items exactly as the Portfolio row of the same project, the engine's summary line and when the state was read.
3. **Given** plans in progress, **When** they are shown, **Then** each plan shows its title, status and milestones done out of the total, with a progress bar.
4. **Given** a current release, **When** it is shown, **Then** the view shows its version, name and open items (id, priority or severity, title); without a roadmap it says "No roadmap".
5. **Given** a release candidate, **When** it is shown, **Then** the view shows its run, version, tests not run, tests failed and whether it is released.
6. **Given** open pull requests, **When** they are shown, **Then** each shows its number, title, summary, whether it needs the user's action and how many review threads wait for the user.
7. **Given** blocking or untriaged bugs, **When** they are shown, **Then** each shows its id, severity and title, and an untriaged bug shows why it counts as untriaged.
8. **Given** a section with nothing to show, **When** the view is shown, **Then** the section says so in one line (for example "No plans in progress") instead of disappearing.

---

### User Story 2 - Know which decision is the user's (Priority: P2)

The developer sees the gates that wait for them and the step the protocol recommends, so they know what to decide before work goes on.

**Why this priority**: the cycle's gates (release candidate acceptance, the release gate, a choice the protocol asks for) are the user's decisions and the reason to open a project.

**Independent Test**: In the preview, open `ai-code-agent-skills` and check that the gate "Release candidate acceptance" and the recommended step are shown; open `any-context` and check that no gate is shown.

**Acceptance Scenarios**:

1. **Given** the protocol asks the user to choose (`next.ask`), **When** the view is shown, **Then** a "Decision" gate names the recommended option.
2. **Given** a release candidate not released with tests not run or failed, **When** the view is shown, **Then** a "Release candidate acceptance" gate says how many tests are not run and how many failed.
3. **Given** a release candidate not released whose tests all passed, **When** the view is shown, **Then** a "Release gate" says the release waits for the user's approval.
4. **Given** the recommended next step and its options, **When** they are shown, **Then** the recommended option comes first and is marked, each with its reason.
5. **Given** nothing to decide, **When** the view is shown, **Then** it says "No gate waits for you".

---

### User Story 3 - Switch, refresh and understand failures (Priority: P3)

The developer switches between projects inside the view, refreshes the state, and when the state cannot be read the view says why, as the Portfolio does.

**Why this priority**: a view that cannot be refreshed or that hides why it is empty reads as broken.

**Independent Test**: In the preview, open the Project view without choosing a project, choose `levix-platform` (missing) and then `any-context`, and press "Refresh".

**Acceptance Scenarios**:

1. **Given** the Project view opened from the navigation with no project chosen, **When** it opens, **Then** it shows the first registered project and a list to choose another; with no registered project it says to add one in the Portfolio.
2. **Given** the user chooses another project, **When** its answer arrives, **Then** only the latest choice is shown; an answer for an earlier choice is dropped.
3. **Given** the user presses "Refresh", **When** the read runs, **Then** the last state stays, marked as refreshing, until the new answer arrives.
4. **Given** a read that fails (unavailable, project missing, timed out, too large, failed, invalid, unsupported), **When** the answer arrives, **Then** the view shows the same reason as the Portfolio row, with the engine's error output available to open.
5. **Given** the engine reports findings, **When** the state is shown, **Then** each finding is listed.

---

### Edge Cases

- The chosen project is removed from the registry: the view falls back to the first registered project.
- A project is renamed: the view keeps it chosen (projects are chosen by id).
- `dcp` unavailable: the availability line at the top says what to install, once, and the view says "Cycle state unavailable".
- A plan with zero milestones: the bar is empty and the text reads "No milestones".
- The recommended key is not among the options: the options are shown in their order, none marked.
- A long list (many pull requests or open items): every item is shown; nothing is truncated.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Each Portfolio row MUST offer "Open", which opens the Project view on that project.
- **FR-002**: The Project view MUST read one project's cycle state through the existing `project_cycle_state` command and MUST drop an answer that is older than the latest request.
- **FR-003**: The view MUST show the phase and the waiting items with the rules of FEAT-014 (`CycleSummary`), so both views always agree.
- **FR-004**: The view MUST show the sections: overview (phase, release, summary line, read time, findings), gates, next step, what waits for you, plans and milestones, roadmap (current release and open items), release candidate, pull requests, blocking bugs, untriaged bugs.
- **FR-005**: The gates MUST be derived in the core from the cycle state: "Decision" when the protocol asks the user to choose, "Release candidate acceptance" for an unreleased candidate with tests not run or failed, "Release gate" for an unreleased candidate whose tests all passed.
- **FR-006**: The next step MUST list the recommended option first, marked, then the other options in their order, each with its reason when there is one.
- **FR-007**: The view MUST offer a choice of the registered projects and "Refresh", keeping the last state while a read runs.
- **FR-008**: A failure MUST be shown with the same text as in the Portfolio row (one shared rule), with the engine's error output in a disclosure.
- **FR-009**: The chosen project MUST be kept while Aurora runs (switching views and back keeps it) and is not saved between sessions.
- **FR-010**: The preview MUST show the Project view with the sample cycle states and failures.

### Key Entities

- **Gate**: a decision that waits for the user: its kind (decision, release candidate acceptance, release gate), a title and a detail.
- **Candidate stage**: where an unreleased release candidate stands: tests to run, failures to look at, ready for release; or released.
- **Plan progress**: milestones done out of the total, as a fraction for the bar.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In the preview, every sample project's Project view shows the phase and waiting count of its Portfolio row and every field of its sample state.
- **SC-002**: Choosing projects quickly one after the other always ends on the last one chosen.
- **SC-003**: With `dcp` missing, the view starts no process and shows the unavailable state within one second.
- **SC-004**: 100% of the automated tests pass on Windows and Linux with no `dcp` installed.

## Assumptions

- The data comes only from `dcp.diagnose` 1.x (FEAT-016); plan approval (gate 3) is not in the contract, so a plan waiting for approval shows as its status, not as a gate.
- The backlog summary is what the contract carries: the current release's open items and the bugs; a full backlog listing would need a later contract MINOR.
- Actions (approve, run the tests, answer a review) come in v0.3.0 through `dcp`; this view only reads.
- The chosen project is not stored (ADR-0004).
