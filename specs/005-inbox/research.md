# Research: Inbox

## R1. The Inbox items are the Portfolio's waiting items

**Decision**: the Inbox lists `WaitingItem::all` of each project (FEAT-014): the unreleased release candidate, each pull request that needs action, each blocking bug and each untriaged bug. The protocol's "Decision" (`next.ask`) is not an item.

**Rationale**: one rule keeps the Inbox total equal to the Portfolio's waiting counts (SC-002). The decision's options point at the same items (FEAT-014 research R2), so listing it would count them twice; the Decision gate stays in the Project view, one "Open" away. The release candidate's acceptance and release gate are the candidate item, in its stage (FEAT-015 research R2).

**Alternatives considered**: a separate Inbox rule set over the cycle state (two rules to keep in agreement); one Decision item per project that asks (duplicates).

## R2. Urgency

**Decision**: four levels, most urgent first, computed in the core with each waiting item:

| Urgency | Label | Items |
|---|---|---|
| `BlocksRelease` | "Blocks a release" | blocking bugs; release candidates with failed tests (`FailuresToReview`) |
| `Acceptance` | "Waits for your acceptance" | release candidates with tests to run (`TestsToRun`) or ready for release (`ReadyForRelease`) |
| `Review` | "Waits for your review" | pull requests that need action |
| `Triage` | "To triage" | untriaged bugs |

**Rationale**: the backlog item asks for "ordered by urgency"; the Dev Cycle Protocol puts bugs before new work and finishing work in flight before starting new work. A blocking bug or a failed candidate stops a release; a candidate to test or to release is the user's gate at the end of the cycle; a review unblocks one change; triage is bookkeeping that no release waits for. Levels instead of a score keep the order explainable in one line per group.

**Alternatives considered**: the protocol's `next.options` order (per project only, and not available across projects); a numeric score with age (the contract has no dates per item).

## R3. Order across projects

**Decision**: urgency first; inside a level, the projects' order in the registry (the Portfolio's order), then the item's order in its project. A stable sort by urgency over the items taken project by project gives it.

**Rationale**: predictable, the same order on every refresh, and it matches the Portfolio's rows.

## R4. Reading every project

**Decision**: move the Portfolio's row state and its read and run functions over the loader (`views/portfolio.rs`) into `views/reads.rs`, and use them in both views. Each view reads when it opens, as the Portfolio does today; there is no cache shared between views.

**Rationale**: the same scheduling rules (FR-001) without a copy. A shared cache in `AppContext` would avoid a second read when switching views, but it changes the Portfolio's behavior and adds invalidation rules; it can come later if reads prove slow.
