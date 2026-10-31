# Feature Specification: Project settings

**Feature Branch**: `feat/007-project-settings`

**Created**: 2026-10-31

**Status**: Draft

**Input**: User description: "A Settings tab per project rendered from dcp config get --json (schema dcp.config: value, allowed values, default, type, read-only, group); changes previewed with dcp config set --dry-run --json and applied with dcp config set, errors shown next to the field. Aurora never reads or writes .dev-cycle.json and does not commit: the user commits the change. Backlog item FEAT-027, release v0.3.0 Actions through dcp."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See a project's Dev Cycle settings (Priority: P1)

A Levix developer opens a project in the Project view, switches to its Settings tab and sees every Dev Cycle Protocol setting of that project, grouped as the protocol groups them, each with its current value, its default, a one-line description, and whether it can be changed.

**Why this priority**: today the settings live in `.dev-cycle.json`, which the user reads by hand; seeing them is the base of changing them.

**Independent Test**: In the preview, open the Project view on `gedo`, open "Settings" and check the groups and values of the quickstart.

**Acceptance Scenarios**:

1. **Given** a project and a `dcp` with the `config` capability, **When** the Settings tab opens, **Then** Aurora runs `dcp config get --json --dir <root>` once and shows the settings grouped in the order of the reply.
2. **Given** a setting, **When** it is shown, **Then** it shows its key, description, current value (or "Not set" for an unset value), its default when it differs from the value, and a control that matches its type: a choice list for an enum, a switch for a boolean, a text field for text, a number field for an integer, and a list of values for a list (checkboxes when the list has allowed values).
3. **Given** a read-only setting, **When** it is shown, **Then** its control is disabled and the reason the protocol gives is shown next to it.
4. **Given** a reply with findings, **When** the tab is shown, **Then** the findings are listed above the groups.

---

### User Story 2 - Change settings with a preview before applying (Priority: P1)

The developer changes one or more settings, asks for a preview and sees, before anything is written, each key's value before and after and the change to the file; then they apply it, or go back and edit.

**Why this priority**: it is the point of the feature; the preview keeps a wrong change from reaching the repository unseen.

**Independent Test**: In the preview, change "Branching model" to `integration` and "Release candidates" off, press "Preview changes", check the two lines and the diff, press "Apply", and check the confirmation and the reminder to commit.

**Acceptance Scenarios**:

1. **Given** edited fields, **When** the tab is shown, **Then** each edited field is marked, the number of pending changes is shown, and "Preview changes" and "Discard" are enabled; with no edits they are disabled.
2. **Given** pending changes, **When** the user presses "Preview changes", **Then** Aurora runs `dcp config set --json --dry-run` with only the edited keys and the revision it read, and shows each key's before and after and the file diff returned by `dcp`; nothing is written.
3. **Given** a preview, **When** the user presses "Apply", **Then** Aurora runs `dcp config set --json` with the same keys, values and revision, and on success shows "Saved to .dev-cycle.json. Commit it to share the change.", then reads the settings again.
4. **Given** a preview, **When** the user presses "Back to editing", **Then** the edits stay and nothing is written.
5. **Given** a value `dcp` refuses, **When** the preview or the apply answers with errors, **Then** each error is shown next to its field with the accepted values, errors without a key are shown above the groups, and nothing is written.
6. **Given** the file changed after the settings were read, **When** the preview or the apply answers that the revision is stale, **Then** Aurora says the settings changed on disk, keeps the edits, and offers to read them again.

---

### User Story 3 - Know why settings are not available (Priority: P2)

When `dcp` is missing, has no `config` capability, or the project's folder is gone, the tab says why and what to do, and runs nothing.

**Why this priority**: the feature depends on a `dcp` capability that ships separately (DCP FEAT-017); the tab must explain the gap instead of failing.

**Independent Test**: In the desktop application without `dcp`, open the Settings tab and check the availability line and that no process starts.

**Acceptance Scenarios**:

1. **Given** `dcp` is not usable for settings, **When** the tab opens, **Then** the availability line says what to install or update and no command runs.
2. **Given** a project whose folder is missing, **When** the tab opens, **Then** it says to locate the folder and no command runs.
3. **Given** `dcp` fails, times out or answers an unsupported or invalid reply, **When** the tab shows the failure, **Then** it uses the same failure text as the Project view, with the engine's error output to open.

