---
title: Engine Adapters
summary: Driven adapters that read the Dev Cycle, Gedo and AnyContext through their command lines, plus stand-in adapters until each contract exists.
code: [crates/aurora-engines]
---

# Engine Adapters

Driven adapters that read the Dev Cycle, Gedo and AnyContext through their command lines, plus stand-in adapters until each contract exists. Not built yet: this page describes the design of the foundations.

## Responsibilities

- Run engine commands safely and parse their versioned JSON.
- Implement the core's ports for each engine and contract version.
- Provide stand-ins while an engine has no contract.
- It owns no product rules: what to show and when lives in the core.

## Structure

| Module | Holds |
|---|---|
| `runner` | process execution without a shell, timeout, working directory, standard error capture, JSON envelope (`schema`, `schema_version`) check |
| `probe` | `EngineProbe`: `<engine> version --json`, binary discovery on `PATH` and in the suite's install directory |
| `devcycle::legacy` | stand-in `CycleSource`: reads `.dev-cycle.json`, `docs/BACKLOG.md`, `docs/ROADMAP.md`, `docs/plans/*.md` frontmatter, and runs the current Python scripts' `--json` when available |
| `devcycle::dcp` | `CycleSource` over the future `dcp` binary |
| `gedo` | `CollaborationSource` over `gedo --json` (from Gedo v0.4.0) |
| `actx` | `KnowledgeSource` over `actx --json` (once AnyContext has it) |
| `fixtures` | fixture adapters for every port, used by tests and by the interface in development |
| `registry` | `ProjectRegistry` over a TOML file in the user's configuration directory |

## Behavior

The composition root picks, per engine, the newest adapter whose contract the installed engine reports, else the stand-in. Unknown JSON fields are ignored; an unsupported MAJOR version marks the engine as degraded with an "update" message.

## Interfaces

Implements the ports of `aurora-core`. Calls the engines' command lines and reads repository files (stand-in only) and the registry file.

## Decisions

- [ADR-0003](../adr/0003-engines-integrated-only-through-their-command-lines-with-ver.md): command lines with versioned JSON only, stand-ins until then.
- [ADR-0004](../adr/0004-local-first-state-derived-from-the-repositories.md): the registry file is the only own state.
