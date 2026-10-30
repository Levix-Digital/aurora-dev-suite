# Data Model: Portfolio view

All types live in `aurora-core` (`domain/portfolio.rs`) and are derived from `CycleState` (FEAT-016); nothing is stored.

## Phase

`CyclePhase`: `ReleaseCandidate`, `Implementation`, `Review`, `Planning`, `Idle`.

| Phase | When (first that applies) | Label |
|---|---|---|
| `ReleaseCandidate` | `release_candidate` present and not `released` | Release candidate |
| `Implementation` | `plans` not empty | Implementation |
| `Review` | `pull_requests` not empty | Review |
| `Planning` | `release` present with `open_items` not empty | Planning |
| `Idle` | otherwise | Idle |

## Waiting item

`WaitingItem { kind: WaitingKind, key: String, title: String }`

| Kind | One per | Key | Title |
|---|---|---|---|
| `ReleaseCandidate` | unreleased candidate | `rc:<run>` | "Run the manual tests of <run>" when tests are not run, "Look at the failed tests of <run>" when some failed and all ran, "Approve the release of <version>" when all passed |
| `PullRequest` | pull request with `needs_action` | `pr:<number>` | "#<number> <title>" |
| `BlockingBug` | blocking bug | `bug:<id>` | "<id> <title>" |
| `UntriagedBug` | untriaged bug | `triage:<id>` | "<id> <title>" |

`WaitingKind::label(count)` gives the plural-aware name shown in a row ("1 pull request", "2 blocking bugs").

## Cycle summary

`CycleSummary { phase: CyclePhase, release: Option<CurrentRelease>, waiting: Vec<WaitingItem>, has_findings: bool }`, built by `CycleSummary::of(&CycleState)`.

## Row state (interface only)

`RowState`: `Reading`, `Read { summary, refreshing }`, `Failed { error }`; each row also keeps the generation of its latest request.
