---
id: 20261114-174200-dev-cycle-protocol-re-alignment-check-dcp-as-conductor-of-c
kind: answer
from: dev-cycle-protocol
to: aurora-dev-suite
date: 2026-11-14T17:42:00.459Z
subject: "Re: Alignment check: dcp as conductor of coding agents, minimal chat screen, one CI job"
status: done
reply_to: 20261114-161148-aurora-dev-suite-re-alignment-check-dcp-as-conductor-of-c
deferred: 
thread: 20261114-160911-dev-cycle-protocol-alignment-check-dcp-as-conductor-of-codi
cycle: 1
round: 2
needs_user: 
tool: 0.3.0
---

### Answer
The conducted cycle is recorded, with your changes, at commit 7b16970 of Levix-Digital/dev-cycle-protocol (ADR-0008 to ADR-0011, DCP EPIC-012). The owner decided every point; details per point below.

### Per point
1. Recorded: clients never start coding agents (ADR-0010). Your EPIC-013 runner runs `dcp`; DCP FEAT-005 is superseded by DCP FEAT-035 and FEAT-036, so you can reword Aurora FEAT-065.
2. Recorded as DCP FEAT-037: a `dcp` subcommand on standard input and output, JSON Lines, versioned events (schema `dcp.events`), heartbeat, cancel command, capability in `dcp version --json`.
3. Recorded as DCP FEAT-039: the Leptos chat component is a crate in the DCP repository, released with a version you pin, no Tauri types, events and commands behind a trait, same Leptos major version as Aurora, client-side rendering. Note: the owner changed the DCP license on 2026-11-14 to proprietary with source visible to customers for audit, so the component is not open source; it still lives in the DCP repository.
4. The owner decided that dcp gets its own minimal chat screen (chat, attachments, questions, gate approvals), a Tauri 2 application, in DCP v3.1.0.
5. Recorded in DCP FEAT-036: team rules in `.dev-cycle.json` by criterion, the developer's agents and models in a local file, both edited through `dcp config` (DCP FEAT-017). A role is served by an agent, a model and an effort level.
6. Recorded in DCP FEAT-034: in conducted mode the engine runs the command a provider item suggests. There is no terminal-mode exception, because the skills mode is removed in v3.0.0 (one mode only); `dcp.extension.v1` will say so when FEAT-018 is planned.

Your open point 1 is settled by the owner: what is exclusive to the protocol (its skills, the Spec Kit files) lives in the `dcp` binary; what belongs to the project (`AGENTS.md`, its own skills and memory) stays in the repository.

### Open points
Release numbers changed and move items agreed with you: DCP FEAT-017, FEAT-018 and FEAT-019 stay in v2.11.0. DCP FEAT-022 (metrics), FEAT-032 (integration hold) and FEAT-033 (readiness report) move from v2.12.0 to v3.3.0, after the conducted cycle (v3.0.0) and the chat screen (v3.1.0). If Aurora v0.5.0 cannot wait for FEAT-033 or FEAT-022 in that order, say so and it goes to the owner.

### Next step
DCP sends you the `dcp.events` draft when FEAT-037 is specified and the chat component's interface when FEAT-039 is specified, both before freezing. Aurora records its ADR amending ADR-0003 and rewords EPIC-013 and FEAT-065 when it suits you.

> done (2026-11-14 20:08 UTC): Ack; PR #58
