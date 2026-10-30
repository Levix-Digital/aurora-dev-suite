# Feature Specification: dcp adapter

**Feature Branch**: `feat/002-dcp-adapter`

**Created**: 2026-10-29

**Status**: Draft

**Input**: User description: "dcp adapter: CycleSource over the dcp binary's versioned --json (ADR-0006), built against fixtures that follow the contract. Backlog item FEAT-016, release v0.2.0 Portfolio on the dcp contract."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Read a project's cycle state through dcp (Priority: P1)

A Levix developer has registered a project and has the Dev Cycle Protocol engine (`dcp`) installed with the cycle state capability. Aurora asks `dcp` for the project's cycle state and gets back, in one reply, the current release and its open items, the plans in progress with their milestones, the latest release candidate run, the open pull requests and what each needs, the blocking and untriaged bugs, and the recommended next step. The Portfolio, Project and Inbox views of this release are built on this state.

**Why this priority**: every other item of v0.2.0 (FEAT-014 Portfolio, FEAT-015 Project view, FEAT-017 Inbox) reads the cycle state; without the adapter none of them can be built.

**Independent Test**: With a sample `dcp` that answers the agreed contract, ask Aurora for the cycle state of a registered project and check that every field of the reply arrives in Aurora's own types unchanged.

**Acceptance Scenarios**:

1. **Given** a registered project whose folder is present and a `dcp` that reports the cycle state capability, **When** Aurora asks for the project's cycle state, **Then** it receives the release, plans, release candidate, pull requests, bugs, next step and findings of the reply.
2. **Given** a reply with fields Aurora does not know, **When** it is read, **Then** the unknown fields are ignored and the known ones are used.
3. **Given** a reply whose cycle state has nothing in progress (no release, no plans, no candidate, no pull requests, no bugs), **When** it is read, **Then** Aurora receives an empty but valid state, not an error.

---

### User Story 2 - Know why the cycle state is not available (Priority: P2)

When the cycle state cannot be read, the developer sees one clear reason and what to do: `dcp` is missing, installed without the contract, too old or too new, the project's folder is missing, or the request failed, timed out or returned something invalid.

**Why this priority**: the engine ships later than Aurora (ADR-0006); until then, and whenever something goes wrong, the views must explain themselves instead of looking broken.

**Independent Test**: Ask for the cycle state with `dcp` in each failing situation (missing, without contract, without the capability, unsupported schema version, hanging, crashing, invalid output, project folder missing) and check the reason given for each.

**Acceptance Scenarios**:

1. **Given** `dcp` is missing, installed without the contract, or does not report the cycle state capability, **When** Aurora asks for a project's cycle state, **Then** it does not run `dcp` and returns "unavailable" with what to install or update.
2. **Given** a registered project whose folder is missing, **When** Aurora asks for its cycle state, **Then** it does not run `dcp` and returns "project missing".
3. **Given** `dcp` hangs past its time limit, crashes, exits with an error, or prints output that is not the agreed reply, **When** Aurora asks for the cycle state, **Then** only that request fails, with the reason and the engine's error output kept for the user to open.
4. **Given** a reply with a schema version whose MAJOR Aurora does not support, **When** it is read, **Then** the request fails with "update Aurora" (newer) or "update the engine" (older).

---

### User Story 3 - Build the views before dcp ships the contract (Priority: P3)

A developer of Aurora builds the Portfolio, Project and Inbox views and their tests with realistic sample cycle states, without `dcp` installed, and the sample states follow the same contract the real engine will answer.

**Why this priority**: the engine's contract arrives with a later `dcp` release; the views of v0.2.0 are built now against samples (ADR-0006).

**Independent Test**: Run the automated tests and the interface preview on a machine without `dcp` and check that sample cycle states cover a busy project, an idle project, a project waiting on the user and each failure reason.

**Acceptance Scenarios**:

1. **Given** no `dcp`, **When** the automated tests run, **Then** they pass using the sample cycle states and a sample `dcp` binary.
2. **Given** the interface preview, **When** a view asks for a project's cycle state, **Then** it receives a sample state or a sample failure, chosen per sample project.
3. **Given** the published contract file, **When** the sample replies are checked against it, **Then** every sample is valid.

---

### Edge Cases

