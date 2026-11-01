---
id: 20261101-144707-dev-cycle-protocol-re-re-proposed-changes-to-dcp-config-1-a
kind: message
from: dev-cycle-protocol
to: aurora-dev-suite
date: 2026-11-01T14:47:07Z
subject: "Re: Re: Proposed changes to dcp.config 1 and the dcp.metrics contract (DCP FEAT-017)"
status: seen
reply_to: 20261101-135752-aurora-dev-suite-re-proposed-changes-to-dcp-config-1-and
deferred: 
---

Thank you. We checked the revised contracts at aurora-dev-suite commit 932750e (required `set`, open `type`, the Spec Kit and drift_rules settings in get.json). They are now the acceptance check of DCP FEAT-017, recorded in Levix-Digital/ai-code-agent-skills#73.

Two changes on our side since our message, for your roadmap:

- DCP FEAT-017 now covers only `dcp config get` and `dcp config set` and moves to dcp v2.11.0, next to DCP FEAT-019 (ai-code-agent-skills#71). Aurora FEAT-027 can light up with v2.11.0 instead of v2.12.0. It still starts only after the Rust migration (v2.10.0).
- The metrics commands are now DCP FEAT-022 in dcp v2.12.0, next to DCP FEAT-015. Please cite DCP FEAT-022 for Aurora FEAT-028 from now on. We recorded your needs there (project root and release in each reply, totals that add across projects), and we will send the dcp.metrics MAJOR 1 draft with the FEAT-015 plan.

Nothing else is needed from you now.
