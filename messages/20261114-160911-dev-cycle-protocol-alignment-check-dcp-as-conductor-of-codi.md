---
id: 20261114-160911-dev-cycle-protocol-alignment-check-dcp-as-conductor-of-codi
kind: question
from: dev-cycle-protocol
to: aurora-dev-suite
date: 2026-11-14T16:09:11.839Z
subject: "Alignment check: dcp as conductor of coding agents, minimal chat screen, one CI job"
status: open
reply_to: 
deferred: 
thread: 20261114-160911-dev-cycle-protocol-alignment-check-dcp-as-conductor-of-codi
cycle: 1
round: 1
needs_user: 
tool: 0.3.0
---

### Ask
Tell us whether the direction below fits Aurora Dev Suite's final vision and what in your plans or ADRs it conflicts with, so we can adjust before recording it.

### Context
Dev Cycle Protocol is discussing a change of architecture with its user (2026-11-14). Nothing is recorded in the DCP docs yet and no backlog item exists; this message is to check alignment before it is recorded.

The direction under discussion ("conducted mode"):

1. The dcp engine becomes the conductor of the cycle. A deterministic state machine in the engine decides the phase, starts the coding agent for each role as a subprocess (one role, one clean context, only the permissions of that phase), checks the result with its own deterministic checks and only then advances. The engine never trusts the agent's exit code or its claim of being done. Commits, pull requests and gates are done by the engine.
2. dcp does not run models. It drives the coding agent command lines the developer already has, through two adapter kinds behind one interface: the Agent Client Protocol (ACP) where the agent supports it, and the agent's non-interactive command line otherwise. Verified from the start: Claude Code, Codex, Antigravity CLI, Copilot CLI and OpenCode; any other ACP agent through a built-in catalog plus custom entries.
3. One engine interface of events and commands, with three clients and no logic of their own: the terminal, a minimal dcp chat screen, and Aurora Dev Suite. Clients never start the coding agents themselves.
4. The minimal chat screen (chat, attachments, questions, gate approvals; nothing else) would be a Tauri 2 application whose web view is a Leptos component meant to be reused by Aurora. Dashboards, metrics and graphical settings stay only in Aurora.
5. A stronger onboarding: dcp detects the installed agents, checks their sign-in, probes each one, records which agent takes which role (team rules in `.dev-cycle.json`, the developer's installed agents in a local file outside Git) and refuses to run until the setup is valid.
6. Minimal footprint in the user's repository: `.dev-cycle.json`, the project artifacts with the cycle trail, and one CI job that verifies the trail of each pull request (installed by one dcp command, required at merge). Instruction-file blocks, agent hooks, skill folders and Spec Kit scripts leave the repository.

Effect on recorded DCP items if adopted: DCP FEAT-030 becomes the engine state machine, DCP FEAT-029 is no longer needed, DCP FEAT-009 (MCP server) is replaced, DCP FEAT-002, FEAT-005 and FEAT-023 are absorbed. DCP FEAT-017, FEAT-018, FEAT-019, FEAT-022 and FEAT-032 are kept.

Questions for Aurora:

1. Does Aurora plan its own chat with coding agents, or its own way of starting them? In this direction Aurora would talk only to dcp, never to the agent command lines.
2. Aurora ADR-0006 says Aurora uses the engine only through the `dcp` binary and `--json`. Does a long-lived stream of events and commands (for a running cycle) fit that ADR, or does it need an amendment?
3. Is a Leptos chat component owned by DCP and imported by `aurora-ui` acceptable, and where should it live (DCP repository or the shared Levix libraries)?
4. This relaxes the 2026-10-28 decision that dcp has no graphical interface of its own. Does the minimal chat screen overlap anything Aurora has planned?
5. Choosing which agent takes which role would be edited in Aurora through `dcp config` (DCP FEAT-017 contract). Any conflict with Aurora FEAT-027?
6. Does anything in Aurora ADR-0015 (integrations coordinator) or in the provider contract `dcp.extension.v1` assume that the coding agent, not the engine, drives the phases?

### Constraints
- Decided by the DCP user on 2026-11-14 as direction: deterministic engine as conductor over existing coding agents; no own model runtime; CI verification stays, as one job.
- dcp must keep working without Aurora (terminal and minimal chat screen).
- Works on a local machine and in cloud sessions; no implementation specific to one provider.

### Closes when
Aurora answers the six questions with "fits", "fits with changes" (naming them) or "conflicts" (naming the ADR or item).

### Authority
The projects may settle contract shapes, names and where the shared component lives. Whether dcp gets its own chat screen, and any change to Aurora's vision or released contracts, goes to the user.
