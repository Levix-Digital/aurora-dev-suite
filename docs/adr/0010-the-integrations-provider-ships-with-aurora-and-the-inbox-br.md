---
id: 10
title: The integrations provider ships with Aurora and the inbox branch holds only data
status: accepted
date: '2026-11-06'
version: null
subsystems: [engines, desktop]
supersedes: []
superseded_by: []
amends: [7]
amended_by: [11]
---

# ADR-0010: The integrations provider ships with Aurora and the inbox branch holds only data

## Context

The provisional messages tool (IMP-001, IMP-002) is a Python script that every project clones from the `aurora/inbox` branch of this repository, and the list of projects that can talk (`tool/projects.json`) lives there too. That works while every project belongs to one owner who can read this repository. It does not work for the product: Aurora is installed on developers' machines, its repository is not available to the projects of a customer, and Aurora's integration uses no Python and no stand-in adapters (ADR-0006).

Levi proposed on 2026-11-06 to keep a Rust executable on the `aurora/inbox` branch of each project. Discussed and set aside, with the reasons below.

## Decision

This amends ADR-0007.

- **The executable ships with Aurora.** `dcp-integrations` (FEAT-034) is built in this repository, signed (FEAT-035), installed by Aurora's installer next to the desktop application (ADR-0008) and updated with it. For machines without Aurora (a CI runner, a server), the same release publishes it as a separate download per platform. Nothing executable is ever stored on an `aurora/inbox` branch.
- **Each project keeps its own list of counterparts.** `.aurora/integrations.json` on the project's work branch (FEAT-029) names the projects it talks to and their repositories; the provider reads it from the project's clone. No central registry, and no project depends on Aurora's repository.
- **The `aurora/inbox` branch holds only data.** Messages and a small `inbox.json` with the format version (`schema: aurora.inbox`, `schema_version`). Any version of the provider reads a format it knows and refuses, with a clear message, one it does not; the format follows the same rules as the other versioned contracts (MINOR adds optional fields, MAJOR breaks).
- **Same commands as the provisional tool.** The provider offers the commands of tool 0.3.0 with the same header and kinds (ADR-0009), so messages already written stay readable.
- **Transition.** The Python tool stays on this repository's `aurora/inbox` branch until the provider is released, then it is removed and the Levix projects' instructions point to `dcp-integrations`.

## Alternatives Considered

- **A Rust executable on each project's `aurora/inbox` branch** (Levi's proposal): available wherever the repository is, but one build per platform, a commit in every repository at each update, binaries growing the history for ever, and an executable that anyone with write access to the branch can replace, run by every developer; customers would not accept it.
- **Keep the registry and the tool in Aurora's repository**: works only while every project can read it, which is not the case for customers.
- **Python tool used by Aurora until the provider exists**: contradicts ADR-0006 and the decision of 2026-10-27 to keep Python out of Aurora's integration.

## Consequences

- Developers get the provider with Aurora; a machine without Aurora downloads one signed binary.
- FEAT-029 gains the counterparts' repositories as the provider's only registry; FEAT-034 gains signing, the separate download and `inbox.json`.
- A project whose counterpart's provider is older than the branch format sees a clear refusal instead of a misread message.
- Verification: the provider's tests cover reading and refusing `inbox.json` versions, the registry read from `.aurora/integrations.json`, and the release check that the provider binary is in every package and in the separate downloads.
