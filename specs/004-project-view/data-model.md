# Data Model: Project view

The core types live in `aurora-core` (`domain/project_view.rs`) and are derived from `CycleState` (FEAT-016); nothing is stored. The phase, waiting items and summary are those of FEAT-014 (`domain/portfolio.rs`).

## Candidate stage

`CandidateStage`: `TestsToRun`, `FailuresToReview`, `ReadyForRelease`, `Released`, from `CandidateRun::stage()`, first that applies:

| Stage | When |
|---|---|
| `Released` | `released` |
| `TestsToRun` | `not_run > 0` |
| `FailuresToReview` | `failed > 0` |
| `ReadyForRelease` | otherwise |

The Portfolio's waiting title of a candidate follows the stage (texts unchanged).

## Gate

`Gate { kind: GateKind, detail: String }`; the title is `GateKind::label()`, from `Gate::all(&CycleState)`, in this order:

| Kind | When | Title | Detail |
|---|---|---|---|
| `Decision` | `next.ask` | "Decision" | "Choose how to go on: <recommended option title>", or "Choose how to go on" when no option is recommended |
| `CandidateAcceptance` | candidate in `TestsToRun` or `FailuresToReview` | "Release candidate acceptance" | "<run>: <n> not run, <m> failed" |
| `Release` | candidate in `ReadyForRelease` | "Release gate" | "<version> passed its manual tests and waits for your approval" |

## Next step

`NextStep::ordered()` returns the options with the recommended one first, each paired with whether it is the recommended one.

## Plan progress

`PlanProgress::fraction()` is `milestones_done / milestones_total` clamped to 0..=1, 0 without milestones; `PlanProgress::progress_label()` reads "3 of 5 milestones" or "No milestones".

## View state (interface only)

`Selected`: the chosen project id in `AppContext`. `DetailState`: `Reading`, `Read { state, refreshing }`, `Failed { error, refreshing }`, with the number of the latest request.
