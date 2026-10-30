# Implementation Plan: dcp adapter

**Branch**: `feat/002-dcp-adapter` | **Date**: 2026-10-29 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/002-dcp-adapter/spec.md`

## Summary

Add the `CycleSource` port to the core with Aurora's own cycle state types, a use case that gates the request on the `dcp` report and the project's presence, and sample cycle states in the fixtures. Add `DcpCycleSource` in `aurora-engines`, which runs `dcp diagnose --json --dir <root>` through the existing command runner and translates the `dcp.diagnose` reply (MAJOR 1) into the core types. Publish the reply's JSON Schema with the specification, teach the fake engine to answer it, expose one desktop command for the interface and the preview, and propose the contract to the Dev Cycle Protocol project through the cross-project inbox. No view changes: FEAT-014, FEAT-015 and FEAT-017 build on this.

## Technical Context

**Language/Version**: Rust stable, edition 2024, MSRV 1.95 (unchanged)

**Primary Dependencies**: existing ones (`serde`, `serde_json`, `thiserror`, `tracing`, `insta`); new dev-dependency `jsonschema` (MIT) with default features off, to validate the sample replies against the published schema in tests. If `cargo deny` finds a license outside the allowed list in its tree, the validation is dropped and Levi is asked before any exception.

**Storage**: none (the cycle state is read on demand and not stored; ADR-0004)

**Testing**: core unit tests over the fixtures; adapter integration tests with the fake engine as `dcp` in every mode; `insta` snapshots of parsed replies; schema validation of every sample reply

**Target Platform**: Windows 10 22H2+ and 11, Linux x86_64 (unchanged)

**Project Type**: desktop application (unchanged)

**Performance Goals**: no process started when the capability is missing (SC-002); a hanging `dcp` is reported within its limit plus one second (SC-003)

**Constraints**: no shell; reply capped at 1 MiB, error output at 64 KiB; Aurora never reads the project's protocol files (ADR-0006)

**Scale/Scope**: one request per registered project (up to about 50), run in parallel by the caller

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | How this plan complies | Status |
|---|---|---|
| I. Design: one responsibility per unit, explicit errors | Translation of the reply in one module (`cycle`), the gate in the use case, errors as one `CycleError` enum with a message per variant; no `unwrap` on engine output | Pass |
| II. Simplicity First | One port with its first caller (the desktop command); no cache, no polling, no derived counts (those come with the views that need them) | Pass |
| III. Hexagonal | Types and port in the core with no I/O; the adapter in `aurora-engines`; the desktop is the composition root; `xtask arch-check` unchanged and passing | Pass |
| IV. Development cycle: evidence | Every FR has an automated test; nothing visual changes, so no manual test is added | Pass |
| IV. Security | Engine output is untrusted: size limits, typed parsing, root check; no secrets; nothing leaves the machine | Pass |
| IV. 9. No AI attribution | Plan, code, commits and pull request carry none; the shared attribution-off settings file is already committed | Pass |
| ADR-0003, ADR-0006 | Only the `dcp` binary and its versioned JSON; no stand-in, no Python, no protocol files | Pass |

Re-checked after Phase 1 design: no violation, nothing in Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/002-dcp-adapter/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── dcp-diagnose.schema.json
│   └── desktop-commands.md
├── checklists/requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/
├── aurora-core/src/
│   ├── domain/cycle.rs          CycleState and its parts, CycleError (new)
│   ├── domain/mod.rs            re-exports
│   ├── ports.rs                 + CycleSource
│   ├── usecases/cycle.rs        cycle_state: presence and capability gate (new)
│   ├── capabilities.rs          + the dcp requirement lookup used by the gate
│   └── fixtures.rs              + FixtureCycleSource and sample states
├── aurora-engines/
│   ├── src/cycle.rs             DcpCycleSource and the reply parser (new)
│   ├── src/bin/aurora-fake-engine.rs   + diagnose --json --dir, diagnose capability
│   └── tests/
│       ├── cycle.rs             adapter tests over the fake engine (new)
│       └── fixtures/dcp-diagnose/*.json   sample replies, validated against the schema
├── aurora-desktop/
│   ├── src/state.rs             + the cycle source in the composition root
│   ├── src/commands.rs          + project_cycle_state
│   └── capabilities/default.json   + the command
└── aurora-ui/src/backend/       + project_cycle_state on both backends (no view uses it yet)
```

