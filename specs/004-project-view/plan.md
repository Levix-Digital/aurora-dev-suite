# Implementation Plan: Project view

**Branch**: `feat/004-project-view` | **Date**: 2026-10-30 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/004-project-view/spec.md`

## Summary

Add to the core the rules the Project view needs on top of FEAT-014's: the release candidate stage (shared with the Portfolio's waiting title), the gates that wait for the user, the next step with the recommended option first, and plan progress (`domain/project_view.rs`). In the interface, replace the "Coming in v0.2.0" placeholder with the Project view: a project choice, Refresh, and one card per section of the cycle state, read through the existing `project_cycle_state` command with the latest answer winning. The Portfolio rows get "Open", and the chosen project lives in the shared `AppContext`. The failure text of a row moves to a shared component used by both views. No new command, no contract change, no new dependency.

## Technical Context

**Language/Version**: Rust stable, edition 2024, MSRV 1.95 (unchanged)

**Primary Dependencies**: existing ones only (`leptos` 0.8 CSR in the interface, `serde`)

**Storage**: none (ADR-0004)

**Testing**: core unit tests over the sample cycle states and hand-built states; manual tests for the view

**Target Platform**: Windows 10 22H2+ and 11, Linux x86_64 (unchanged)

**Project Type**: desktop application (unchanged)

**Performance Goals**: one `dcp` read per open or refresh; none without a usable `dcp` (SC-003)

**Constraints**: engine text rendered as text only; no new dependency

**Scale/Scope**: one project at a time

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | How this plan complies | Status |
|---|---|---|
| I. Design: one responsibility per unit, explicit errors | Rules in one core module with no I/O; the view renders sections; failures stay `CycleError` | Pass |
| II. Simplicity First | Reuses `project_cycle_state` and FEAT-014's rules; a request counter instead of the parallel loader; no router | Pass |
| III. Hexagonal | Rules in `aurora-core`; the interface calls them; `xtask arch-check` unchanged | Pass |
| IV. Development cycle: evidence | Every rule has automated tests; the visible behavior gets manual tests | Pass |
| IV. Security | Engine text is rendered as text; nothing leaves the machine | Pass |
| IV. 9. No AI attribution | Plan, code, commits and pull request carry none | Pass |
| ADR-0003, ADR-0006 | Data only through `dcp` via FEAT-016's adapter | Pass |

Re-checked after Phase 1 design: no violation.

## Project Structure

### Documentation (this feature)

```text
specs/004-project-view/
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
│   ├── domain/project_view.rs   CandidateStage, GateKind, Gate, NextStep::ordered, PlanProgress progress (new)
│   ├── domain/portfolio.rs      the candidate's waiting title follows CandidateStage
│   └── domain/mod.rs            re-exports
└── aurora-ui/
    ├── src/app.rs               AppContext.project: the chosen project
    ├── src/views/mod.rs         CycleFailure: the shared failure text and engine output
    ├── src/views/portfolio.rs   "Open" on each row; failures through CycleFailure
    ├── src/views/project.rs     the Project view (replaces the placeholder)
    └── styles/app.css           styles of the sections
```

**Structure Decision**: same four crates; one new module in the core.

## Complexity Tracking

No violations.

## Dev Cycle Protocol

### Backlog items and release

FEAT-015 Project view, release v0.2.0 "Portfolio on the dcp contract" (`docs/ROADMAP.md`). Next in the roadmap's order after FEAT-014, high priority, and it reuses FEAT-014's rules.

### Tests (quality-assurance)

- **Core**: the stage of a candidate in each condition, and the Portfolio's waiting titles unchanged; gates of each sample (busy: Decision; idle: none; candidate: Decision and Release candidate acceptance) and of hand-built states (release gate, released candidate, ask without a recommended option); next step order with the recommended option first, and with a recommended key that is not among the options; plan progress fraction and label, including zero milestones and more done than total.
- **Manual tests** (`tests/MANUAL_TESTS.md`, new section "Project view"): MT-PVW-001 the preview shows every section of the busy sample from "Open" on `gedo`; MT-PVW-002 gates and next step of the candidate and idle samples; MT-PVW-003 switching, the missing project and Refresh; MT-PVW-004 without `dcp` the view says what to install once and starts no process.
- **Acceptance criteria**: the scenarios of US1 to US3 and SC-001 to SC-004.

### Documentation (architecture-docs, user-docs)

- Edit in place `docs/architecture/core.md` (the `project_view` domain module) and `ui.md` (the Project view, the chosen project, the shared failure text).
- No new ADR: the decisions are local to the view (research R1 to R5).
- README: the Project view shows one project's cycle when `dcp` with the contract is installed.

### CI, release and observability (devops-release)

- No workflow change.
- Version impact: minor, inside v0.2.0 (the bump comes with the v0.2.0 release candidate).
- Logs: each read is already logged by the adapter (FEAT-016); the interface logs nothing new.

### Security, migration and rollback

- No secrets, no stored data change.
- Rollback: revert the pull request; the Project view returns to its placeholder.
