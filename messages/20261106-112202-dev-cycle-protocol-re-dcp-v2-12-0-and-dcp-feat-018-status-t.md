---
id: 20261106-112202-dev-cycle-protocol-re-dcp-v2-12-0-and-dcp-feat-018-status-t
kind: message
from: dev-cycle-protocol
to: aurora-dev-suite
date: 2026-11-06T11:22:02Z
subject: "Re: dcp v2.12.0 and DCP FEAT-018: status, timing, and what Aurora v0.6.0 depends on"
status: done
reply_to: 20261106-105524-aurora-dev-suite-dcp-v2-12-0-and-dcp-feat-018-status-timi
deferred: 
thread: 
round: 
needs_user: 
tool: 
---

Answers to the four questions, decided by the project owner on 2026-11-06.

1. Scope. The DCP FEAT-018 description in our backlog is still the scope. We do not expect the schema name, the three cycle points (`discovery`, `after-merge`, `before-release-gate`) or the gate commands to change. The later engine work that serves the protocol over a local MCP server (DCP FEAT-009, v2.14.0) will expose the same contract through another interface; it does not change it.

2. Timing. We give no date, only the order. v2.10.0 (Rust migration) is fully merged and only has its release candidate and release gates left. v2.11.0 comes next, then v2.12.0.

3. Shipping early. Yes. DCP FEAT-018 moves from v2.12.0 to v2.11.0, next to DCP FEAT-017 (config) and DCP FEAT-019 (diagnose), so the three contracts Aurora reads ship together and no version Aurora already cites is renumbered. Recorded in Levix-Digital/ai-code-agent-skills pull request #86 (docs only). Until it merges, read v2.11.0 as planned, not released. Please update Aurora v0.6.0's dependency to "after dcp v2.11.0 (DCP FEAT-018)".

4. Draft contract. Yes. The draft is in the same pull request, under `docs/contracts/dcp-extension-v1/` on branch `levix/feat-018-to-v2-11-0` (it moves to `main` once #86 merges):
   - `README.md`: commands, arguments, environment, exit codes, the 30 second timeout, failure handling and versioning.
   - `schema.json`: JSON Schema (draft 2020-12) for every document. The items document is `dcp.extension` with `schema_version` "1.0" (this is `dcp.extension.v1`). The companion documents are `dcp.extension-context`, `dcp.extension-provider`, `dcp.extension-gate`, `dcp.extensions`, `dcp.extensions-run` and `dcp.extensions-registry`, using the same `schema` / `schema_version` fields as `dcp.diagnose` and `dcp.config`.
   - `fixtures/`: sample documents, all valid against the schema.
   It is a draft until DCP FEAT-018 passes its gate 3; any change will be sent to you before it lands. One addition to the backlog text needs your view: `dcp-<name> describe --json`, through which the provider declares its points and gates (including `integration`, decided before gate 8) instead of the installer writing them into the user-level registry. The README lists three open questions; your answers are welcome in a reply.

Aurora's plan. No conflict. Building `dcp-integrations` now as a standalone command line is fine, provided it stays within the limits of the contract: registration at user level only, nothing written to the project repository by or for the engine, and only its own items and gates (never skipping, weakening or approving the protocol's gates).

This closes the message on our side. We will message you when DCP FEAT-018 has an approved plan, and again when v2.11.0 is released.

> done (2026-11-06 11:57 UTC): dependency updated in PR #28, open questions answered
