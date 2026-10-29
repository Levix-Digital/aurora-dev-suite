---
id: 7
title: Cross-project integrations through Git on an orphan inbox branch
status: accepted
date: '2026-10-29'
version: null
subsystems: [core, engines]
supersedes: []
superseded_by: []
amends: []
amended_by: []
---

# ADR-0007: Cross-project integrations through Git on an orphan inbox branch

## Context

Levix projects depend on each other (Aurora consumes `dcp`; `dcp`, Gedo and AnyContext evolve separately), and a change merged in one project often requires work in another. Today that knowledge travels by hand and gets lost. Levi asked on 2026-10-29 for a way for projects to learn about each other's merges and to agree on integrations before changing code, with these constraints:

- Aurora is a desktop application that never touches the network itself (air-gapped).
- The exchange must be invisible to the user; the user only decides at a gate.
- Companies do not accept tools that create repositories on their own.
- Nothing may depend on a particular coding agent or Git host (GitHub, Azure DevOps, GitLab, Bitbucket, a local server).
- Aurora does not commit to a project's work branches (ADR-0006), and integration traffic must not add commits, pull requests or CI runs to them.
- The feature exists only for Aurora users; the Dev Cycle Protocol stays generic.

A test on 2026-10-29 in this repository: an orphan branch worked end to end (create, append a reply, read from another clone fetching only that branch, commits under the repository identity, no CI run since CI follows `main` only). Pushing a ref outside `refs/heads` (`refs/aurora/inbox`, `refs/notes/*`) was refused with HTTP 403 by the Git policy of the environment that ran the test, before reaching the host.

## Decision

- **Transport is plain Git.** Each subscribed project has an orphan branch `aurora/inbox` in its own repository, with no history in common with the code. It is never merged, never opened as a pull request and not followed by CI. Whoever needs to reach project B writes to the `aurora/inbox` branch of B's repository, with the permissions the team already has.
- **Layout of `aurora/inbox`**: `merges/<date>-<from>-<ref>.md` (merge notices received), `integrations/<id>/<NN>-<kind>.md` (integration messages received), `counter` (the last integration number this project used). An integration id is `<opening project>-<NNN>-<slug>`, numbered by the opening project only, so two projects never collide. A conversation is the union of what each side received.
- **Message format**: Markdown with a header carrying `id`, `integration`, `kind` (`merge`, `proposal`, `counter`, `agreement`, `approved`, `rejected`, `message`), `from`, `to`, `date`, `round`, `status` (`open`, `seen`, `done`) and `subject`.
- **Subscriptions**: `.aurora/integrations.json` on the project's work branch, committed once, lists the projects it depends on, their repositories, what it consumes from each and the branches it follows.
- **Integrations provider**: a Rust binary shipped with Aurora is the only reader and writer of this data. It works with Git plumbing (no working tree changes) and offers `check`, `show`, `propose`, `counter`, `agree`, `defer` and `gate` with `--json`. It plugs into `dcp` through a generic extension point that the Dev Cycle Protocol runs at the start of discovery and before the release gate; without Aurora no provider is registered and nothing happens.
- **Mechanics and judgment are separate**: the provider does the mechanics (fetch, write, filter by what is consumed, number, count rounds, keep gate state). Deciding whether a merge affects the project, planning the integration and agreeing or countering belong to whoever runs the protocol: any agent or a person. Agent-specific hooks or editor rules are optional shortcuts that call the same commands.
- **Negotiation** has at most three rounds; then the user decides. The user approves or rejects every agreed integration at the integration gate; both projects receive the decision.
- **Aurora's interface** reads and writes local clones through the provider and never fetches or pushes; Git does, through the provider.

## Alternatives Considered

- **A folder in each project's work branch (`.aurora/inbox/`)**: every merge would add commits to every subscribed repository, trigger CI and collide with branch protection and with ADR-0006.
- **A central mailbox repository**: one place to read, but it must be created in the customer's Git, which companies do not accept from a tool, and it needs access to one more repository.
- **A hidden ref outside branches (`refs/aurora/inbox`)**: invisible in the branch list and untouched by branch rules, but refused by Git policies (seen in the test above); kept as a possible later optimisation where the server accepts it.
- **Host features (issues, pull request comments, workflow automation)**: tie the feature to one host and its APIs, and comments posted by tools can receive footers that break principle 9.
- **A service run by Aurora**: contradicts the air-gapped design.

## Consequences

- Works on any Git host and with any agent; the only requirement is write access to the repositories of the projects one talks to.
- The `aurora/inbox` branch shows in the host's branch list, and right after a push some hosts suggest opening a pull request; branch protection rules with a wildcard must exclude it (installation documentation).
- The Dev Cycle Protocol needs a generic extension point for input providers and gates (suggestion to the Dev Cycle Protocol of 2026-10-29); until it exists, the provider can only be run by hand.
- Until the provider exists, a provisional tool on this repository's `aurora/inbox` branch exchanges free messages in the same format (IMP-001); it is not part of the product and is removed when the provider ships.
- Verification: the provider's tests run against bare repositories created in a temporary folder, covering concurrent writers (a rejected push is rebuilt on the new tip), the round limit and a missing `aurora/inbox` branch.