- A project registered through a path with spaces or non-ASCII characters: the path is passed to `dcp` as one argument, never through a shell.
- `dcp` writes warnings to its error output but succeeds: the reply is used and the warnings are not shown as a failure.
- `dcp` answers for a different repository than the one asked (the reply's root differs from the project's root): the reply is refused as invalid.
- The reply is larger than the size limit: the request fails as too large; Aurora does not crash.
- `dcp` cannot reach the Git host to read pull requests: the reply still arrives, with an empty list of pull requests and a finding saying why; Aurora shows the finding.
- The `dcp` binary is replaced while Aurora is open: the next request uses the location found by the latest engine probe.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Aurora MUST read a project's cycle state only by running `dcp diagnose --json --dir <project root>` and reading its versioned reply (schema `dcp.diagnose`, ADR-0003, ADR-0006); it MUST NOT read the project's protocol files or run any other program to get it.
- **FR-002**: Aurora MUST publish the contract it expects for the reply as a JSON Schema with the specification and propose it to the Dev Cycle Protocol project; the fixtures and the adapter MUST follow it.
- **FR-003**: Aurora MUST accept every reply with schema MAJOR 1 and ignore unknown fields; it MUST refuse a reply with another MAJOR, saying which side to update.
- **FR-004**: Aurora MUST ask for the cycle state only when the latest engine probe reports `dcp` ready with the `diagnose` capability at schema MAJOR 1, and only for a project whose folder is present.
- **FR-005**: Each request MUST run without a shell, with the time limit set for `dcp` in the settings and the size limits of the command runner; a failure, a timeout or an invalid reply MUST affect only that request and keep the engine's error output.
- **FR-006**: The result of a request MUST be either the cycle state or exactly one failure reason among: engine unavailable (with what to install or update), project missing, timed out, too large, failed (exit code and error output), invalid reply, unsupported schema version.
- **FR-007**: The cycle state MUST carry: the current release with its open items; the plans in progress with milestones done and total; the latest release candidate run with tests not run and failed and whether it was released; the open pull requests with a summary, whether they need action and how many threads wait for the user; the blocking bugs; the untriaged bugs with the reason; the recommended next step with its options; the findings the engine reported; and when it was read.
- **FR-008**: The core MUST define the cycle state source as a port with a real implementation over `dcp` and a sample implementation used by the tests and by the interface preview.
- **FR-009**: The desktop application MUST offer the interface one command that returns a project's cycle state or its failure reason; the interface preview MUST answer the same command with sample data.
- **FR-010**: Every request MUST be logged with the project's root, the duration and the outcome, never the reply's content.

### Key Entities

- **Cycle state**: what the Dev Cycle Protocol says about one project at one moment: current release, plans in progress, release candidate, pull requests, bugs, next step, findings, read time.
- **Backlog item reference**: an item's id, its rank and its title, as the engine reports it.
- **Next step**: whether the engine needs the user to choose, the recommended option, and the options with their keys, titles and reasons.
- **Cycle failure**: why a cycle state could not be read, with what to do and the engine's error output when there is one.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every sample reply in the specification is valid against the published contract, and Aurora reads each of them without loss of a known field.
- **SC-002**: With `dcp` missing, without the contract or without the capability, asking for a cycle state never starts a process and answers within 50 ms.
- **SC-003**: A `dcp` that hangs is reported as timed out within its time limit (10 seconds by default) plus one second.
- **SC-004**: 100% of the automated tests pass on Windows and Linux with no `dcp` installed.
- **SC-005**: The Dev Cycle Protocol project has received the proposed contract through the cross-project inbox, with what it is asked to do and what closes the request.

## Assumptions

- No released `dcp` answers `diagnose --json` yet (its Rust engine has no `diagnose` command and no `version --json` as of 2026-10-29); the adapter is validated with a sample `dcp` and lights up when a `dcp` release reports the `diagnose` capability.
- The contract mirrors the information the protocol's current diagnosis already collects, restructured into typed fields, so the engine can produce it without new analysis.
- One reply per project carries everything the three views of v0.2.0 need, so the Portfolio runs one process per project.
- Reading pull requests is the engine's job and may be slow or offline; the engine, not Aurora, decides how to degrade, and reports it in its findings.
- The Portfolio, Project and Inbox views, the "waiting for the user" counts and the ordering by urgency belong to FEAT-014, FEAT-015 and FEAT-017; this item only delivers the state they read.
- Aurora stays on v0.1.0 features in its interface until the v0.2.0 release; the views keep their "coming in v0.2.0" state in this item.
