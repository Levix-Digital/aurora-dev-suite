---
id: 1
title: Hexagonal architecture with the engines behind ports
status: accepted
date: '2026-10-27'
version: null
subsystems: [core, engines, desktop, ui]
supersedes: []
superseded_by: []
amends: []
amended_by: []
---

# ADR-0001: Hexagonal architecture with the engines behind ports

## Context

Aurora joins three engines that are not ready yet and evolve on their own: the Dev Cycle Protocol (Python scripts today, a Rust `dcp` binary later), Gedo (a library until its v0.4.0 command line) and AnyContext (no machine-readable output yet). Levi decided on 2026-10-27 to build Aurora's foundations now and plug each engine in when it is ready. The interface is a desktop application today; other surfaces (a command line for scripts, a tray view) may follow.

## Decision

Aurora is a hexagonal Rust workspace:

- `crates/aurora-core`: the domain (projects, cycle state, gates, engine health, capabilities, decision inbox items), the use cases and the **ports** as traits: `ProjectRegistry`, `CycleSource`, `CollaborationSource`, `KnowledgeSource` and `EngineProbe`. It depends on no UI, framework, process or file system code, so it also compiles to WebAssembly and its types are shared with the interface.
- `crates/aurora-engines`: the driven adapters that implement the ports (one per engine and per contract version) and the stand-in adapters used until an engine's contract exists.
- `crates/aurora-desktop`: the Tauri application and the composition root that chooses the adapters at start-up.
- `crates/aurora-ui`: the driving adapter, the Leptos interface.

`architecture.forbidden_core_imports` lists the adapter crates, the UI frameworks and process spawning, and `check_design.py` enforces it.

## Alternatives Considered

- **Layered application around Tauri commands**: simpler at first, but each engine's command line would leak into the commands, and replacing a stand-in by the real engine would touch the interface.
- **Modular monolith by feature** (portfolio, agents, knowledge): fits the screens, but every feature talks to several engines, so the engine boundary would be repeated in each module.

## Consequences

- An engine's arrival is one new adapter and one line in the composition root; the core and the interface do not change.
- Every view can be built and tested against fixtures before the engines exist.
- The core must stay free of I/O, which costs some indirection for simple reads.
