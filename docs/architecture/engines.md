---
title: Engine Adapters
summary: Driven adapters that read the Dev Cycle, Gedo and AnyContext through their Rust binaries' versioned JSON.
code: [crates/aurora-engines]
---

# Engine Adapters

Driven adapters that read the Dev Cycle, Gedo and AnyContext through their Rust binaries' versioned JSON. Not built yet: this page describes the design of the foundations.

## Responsibilities

- Run engine commands safely and parse their versioned JSON.
- Implement the core's ports for each engine and contract version.
- It owns no product rules: what to show and when lives in the core.

## Structure

| Module | Holds |
|---|---|
| `runner` | process execution without a shell, timeout, working directory, standard error capture, JSON envelope (`schema`, `schema_version`) check |
| `probe` | `EngineProbe`: `<engine> version --json`, binary discovery on `PATH` and in the suite's install directory |
| `devcycle::dcp` | `CycleSource` over the future `dcp` binary |
| `gedo` | `CollaborationSource` over `gedo --json` (from Gedo v0.4.0) |
| `actx` | `KnowledgeSource` over `actx --json` (once AnyContext has it) |
| `registry` | `ProjectRegistry` over a TOML file in the user's configuration directory |

## Behavior

The composition root picks, per engine, the newest adapter whose contract the installed engine reports; without one, the engine's features are unavailable (ADR-0006). Fixtures for tests and the interface preview live in `aurora-core` behind its `fixtures` feature. Unknown JSON fields are ignored; an unsupported MAJOR version marks the engine as degraded with an "update" message.

## Interfaces

Implements the ports of `aurora-core`. Calls the engines' command lines and reads and writes the registry and settings files.

## Decisions

- [ADR-0003](../adr/0003-engines-integrated-only-through-their-command-lines-with-ver.md): command lines with versioned JSON only.
- [ADR-0006](../adr/0006-no-stand-in-adapters-engines-plugged-in-only-through-their-r.md): no stand-in adapters; engines only through their Rust binaries.
- [ADR-0004](../adr/0004-local-first-state-derived-from-the-repositories.md): the registry file is the only own state.
