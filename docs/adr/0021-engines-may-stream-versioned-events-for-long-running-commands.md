---
id: 21
title: Engines may stream versioned events for long-running commands
status: accepted
date: '2026-11-14'
version: null
subsystems: [engines]
supersedes: []
superseded_by: []
amends: [3]
amended_by: []
---

# ADR-0021: Engines may stream versioned events for long-running commands

## Context

ADR-0003 has Aurora run each engine command with `--json`, read one JSON object and stop the command at a time limit. The Dev Cycle Protocol now conducts the cycle itself: its engine starts the coding agent for each role and advances the phases (DCP ADR-0008 to ADR-0011, recorded on 2026-11-14 after the alignment with Aurora, conversation 20261114-160911). A running cycle produces events for minutes or hours, and its clients (the terminal, the protocol's own chat screen and Aurora) only show them and send commands; they never start the agents. DCP FEAT-037 adds that interface as a `dcp` subcommand.

## Decision

- **A streamed command** is allowed besides the single-answer commands of ADR-0003: still the engine's own binary, run without a shell from the project's directory, but it writes JSON Lines on standard output, each line one event object with `schema` and `schema_version` (for the Dev Cycle Protocol, `dcp.events`), and reads commands as JSON Lines on standard input.
- **No time limit on the whole command;** instead the engine sends a heartbeat event at a fixed interval and Aurora treats a missing heartbeat as a failure. Aurora stops a stream with the engine's cancel command, then by ending the process.
- **Capabilities** stay the rule: Aurora uses a stream only when `dcp version --json` lists its capability, and an unknown event type or field is ignored.
- **Aurora never starts coding agents** and keeps no logic of the cycle: it shows events and sends the user's commands (answers, approvals). Aurora's server runs the stream and relays it to browsers through its own push channel (ADR-0014).
- **This amends ADR-0003** only for streamed commands; single-answer commands keep their time limit.

## Alternatives Considered

- **A socket or daemon exposed by the engine:** more to secure and to keep alive on Aurora's server, with no gain while one stream per running cycle is enough.
- **Polling single-answer commands:** slow for a live cycle, and every poll re-reads the project.

## Consequences

- The engines adapter gains a stream runner beside the single-answer runner, with the heartbeat and cancel handling.
- The `dcp.events` draft (DCP FEAT-037) and the protocol's Leptos chat component (DCP FEAT-039) come to Aurora for review before they are frozen.
- EPIC-013 runs `dcp` in conducted mode on a registered runner (FEAT-065); Aurora's metrics and readiness release (v0.5.0) waits for dcp v3.3.0.
