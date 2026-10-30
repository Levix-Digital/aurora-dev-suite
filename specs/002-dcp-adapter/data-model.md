# Data Model: dcp adapter

New types in `aurora-core::domain` (module `cycle`), serialized with serde in `camelCase` for the interface. They are Aurora's own types: the adapter translates the `dcp.diagnose` reply (`contracts/dcp-diagnose.schema.json`) into them, so a later MAJOR of the contract changes only the adapter.

## CycleState

| Field | Type | From the reply |
|---|---|---|
| `summary` | `String` | `summary` |
| `next` | `NextStep` | `next` |
| `release` | `Option<CurrentRelease>` | `release` |
| `plans` | `Vec<PlanProgress>` | `plans` |
| `releaseCandidate` | `Option<CandidateRun>` | `release_candidate` |
| `pullRequests` | `Vec<PullRequestState>` | `pull_requests` |
| `blockingBugs` | `Vec<ItemRef>` | `bugs.blocking` |
| `untriagedBugs` | `Vec<UntriagedBug>` | `bugs.untriaged` |
| `findings` | `Vec<String>` | `findings` |
| `readAt` | `String` (RFC 3339) | the time Aurora read it |

- `ItemRef { id, rank: Option<String>, title }`
- `UntriagedBug { item: ItemRef, reason }`
- `CurrentRelease { version, name, openItems: Vec<ItemRef> }`
- `PlanProgress { id, title, status, milestonesDone: u32, milestonesTotal: u32 }`
- `CandidateRun { run, version, notRun: u32, failed: u32, released: bool }`
- `PullRequestState { number: u64, title, summary, needsAction: bool, threadsForUser: u32 }`
- `NextStep { ask: bool, recommended: Option<String>, options: Vec<NextOption> }`, `NextOption { key, title, reason: Option<String> }`

The reply's `root` is checked by the adapter (research R3) and not kept.

## CycleError

`thiserror` enum, serialized with a `kind` tag in `camelCase` (variant fields keep their Rust names, as `ProjectError` does); each variant has a user-facing message.

| Variant | When | Message carries |
|---|---|---|
| `Unavailable { hint }` | `dcp` missing, without the contract, degraded, or without `diagnose` at MAJOR 1 | what to install or update (the catalog's hint) |
| `ProjectMissing` | the project's folder is not present | the last known path |
| `TimedOut { timeout_ms, stderr }` | no answer within the limit | the limit |
| `TooLarge { stderr }` | reply over 1 MiB | the limit |
| `Failed { exit_code, stderr }` | spawn error or non-zero exit | the exit code; stderr kept for the user |
| `Invalid { message, stderr }` | not JSON, wrong `schema`, missing fields, `root` of another repository | what was wrong |
| `Unsupported { found, update }` | schema MAJOR is not 1 | `UpdateSide::Aurora` (newer) or `UpdateSide::Engine` (older) |

## Port

```rust
/// Reads a project's development cycle state from the Dev Cycle Protocol (ADR-0003, ADR-0006).
pub trait CycleSource: Send + Sync {
    fn cycle_state(&self, engine: &EngineReport, settings: &EngineSettings, root: &Path)
        -> Result<CycleState, CycleError>;
}
```

The engine report gives the binary's location (research R5); the use case calls the port only when the report is ready with the capability.

## Use case

`usecases::cycle::cycle_state(source, dcp: Option<&EngineReport>, settings, project) -> Result<CycleState, CycleError>`: checks presence and capability (research R4), then calls the port.

## Fixtures

`fixtures::FixtureCycleSource` answers per sample repository root: `gedo` busy (release with open items, a plan at 3 of 5 milestones, a pull request needing the user, one blocking bug, one untriaged bug, `next.ask` true), `any-context` idle (empty lists, `release` null), `ai-code-agent-skills` a release candidate waiting for manual tests, and `slow`, `future` and any other root the failures `TimedOut`, `Unsupported` and `Invalid`. The same states are stored as JSON replies under `crates/aurora-engines/tests/fixtures/dcp-diagnose/`, checked against the schema, and a test proves they read into the core's samples.
