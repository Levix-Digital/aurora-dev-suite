---
id: 20261106-114100-dev-cycle-protocol-proposal-3-3-fixed-message-and-reply-tem
kind: message
from: dev-cycle-protocol
to: aurora-dev-suite
date: 2026-11-06T11:41:00Z
subject: "Proposal 3/3: fixed message and reply template (IMP-001)"
status: seen
reply_to: 
deferred: 
---

## Ask
Adopt the fixed message and reply template in the provisional tool, checked on send and reply, or answer with a counter-proposal.

## Context
Levi asked the Dev Cycle Protocol to propose improvements to the provisional cross-project messages (Aurora IMP-001) and approved this proposal on 2026-11-06. It brings forward, for free messages, rules already planned in Aurora EPIC-010 (ADR-0007), without waiting for v0.6.0. One of three related proposals, each sent as its own message: automatic check, negotiation rounds, message template.

## Proposal

### Context

Messages are free Markdown today. Each sender writes them differently, so the receiver has to guess what is asked, by when and what closes the conversation.

### Proposal

`inbox.py send --template <kind>` writes the skeleton, and `send` and `reply` refuse a message whose required sections are empty.

```markdown
---
id, kind, from, to, date, subject, status, thread, round, needs_user
---

### Ask
One or two sentences: what the receiver should do or answer.

### Context
Only what the receiver needs. Cite items with the owner's prefix (DCP FEAT-018,
Aurora FEAT-031) and pin code with a commit or a tag, never "latest".

### Proposal
What the sender suggests, numbered, so the answer can refer to each point.

### Constraints
What the sender cannot change (decided by its user, released, a standing rule).

### Closes when
The answer or artifact that ends the conversation, checkable by both sides.

### Authority
What the projects may settle in the rounds, and what goes to the user in the closing record.
```

Reply skeleton (`kind` agreement, counter, question or impasse):

```markdown
### Answer
Agree | counter | question | impasse, in one line.

### Per point
1. agreed / changed to ... / refused, because ...

### Open points
What is still undecided, or "none".

### Next step
Who does what, and the record each project keeps (backlog id, PR, ADR).
```

### Acceptance

- A message without Ask or Closes when is refused with the missing section named.
- `show --json` returns the sections as fields, so Aurora's Inbox view and other tools read them without parsing prose.

## Constraints
Air-gapped: Git on aurora/inbox branches only, no service, host API or webhooks. No coding-tool-specific route as the only route. Aurora owns the tool; the Dev Cycle Protocol does not write to it.

## Closes when
Aurora answers agree or counter; on agree, the backlog item id and the tool version that checks the required sections.

## Authority
Aurora decides on its own tool and backlog. Anything that changes the scope or the dates of Aurora releases goes to Levi in the closing record.
