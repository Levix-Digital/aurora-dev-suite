# Implementation Plan: Inbox

**Branch**: `feat/005-inbox` | **Date**: 2026-10-30 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/005-inbox/spec.md`

## Summary

Add to the core the urgency of each waiting item and its detail (`domain/portfolio.rs`), and the Inbox: every project's waiting items in one list ordered by urgency, then registry order, then item order (`domain/inbox.rs`). In the interface, replace the "Coming in v0.2.0" placeholder with the Inbox: a header with the totals and Refresh, one group per urgency with each item's project, kind, title, detail and "Open", and a "Not read" list with the shared failure text. The Portfolio's parallel reads (row state, read and run over the loader) move to a shared module used by both views. No new command, no contract change, no new dependency.

## Technical Context

**Language/Version**: Rust stable, edition 2024, MSRV 1.95 (unchanged)

**Primary Dependencies**: existing ones only (`leptos` 0.8 CSR in the interface, `serde`)

**Storage**: none (ADR-0004)

**Testing**: core unit tests over the sample cycle states and hand-built states; interface unit tests of the shared reads where they have no browser dependency; manual tests for the view

**Target Platform**: Windows 10 22H2+ and 11, Linux x86_64 (unchanged)

**Project Type**: desktop application (unchanged)

**Performance Goals**: one `dcp` read per project per open or refresh, at most four at once; none without a usable `dcp` (SC-003)

**Constraints**: engine text rendered as text only; no new dependency

**Scale/Scope**: every registered project (tens)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | How this plan complies | Status |
|---|---|---|
| I. Design: one responsibility per unit, explicit errors | Urgency and order in the core with no I/O; the view renders groups; failures stay `CycleError` | Pass |
| II. Simplicity First | Reuses `project_cycle_state`, the loader, `WaitingItem` and `CycleFailure`; no shared cache between views | Pass |
| III. Hexagonal | Rules in `aurora-core`; the interface calls them; `xtask arch-check` unchanged | Pass |
| IV. Development cycle: evidence | Every rule has automated tests; the visible behavior gets manual tests | Pass |
| IV. Security | Engine text is rendered as text; nothing leaves the machine | Pass |
| IV. 9. No AI attribution | Plan, code, commits and pull request carry none | Pass |
| ADR-0003, ADR-0006 | Data only through `dcp` via FEAT-016's adapter | Pass |

Re-checked after Phase 1 design: no violation.

## Project Structure

### Documentation (this feature)

```text
specs/005-inbox/
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
│   ├── domain/inbox.rs          Urgency, InboxEntry, Inbox (new)
│   ├── domain/portfolio.rs      WaitingItem gains its urgency and detail
│   └── domain/mod.rs            re-exports
└── aurora-ui/
    ├── src/views/reads.rs       the Portfolio's parallel reads, shared (new)
    ├── src/views/portfolio.rs   uses views::reads
    ├── src/views/inbox.rs       the Inbox (replaces the placeholder)
    └── styles/app.css           styles of the groups
```

**Structure Decision**: same four crates; one new module in the core and one in the interface.

## Complexity Tracking

No violations.

## Dev Cycle Protocol

### Backlog items and release

FEAT-017 Inbox view (EPIC-005 Decision inbox), release v0.2.0 "Portfolio on the dcp contract" (`docs/ROADMAP.md`). Next in the roadmap's order after FEAT-015; it reuses FEAT-014's waiting rules and FEAT-015's candidate stage. FEAT-018 (Inbox actions) stays in v0.3.0.

### Tests (quality-assurance)

- **Core**: the urgency and detail of each kind of waiting item (blocking bug, candidate in each stage, pull request with and without threads for the user, untriaged bug), Portfolio titles unchanged; the Inbox order across projects (urgency first, then registry order, then item order), the groups in urgency order without empty groups, the project count; the samples in the quickstart's order.
- **Interface**: the shared row state keeps the last answer while refreshing (moved with its code).
- **Manual tests** (`tests/MANUAL_TESTS.md`, new section "Inbox"): MT-INB-001 the preview lists the sample items by urgency with project, kind, title and detail; MT-INB-002 "Open" on an item opens its project; MT-INB-003 Refresh and "Not read"; MT-INB-004 without `dcp` the Inbox says what to install once and starts no process.
- **Acceptance criteria**: the scenarios of US1 to US3 and SC-001 to SC-004.

### Documentation (architecture-docs, user-docs)

- Edit in place `docs/architecture/core.md` (urgency and the `inbox` domain module) and `ui.md` (the Inbox, the shared reads).
- No new ADR: the decisions are local to the view (research R1 to R4).
- README: the Inbox lists what waits for the user across projects when `dcp` with the contract is installed.

### CI, release and observability (devops-release)

- No workflow change.
- Version impact: minor, inside v0.2.0 (the bump comes with the v0.2.0 release candidate).
- Logs: each read is already logged by the adapter (FEAT-016); the interface logs nothing new.

### Security, migration and rollback

- No secrets, no stored data change.
- Rollback: revert the pull request; the Inbox returns to its placeholder.