**Structure Decision**: same four crates; one new module per crate where the feature lives.

## Complexity Tracking

No violations.

## Dev Cycle Protocol

### Backlog items and release

FEAT-016 dcp adapter, release v0.2.0 "Portfolio on the dcp contract" (`docs/ROADMAP.md`). Release v0.1.0 stays unreleased for now (Levi's choice, 2026-10-29).

### Tests (quality-assurance)

- **Core**: `cycle_state` returns `Unavailable` for `dcp` missing, without contract, degraded, and ready without `diagnose` or with `diagnose` at another MAJOR, without calling the port (a counting source proves it); `ProjectMissing` for a missing folder; passes through the source's state and errors otherwise; every `CycleError` has a message.
- **Adapter** (`aurora-engines/tests/cycle.rs`, fake engine as `dcp`): a valid reply becomes the expected `CycleState` (`insta` snapshot); unknown fields ignored; empty state; reply for another root refused; path with spaces and non-ASCII characters passed intact; `hang` gives `TimedOut` within the limit plus one second; `crash` gives `Failed` with the error output; `huge` gives `TooLarge`; `invalid` gives `Invalid`; `unsupported` gives `Unsupported` with the side to update; warnings on standard error with exit 0 still succeed.
- **Contract**: every sample reply under `tests/fixtures/dcp-diagnose/` and the fake engine's reply are valid against `contracts/dcp-diagnose.schema.json`; the fixtures in the core are built from the same samples, so the preview and the tests cannot drift from the contract.
- **Desktop and interface**: the command error carries the serialized `CycleError`; the fixture backend answers `project_cycle_state` for every sample project.
- **Manual tests**: none added; nothing visible changes in this item. The views' manual tests come with FEAT-014, FEAT-015 and FEAT-017.
- **Acceptance criteria**: the acceptance scenarios of US1 to US3 and SC-001 to SC-005.

### Documentation (architecture-docs, user-docs)

- Edit in place `docs/architecture/core.md` (the `cycle` domain module, the `CycleSource` port and the use case), `engines.md` (the `cycle` module, the `dcp.diagnose` contract, the fake engine's new command), `desktop.md` (the new command) and `ui.md` (the backend method).
- The contract's source of truth is `specs/002-dcp-adapter/contracts/dcp-diagnose.schema.json`; `engines.md` links it.
- No new ADR: ADR-0003 and ADR-0006 already decide how engines are read and that features are built against the agreed contract.
- README: no change (nothing user-visible yet).

### Cross-project request

After this plan is approved, send to `dev-cycle-protocol` through the inbox tool: the schema, the command (`dcp diagnose --json --dir <root>`, exit codes), the `diagnose` capability at `schema_major` 1 in `dcp version --json` (the version reply of `specs/001-foundations/contracts/engine-version.schema.json`), the reason (base of Aurora v0.2.0) and what closes it (an item in their backlog and roadmap, or a counter-proposal Aurora adopts in a MINOR change of this plan).

### CI, release and observability (devops-release)

- No workflow change; the existing jobs run the new tests on Windows and Linux.
- Version impact: minor, inside v0.2.0 (no version bump in this pull request; the bump comes with the v0.2.0 release candidate).
- Logs: each request logs the project id, the duration and the outcome kind at `info`, failures at `warn`, never the reply's content; covered by the adapter tests through the outcome and checked in the log during the quickstart.
- No telemetry.

### Security, migration and rollback

- No secrets; the reply is untrusted (size limits, typed parsing, root check, rendered as text by the views later).
- No stored data changes: `projects.toml` and `settings.toml` are untouched.
- Rollback: revert the pull request; nothing depends on it until FEAT-014.
