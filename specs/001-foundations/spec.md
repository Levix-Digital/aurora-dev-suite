# Feature Specification: Foundations

**Feature Branch**: `feat/001-foundations`

**Created**: 2026-10-27

**Status**: Draft

**Input**: User description: "v0.1.0 Foundations of Aurora Dev Suite: the Levix team opens Aurora on Windows and Linux, registers its projects and sees which engines are installed and what each feature needs, with every engine behind a port and fixtures in place. Backlog items FEAT-001 to FEAT-004 and FEAT-006 to FEAT-012."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Open Aurora and move between its views (Priority: P1)

A Levix developer installs a build of Aurora on Windows or Linux, opens it and finds a window in the Aurora Borealis look with four places to go: Portfolio, Project, Engines and Inbox. Views whose data comes in later releases say so plainly instead of looking broken.

**Why this priority**: Everything else lives inside this window; without it nothing can be shown or tested on real machines.

**Independent Test**: Build and open the application on Windows and on Linux, move through the four views with the mouse and with the keyboard only, switch between light and dark, and check that each view either shows its content or an explanatory empty state.

**Acceptance Scenarios**:

1. **Given** a fresh build on Windows or Linux, **When** the user opens Aurora, **Then** the main window appears with the navigation and the Portfolio view selected.
2. **Given** the main window, **When** the user moves between views using only the keyboard, **Then** every view is reachable and the focused element is always visible.
3. **Given** the operating system is set to dark (or light) appearance, **When** Aurora opens, **Then** it uses the matching theme, and the user can override it in the settings.
4. **Given** the Project or Inbox view in this release, **When** the user opens it, **Then** it explains what it will show and which release brings it.

---

### User Story 2 - Register the projects to follow (Priority: P2)

The developer adds the repositories they work on (for example the local clones of gedo, any-context and ai-code-agent-skills), gives them friendly names, and removes the ones they no longer follow. The list is there again the next time Aurora opens.

**Why this priority**: The project list is Aurora's only own data and the entry point of every later view (portfolio, agents, knowledge).

**Independent Test**: Add three repositories, rename one, remove one, close and reopen Aurora, and check that the list survived; then try to add a folder that is not a repository and check that it is refused with a reason.

**Acceptance Scenarios**:

1. **Given** an empty list, **When** the user picks a folder that is a Git repository, **Then** it appears in the Portfolio with the folder name as its display name.
2. **Given** a registered project, **When** the user renames or removes it, **Then** the Portfolio shows the change at once and it is kept after a restart; the repository on disk is never touched.
3. **Given** the user picks a folder that does not exist, is not inside a Git repository, or is already registered, **When** they confirm, **Then** nothing is added and a message says why.
4. **Given** a registered project whose folder was moved or deleted, **When** Aurora opens, **Then** the project is listed as missing with its last known path, and the user can locate it again or remove it.
5. **Given** the saved list is unreadable (damaged by hand or by a crash), **When** Aurora opens, **Then** the damaged file is kept aside under a new name, the list starts empty, and a message tells the user where the old file is.

---

### User Story 3 - See which engines are installed and what they enable (Priority: P3)

The developer opens the Engines view and sees, for the Dev Cycle Protocol, Gedo and AnyContext, whether each is installed, its version, what it can do, and which Aurora features it enables or blocks. When something is missing or too old, the view says what to install or update.

**Why this priority**: Aurora's value depends on the engines; knowing exactly what each one provides is the base for every feature that is plugged in later, and it already tells the team what is missing on each machine.

**Independent Test**: Run Aurora on a machine without any engine, then with sample engines that report different versions and capabilities, and check that each engine's status, version, capabilities and enabled features match what was installed.

**Acceptance Scenarios**:

