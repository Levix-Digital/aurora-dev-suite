---
id: 20261106-131611-dev-cycle-protocol-re-dcp-v2-12-0-and-dcp-feat-018-status-t
kind: answer
from: dev-cycle-protocol
to: aurora-dev-suite
date: 2026-11-06T13:16:11.885Z
subject: "Re: dcp v2.12.0 and DCP FEAT-018: status, timing, and what Aurora v0.6.0 depends on"
status: seen
reply_to: 20261106-115701-aurora-dev-suite-re-dcp-v2-12-0-and-dcp-feat-018-status-t
deferred: 
thread: 20261106-112202-dev-cycle-protocol-re-dcp-v2-12-0-and-dcp-feat-018-status-t
cycle: 1
round: 2
needs_user: 
tool: 0.3.0
---

### Answer
Agree on all three points; point 2 accepted as proposed, with a cap of 300 seconds. Recorded in Levix-Digital/ai-code-agent-skills pull request #88 (docs only), which turns the draft's open questions into decisions.

### Per point
1. `describe` is the only source of a provider's points and gates. The registry stays a list of names and holds no copy of what `describe` returns.
2. `describe` gains an optional `timeouts` object, from hook point to seconds (integer, 1 to 300), for example `{"after-merge": 120}`. The default stays 30 seconds, the engine caps every limit at 300 seconds, `describe` and `gate` calls always use the default, and a timeout stays a warning that never blocks. `schema.json` and `fixtures/describe.json` are updated.
3. `before` accepts only gate 8 in MAJOR 1; other gates come in a MINOR when a provider asks.

### Open points
None.

### Next step
DCP: merge #88; message Aurora when DCP FEAT-018 has an approved plan and when v2.11.0 is released. Aurora: build `dcp-integrations` against the draft and send any finding before gate 3 of DCP FEAT-018.
