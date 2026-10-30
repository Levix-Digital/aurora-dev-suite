# Implementation Plan: Portfolio view

**Branch**: `feat/003-portfolio-view` | **Date**: 2026-10-30 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/003-portfolio-view/spec.md`

## Summary

Add to the core the rules that turn a `CycleState` into what the Portfolio shows: the phase, the items waiting for the user and a summary (`domain/portfolio.rs`), shared later by the Project view and the Inbox. In the interface, add a loader that reads every project's cycle state through the existing `project_cycle_state` command, at most four at a time, and applies each answer as it arrives; the Portfolio rows show reading, the summary or the failure reason, with a Refresh action and a header total. In the desktop, make concurrent reads share a single first engine probe. No new command and no contract change.

## Technical Context

**Language/Version**: Rust stable, edition 2024, MSRV 1.95 (unchanged)

**Primary Dependencies**: existing ones only (`leptos` 0.8 CSR in the interface, `serde`, `thiserror`)

**Storage**: none (cycle states are read on demand; ADR-0004)

**Testing**: core unit tests over the sample cycle states; loader unit tests in the interface crate (scheduling limit, order of answers, stale answers dropped); desktop unit test of the single probe; manual tests for the view

**Target Platform**: Windows 10 22H2+ and 11, Linux x86_64 (unchanged)

**Project Type**: desktop application (unchanged)

**Performance Goals**: rows update as their answers arrive; at most four reads at once (SC-003); no process without a usable `dcp` (SC-004)

**Constraints**: the interface renders engine text as text only; no new dependency

**Scale/Scope**: up to about 50 registered projects

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | How this plan complies | Status |
|---|---|---|
| I. Design: one responsibility per unit, explicit errors | Rules in one core module with no I/O; the loader only schedules and applies answers; the row renders one state; failures stay `CycleError` | Pass |
| II. Simplicity First | Reuses `project_cycle_state`; no new command, event, cache or dependency | Pass |
| III. Hexagonal | Rules in `aurora-core`; the interface calls them; `xtask arch-check` unchanged | Pass |
| IV. Development cycle: evidence | Every rule and the loader have automated tests; the visible behavior gets manual tests | Pass |
| IV. Security | Engine text is rendered as text; nothing leaves the machine | Pass |
| IV. 9. No AI attribution | Plan, code, commits and pull request carry none | Pass |
| ADR-0003, ADR-0006 | Data only through `dcp` via FEAT-016's adapter | Pass |

Re-checked after Phase 1 design: no violation.

## Project Structure

### Documentation (this feature)

```text
specs/003-portfolio-view/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── checklists/requirements.md
└── tasks.md
```

No `contracts/`: no desktop command or engine contract changes.

### Source Code (repository root)

```text
crates/
├── aurora-core/src/
│   ├── domain/portfolio.rs      CyclePhase, WaitingKind, WaitingItem, CycleSummary (new)
│   └── domain/mod.rs            re-exports
├── aurora-desktop/src/
│   ├── commands.rs              engines_overview: single first probe
│   └── state.rs                 + the probe lock
└── aurora-ui/
    ├── src/views/loader.rs      parallel loader with a limit and generations (new)
    ├── src/views/portfolio.rs   cycle summary per row, total, Refresh
    ├── src/backend/mod.rs       project_cycle_state no longer dead code
    └── styles/app.css           styles of the summary
```

**Structure Decision**: same four crates; one new module in the core and one in the interface.

## Complexity Tracking

No violations.

## Dev Cycle Protocol

### Backlog items and release

FEAT-014 Portfolio view, release v0.2.0 "Portfolio on the dcp contract" (`docs/ROADMAP.md`). Chosen first among the open items of v0.2.0: high priority, first in the roadmap's order, and its core rules (phase, waiting items) are reused by FEAT-015 and FEAT-017.

### Tests (quality-assurance)

- **Core**: the phase of each sample (busy: Implementation, idle: Idle, candidate: Release candidate) and of hand-built states for every precedence step; waiting items of each sample (busy: 3, idle: 0, candidate: 1) and the three titles of a candidate; a released candidate does not wait; a pull request without `needs_action` does not wait; findings flag.
- **Interface loader**: never more than the limit in flight; every project answered once; answers applied as they arrive in any order; an answer from an older generation or for a project no longer listed is dropped.
- **Desktop**: two concurrent first calls probe once.
- **Manual tests** (`tests/MANUAL_TESTS.md`, new section "Portfolio"): MT-POR-001 the preview shows each sample's phase, release and count; MT-POR-002 without `dcp` the view says what to install once and rows say unavailable; MT-POR-003 with the fake engine as `dcp`, every present project shows the busy sample; MT-POR-004 a hanging `dcp` only delays its own rows and Refresh keeps the last state.
- **Acceptance criteria**: the scenarios of US1 to US3 and SC-001 to SC-005.

### Documentation (architecture-docs, user-docs)

- Edit in place `docs/architecture/core.md` (the `portfolio` domain module), `ui.md` (the loader and the Portfolio rows) and `desktop.md` (the single probe).
- No new ADR: the decisions are local to the view (research R1 to R5).
- README: the Portfolio now shows each project's cycle summary when `dcp` with the contract is installed.

### CI, release and observability (devops-release)

- No workflow change.
- Version impact: minor, inside v0.2.0 (the bump comes with the v0.2.0 release candidate).
- Logs: each read is already logged by the adapter (FEAT-016); the interface logs nothing new.

### Security, migration and rollback

- No secrets, no stored data change.
- Rollback: revert the pull request; the Portfolio returns to the v0.1.0 list.
