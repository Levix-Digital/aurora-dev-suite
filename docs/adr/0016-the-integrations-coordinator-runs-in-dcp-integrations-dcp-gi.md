---
id: 16
title: The integrations coordinator runs in dcp-integrations; dcp gives the phase points and enforces the hold
status: accepted
date: '2026-11-10'
version: null
subsystems: [integrations]
supersedes: []
superseded_by: []
amends: [15]
amended_by: []
---

# ADR-0016: The integrations coordinator runs in dcp-integrations; dcp gives the phase points and enforces the hold

## Context

ADR-0015 gave the Dev Cycle Protocol the phase check, the coordinator, the round table, the hold, `verify` and the mail fallback. Aurora sent that split to dev-cycle-protocol on 2026-11-10 (message 20261110-222720). The Dev Cycle Protocol countered on the split only: its engine stays free of agent orchestration and network calls, and the judgment, the conversation and the verification belong to the integrations provider that already owns the inbox, the messages and the templates. The projects agreed and Levi approved the split on 2026-11-10.

## Decision

- **dcp gives the moment.** DCP FEAT-018 (dcp v2.11.0) adds the cycle points `phase-enter` and `phase-exit` at every phase of the full and short paths, with the phase, the active plan id and the branch in the `dcp.extension-context` document.
- **dcp enforces the hold.** DCP FEAT-032 (dcp v2.12.0) lets the provider declare its gate before any phase exit; while the hold is open the status line shows it, `dcp phase exit` refuses to advance and `dcp next` names the provider's resume command. DCP FEAT-032 also adds the `dcp integrations verify` step to the protocol's CI templates when dcp-integrations is configured, and the diagnosis warns about a long-lived branch without it. dcp never records approvals; it asks the provider for the gate status.
- **dcp-integrations judges, talks and verifies.** The phase-check judgment (projects involved and informed, given in the provider's items), the coordinator with its chair and seats (`dcp integrations table <id>`), the templates and storage, the hold record, its watch and resumption, `verify` and `verify --branches`, and the mail fallback (`dcp integrations mail setup`) are dcp-integrations work. Every command runs as `dcp integrations ...` through the FEAT-018 external subcommands, and the protocol tells whoever runs the cycle to run the command a blocking provider item suggests, so the table starts in the developer's session with the developer's agent.
- **Unchanged from ADR-0015:** the dynamics, the templates, approval by every involved project through the Admin Hub, the required check on every long-lived branch, and standalone operation (with no provider registered, dcp behaves as today; with dcp-integrations and no `.aurora/approvers.json`, the developer's acceptance closes the hold).

## Alternatives Considered

- **Everything in dcp** (ADR-0015 as first written): the cycle engine would orchestrate agents and send mail, and would carry integration logic for teams that never use it.

## Consequences

- Detecting integrations needs dcp-integrations installed; it is released standalone, so a team without Aurora can still use it.
- Aurora's dcp-integrations backlog grows: the phase-check judgment, the table runner and the mail setup (EPIC-010).
- The dependency on DCP is narrower: FEAT-018 phase points (v2.11.0) and FEAT-032 hold gate (v2.12.0).
