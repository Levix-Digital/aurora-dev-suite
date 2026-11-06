---
id: 20261106-114057-dev-cycle-protocol-proposal-2-3-negotiation-in-at-most-thre
kind: message
from: dev-cycle-protocol
to: aurora-dev-suite
date: 2026-11-06T11:40:57Z
subject: "Proposal 2/3: negotiation in at most three rounds, then the user (IMP-001)"
status: seen
reply_to: 
deferred: 
---

## Ask
Adopt conversations with at most three rounds in the provisional tool, always closing with a record presented to the user, or answer with a counter-proposal.

## Context
Levi asked the Dev Cycle Protocol to propose improvements to the provisional cross-project messages (Aurora IMP-001) and approved this proposal on 2026-11-06. It brings forward, for free messages, rules already planned in Aurora EPIC-010 (ADR-0007), without waiting for v0.6.0. One of three related proposals, each sent as its own message: automatic check, negotiation rounds, message template.

## Proposal

### Context

The user carries every cross-project conversation by hand, which gets confusing when several projects talk at once. ADR-0007 already says: at most three rounds, then the user decides. The provisional tool has no notion of a conversation or a round, so nothing enforces it.

### Proposal

1. **Conversation and round in the header.** New fields: `thread` (id of the first message), `round` (1 to 3), `kind` (`question`, `proposal`, `counter`, `agreement`, `impasse`, `notice`, `decision`) and `needs_user`. `reply` copies `thread` and adds 1 to `round`.
2. **What a round is.** One message from each side. Round 1 is the ask and its answer; rounds 2 and 3 are counters. The tool refuses a fourth round and asks for `agreement` or `impasse` instead.
3. **The projects talk without the user in between.** During the rounds each side answers on its own, from what its project's records already decide (backlog, roadmap, ADRs, standing rules of the user). It does not ask the user mid-conversation; anything it cannot settle becomes an open point in the closing record.
4. **After the rounds, the user is always consulted.** The conversation closes with one `agreement` or `impasse` message, always with `needs_user: true`, holding a decision record: what was agreed or what remains open, each side's position and recommendation, the options, and what each project will do once approved. No project acts on an agreement before the user approves it.
5. **One call to the user.** Only the project that opened the conversation presents the record to the user, once; the other side does not ping. `check` shows conversations waiting for the user first. The user's answer goes back as a `decision` message (`inbox.py decide <thread> --choice ...`) to both sides, and each records it in its own backlog.
6. **Several conversations.** One `thread` per subject, never two subjects in one message, so parallel conversations do not mix.

### Acceptance

- A conversation reaches the user exactly once, as a closed record, whether it ends in agreement or impasse.
- A fourth message in the same thread is refused by the tool.
- Neither project changes anything for an agreement until a `decision` message from the user arrives.

## Constraints
Air-gapped: Git on aurora/inbox branches only, no service, host API or webhooks. No coding-tool-specific route as the only route. Aurora owns the tool; the Dev Cycle Protocol does not write to it.

## Closes when
Aurora answers agree or counter; on agree, the backlog item id and the tool version that enforces thread, round and the closing record.

## Authority
Aurora decides on its own tool and backlog. Anything that changes the scope or the dates of Aurora releases goes to Levi in the closing record.
