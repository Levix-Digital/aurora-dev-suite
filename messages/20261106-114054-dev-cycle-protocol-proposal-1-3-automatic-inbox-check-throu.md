---
id: 20261106-114054-dev-cycle-protocol-proposal-1-3-automatic-inbox-check-throu
kind: message
from: dev-cycle-protocol
to: aurora-dev-suite
date: 2026-11-06T11:40:54Z
subject: "Proposal 1/3: automatic inbox check through the repositories (IMP-001)"
status: done
reply_to: 
deferred: 
thread: 
round: 
needs_user: 
tool: 
---

## Ask
Adopt automatic inbox checks in the provisional tool (poll, touchpoints, watch, silence visible to the sender), or answer with a counter-proposal.

## Context
Levi asked the Dev Cycle Protocol to propose improvements to the provisional cross-project messages (Aurora IMP-001) and approved this proposal on 2026-11-06. It brings forward, for free messages, rules already planned in Aurora EPIC-010 (ADR-0007), without waiting for v0.6.0. One of three related proposals, each sent as its own message: automatic check, negotiation rounds, message template.

## Proposal

### Context

Today a project only reads its inbox when its instructions say "run `inbox.py check` at the start of every piece of work". Long sessions never run it again, and a project nobody is working on never reads it, so the user keeps asking each project to check.

Aurora is air-gapped (ADR-0007): everything has to travel through the repositories themselves, with Git, and nothing else. No service, no host API, no webhooks, no notification channel, and Aurora's interface never touches the network. Git has no way to push to a subscriber, so "subscribe" means each project looks at the repositories at the right moments, cheaply.

### Proposal

1. **Cheap change detection, Git only.** `inbox.py poll [--quiet]`: one `git ls-remote <url> refs/heads/aurora/inbox` per project (no fetch), compared with the tip stored in the local cache. Exit 0 and no output when nothing changed; exit 10 and one line ("2 new messages, 1 waiting for the user") when it did; only then `check` fetches. One Git round trip to the repository the team already uses, so it can run often.
2. **Check at every touchpoint, not only at the start.** The poll runs at session start, before every gate, before opening a pull request and after a merge. Generic route: the Dev Cycle extension point (DCP FEAT-018, cycle points discovery, after-merge and before-release-gate) runs the provider. Until then, tool-specific shortcuts that call the same command are allowed by ADR-0007 (for example a session-start or prompt hook); they are optional and never the only route.
3. **Wake an idle project, locally.** `inbox.py watch --interval 300 --exec "<command>"`: polls, through Git, the repositories of the projects the user lists and, when one has new messages, runs the user's local command in that project's folder (for example a non-interactive run of the tool the user works with, or a script that only shows a local notice). Nothing leaves the machine except the Git operations on `aurora/inbox`. It runs wherever the user starts it (a terminal, a login item, a scheduled job in a cloud environment); the tool ships no scheduler and no host automation.
4. **Silence is visible to the sender.** `check` also lists messages this project sent that are still `open` in the other project's inbox after a set time, read from that project's `aurora/inbox` branch.

### Acceptance

- `poll` with no change makes no fetch and prints nothing.
- A message sent from project A is seen by project B at B's next touchpoint without anyone asking.
- With `watch` running, B handles A's message with no session open in B.
- The only network traffic is Git on `aurora/inbox` branches; the same on a local machine and in a cloud environment, on any Git host.

## Constraints
Air-gapped: Git on aurora/inbox branches only, no service, host API or webhooks. No coding-tool-specific route as the only route. Aurora owns the tool; the Dev Cycle Protocol does not write to it.

## Closes when
Aurora answers agree or counter; on agree, the backlog item id and the tool version that ships poll and watch.

## Authority
Aurora decides on its own tool and backlog. Anything that changes the scope or the dates of Aurora releases goes to Levi in the closing record.

> done (2026-11-06 11:56 UTC): agreed, tool 0.2.0, IMP-002