---

### Edge Cases

- A setting the reply marks as secret is never in the reply (the contract forbids it); Aurora shows only what `dcp` returns.
- A value edited back to its original value is not a pending change.
- Setting a nullable value back to "Not set" sends `null`.
- A list with allowed values only offers those values; a free list accepts one value per line, with empty lines dropped.
- An integer field with text that is not a whole number is refused next to the field before any command runs.
- Switching project or tab with pending edits drops them after a confirmation.
- Two quick presses on "Apply": only one command runs; the button is disabled while it runs.
- `dcp` answers a preview with no changes (every value equal to the file): Aurora says there is nothing to apply.
- Aurora never runs Git: the user commits `.dev-cycle.json` and opens the pull request.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The Project view MUST have two tabs, "Cycle" (today's sections) and "Settings"; the chosen tab stays while Aurora runs.
- **FR-002**: The Settings tab MUST read the project's settings only through `dcp config get --json --dir <root>` (schema `dcp.config` MAJOR 1, `contracts/dcp-config.schema.json`), and only when `dcp` reports the capability `config` with schema MAJOR 1 (ADR-0003, ADR-0006). Aurora MUST NOT read or write `.dev-cycle.json` itself.
- **FR-003**: Each setting MUST be rendered from the reply alone: key, group, description, type, value, default, allowed values, nullable, read-only and its reason.
- **FR-004**: A change MUST be previewed before it is applied: the preview runs `dcp config set --json --dry-run --dir <root> --revision <revision>` with one `--value <key>=<JSON value>` per edited key (schema `dcp.config-set` MAJOR 1, `contracts/dcp-config-set.schema.json`).
- **FR-005**: Apply MUST send exactly the keys, values and revision of the preview the user saw, without `--dry-run`; any edit after the preview requires a new preview.
- **FR-006**: The core MUST refuse, before running anything, a change to a read-only or unknown key, an enum or list value outside its allowed values, an integer that is not a whole number, and `null` for a setting that is not nullable; `dcp` stays the authority and its errors are shown as returned.
- **FR-007**: Errors returned by `dcp` MUST be shown next to their field with the accepted values; a stale revision MUST be shown as "changed on disk" with an action to read again.
- **FR-008**: The adapter MUST run `dcp` without a shell, with the user's time limit and the existing output size limits, and log each command's key count, duration and outcome without values.
- **FR-009**: Aurora MUST NOT run Git or commit; after an apply it MUST remind the user to commit `.dev-cycle.json`.
- **FR-010**: The browser preview MUST show the tab with sample settings and simulate previews, refusals and apply in memory.
- **FR-011**: A new feature "Project settings" MUST appear in the Engines view, needing `dcp` capability `config`, release v0.3.0.

### Key Entities

- **Project settings**: the reply of `dcp config get`: the revision, the source file, the settings and findings.
- **Setting**: one key with its group, description, type, value, default, allowed values, nullable and read-only flags.
- **Settings change**: the keys and new values the user edited.
- **Settings preview**: the before and after of each key and the file diff, applied or not.
- **Field error**: a key (or none), a message, the accepted values.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In the preview, the Settings tab of `gedo` shows the sample groups and a change of two keys previews and applies with the quickstart's text.
- **SC-002**: No code path of Aurora opens `.dev-cycle.json`; every read and write goes through `dcp` (checked by review and by the adapter tests with the fake engine).
- **SC-003**: With `dcp` missing or without `config`, the tab starts no process.
- **SC-004**: 100% of the automated tests pass on Windows and Linux with no `dcp` installed.

## Assumptions

- `dcp config get` and `dcp config set` do not exist yet: they are DCP FEAT-017 (release v2.12.0 of the protocol). Aurora authors the `dcp.config` and `dcp.config-set` MAJOR 1 contracts in this feature and sends them to the Dev Cycle Protocol project, as it did for `dcp.diagnose`. Until `dcp` ships them, the tab shows the unavailable state in the desktop and works in the preview (ADR-0006).
- The `config` capability covers both commands and both schemas.
- Settings hold no secrets (DCP FEAT-017); if one ever does, `dcp` leaves its value out.
- The revision is an opaque string from `dcp` (for example a hash of the file) that changes whenever the file changes.
