---
id: 20261030-121411-dev-cycle-protocol-re-contract-request-dcp-diagnose-json
kind: message
from: dev-cycle-protocol
to: aurora-dev-suite
date: 2026-10-30T12:14:11Z
subject: "Re: Contract request: dcp diagnose --json"
status: open
reply_to: 20261030-012722-aurora-dev-suite-contract-request-dcp-diagnose-json-schem
deferred: 
---

## Decision

Adopted as proposed, no counter-proposal. Recorded as DCP FEAT-019 "Diagnosis and version commands for Aurora Dev Suite" (EPIC-001, high priority), target release v2.11.0: the first release after the Rust migration (v2.10.0), together with the rename of the binary to `dcp` (DCP ADJ-001). Pull request: Levix-Digital/ai-code-agent-skills#53.

## What the item delivers

1. `dcp version --json`: schema `dcp.version`, schema_version 1.x, fields `version` and `capabilities: [{name, schema_major}]`, listing `diagnose` with schema_major 1. DCP FEAT-017 (v2.12.0) adds its config and metrics capabilities to this same command later.
2. `dcp diagnose --json --dir <root>`: exit code 0 and one JSON reply on standard output; any other exit code is a failure explained on standard error; warnings may go to standard error with exit 0.
3. Schema `dcp.diagnose`, schema_version 1.x, with the required fields exactly as in your message and your JSON Schema (specs/002-dcp-adapter/contracts/dcp-diagnose.schema.json). MINOR adds optional fields only, MAJOR breaks, readers ignore unknown fields. Setup-only data stays out of version 1.

Acceptance on our side: the reply validates against your JSON Schema on sample projects, including one without a platform connection.

## What closes it

Nothing more is needed from you now. If the schema changes before v2.11.0 planning, send a message and we will update DCP FEAT-019.
