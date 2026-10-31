# Implementation Plan: Project settings

**Branch**: `feat/007-project-settings` | **Date**: 2026-10-31 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/007-project-settings/spec.md`

## Summary

Add to the core the project settings domain (`domain/config.rs`: values, settings, draft, change, outcome, errors), a `ConfigSource` port, the `config` capability with the feature "Project settings" (v0.3.0), and the use cases read, preview and apply. Add a `dcp` adapter (`aurora-engines/src/config.rs`) over `dcp config get --json` and `dcp config set --json [--dry-run]`, sharing the runner and the envelope checks with the cycle adapter (`dcp.rs`), three desktop commands, the fake engine's answers, and the contracts `dcp.config` and `dcp.config-set` MAJOR 1 with validated samples. In the interface, the Project view gets the tabs "Cycle" and "Settings"; the Settings tab renders the settings by group from the reply, keeps the user's edits, previews them and applies them. The contracts go to the Dev Cycle Protocol project (DCP FEAT-017). No new dependency.

## Technical Context

**Language/Version**: Rust stable, edition 2024, MSRV 1.95 (unchanged)

**Primary Dependencies**: existing ones only (`leptos` 0.8 CSR, `serde`, `serde_json` in the adapter, `jsonschema` in tests)

**Storage**: none in Aurora (ADR-0004); `.dev-cycle.json` is written only by `dcp`

**Testing**: core unit tests (checks, draft, use cases with a counting source); adapter tests against the fake `dcp`; sample replies validated against the JSON Schemas; manual tests for the tab

**Target Platform**: Windows 10 22H2+ and 11, Linux x86_64 (unchanged)

**Project Type**: desktop application (unchanged)

**Performance Goals**: one `dcp config get` per tab open or refresh; one `dcp config set` per preview or apply; none without a usable `dcp` (SC-003)

**Constraints**: no shell, the user's time limit and the existing output limits; values never logged; engine text rendered as text only

**Scale/Scope**: one project at a time, tens of settings

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | How this plan complies | Status |
|---|---|---|
| I. Design: one responsibility per unit, explicit errors | Checks and draft in the core; adapter only runs and parses; `ConfigError` explicit with field errors | Pass |
| II. Simplicity First | Reuses the runner, the envelope checks, `CycleError` texts and the Project view's picker; no cache, no Git | Pass |
| III. Hexagonal | Rules and port in `aurora-core`; `dcp` behind `ConfigSource` in `aurora-engines`; `xtask arch-check` unchanged | Pass |
| IV. Development cycle: evidence | Every rule has automated tests; contracts validated by samples; manual tests for the tab | Pass |
| IV. Security | No shell; arguments built from the reply's keys; values not logged; nothing leaves the machine; no secrets in the contract | Pass |
| IV. 9. No AI attribution | Plan, code, commits and pull request carry none | Pass |
| ADR-0003, ADR-0006 | Settings only through `dcp` with a versioned capability; no stand-in; unavailable until `dcp` ships `config` | Pass |

Re-checked after Phase 1 design: no violation.

## Project Structure

### Documentation (this feature)

```text
specs/007-project-settings/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── dcp-config.schema.json
│   ├── dcp-config-set.schema.json
│   └── desktop-commands.md
├── checklists/requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/
├── aurora-core/src/
│   ├── domain/config.rs          settings, draft, change, outcome, errors (new)
│   ├── domain/feature.rs         Feature::ProjectSettings
│   ├── capabilities.rs           capability `config`, usable_dcp_for
│   ├── ports.rs                  ConfigSource
│   ├── usecases/config.rs        read, preview, apply (new)
│   └── fixtures.rs               sample settings, FixtureConfigSource
├── aurora-engines/
│   ├── src/dcp.rs                running dcp and checking the envelope, shared (new)
│   ├── src/cycle.rs              uses dcp.rs
│   ├── src/config.rs             DcpConfigSource (new)
│   ├── src/bin/aurora-fake-engine.rs   answers config get and config set
│   └── tests/config.rs, tests/fixtures/dcp-config/*.json, tests/contract_samples.rs
├── aurora-desktop/               three commands, permissions, state
└── aurora-ui/
    ├── src/backend/              three calls, fixture answers
    ├── src/views/project.rs      tabs
    ├── src/views/project_settings.rs   the Settings tab (new)
    └── styles/app.css
```

**Structure Decision**: same four crates; one new domain module, one adapter, one view.

## Complexity Tracking

No violations.

## Dev Cycle Protocol

### Backlog items and release

FEAT-027 Project settings (EPIC-004), release v0.3.0 "Actions through dcp" (`docs/ROADMAP.md`), the first of its two items (FEAT-018 next). Depends on DCP FEAT-017 (protocol v2.12.0) for the real `dcp`; the contracts are sent to the Dev Cycle Protocol project after gate 3.

### Tests (quality-assurance)

- **Core**: each check of `ConfigSetting` (read-only, type, allowed values for enum and list, null), `parse_input` per type, the draft (edit back to the current value drops it, order of the change, unknown keys), `ConfigError::from_field_errors`, groups in order; the use cases run nothing without a usable `dcp` or with a missing folder, refuse an empty change, and pass `dry_run` through.
- **Adapter**: against the fake `dcp`: the read, the arguments of a preview and an apply, a refusal, a stale revision, a reply about another root, an unsupported schema, a timeout.
- **Contracts**: every sample reply validates against its schema.
- **Interface**: the fixture backend answers every command of the contracts, and simulates preview, refusal and apply.
- **Manual tests** (`tests/MANUAL_TESTS.md`, new section "Project settings"): MT-SET-001 the preview shows the sample settings by group; MT-SET-002 edit, preview and apply; MT-SET-003 errors next to the field and the integer check; MT-SET-004 without `dcp` or without `config`, the tab says what to install and starts no process.
- **Acceptance criteria**: the scenarios of US1 to US3 and SC-001 to SC-004.

### Documentation (architecture-docs, user-docs)

- Edit in place `docs/architecture/core.md` (config domain, port, use cases), `engines.md` (the `dcp` config adapter and `dcp.rs`), `ui.md` (tabs, Settings tab) and `desktop.md` (commands).
- No new ADR: the decisions follow ADR-0003 and ADR-0006 (research R1 to R6).
- README: the Project view's Settings tab, with a `dcp` that has the `config` capability.

### CI, release and observability (devops-release)

- No workflow change.
- Version impact: minor, inside v0.3.0 (the bump comes with the v0.3.0 release candidate).
- Logs: each `dcp config` command is logged with the root, the number of keys, dry run or not, the duration and the outcome; never the values.

### Security, migration and rollback

- No secrets; the contract forbids secret values in the reply.
- `.dev-cycle.json` is written only by `dcp`, all or nothing, guarded by the revision.
- Rollback: revert the pull request; the Project view returns to one page. A change applied by a user is in their working tree, not committed, and can be discarded with Git.
