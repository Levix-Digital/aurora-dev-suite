# Data Model: Inbox

The core types live in `aurora-core` and are derived from `CycleState` (FEAT-016); nothing is stored. The waiting items are those of FEAT-014 (`domain/portfolio.rs`), the candidate stage that of FEAT-015 (`domain/project_view.rs`).

## Urgency

`Urgency`: `BlocksRelease`, `Acceptance`, `Review`, `Triage`, ordered most urgent first (`Ord`), with `label()` and `ALL`.

## Waiting item (extended)

`WaitingItem { kind, key, title, detail, urgency }`; `kind`, `key` and `title` are unchanged.

| Item | Urgency | Detail |
|---|---|---|
| Candidate `TestsToRun` | `Acceptance` | "<n> not run, <m> failed" |
| Candidate `FailuresToReview` | `BlocksRelease` | "<n> not run, <m> failed" |
| Candidate `ReadyForRelease` | `Acceptance` | "Passed its manual tests" |
| Pull request that needs action | `Review` | its summary, then ". 1 review thread for you" or ". <n> review threads for you" when there are any; `None` when both are empty |
| Blocking bug | `BlocksRelease` | "Severity <rank>", `None` without one |
| Untriaged bug | `Triage` | its reason |

## Inbox

`InboxEntry { project_id: String, project_name: String, item: WaitingItem }`.

`Inbox { entries: Vec<InboxEntry> }`:

- `Inbox::of(projects)`: takes `(project id, project name, &CycleSummary)` in registry order; the entries sorted stably by urgency (research R3).
- `groups()`: `(Urgency, entries)` in urgency order, without empty levels.
- `project_count()`: the projects with at least one entry.

## View state (interface only)

`RowState`: `Reading`, `Read { summary, refreshing }`, `Failed { error, refreshing }` per project id, moved from the Portfolio to `views/reads.rs` with `read` and `run`.
