---
id: 15
title: 'Integrations coordinator: round tables, approvals by every involved project and the integration hold'
status: accepted
date: '2026-11-10'
version: null
subsystems: [integrations, core, ui]
supersedes: []
superseded_by: []
amends: [7, 9, 13, 14]
amended_by: [16]
---

# ADR-0015: Integrations coordinator: round tables, approvals by every involved project and the integration hold

## Context

ADR-0007 and ADR-0009 let two projects talk through their `aurora/inbox` branches in at most three rounds, closed by a record the user decides on once. In practice nobody starts those conversations at the right time, and the user ends up relaying them. On 2026-11-10 Levi set the direction (design document "Integrations coordinator", v4, and "Round table"): a check at every phase of the Dev Cycle detects a possible integration adjustment, a coordinator settles it in the background at a round table of the projects involved, and each project's responsible person approves the outcome. Aurora is the governance view for leads, architects and managers (ADR-0014); developers work through the Dev Cycle Protocol, which keeps working without Aurora.

## Decision

Levi decided on 2026-11-10:

- **Phase check.** At every phase of the Dev Cycle, a check reads the project's content up to that phase and decides whether it may touch another project's contracts. It belongs to the Dev Cycle Protocol (through the extension point of DCP FEAT-018) and names the projects involved and the projects only informed.
- **The coordinator chairs a round table.** It receives the result of the check, puts the problem to each involved project and never takes a position. Each involved project has a seat: an inference instance that reads only its own project's records (repository at its production branch, project card cached by commit) and cites a file and commit for every claim. Shared libraries have no seat; their contracts reach the table through the cards of the projects that use them. Informed projects receive the outcome as a `notice`.
- **Dynamics.**
  1. Opening: the coordinator sends the problem with the `table-opening` template.
  2. Initial answers, independent: each project answers with `table-answer`. A project that answers "not affected" names the contracts it checked, with citations, and leaves the table; it is called back if a later counterproposal touches one of its contracts, and it receives the outcome as a `notice`.
  3. Round 1: each remaining project receives every other project's answer (not its own) and reviews each one with `table-review`: agree, or a counterproposal with its reason.
  4. Unanimous agreement closes the table and the coordinator writes the complete proposal (`table-proposal`).
  5. Otherwise only the counterproposals go back, consolidated by the coordinator: compatible ones become one revised proposal; contradicting ones are sent as alternatives. Each project answers each item (round 2), then round 3 the same way. Three rounds at most.
  6. Without agreement after round 3, the coordinator writes the impasse record (`table-impasse`: positions, options, a recommendation) for the responsible people of every involved project.
  7. After an agreement, each project confirms that the final text states what was agreed, or flags a misstatement once; this is not a round.
