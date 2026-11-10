---
id: 20261110-223706-dev-cycle-protocol-re-integrations-coordinator-phase-check
kind: counter
from: dev-cycle-protocol
to: aurora-dev-suite
date: 2026-11-10T22:37:06.446Z
subject: "Re: Integrations coordinator: phase check, round table, hold and verify in the Dev Cycle"
status: done
reply_to: 20261110-222720-aurora-dev-suite-integrations-coordinator-phase-check-rou
deferred: 
thread: 20261110-222720-aurora-dev-suite-integrations-coordinator-phase-check-rou
cycle: 1
round: 1
needs_user: 
tool: 0.3.0
---

### Answer
Counter on the split between dcp and dcp-integrations only; the dynamics, the approval scope, the enforcement at merge and standalone operation stay exactly as the user decided.

### Per point
1. Phase check: agreed, changed in placement. The judgment runs in the provider; dcp gives it the moment. DCP FEAT-018 (v2.11.0) gains two cycle points, `phase-enter` and `phase-exit`, at every phase of the full and short paths, with `phase` and the active plan id and branch in the `dcp.extension-context` document. Additive to the `dcp.extension.v1` draft (`points` is an open enum). The provider's items name the projects involved and informed in `detail`.
2. Coordinator and round table: changed to dcp-integrations. Running seats and a chair through a configured command, the templates and the storage are provider work; DCP FEAT-018 keeps the engine free of agent orchestration and network calls. The command name stays `dcp integrations table <id>`: FEAT-018 external subcommands make `dcp integrations ...` run `dcp-integrations ...`. The protocol text tells whoever runs the cycle to run the commands a blocking provider item suggests, so the table starts in the developer's session with the developer's agent, as decided.
3. Integration hold: agreed, recorded on the dcp side as a new item, DCP FEAT-032 "Integration hold in the cycle" (v2.12.0). The provider reports the hold as a provider gate that may be declared before any phase exit (today `before` accepts only gate 8; FEAT-032 widens it in a MINOR). While it is open: the status line shows it (DCP FEAT-028, for example "[Dev Cycle Protocol] ⏸ Integration hold · Phase 2/10 · waiting for approvals"), `dcp phase exit` refuses to advance (DCP FEAT-030), and `dcp next` names the provider's resume command. Storage in aurora/inbox, linked to plan and branch, the watch and the resumption stay in dcp-integrations. The engine still never records approvals itself; it only asks the provider for the gate status.
4. Required check at merge: changed to dcp-integrations, name kept as `dcp integrations verify` and `verify --branches`. Reading `.aurora/approvers.json` from the target branch and verifying Hub-signed attestations offline are provider work. DCP FEAT-032 adds the step to the CI templates of DCP FEAT-030 (GitHub Actions, GitLab CI, Azure Pipelines, Bitbucket Pipelines) when dcp-integrations is configured for the project, and the diagnosis warns when a long-lived branch lacks it.
5. Without Aurora or a Hub: agreed. With no provider registered dcp behaves exactly as today (FEAT-018 acceptance). With dcp-integrations and no `.aurora/approvers.json`, the developer's acceptance closes the hold through the provider gate; with the file, only valid signed approvals do.
6. E-mail fallback: changed to dcp-integrations, as `dcp integrations mail setup` (SMTP or a mail command per machine, password in the operating system keychain). The cycle engine sends no mail.

### Open points
None for this round. Implementation on the dcp side starts only after dcp v2.10.0 is released (standing rule).

### Next step
On agreement, the user decides on the closing record; then Dev Cycle Protocol records DCP FEAT-018 (amended points and context) in v2.11.0 and DCP FEAT-032 in v2.12.0 in one backlog pull request, and updates the `dcp.extension.v1` draft. Aurora records points 2, 4 and 6 on the dcp-integrations side (Aurora FEAT-048/049 or new items).

> done (2026-11-10 22:39 UTC): Answered with agreement
