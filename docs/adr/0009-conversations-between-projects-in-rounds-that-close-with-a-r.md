---
id: 9
title: Conversations between projects in rounds that close with a record for the user
status: accepted
date: '2026-11-06'
version: null
subsystems: [core, engines]
supersedes: []
superseded_by: []
amends: [7]
amended_by: []
---

# ADR-0009: Conversations between projects in rounds that close with a record for the user

## Context

ADR-0007 set the transport (an orphan `aurora/inbox` branch per repository), at most three rounds of negotiation and the integration gate. Running the provisional tool (IMP-001) between six Levix projects from 2026-10-29 to 2026-11-06 showed what it left open:

- A project read its inbox only when a session ran `check` at its start; long sessions never looked again, and a project nobody worked on never looked at all, so the user kept asking each project to check.
- Nothing defined a conversation or a round, so the three-round limit was not enforced, and the user carried conversations by hand, asked in the middle of them.
- Messages were free Markdown; each receiver had to guess what was asked and what closed it.
- The user received the outcome of a conversation in a different shape every time.

On 2026-11-06 the Dev Cycle Protocol proposed an automatic check, rounds and fixed templates (Aurora IMP-002). Levi accepted them with adjustments and asked for a fixed report for the user. The provisional tool 0.2.0 (`aurora/inbox`, 837f072) implements them; 0.3.0 (479da57) adds the user's requests in the middle of a conversation, also asked for by Levi on 2026-11-06. DCP FEAT-018, the extension point, moved to dcp v2.11.0, with a draft contract `dcp.extension` 1.0 that lets a provider declare its points and gates through `describe --json` (Levix-Digital/ai-code-agent-skills pull request #86).

## Decision

This amends ADR-0007; its transport, layout and gate stay.

- **When to look.** A cheap check (`poll`: one `git ls-remote` per inbox, no fetch, silent when nothing changed) runs at every touchpoint: the start of work, before every gate, before opening a pull request and after a merge. The generic route is the extension point (`discovery`, `before-release-gate`, `after-merge`); tool-specific hooks may call the same command, never as the only route. An optional `watch` wakes a project nobody works on by running the user's own local command in its folder. A check also lists messages this project sent that are still unread after a set time.
- **Conversation.** Every message carries `thread` (the id of the first message), `round` and `needs_user`; `kind` takes `question`, `proposal`, `notice` to open, `answer`, `counter`, `agreement`, `impasse`, `question`, `ack` to reply, `decision` for the user's answer and `guidance` for the user's request. ADR-0007's `merge`, `approved` and `rejected` become `notice` and `decision`.
- **Rounds.** A round is one message from each side; round 1 is the ask and its answer. After three rounds only `agreement` or `impasse` is accepted. `ack`, `notice`, `decision` and `guidance` are outside rounds, so an acknowledgement never uses one up. Rounds are counted within the current cycle.
- **No user in the middle, unless the user steps in.** During the rounds each side answers from what its project's records already decide (backlog, roadmap, ADRs, the user's standing rules), without asking the user; what it cannot settle becomes an open point of the closing record. The user, on the other hand, may step in at any time: a request written while the projects talk, or a revision asked for when a closing record does not satisfy (`guide`, `decide --revise`). It goes to every side as a `guidance` message, any open closing record is set aside, and a new cycle starts (`cycle` in the header): up to three new rounds that take the request into account, ending in a new closing record. The cycle repeats until the user decides.
- **Closing record.** A conversation always ends with an `agreement` or an `impasse`, which always needs the user. It carries a fixed report written for the user: the projects and rounds, the subject, the outcome, Conclusion, Justification, Positions (impasse), Decision needed (options with one recommended) and After approval (what each project does). Only the project that opened the conversation presents it, once, translated into the user's language; the user's answer is sent to every side as a `decision`, and each project records it in its own backlog. No project acts on an agreement before the decision arrives. The integration gate of ADR-0007 is this decision for conversations about an integration.
- **Templates.** Each kind has a fixed set of sections (Ask, Context, Proposal, Constraints, Closes when, Authority to open; Answer, Per point, Open points, Next step to reply; the report sections to close). Sending refuses a message whose required sections are empty, and `show --json` returns the sections as fields, so the interface reads them without parsing prose.
- **Provider registration.** The integrations provider declares its cycle points and the `integration` gate through `dcp-integrations describe --json` (draft `dcp.extension` 1.0); the installer only registers its name at user level.

## Alternatives Considered

- **Keep the check at session start only**: cheapest, but it is what failed in practice.
- **Notifications from the Git host (webhooks, workflow runs, e-mail)**: tie the feature to one host and break the air-gapped design (ADR-0007).
- **Ask the user at every round**: keeps the user in control of each step, but makes the user the carrier of every conversation, which is what the change removes; the closing record keeps the control at the end, and the user's requests let the user steer without being asked.
- **A closed conversation, with a new one for every change the user wants**: simpler counting, but the history of the subject would be split across threads; one thread with cycles keeps it together.
- **Free-form closing summaries**: fewer rules, but each project would report differently and the interface could not show them side by side.

## Consequences

- Every Levix project's instructions say when to poll, the reply kinds, the templates and the closing record (updated by Levi on 2026-11-06).
- The provider (FEAT-034) implements the same commands and header as the provisional tool 0.2.0, so moving from one to the other changes no message already written; messages of tool 0.1.x (`kind: message`, no thread) are read as their own threads.
- The Integrations view (FEAT-033) and the Inbox show the conversations waiting for the user first and display the closing report as fields.
- The `after-merge` point may need more than the draft's 30 seconds, since one notice is pushed to every subscribed project; Aurora asked DCP for a per-point limit declared in `describe` (2026-11-06).
- Verification: the provider's tests cover the round limit, the refusal of incomplete templates, `ack` outside rounds, the closing record shown only to the opener, the decision sent to every side, a user request restarting the rounds in a new cycle, and `poll` making no fetch when nothing changed.
