---
description: "Tasks of the Messages between projects feature (v0.3.0, FEAT-029, FEAT-031 to FEAT-034)"
---

# Tasks: Messages between projects

**Input**: Design documents from `/specs/008-messages/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/
**Tests**: required by the constitution (principle IV.2, evidence); test tasks come before the code they prove.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: can run in parallel (different files, no dependency on an unfinished task)
- **[Story]**: US1 to US6 of spec.md

---

## Phase 1: Provider format and rules (M1)

- [ ] T001 [US6] Create `crates/aurora-integrations` (library and binary `dcp-integrations`) in the workspace, `xtask arch-check` aware of it, `aurora-core` not depending on it
- [ ] T002 [P] [US6] Copy sample messages written by the provisional tool 0.1, 0.2 and 0.3 from the six inbox branches into `crates/aurora-integrations/tests/fixtures/messages/`
- [ ] T003 [US6] Write the tests of parse, render and sections (round trip of every sample) and implement `format.rs`
- [ ] T004 [US6] Write the tests of templates, rounds, cycles, waiting for the user and the record (numbered options) and implement `rules.rs`
- [ ] T005 [P] [US6] Write the tests of `init`, `add`, `validate` and reading `.aurora/integrations.json` and implement `registry.rs`

**Checkpoint**: `cargo test -p aurora-integrations` passes.

---

## Phase 2: Provider transport and commands (M2)

- [ ] T006 [US6] Write the bare-repository tests of `transport.rs` (fetch, missing branch, write with retry on a lost race, `inbox.json` absent, 1.0, MAJOR 2 refused, commit identity from the project, `ls-remote` without fetch) and implement it
- [ ] T007 [US6] Write the scenario tests of tool 0.3.0 (template refusal, round limit, ack outside rounds, closing record, decide, revise in cycle 2, guide, check with unanswered sent, poll exit codes, watch once) and implement `commands.rs`
- [ ] T008 [US6] Implement the binary: arguments, `--dir`, `--json`, `--verbose`, exit codes of `contracts/cli.md`; text output equal to the provisional tool's
- [ ] T009 [US6] Validate every `--json` reply of the scenarios against `contracts/dcp-integrations.schema.json`, and the written files against `aurora-inbox.schema.json` and `aurora-integrations.schema.json`

**Checkpoint**: `cargo test -p aurora-integrations` passes on Windows and Linux in CI.

---

## Phase 3: Aurora core, adapter and desktop (M3)

- [ ] T010 [US5] Add `EngineKind::Integrations` (`dcp-integrations`, `dcp-integrations.version`), `Feature::Messages` (capability `messages` MAJOR 1, v0.3.0) and the poll interval setting, with tests
- [ ] T011 [US1] Write the tests and implement `domain/messages.rs` (types, checks before running, overview order, options) re-exported from `domain/mod.rs`
- [ ] T012 [US1] Add the `MessagesSource` port and `usecases/messages.rs` (overview, conversation, record, template, act, poll) with tests, and `FixtureMessages` with the sample conversations of the quickstart in `fixtures.rs`
- [ ] T013 [P] [US6] Write the sample replies in `crates/aurora-engines/tests/fixtures/dcp-integrations/` and validate them against the schema
- [ ] T014 [US1] Implement `ProviderMessagesSource` in `aurora-engines/src/messages.rs`, the fake provider's answers, and the adapter tests (arguments, body file removed, refusal, Git error, unsupported version, timeout, invalid reply)
- [ ] T015 [US5] Find `dcp-integrations` next to Aurora's executable first in `probe.rs`, with tests
- [ ] T016 [US5] Add the six desktop commands, the background poll with the `messages-news` event and the capability file entries, with the timer test

**Checkpoint**: `cargo test --workspace --all-features` and `cargo xtask arch-check` pass.

---

## Phase 4: Messages view (M4)

- [ ] T017 [US1] Backend calls and fixture answers for the six commands and the event in `aurora-ui/src/backend/`
- [ ] T018 [US1] The Messages view: "Waiting for you" with the closing records as fields, decide with options and a free choice, confirmation
- [ ] T019 [US2] "Ask for changes" on a record and the field of every conversation (guide), with cycle and requests shown
- [ ] T020 [US3] Conversation list (newest first, waiting first), filters, the conversation with its messages and sections, "no answer yet", "Not set up" and "not reachable"
- [ ] T021 [US4] New conversation and reply from templates, required sections marked, refusals next to the form, kinds limited after three rounds
- [ ] T022 [US5] Navigation entry "Messages" with the count, updated by `messages-news`; the simulated poll in the preview
- [ ] T023 [P] Styles in `styles/app.css`, light and dark, contrast checked by `theme_contrast.rs`

**Checkpoint**: the quickstart passes in the preview.

---

## Phase 5: Packaging, docs and manual tests (M5)

- [ ] T024 [US6] Tauri `bundle.externalBin` for `dcp-integrations`; `release.yml` builds it, bundles it and uploads it alone with its checksum; `xtask release-check` checks its version
- [ ] T025 [P] `docs/architecture/integrations.md` (new), and in place `core.md`, `engines.md`, `ui.md`, `desktop.md`; `TECDOC.md` index
- [ ] T026 [P] README: the Messages view, the command line, setting up `.aurora/integrations.json`
- [ ] T027 Manual tests MT-MSG-001 to MT-MSG-007 in `tests/MANUAL_TESTS.md`, checked with `quality-assurance manual_tests validate`
- [ ] T028 Mark the milestones done in `docs/plans/messages.md`, the backlog items in the pull request that closes the plan

---

## Dependencies

Phase 1 → Phase 2 → Phase 3 (T013 and T015 may start with Phase 2) → Phase 4 → Phase 5 (T025 and T026 may start with Phase 4).
