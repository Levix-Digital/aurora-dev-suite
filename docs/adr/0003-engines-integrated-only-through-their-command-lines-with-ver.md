---
id: 3
title: Engines integrated only through their command lines with versioned JSON
status: accepted
date: '2026-10-27'
version: null
subsystems: [core, engines]
supersedes: []
superseded_by: []
amends: []
amended_by: []
---

# ADR-0003: Engines integrated only through their command lines with versioned JSON

## Context

Gedo's ADR-0013 makes its versioned `--json` output the contract for tools built on it, and names a team dashboard as one of those tools; the Dev Cycle Protocol's ADR-0004 integrates with Gedo only through its command line. Aurora must not couple its releases to the engines' code, the engines stay separate products, and none of them is ready today.

## Decision

- Aurora reads every engine only by running its command line with `--json`. It never links an engine's crates.
- Every response is one JSON object with `schema` and `schema_version` (`MAJOR.MINOR`). An adapter supports a list of MAJOR versions and ignores unknown fields.
- At start-up and on demand, `EngineProbe` runs `<engine> version --json` and records the engine's version and capabilities. Each Aurora feature declares the capabilities it needs; a missing capability disables that feature with a message that says what to install or update, never an error dialog.
- Commands run with a timeout, without a shell, from the project's directory, and their standard error is kept for diagnostics.
- Until an engine offers the contract, a **stand-in adapter** implements the same port: for the Dev Cycle, a reader of `.dev-cycle.json`, `docs/BACKLOG.md`, `docs/ROADMAP.md` and `docs/plans/*.md`, plus the current scripts' `--json` when Python is available; for Gedo and AnyContext, fixtures used in development and tests only.
- The commands and schemas Aurora needs from each engine are proposed to its repository (suggestion files of 2026-10-27).

## Alternatives Considered

- **Link the engines as crates**: richer access, but couples Aurora to each engine's build, versions and internal APIs, and would break the separation the engines' own ADRs chose.
- **Wait for servers (`actx rpc`, a Gedo daemon API)**: better for live updates, but none exists, and the command line is the contract all engines already plan.

## Consequences

- Aurora works with any engine version that reports the capabilities it needs.
- Each call costs a process start (tens of milliseconds); the core caches results per refresh and long operations stream progress.
- A stand-in adapter for the Dev Cycle must be retired when `dcp` publishes its contract.
