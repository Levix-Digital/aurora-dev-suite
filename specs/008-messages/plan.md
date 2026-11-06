# Implementation Plan: Messages between projects

**Branch**: `feat/008-messages` | **Date**: 2026-11-06 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/008-messages/spec.md`

## Summary

Add the integrations provider as a new crate, `crates/aurora-integrations` (library and binary `dcp-integrations`), porting the provisional tool 0.3.0 to Rust with the same commands, header, kinds, templates, rounds, cycles and closing record, plus `.aurora/integrations.json` (`init`, `add`, `validate`) as its only registry and `inbox.json` as the branch format version (ADR-0009, ADR-0010). Git runs through the `git` command on bare caches, as today. Then add to Aurora the engine `dcp-integrations` (probe, feature "Messages"), a messages domain and a `MessagesSource` port in the core, an adapter over the provider's `--json`, six desktop commands, a background poll with a `messages-news` event, and a Messages view: closing records waiting for the user first with decide and "Ask for changes", a field to step into any conversation, the conversations with their cycles and rounds, and sending and replying from templates. The release workflow ships the provider inside the three packages and as a separate download. No new external dependency.

## Technical Context

**Language/Version**: Rust stable, edition 2024, MSRV 1.95 (unchanged)

**Primary Dependencies**: existing ones only (`serde`, `serde_json`, `thiserror`, `time`, `directories`, `tempfile`, `wait-timeout`, `tracing`, `leptos` 0.8 CSR; `jsonschema` and `insta` in tests); the provider adds no crate outside the workspace's list, and parses arguments by hand

**Storage**: none in Aurora (ADR-0004); messages on `aurora/inbox` branches and `.aurora/integrations.json`, written only by the provider; bare caches under the user's cache directory

**Testing**: provider unit tests (format, templates, rounds, cycles, record) and integration tests on bare repositories in a temporary folder (the scenarios of tool 0.3.0, concurrent writers, `inbox.json` versions, tool 0.1 to 0.3 messages); core unit tests; adapter tests against a fake provider; sample replies validated against the schemas; manual tests for the view

**Target Platform**: Windows 10 22H2+ and 11, Linux x86_64 (unchanged); the provider on the same targets

**Project Type**: desktop application plus a command line engine

**Performance Goals**: a poll of six projects with nothing new under 5 seconds and no fetch (SC-004); one `check` and one `list` per project read; at most four provider processes at once

**Constraints**: Aurora runs no Git and writes no inbox or registry (SC-005); no shell; message bodies never logged and passed by temporary file; the user's time limit and the existing output limits

**Scale/Scope**: six projects, tens of conversations each

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | How this plan complies | Status |
|---|---|---|
| I. Design: one responsibility per unit, explicit errors | Provider split into format, rules, transport and commands; `MessagesError` and the provider's error document explicit | Pass |
| II. Simplicity First | Ports a proven tool; Git through the `git` command; one schema file; no new crate dependency; no cache in Aurora | Pass |
| III. Hexagonal | Messages domain and port in `aurora-core`; the provider behind `MessagesSource` in `aurora-engines`; `aurora-core` does not depend on `aurora-integrations`; `xtask arch-check` gains the new crate | Pass |
| IV. Development cycle: evidence | Every rule tested; scenarios of tool 0.3.0 as integration tests; contracts validated by samples; manual tests for the view | Pass |
| IV. Security | No shell; bodies by temporary file; no secrets; Git uses the user's own credentials; nothing executable on inbox branches (ADR-0010) | Pass |
| IV. 9. No AI attribution | Plan, code, commits, pull request and messages carry none | Pass |
| ADR-0003, ADR-0006, ADR-0007 | The provider only through its command line with versioned JSON; no stand-in; Aurora never fetches or pushes | Pass |

Re-checked after Phase 1 design: no violation.

## Project Structure

### Documentation (this feature)

```text
specs/008-messages/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── cli.md
│   ├── dcp-integrations.schema.json
│   ├── aurora-inbox.schema.json
│   ├── aurora-integrations.schema.json
│   └── desktop-commands.md
├── checklists/requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/
├── aurora-integrations/             the provider (new)
│   ├── src/lib.rs
│   ├── src/format.rs                header, parse, render, sections
│   ├── src/rules.rs                 kinds, templates, rounds, cycles, waiting, record
│   ├── src/registry.rs              .aurora/integrations.json: init, add, validate, read
│   ├── src/transport.rs             git on bare caches: ls-remote, fetch, write with retry, inbox.json
│   ├── src/commands.rs              every command, text and --json output
│   ├── src/bin/dcp-integrations.rs  argument parsing, exit codes
│   └── tests/                       bare-repository scenarios, contract samples
├── aurora-core/src/
│   ├── domain/messages.rs           messages domain (new)
│   ├── domain/engine.rs             EngineKind::Integrations
│   ├── domain/feature.rs            Feature::Messages
│   ├── domain/settings.rs           poll interval
│   ├── ports.rs                     MessagesSource
│   ├── usecases/messages.rs         overview, conversation, record, template, act, poll (new)
│   └── fixtures.rs                  sample conversations, FixtureMessages
├── aurora-engines/
│   ├── src/messages.rs              ProviderMessagesSource over dcp-integrations --json (new)
│   ├── src/probe.rs                 finds dcp-integrations next to Aurora first
│   ├── src/bin/aurora-fake-engine.rs   answers as dcp-integrations
│   └── tests/messages.rs, tests/fixtures/dcp-integrations/*.json
├── aurora-desktop/                  six commands, background poll, event, externalBin
└── aurora-ui/
    ├── src/backend/                 six calls, the event, fixture answers
    ├── src/views/messages.rs        the Messages view (new)
    ├── src/app.rs                   navigation entry with the count
    └── styles/app.css
.github/workflows/release.yml        builds and bundles the provider, uploads it alone
```

**Structure Decision**: one new crate for the provider; in Aurora, one domain module, one adapter, one view.

## Complexity Tracking

| Addition | Why needed | Simpler alternative rejected because |
|---|---|---|
| A fifth crate | The provider is an engine with its own command line (ADR-0010) | Inside `aurora-engines` it would link the desktop's adapters into a command line tool |

## Dev Cycle Protocol

### Backlog items and release

FEAT-029, FEAT-031, FEAT-032, FEAT-033 and FEAT-034 (EPIC-010), release v0.3.0 "Actions through dcp" (`docs/ROADMAP.md`), plan `docs/plans/messages.md`. Needs nothing from another project; plugging into the Dev Cycle is FEAT-036 and FEAT-037 (v0.6.0).

### Tests (quality-assurance)

- **Provider**: parse and render round trips (tool 0.1, 0.2 and 0.3 messages from the six inbox branches as samples); template checks per kind; round counting, the fourth round refused, `ack` outside rounds; a `guidance` starting cycle 2 and the rounds restarting; waiting-for-user only for the opener and cleared by `decision` or `guidance`; the record fields and numbered options; `inbox.json` accepted, absent (read as 1.0, written on next write) and refused for MAJOR 2; registry `init`, `add`, `validate`; on bare repositories: send, reply, record, decide, revise, guide, poll without fetch, a push race rebuilt on the new tip, a missing branch created, the commit identity taken from the project; every `--json` reply validated against the schema; text output equal to the provisional tool's for the same scenario.
- **Core**: the checks before running (empty sections, round limit, empty text, size); ordering of the overview (waiting, user requests in progress, others); options parsed from "Decision needed"; use cases run nothing without a usable provider or for a project not set up.
- **Adapter**: against the fake provider: each command's arguments, the body file removed afterwards, a refusal with missing sections, a Git error with its output, an unsupported version, a timeout, an invalid reply.
- **Desktop**: the poll timer emits `messages-news` only on changes and stops when the interval is off.
- **Interface**: the fixture backend answers every command and simulates decide, revise, guide, send, reply and poll.
- **Manual tests** (`tests/MANUAL_TESTS.md`, new section "Messages"): MT-MSG-001 the waiting record first with its fields, decide; MT-MSG-002 ask for changes and step into a running conversation; MT-MSG-003 follow a conversation and filters; MT-MSG-004 new conversation refused then sent; MT-MSG-005 the count on the navigation after a poll; MT-MSG-006 a project not set up and a missing provider; MT-MSG-007 the packages contain `dcp-integrations` and it answers `version --json`.
- **Acceptance criteria**: the scenarios of US1 to US6 and SC-001 to SC-006.

### Documentation (architecture-docs, user-docs)

- New subsystem page `docs/architecture/integrations.md` (the provider: format, rules, transport, commands); edit in place `core.md` (messages domain, port, use cases), `engines.md` (adapter, discovery next to Aurora), `ui.md` (Messages view), `desktop.md` (commands, poll, externalBin).
- No new ADR: ADR-0007, ADR-0009 and ADR-0010 cover the decisions; research R1 to R9 holds the rest.
- README: the Messages view, the `dcp-integrations` command line, setting up `.aurora/integrations.json`.

### CI, release and observability (devops-release)

- `ci.yml`: the new crate is in the workspace, so build, tests, clippy and `cargo deny` cover it; the integration tests need `git` on the runners (present on GitHub's images).
- `release.yml`: build `aurora-integrations` for the job's target before the bundles, copy it as Tauri's `externalBin`, upload `dcp-integrations-<os>-x86_64[.exe]` with its checksum in `SHA256SUMS`; `xtask release-check` checks the provider's version equals the workspace version.
- Version impact: minor, inside v0.3.0 (the bump comes with the v0.3.0 release candidate).
- Logs: the provider logs to standard error only with `--verbose` (command, project, Git steps and durations, never bodies); Aurora logs each provider command with project, duration and outcome.

### Security, migration and rollback

- No secrets; Git uses the user's credentials; bodies by temporary file, never in arguments or logs.
- Migration: none for Aurora's own files; the provider reads every message the provisional tool wrote and adds `inbox.json` on its next write, which the provisional tool ignores. The projects add `.aurora/integrations.json` when they switch (research R9).
- Rollback: revert the pull request; inbox branches stay readable by the provisional tool, which ignores `inbox.json`, `cycle` and millisecond dates are already its own format.

## Milestones

- **M1 Provider format and rules**: `format.rs`, `rules.rs`, `registry.rs` with unit tests and the samples.
- **M2 Provider transport and commands**: `transport.rs`, `commands.rs`, the binary, the bare-repository scenarios, `--json` validated against the schemas.
- **M3 Aurora core, adapter and desktop**: engine and feature, domain, port, use cases, fixtures, adapter and fake provider, desktop commands, poll and event.
- **M4 Messages view**: navigation count, waiting records, decide and ask for changes, conversation and its field, filters, new conversation and reply, preview fixtures.
- **M5 Packaging, docs and manual tests**: `externalBin`, `release.yml`, `release-check`, subsystem pages, README, manual tests.
