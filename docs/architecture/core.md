---
title: Aurora Core
summary: Domain model of the suite, the use cases and the ports every engine and surface goes through.
code: [crates/aurora-core]
---

# Aurora Core

Domain model of the suite, the use cases and the ports every engine and surface goes through.

## Responsibilities

- The domain types of projects, engines, settings, features and the development cycle state, as plain data with serde (`camelCase` on the wire).
- The use cases, one per user action: list, add, rename, remove and relocate a project; read and save the settings; probe an engine and build the engines overview; read a project's cycle state.
- The ports the use cases need, as traits.
- The catalog of what each feature needs from each engine.
- Sample data and in-memory ports behind the `fixtures` feature, for every crate's tests and the interface preview.
- It does not run processes, read files or know any UI framework; it compiles to WebAssembly so the interface shares its types.

## Structure

| Module | Holds |
|---|---|
| `domain::project` | `Project`, `ProjectId`, `ProjectName` (trimmed, 1 to 80 characters), `Presence` (present or missing), `ProjectList` with an optional `RegistryNotice` (recovered or read-only), `AddOutcome`, `ProjectError` |
| `domain::engine` | `EngineKind` (`dcp`, `gedo`, `actx`), `EngineStatus` (ready, installed without contract, degraded with a reason, missing), `EngineReport`, `Capability`, `EngineSettings` (location, time limit) |
| `domain::cycle` | `CycleState` (summary, `NextStep` with its options, `CurrentRelease` with open items, `PlanProgress`, `CandidateRun`, `PullRequestState`, blocking `ItemRef`s, `UntriagedBug`s, findings, read time) and `CycleError` (unavailable with a hint, project missing, timed out, too large, failed, invalid, unsupported with the side to update) |
| `domain::portfolio` | what the views show of a cycle state: `CyclePhase` (release candidate, implementation, review, planning, idle), `WaitingItem`s by `WaitingKind` (release candidate, pull request, blocking bug, untriaged bug) and `CycleSummary` |
| `domain::inbox` | the Inbox: the `Urgency` of a waiting item (blocks a release, acceptance, review, triage) and the `Inbox` of every project's `InboxEntry`s in order, with its groups and project count |
| `domain::project_view` | what the Project view adds: `CandidateStage` of a release candidate (tests to run, failures to look at, ready for release, released), the `Gate`s by `GateKind` (decision, release candidate acceptance, release), `NextStep::ordered` and the progress of a `PlanProgress` |
| `domain::settings` | `Settings` (version, `ThemeChoice`, per-engine settings) and `Theme` resolution |
| `domain::feature` | `Feature`, `AvailabilityState` (available, unavailable with what is missing and a hint, not in this release), `EnginesOverview` |
| `ports` | `ProjectRegistry`, `SettingsStore`, `EngineProbe`, `CycleSource` |
| `usecases` | `projects`, `settings`, `engines`, `cycle` |
| `capabilities` | the feature catalog and its evaluation; `usable_dcp` for the cycle state |
| `fixtures` | feature `fixtures`: four sample projects (one missing), sample engine reports, `MemoryRegistry`, `MemorySettings`, `FixtureProbe`, three sample cycle states (busy, idle, release candidate) and `FixtureCycleSource` |

```mermaid
flowchart LR
    UI[aurora-ui] -->|Tauri command| D[aurora-desktop]
    D --> U[usecases]
    U --> P[ports]
    E[aurora-engines] -. implements .-> P
    F[fixtures] -. implements .-> P
```

## Behavior

The catalog maps each feature to one engine, the capabilities it needs (schema MAJOR 1) and the release that ships it:

| Feature | Engine | Capabilities | Release |
|---|---|---|---|
| Portfolio, Project view, Decision inbox | Dev Cycle (`dcp`) | `diagnose` | v0.2.0 |
| Agent work | Gedo | `radar`, `scope`, `stream` | v0.4.0 |
| Knowledge | AnyContext (`actx`) | `ask`, `workspaces` | v0.5.0 |

A feature is unavailable, with the hint of its engine, while its engine is not ready or lacks a capability; only a ready engine counts, and only the capabilities of the engine the feature names. When everything is there but the feature ships in a later release, it says "not in this release". Errors are values (`thiserror` enums), never panics.

**Cycle state.** `usecases::cycle::cycle_state` asks the `CycleSource` only when the latest `dcp` report is ready with `diagnose` at schema MAJOR 1 and the project's folder is present; otherwise it returns `Unavailable` with the catalog's hint, or `ProjectMissing`, without running anything. The state is read on demand and never stored. It does not depend on the release gate of the catalog: the views of v0.2.0 decide what to show.

**Phase and waiting items.** `CyclePhase::of` takes the first that applies: a release candidate not released, a plan in progress, an open pull request, a current release with open items, otherwise idle (the `dcp.diagnose` reply has no phase field). `WaitingItem::all` lists what waits for the user: the unreleased release candidate (tests to run, failures to look at, or the release to approve), each pull request that needs action, each blocking bug and each untriaged bug; the recommended next step is not counted, since its options point at the same items. Each item also carries a detail (a bug's severity, a candidate's test counts, a pull request's summary and review threads for the user, an untriaged bug's reason) and an `Urgency`: blocking bugs and candidates with failed tests block a release, other unreleased candidates wait for the user's acceptance, pull requests wait for review, untriaged bugs wait for triage. The Portfolio, the Project view and the Inbox share these rules.

**Inbox.** `Inbox::of` takes each project's id, name and `CycleSummary` in the registry's order and sorts the waiting items stably by urgency, so the order is urgency, then project, then the item's order in its project. The protocol's decision (`next.ask`) is not an item: its options point at the same items, and the Inbox total stays equal to the Portfolio's.

**Gates.** `Gate::all` lists the decisions that wait for the user: a decision when the protocol asks (`next.ask`), naming the recommended option, then the release candidate's acceptance while its tests are not run or failed, or the release gate when all passed (`CandidateRun::stage`, which also gives the candidate's waiting title). Plan approval happens before a plan is saved and is not in the contract. `NextStep::ordered` puts the recommended option first.

`CollaborationSource` and `KnowledgeSource` are added with the first release that calls them.

## Interfaces

Called by `aurora-desktop` (one Tauri command per use case) and, for the preview, by `aurora-ui` through the fixtures. Calls nothing outside itself. `cargo xtask arch-check` fails the build when the core depends on Tauri, Leptos, the adapters or process, file or network APIs.

## Decisions

- [ADR-0001](../adr/0001-hexagonal-architecture-with-the-engines-behind-ports.md): hexagonal workspace, engines behind ports.
- [ADR-0003](../adr/0003-engines-integrated-only-through-their-command-lines-with-ver.md): capabilities decide which features run.
- [ADR-0004](../adr/0004-local-first-state-derived-from-the-repositories.md): the only own state is the project registry and the settings.
- [ADR-0007](../adr/0007-cross-project-integrations-through-git-on-an-orphan-inbox-br.md): cross-project integrations through Git on an orphan `aurora/inbox` branch, written only by the integrations provider.
