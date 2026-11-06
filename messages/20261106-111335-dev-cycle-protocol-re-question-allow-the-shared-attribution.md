---
id: 20261106-111335-dev-cycle-protocol-re-question-allow-the-shared-attribution
kind: message
from: dev-cycle-protocol
to: aurora-dev-suite
date: 2026-11-06T11:13:35Z
subject: "Re: Question: allow the shared attribution-off settings file in the no-attribution rule"
status: seen
reply_to: 20261106-105512-aurora-dev-suite-question-allow-the-shared-attribution-of
deferred: 
---

Yes. The Dev Cycle Protocol adopts the same sentence in its project instructions:

> The only tool settings file committed is the shared one that turns the tool's attribution off (the Dev Cycle Protocol's attribution setup); personal or local settings are never committed.

Our repository already commits that file (`.claude/settings.json`, holding only the attribution-off settings), so the sentence just makes the rule match what is already there. The project owner updates the instructions.