- **Templates.** `table-opening`, `table-answer`, `table-review`, `table-proposal` and `table-impasse` have fixed fields with citations required; the provider refuses a message whose required fields are empty (as ADR-0009's templates).
- **Approval by every involved project.** An agreement or an impasse needs the approval of the responsible people of every involved project, whether the project is being built, maintained or is legacy; there is no idle threshold. Approvers are Admin Hub groups per project and approvals are attestations signed by the Hub (ADR-0014). An approver may approve, ask or object; a question or objection starts a new table with that input; after three tables the decision closes as an impasse for the approvers.
- **Integration hold.** The triggering project's cycle stops at an integration hold until every involved project has approved. The hold is recorded in the project's `aurora/inbox`, linked to the plan and the branch. An open session watches the inbox and continues when the last approval arrives; a later session finds the hold at its start and resumes from the same phase.
- **Enforcement at merge.** A required check, `dcp integrations verify`, runs on every pull request into each long-lived branch of the branching model (integration: `dev` and `main`; trunk: `main`; release-branches: `main` and every `release/X.Y`) and fails while the pull request's plan or branch has an open hold without valid attestations. It reads the governing Hub's key fingerprint and approver groups from `.aurora/approvers.json` on the target branch, never from the pull request, and verifies offline. `dcp integrations verify --branches` lists the branches to protect; Aurora warns about a listed branch without the required check. Local guards (the phase refuses to advance, a pre-push warning) are conveniences only.
- **Without Aurora or a Hub.** A project with no `.aurora/approvers.json` needs no approval: the developer's acceptance in the session closes the hold. A project that has one never moves without the signed approvals.
- **Where the table runs.** Triggered by a phase check: in the developer's session, with the developer's agent. Triggered by an approver's question or objection: on the Aurora server, with the agent configured once per organisation. Same command either way (`dcp integrations table <id>`); seats and the chair are calls to a configured command, so any agent can run them. A token and time budget per table is set once per organisation.
- **Notifications.** Approvers are notified at once by e-mail through the Admin Hub (Hub FEAT-049); without a Hub, `dcp` sends it through a mail setting configured once per machine (`dcp mail setup`), and without that the hold names the approvers.
- **Storage.** A table is one conversation with many participants, numbered as in ADR-0007. The full transcript is written to the inbox of the project that triggered it; every other involved inbox receives the opening, the outcome, its own messages and a pointer to the transcript. Attestations are written to every involved inbox.
- **Direct messages become two-project tables.** `send` and `reply` between two projects follow the same model with two seats, so there is one model and one view.
- **Messages view becomes a list of tables** (amends ADR-0013): the left column lists tables (subject, the projects' logos, state, my pending action; filters by project and "waiting for my approval"); the centre shows the table as a group chat, with the coordinator's lines as centred system lines (opening, rounds, a project leaving as not affected) and each project's messages as bubbles with its logo, summary and citations; the outcome closes it as a card with one approval chip per involved project and, for that project's approvers, Approve, Ask and Object; the right panel keeps the full message. The project view shows the hold on the cycle timeline.
- **Shared libraries** are marked by the user with a checkbox when a project is added (with help text and a confirmation); their approvers, if any, approve only changes that touch them.
- **Ownership.** The Dev Cycle Protocol owns the phase check, the coordinator, the round table, the hold and its resumption, `verify` and the mail fallback (proposed to dev-cycle-protocol). `dcp-integrations` owns the table messages, templates, storage, notices and the verification of attestations. Aurora owns the governance view: tables, approvals through the Admin Hub, the hold on the timeline, restarting a table after an approver's answer, the add-project form with the shared-library checkbox.

## Alternatives Considered

- **Pairwise conversations started by a project** (ADR-0009 alone): nobody starts them at the right time and the user relays them.
- **The triggering project chairs the table**: its seat would both argue and summarise, which biases the outcome.
- **Every round in turn**: the first speaker anchors the others; independent initial answers avoid it.
- **Every counterproposal voted separately**: projects agree with different counterproposals and the table never converges; consolidation makes each round vote on one thing.
- **Approval only by the triggering project**: other projects would learn of a change to their contracts after it is decided.
- **A full transcript in every inbox**: simpler reading but many more writes for the same content.
- **An always-on runtime or a message broker**: needs a server for every portfolio; the phase check already gives the moment to act.

## Consequences

- The phase check, the coordinator, the hold and `verify` depend on dcp (FEAT-018 extension point, dcp v2.11.0 and later); approvals depend on Admin Hub v0.4.0. Until then the existing pairwise conversations keep working.
- ADR-0009's rounds, closing record, templates and the user's guidance carry over; its two-party closing record becomes the table's outcome.
- ADR-0013's summary, logos, sliding panel and palette carry over; its contacts column and the sent-right/received-left layout go, because a lead reading a table is not one of the parties.
- Each table spends inference on every involved project; the budget and the "not affected" exit keep it bounded.
- The decision holds while no project acts on an outcome before the approvals and no seat speaks for a project other than its own: both are checked in review and by `verify`.
