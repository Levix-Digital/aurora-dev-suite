---
id: 20
title: The Dev Cycle Protocol is proprietary from its next versions; Aurora uses it only through its releases
status: accepted
date: '2026-11-14'
version: null
subsystems: [engines]
supersedes: []
superseded_by: []
amends: [5, 18]
amended_by: []
---

# ADR-0020: The Dev Cycle Protocol is proprietary from its next versions; Aurora uses it only through its releases

## Context

ADR-0018 described the Dev Cycle Protocol as open source and built the Community edition on it. On 2026-11-14 the owner decided that the Dev Cycle Protocol is proprietary for every version after engine-v0.46.1 and v2.10.0-rc.1 (message 20261114-163846 from dev-cycle-protocol): all rights reserved, private repository, customer source access only under a written agreement for audit. Versions published up to that cut-off stay under MIT OR Apache-2.0. The protocol is distributed as signed engine binaries and a skills install package in the public releases repository `Levix-Digital/dcp-releases`.

## Decision

- **Aurora uses the Dev Cycle Protocol only through its released binaries and its published contracts** (`dcp.diagnose`, `dcp.config`, `dcp.metrics`, `dcp.readiness`, `dcp.extension.v1` and the coming `dcp.events`), never through its source. This is what ADR-0003 and ADR-0006 already require; the contracts must stay published with each release.
- **Suite texts cite the Dev Cycle Protocol as proprietary;** releases up to the cut-off remain MIT OR Apache-2.0.
- **The Community edition** keeps its place in ADR-0018 with the Dev Cycle Protocol's free tier, whose terms the protocol defines, instead of an open-source protocol.
- **A chat component from the Dev Cycle Protocol for Aurora** (asked in the conducted-mode alignment of 2026-11-14) is proprietary like the rest of the new protocol code; how it reaches Aurora (a private registry, vendoring or a binary component) is the owner's decision with the conductor design.
- **This amends ADR-0005 and ADR-0018** where they say the Dev Cycle Protocol is open source.

## Alternatives Considered

- **Keep relying on the protocol's source** (for example to read its files or link its crates): excluded by ADR-0003 and ADR-0006 and now by the license.

## Consequences

- `docs/product/VISION.md` no longer calls the Dev Cycle Protocol open source.
- Nothing in Aurora's code reads the protocol's source; no code change.
