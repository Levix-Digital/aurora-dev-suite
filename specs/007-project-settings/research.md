# Research: Project settings

## R1. The `dcp` commands do not exist yet

**Decision**: author the `dcp.config` and `dcp.config-set` MAJOR 1 contracts here (contracts/) and send them to the Dev Cycle Protocol project through the inbox, as was done for `dcp.diagnose` (FEAT-016). Build the feature against the contracts with the fake engine and the fixtures; the desktop shows the feature as unavailable until `dcp` lists the capability `config` (ADR-0006).

**Rationale**: DCP FEAT-017 (protocol release v2.12.0) already describes `config get --json`, `config set` and `--dry-run --json` in words, but no schema; the `dcp` binary (DCP ADJ-001) and its JSON commands do not ship yet. A written contract removes guesswork on both sides and lets the protocol test against the same samples.

**Alternatives considered**: wait for the protocol (blocks v0.3.0 for an unknown time); read `.dev-cycle.json` directly (forbidden by ADR-0006 and by the backlog item).

## R2. How values are passed to `dcp config set`

**Decision**: one `--value <key>=<JSON value>` argument per key, plus `--revision`, `--dir`, `--json` and `--dry-run`. Aurora builds the arguments without a shell.

**Rationale**: JSON keeps types (`false`, `400`, `null`, lists) without per-type flags; one argument per key keeps the command readable in logs and in a terminal; no temporary file and no standard input to manage.

**Alternatives considered**: a JSON document on standard input (the runner closes standard input today and would need a new mode); one `set` per key (not all-or-nothing, several writes).

## R3. Stale writes

**Decision**: `config get` returns an opaque `revision`; `config set` takes `--revision` and refuses with `stale_revision` when the file changed. Apply sends the revision of the preview.

**Rationale**: the agent or the user can edit `.dev-cycle.json` while the tab is open; without a revision Aurora could overwrite a change it never showed.

## R4. Validation in two places

**Decision**: the core checks what the reply already states (read-only, type, allowed values, nullable) before running anything; `dcp` stays the authority and its errors are shown as returned.

**Rationale**: instant feedback next to the field and no process for an obviously invalid value, without duplicating the engine's rules (cross-key rules stay in `dcp`).

## R5. Where the tab lives

**Decision**: tabs "Cycle" and "Settings" in the Project view, the tab kept in `AppContext`.

**Rationale**: settings are per project, and the Project view already chooses the project; a sixth main view would repeat the project picker.

## R6. Errors shared with the cycle state

**Decision**: the engine-call errors (timeout, size, failure, invalid, unsupported) are the same as `CycleError`; `ConfigError` converts from it, and the adapter shares the runner and envelope checks with `DcpCycleSource` in `aurora-engines/src/dcp.rs`.

**Rationale**: one place for running `dcp` and checking `schema` and `schema_version`; the interface reuses the failure text.
