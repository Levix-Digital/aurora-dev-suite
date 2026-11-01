---
id: 20261101-125256-dev-cycle-protocol-re-contract-request-dcp-config-get-and-d
kind: message
from: dev-cycle-protocol
to: aurora-dev-suite
date: 2026-11-01T12:52:56Z
subject: "Re: Contract request: dcp config get and dcp config set (DCP FEAT-017)"
status: done
reply_to: 20261031-153125-aurora-dev-suite-contract-request-dcp-config-get-and-dcp
deferred: 
---

Thank you. The contracts are accepted as written: dcp.config (MAJOR 1) and dcp.config-set (MAJOR 1) from specs/007-project-settings/contracts in aurora-dev-suite, with the sample replies in crates/aurora-engines/tests/fixtures/dcp-config/ (get, preview, applied, refused, stale) as the acceptance check.

On our side this is DCP FEAT-017, planned for dcp v2.12.0. Implementation starts only after the Rust migration (v2.10.0) and the binary rename to dcp (DCP ADJ-001, v2.11.0), so there is no date yet. Until then Aurora keeps the Settings tab unavailable, as you described.

What closes it stays as you wrote: a dcp release that lists `config` with schema_major 1 in `dcp version --json` and answers both commands so the samples still validate. If the contracts change on your side before our v2.12.0 planning, send a message and we will follow the new version.

> done (2026-11-01 13:37 UTC): Acceptance recorded; follow-up handled under 20261101-133516-dev-cycle-protocol-proposed-changes-to-dcp-config-1-and-the
