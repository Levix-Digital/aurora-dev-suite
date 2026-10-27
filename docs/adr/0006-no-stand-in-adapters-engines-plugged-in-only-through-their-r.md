---
id: 6
title: 'No stand-in adapters: engines plugged in only through their Rust binaries'
status: accepted
date: '2026-10-27'
version: null
subsystems: [engines, core]
supersedes: []
superseded_by: []
amends: [3]
amended_by: []
---

# ADR-0006: No stand-in adapters: engines plugged in only through their Rust binaries

## Context

ADR-0003 allowed a stand-in adapter while an engine has no contract: for the Dev Cycle, a reader of the repository's Markdown files plus the current Python scripts' `--json`. Levi decided on 2026-10-27 that the Dev Cycle is plugged into Aurora only once it is fully migrated to Rust (`dcp`), and that Aurora's interfaces are designed expecting all engines to be Rust binaries. A stand-in would also tie Aurora to file formats and scripts that `dcp` will replace.

## Decision

- Aurora has no stand-in adapter for any engine. An engine is used only through its Rust binary (`dcp`, `gedo`, `actx`) and the versioned `--json` contract of ADR-0003.
- Until an engine offers the contract, it is shown as missing or as installed without the contract, and the features that need it are unavailable with a message.
- Features are built and tested before their engine exists against the core's fixtures, which follow the agreed contract; the real adapter is added when the engine ships it.
- Aurora never runs Python or reads an engine's own files.

This amends ADR-0003 in its stand-in part only; the rest of ADR-0003 holds.

## Alternatives Considered

- **Keep the Dev Cycle stand-in** (Markdown files and Python scripts): a portfolio view earlier, but Aurora would depend on Python and on formats `dcp` replaces, and the work would be thrown away.

## Consequences

- No Python, Node or other runtime is ever needed by Aurora.
- The portfolio, project and inbox views stay unavailable until the first `dcp` release with the contract, while being ready against fixtures.
- The contract proposed to each engine (suggestion files of 2026-10-27) becomes the only source of truth for those features.
