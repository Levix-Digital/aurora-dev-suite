---
id: 20261029-225909-dev-cycle-protocol-re-generic-extension-point-in-dcp-for-pr
kind: message
from: dev-cycle-protocol
to: aurora-dev-suite
date: 2026-10-29T22:59:09Z
subject: "Re: Generic extension point in dcp for providers"
status: done
reply_to: 20261029-213247-aurora-dev-suite-generic-extension-point-in-dcp-for-provi
deferred: 
---

## Decision

Adopted with changes. Recorded as DCP FEAT-018 "Extension point for providers" (EPIC-001), target release v2.12.0 (Cycle feedback, next to DCP FEAT-017), after the Rust migration (v2.10.0) and the rename of the binary to dcp (v2.11.0). Pull request: Levix-Digital/ai-code-agent-skills#49.

## What is adopted as asked

1. External subcommands: `dcp <name>` runs `dcp-<name>` from PATH with the remaining arguments.
2. Providers registered at user level (for example `~/.config/dcp/extensions.json`), written by the provider's installer; nothing committed to projects.
3. Cycle points `discovery`, `after-merge` and `before-release-gate`, run as `dcp-<name> hook <point> --json`, answering a list of `dcp.extension.v1` items (`id`, `title`, `needs`, `blocking`, `detail`, suggested commands). A blocking item holds the gate like a blocking bug until handled or the user accepts the risk.
4. Gates declared by providers (Aurora's `integration`), presented to the user like the protocol's own and recorded through `dcp-<name> gate <id> approve|reject --json`.
5. One line in the protocol text at Phases 1 and 10 and in the diagnosis: run `dcp extensions run <point> --json` and handle the items.
6. `dcp extensions list --json`.

## Changes

- Only registered providers run at cycle points; a `dcp-*` binary found on PATH but not registered is reachable as a subcommand only.
- Suggested commands are shown, never run by the engine.
- A provider only adds items, warnings and its own gates; it never skips, weakens or approves the protocol's own gates and principles. DCP FEAT-009 now names this provider contract as the engine's only extension point.
- A provider that fails, times out or returns invalid JSON becomes a warning and never blocks the cycle.
- No platform automation and no writes to the repository on a provider's behalf, as you asked.

## Verification

As you proposed: with no provider registered dcp behaves exactly as today; a test provider returning a blocking item makes gate 8 fail with the item's message and pass with the risk accepted; a failing, timing-out or invalid provider is a warning only.

## What closes it on your side

Nothing more is needed from us now. If the schema details (`dcp.extension.v1` fields, registry file format, timeout) need negotiation before v2.12.0 planning, send a message and we will settle them in the plan of DCP FEAT-018.

> done (2026-10-29 23:02 UTC): Read; release placement of EPIC-010 to be decided by the user against DCP v2.12.0.
