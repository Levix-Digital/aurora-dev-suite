# Feature Specification: Inbox

**Feature Branch**: `feat/005-inbox`

**Created**: 2026-10-30

**Status**: Draft

**Input**: User description: "Inbox view: gates to approve, release candidates to test, review threads, untriaged and blocking bugs, across projects, ordered by urgency. Backlog item FEAT-017, release v0.2.0 Portfolio on the dcp contract."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See everything that waits for me, most urgent first (Priority: P1)

A Levix developer opens the Inbox and sees, in one list across every registered project, what waits for them in the Dev Cycle Protocol (`dcp`): blocking bugs, release candidates with failed tests, release candidates to test or to release, pull requests with review threads for them, and bugs to triage, ordered by urgency.

**Why this priority**: the Portfolio counts what waits per project; the Inbox answers "what do I do first" without opening each project.

**Independent Test**: In the preview, open the Inbox and check that the four sample items are listed in the order of the quickstart.

**Acceptance Scenarios**:

1. **Given** registered projects, **When** the Inbox opens, **Then** it reads every project's cycle state, at most four at once, and lists each item as its answer arrives.
2. **Given** the items of every project read so far, **When** they are shown, **Then** they are grouped by urgency in this order: "Blocks a release", "Waits for your acceptance", "Waits for your review", "To triage"; inside a group, by the projects' order in the Portfolio, then by the item's order in its project.
3. **Given** an item, **When** it is shown, **Then** it shows its project, its kind, its title (the same title as the Project view's "What waits for you") and its detail: a bug's severity, a release candidate's tests not run and failed, a pull request's summary and review threads for the user, an untriaged bug's reason.
4. **Given** the items, **When** the header is shown, **Then** it says how many items wait across how many projects, and the total equals the sum of the Portfolio's waiting counts.
5. **Given** nothing waits in any project read, **When** the Inbox is shown, **Then** it says "Nothing waits for you".

---

### User Story 2 - Go to the project of an item (Priority: P2)

From an item, the developer opens its project to see the context (gates, next step, plans) before deciding.

**Why this priority**: the Inbox only reads in v0.2.0 (actions come with FEAT-018 in v0.3.0); the Project view is where the context is.

**Independent Test**: In the preview, press "Open" on the release candidate item and check that the Project view opens on `ai-code-agent-skills`.

**Acceptance Scenarios**:

1. **Given** an item, **When** the user presses "Open", **Then** the Project view opens on the item's project.

---

### User Story 3 - Refresh and know what could not be read (Priority: P3)

The developer refreshes the Inbox, and when a project cannot be read the Inbox says which one and why, so an empty list is never mistaken for nothing to do.

**Why this priority**: a project that fails silently would hide its waiting items.

**Independent Test**: In the preview, check that `levix-platform` is listed under "Not read" with the text to locate its folder, and press "Refresh".

**Acceptance Scenarios**:

1. **Given** the user presses "Refresh", **When** the reads run, **Then** the last items stay until each new answer arrives, and the header says how many projects are still being read.
2. **Given** a project whose read fails, **When** its answer arrives, **Then** it is listed under "Not read" with the same failure text as its Portfolio row, the engine's error output available to open, and "Open".
3. **Given** `dcp` is unavailable, **When** the Inbox is shown, **Then** the availability line says what to install once and the Inbox says "Cycle state unavailable" once, not once per project.
4. **Given** a project whose state has findings, **When** its items are shown, **Then** the header notes that part of a project could not be read.

---

### Edge Cases

- No registered project: the Inbox says to add one in the Portfolio.
- A project is removed or renamed in the Portfolio: the Inbox lists the projects again when it is opened next; a renamed project keeps its items (projects are known by id).
- A pull request open but not needing the user's action, a released candidate, a plan in progress: not in the Inbox (they wait for nobody, or for the agent).
- The protocol asks the user to choose (`next.ask`): not an Inbox item, since its options point at the same items; the Decision gate stays in the Project view.
- A long list: every item is shown; nothing is truncated.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The Inbox MUST read every registered project's cycle state through the existing `project_cycle_state` command, with the Portfolio's parallel loader (at most four reads at once, latest answer wins, answers for projects no longer listed dropped).
- **FR-002**: The Inbox items MUST be the Portfolio's waiting items (FEAT-014 `WaitingItem`), so the Inbox total and the Portfolio's waiting counts always agree.
- **FR-003**: Each waiting item MUST carry an urgency derived in the core: blocking bugs and release candidates with failed tests block a release; release candidates with tests to run or ready for release wait for the user's acceptance; pull requests that need action wait for the user's review; untriaged bugs are to triage.
- **FR-004**: The items MUST be ordered by urgency, then by the projects' registry order, then by the item's order in its project; the order MUST be computed in the core.
- **FR-005**: Each item MUST show its project name, kind, title and detail, and offer "Open", which opens the Project view on its project.
- **FR-006**: The header MUST show the total of items and of projects with items, and how many projects are still being read; Refresh MUST keep the last items until the new answers arrive.
- **FR-007**: Projects that could not be read MUST be listed under "Not read" with the shared failure text (`CycleFailure`); when every failure is `dcp` unavailable, one line says so.
- **FR-008**: The preview MUST show the Inbox with the sample cycle states and failures.

### Key Entities

- **Urgency**: how soon an item needs the user: blocks a release, waits for acceptance, waits for review, to triage.
- **Inbox entry**: a waiting item with its project id and name.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In the preview, the Inbox lists the 4 sample items in the quickstart's order and "Not read" lists `levix-platform`.
- **SC-002**: The Inbox total always equals the Portfolio header's total for the same answers.
- **SC-003**: With `dcp` missing, the Inbox starts no process and shows the unavailable state within one second.
- **SC-004**: 100% of the automated tests pass on Windows and Linux with no `dcp` installed.

## Assumptions

- The data comes only from `dcp.diagnose` 1.x (FEAT-016); review threads are the `threads_for_user` count of a pull request, without the threads' text.
- Plan approval (gate 3) is not in the contract, as in the Project view.
- Actions (approve, record a result, open the pull request) come with FEAT-018 in v0.3.0; this view only reads.
- The Inbox reads on its own when opened, as the Portfolio does; nothing is stored (ADR-0004).