1. **Given** no engine is installed, **When** the user opens the Engines view, **Then** each engine is shown as missing with what to install, and Aurora keeps working.
2. **Given** an engine that reports its version and capabilities in the agreed machine-readable form, **When** the view loads, **Then** it shows the engine as ready with its version and capabilities and lists the Aurora features it enables.
3. **Given** an engine that is installed but does not offer the agreed machine-readable form yet (for example today's AnyContext), **When** the view loads, **Then** it shows the engine as installed without the contract and says which Aurora features are unavailable and what to update.
4. **Given** an engine that reports a contract version Aurora does not support, **When** the view loads, **Then** it shows the engine as degraded with "update Aurora" or "update the engine", whichever applies.
5. **Given** an engine that hangs or crashes when asked, **When** the view loads, **Then** only that engine is shown as degraded after the time limit, the others load normally, and the user can open the engine's last error output.
6. **Given** the user installs or updates an engine while Aurora is open, **When** they ask to refresh, **Then** the view reflects the new state without restarting Aurora.

---

### User Story 4 - Develop and demonstrate views without the engines (Priority: P4)

A developer of Aurora works on the interface with realistic sample data, without installing any engine and without starting the desktop application, and every automated test runs against the same sample data.

**Why this priority**: The engines arrive over the coming months; the views and the rules behind them must be built and tested before that.

**Independent Test**: Start the interface on its own in a browser and check that every view shows the sample data; run the automated tests on a machine without any engine.

**Acceptance Scenarios**:

1. **Given** no engine and no desktop application, **When** the developer starts the interface preview, **Then** every view renders with sample projects and engines covering the ready, degraded and missing states.
2. **Given** a machine without any engine, **When** the automated tests run, **Then** they all pass using sample data only.

---

### Edge Cases

- Two engines report the same capability name: each feature checks the capability of the engine it needs, never another engine's.
- An engine prints far more output than expected or output that is not valid: the reply is cut at a size limit and the engine is shown as degraded, never crashing Aurora.
- The user registers a sub-folder of a repository: Aurora registers the repository root and says so.
- The same repository is reached through two paths (a link or a different letter case on Windows): it is detected as already registered.
- The configuration directory is not writable: Aurora opens, shows the list for this session only and warns that changes will not be kept.
- Linux without the required web engine component installed: the application tells the user which system package is missing instead of failing silently.

## Requirements *(mandatory)*

### Functional Requirements

**Application and navigation**

- **FR-001**: Aurora MUST open as a desktop application on Windows 10 22H2 or later, Windows 11, and Linux (Ubuntu 22.04 or later and equivalents).
- **FR-002**: Aurora MUST offer navigation between the Portfolio, Project, Engines and Inbox views, reachable with the mouse and with the keyboard only.
- **FR-003**: Aurora MUST follow the operating system's light or dark appearance by default and let the user choose light, dark or system in its settings, remembered across restarts.
- **FR-004**: Every view MUST show, per section, one of: ready, degraded with the reason, unavailable with what is needed, or not yet available in this release.

**Projects**

- **FR-005**: Users MUST be able to add a project by choosing a folder; Aurora MUST accept only folders inside a Git repository and register the repository's root.
- **FR-006**: Users MUST be able to rename and remove a project; removing never changes anything on disk outside Aurora's own files.
- **FR-007**: Aurora MUST refuse a project that is already registered, also when reached through another path to the same folder.
- **FR-008**: Aurora MUST keep the list of projects and the settings across restarts, in the user's own configuration area, as its only stored data.
- **FR-009**: Aurora MUST show a registered project whose folder is missing as missing, and let the user locate it again or remove it.
- **FR-010**: When the stored list cannot be read, Aurora MUST keep the damaged file aside, start with an empty list and tell the user where the damaged file is.

**Engines**

- **FR-011**: Aurora MUST look for each engine (Dev Cycle Protocol, Gedo, AnyContext) in the system's search path and in a location the user can set per engine in the settings.
- **FR-012**: Aurora MUST ask each found engine for its version and capabilities in the versioned machine-readable form agreed with the engines (ADR-0003), and accept every contract version it supports.
- **FR-013**: For an engine found without that form, Aurora MUST report it as installed without the contract. Aurora MUST NOT use any other way to read an engine (no scripts, interpreters or engine files; ADR-0006).
- **FR-014**: Aurora MUST know which capabilities each of its features needs, show for each engine the features it enables, and for a missing capability say what to install or update.
- **FR-015**: Every request to an engine MUST run without a shell, with a time limit and a size limit on its reply; a failure, a timeout or an invalid reply affects only that engine and keeps its error output for the user to open.
- **FR-016**: Users MUST be able to refresh engine detection without restarting Aurora.

**Development and quality**

- **FR-017**: Every source of data (projects, engines, cycle state, agent work, knowledge) MUST have a sample implementation with realistic data covering the ready, degraded and missing states, used by the automated tests and by the interface preview.
- **FR-018**: The interface MUST be runnable on its own in a browser with the sample data, without the desktop application.
- **FR-019**: Every change MUST be verified automatically on Windows and on Linux before it is merged: formatting, lint, tests, dependency licenses and advisories, the architecture dependency rule, and a desktop build.
- **FR-020**: The core of Aurora MUST not depend on the desktop framework, the interface framework, the engine adapters or process execution, and this MUST be checked automatically.

### Key Entities

- **Project**: a Git repository the user follows: root path, display name, date added, and whether its folder is currently present.
- **Engine**: one of the Dev Cycle Protocol, Gedo or AnyContext: where it was found, its version, its contract versions, its capabilities, its status (ready, installed without contract, degraded, missing) and its last error output.
- **Capability**: a named ability an engine reports, with the contract version of its machine-readable output.
- **Feature requirement**: an Aurora feature and the engine capabilities it needs, with the message shown when one is missing.
- **Settings**: theme choice and the per-engine locations and time limits.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On a mid-range laptop, the Aurora window is visible and usable within 1.5 seconds of launching, on both Windows and Linux.
- **SC-002**: A developer registers three projects in under one minute, and the list is intact after a restart in 100% of tries.
- **SC-003**: With all engines missing, hanging or broken, Aurora still opens and every view is usable; an engine that hangs is reported as degraded within its time limit (10 seconds by default) without blocking the other engines.
- **SC-004**: The Engines view reflects an engine installed or updated during the session within one refresh, with no restart.
- **SC-005**: Every view can be reached and used with the keyboard only, and text contrast meets WCAG 2.2 AA in both themes.
- **SC-006**: 100% of the automated tests pass on a machine with no engine installed, on both Windows and Linux.
- **SC-007**: Aurora itself (without the engines) takes under 30 MB once installed.

## Assumptions

- Users are developers of the Levix team using their own machines; there are no accounts, sign-in or licenses in this release (ADR-0005: licensing comes with `levix-platform`).
- None of the three engines offers the versioned machine-readable contract yet; this release is validated with sample engines, and the real ones are recognized as "installed without the contract" when present.
- Registering a project only records it; reading its development cycle state is the next release (v0.2.0), so the Project and Inbox views show their "coming in v0.2.0" state here.
- The Agents and Knowledge views are not part of this release; they arrive with Gedo and AnyContext.
- Desktop packages for distribution are part of v0.2.0; in this release Aurora is built from source and run by the team.
- The interface is in English; other languages come later.
- Engines are looked up by their command names `dcp`, `gedo` and `actx`, the Rust binaries of each product; the Dev Cycle Protocol counts as installed only through `dcp`, never through its Python scripts (ADR-0006).
